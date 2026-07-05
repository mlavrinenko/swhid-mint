//! Git backend: derive and resolve SWHIDs against a local git repository via the
//! `gix` (gitoxide) crate — a real git object model, no subprocess, no porcelain
//! parsing.
//!
//! For content/directory/revision the SWHID hash is `sha1_git`, identical to
//! git's own object SHA-1, so we read commit/blob ids straight from the object
//! database and prefix the kind: no libgit2, no hashing of our own beyond the
//! working blob.

use std::path::Path;

use gix::ObjectId;

use crate::{Kind, Swhid};

/// Why a git-backed SWHID operation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A `gix` operation failed (repository open, object lookup, revision walk).
    Git(String),
    /// No commit touches the path — nothing to pin a revision id to.
    NotTracked(String),
    /// The working-tree blob differs from the blob recorded at the commit, so a
    /// minted id would not recover the current bytes.
    Dirty(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Git(msg) => write!(f, "git: {msg}"),
            Self::NotTracked(path) => write!(f, "{path}: no commit touches this path"),
            Self::Dirty(path) => write!(
                f,
                "{path}: working tree differs from the recorded blob; commit first"
            ),
        }
    }
}

impl std::error::Error for Error {}

fn git_err(err: impl std::fmt::Display) -> Error {
    Error::Git(err.to_string())
}

/// Mint a revision-scoped SWHID for `path` (repository-relative): its last-touch
/// commit, qualified by path. Verifies the working-tree blob still equals the
/// blob recorded at `HEAD`, so the id recovers the same bytes.
///
/// # Errors
/// [`Error::NotTracked`] when `HEAD` has no entry for the path, [`Error::Dirty`]
/// when the working tree diverges from `HEAD`'s recorded blob, [`Error::Git`] on
/// any git failure.
pub fn mint_rev_path(root: &Path, path: &str) -> Result<Swhid, Error> {
    let repo = gix::discover(root).map_err(git_err)?;
    let head = repo.head_commit().map_err(git_err)?;
    let head_blob =
        entry_oid(&repo, head.id, path)?.ok_or_else(|| Error::NotTracked(path.to_owned()))?;
    let working = working_blob_id(&repo, root, path)?;
    if working != head_blob {
        return Err(Error::Dirty(path.to_owned()));
    }
    let commit = last_touch_commit(&repo, path, head_blob)?
        .ok_or_else(|| Error::NotTracked(path.to_owned()))?;
    Ok(Swhid {
        kind: Kind::Revision,
        hash: commit.to_hex().to_string(),
        path: Some(path.to_owned()),
    })
}

/// Mint a content (`cnt`) SWHID for `path` from its working-tree blob.
///
/// # Errors
/// [`Error::Git`] when the working file cannot be read or hashed.
pub fn mint_content(root: &Path, path: &str) -> Result<Swhid, Error> {
    let repo = gix::discover(root).map_err(git_err)?;
    let hash = working_blob_id(&repo, root, path)?;
    Ok(Swhid {
        kind: Kind::Content,
        hash: hash.to_hex().to_string(),
        path: None,
    })
}

/// True when `id` still resolves in `root`: its object exists, and when a `path`
/// qualifier is present, the path exists at that revision.
///
/// # Errors
/// [`Error::Git`] only when the repository cannot be opened or the hash is
/// malformed; a clean "does not resolve" is `Ok(false)`.
pub fn resolves(root: &Path, id: &Swhid) -> Result<bool, Error> {
    let repo = gix::discover(root).map_err(git_err)?;
    let oid = ObjectId::from_hex(id.hash.as_bytes()).map_err(git_err)?;
    if repo.find_object(oid).is_err() {
        return Ok(false);
    }
    match &id.path {
        Some(path) => Ok(entry_oid(&repo, oid, path)?.is_some()),
        None => Ok(true),
    }
}

/// Compute the `sha1_git` of `path`'s current working-tree bytes (a git blob id).
fn working_blob_id(repo: &gix::Repository, root: &Path, path: &str) -> Result<ObjectId, Error> {
    let data = std::fs::read(root.join(path)).map_err(git_err)?;
    gix::objs::compute_hash(repo.object_hash(), gix::object::Kind::Blob, &data).map_err(git_err)
}

/// The commit reachable from `HEAD` that introduced `target_blob` at `path` —
/// its blob matches `target_blob` while its first parent's does not (or it has
/// no parent). `None` when no reachable commit matches.
///
/// `gix`'s ancestor walk defaults to [`Sorting::BreadthFirst`], which orders by
/// graph distance rather than history: in a diamond, a stale-content commit
/// reachable via a short merge-side route can be visited before the commit that
/// actually last set `path`'s content. Gating on `target_blob` (`HEAD`'s current
/// blob, already confirmed clean by the caller) rather than "any change" makes
/// the search immune to that ordering — a distance-nearer commit with a
/// different blob is skipped, not returned.
fn last_touch_commit(
    repo: &gix::Repository,
    path: &str,
    target_blob: ObjectId,
) -> Result<Option<ObjectId>, Error> {
    let head = repo.head_commit().map_err(git_err)?;
    for info in head.ancestors().all().map_err(git_err)? {
        let info = info.map_err(git_err)?;
        let Some(cur) = entry_oid(repo, info.id, path)? else {
            continue;
        };
        if cur == target_blob && touched_here(repo, &info, path, cur)? {
            return Ok(Some(info.id));
        }
    }
    Ok(None)
}

/// Whether `commit` changed `path`'s blob relative to its first parent (or has no
/// parent, introducing the path).
fn touched_here(
    repo: &gix::Repository,
    info: &gix::revision::walk::Info<'_>,
    path: &str,
    cur: ObjectId,
) -> Result<bool, Error> {
    match info.parent_ids().next() {
        None => Ok(true),
        Some(parent) => Ok(entry_oid(repo, parent.detach(), path)? != Some(cur)),
    }
}

/// The blob id recorded at `path` in `commit`'s tree, or `None` if absent.
fn entry_oid(
    repo: &gix::Repository,
    commit: ObjectId,
    path: &str,
) -> Result<Option<ObjectId>, Error> {
    let tree = repo
        .find_commit(commit)
        .map_err(git_err)?
        .tree()
        .map_err(git_err)?;
    let entry = tree.lookup_entry_by_path(path).map_err(git_err)?;
    Ok(entry.map(|e| e.oid().to_owned()))
}

#[cfg(test)]
#[path = "git_tests.rs"]
mod tests;
