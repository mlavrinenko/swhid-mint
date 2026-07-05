#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::{Error, mint_content, mint_rev_path, resolves};
use crate::Kind;
use crate::test_support::{git, repo};

#[test]
fn mints_and_resolves_rev_path() {
    let dir = repo();
    let root = dir.path();
    std::fs::write(root.join("a.typ"), "hello").expect("write");
    git(root, &["add", "a.typ"]);
    git(root, &["commit", "-q", "-m", "add a"]);

    let id = mint_rev_path(root, "a.typ").expect("mint");
    assert_eq!(id.kind, Kind::Revision);
    assert_eq!(id.hash.len(), 40);
    assert_eq!(id.path.as_deref(), Some("a.typ"));
    assert!(resolves(root, &id).expect("resolves"));
}

#[test]
fn refuses_untracked_path() {
    let dir = repo();
    let root = dir.path();
    // Repo needs history; an untracked file then has no touching commit.
    std::fs::write(root.join("seed.typ"), "seed").expect("write");
    git(root, &["add", "seed.typ"]);
    git(root, &["commit", "-q", "-m", "seed"]);
    std::fs::write(root.join("u.typ"), "x").expect("write");
    assert!(matches!(
        mint_rev_path(root, "u.typ"),
        Err(Error::NotTracked(_))
    ));
}

#[test]
fn refuses_dirty_working_tree() {
    let dir = repo();
    let root = dir.path();
    std::fs::write(root.join("a.typ"), "v1").expect("write");
    git(root, &["add", "a.typ"]);
    git(root, &["commit", "-q", "-m", "v1"]);
    std::fs::write(root.join("a.typ"), "v2-uncommitted").expect("write");
    assert!(matches!(mint_rev_path(root, "a.typ"), Err(Error::Dirty(_))));
}

#[test]
fn last_touch_ignores_merge_shortcut_to_stale_commit() {
    // Regression: a diamond where the path's stale-content commit is reachable
    // from HEAD by a shorter (merge second-parent) route than its newest change.
    // A distance-ordered walk pops the stale commit first and would flag the
    // clean tree dirty; the last-touch must be the newest commit that changed
    // the path, so a clean tree mints cleanly.
    let dir = repo();
    let root = dir.path();

    // seed (no note.typ yet), then C1 introduces note.typ = v1 (stale content).
    std::fs::write(root.join("seed.typ"), "s").expect("write");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "seed"]);
    std::fs::write(root.join("note.typ"), "v1").expect("write");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "note v1"]);

    // Mainline: C2 sets note.typ = v2 (the true last-touch), then two commits
    // that leave note.typ alone — lengthening the route back to note's history.
    std::fs::write(root.join("note.typ"), "v2").expect("write");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "note v2"]);
    std::fs::write(root.join("other.typ"), "a").expect("write");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "other a"]);
    std::fs::write(root.join("other.typ"), "b").expect("write");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "other b"]);

    // Side branch off C1 (HEAD~3): touch a different file, so note.typ stays v1.
    // One hop from the merge — shorter than the mainline route to note's v1.
    git(root, &["checkout", "-q", "-b", "side", "HEAD~3"]);
    std::fs::write(root.join("side.typ"), "x").expect("write");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "side"]);

    // Merge side into mainline (first parent = mainline). note.typ resolves to
    // v2; the working tree is clean.
    git(root, &["checkout", "-q", "main"]);
    git(root, &["merge", "-q", "--no-ff", "-m", "merge", "side"]);

    let id = mint_rev_path(root, "note.typ").expect("clean tree mints");
    assert_eq!(id.kind, Kind::Revision);
    assert!(resolves(root, &id).expect("resolves"));
}

#[test]
fn resolve_is_false_for_absent_path_at_commit() {
    let dir = repo();
    let root = dir.path();
    std::fs::write(root.join("a.typ"), "x").expect("write");
    git(root, &["add", "a.typ"]);
    git(root, &["commit", "-q", "-m", "a"]);
    let mut id = mint_rev_path(root, "a.typ").expect("mint");
    id.path = Some("missing.typ".to_owned());
    assert!(!resolves(root, &id).expect("resolves"));
}

#[test]
fn mints_content_id_without_commit() {
    let dir = repo();
    let root = dir.path();
    std::fs::write(root.join("a.typ"), "hello").expect("write");
    let id = mint_content(root, "a.typ").expect("mint cnt");
    assert_eq!(id.kind, Kind::Content);
    assert_eq!(id.hash.len(), 40);
}
