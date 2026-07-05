# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-07-06

### Fixed

- `mint_rev_path`: the last-touch commit search relied on `gix`'s default
  breadth-first ancestor walk, which orders by graph distance rather than
  history. In a diamond (two merged branches where the file's true last change
  sits further from `HEAD` than a stale copy reachable via the other parent),
  the walk could return the stale commit's blob and falsely report a clean
  working tree as `Dirty`. The dirty check now compares the working blob
  directly against `HEAD`'s recorded blob (matching `git status` semantics),
  and the last-touch search is anchored to that confirmed-current blob instead
  of walk order, so it is immune to traversal ordering.

## [0.1.0] - 2026-07-05

### Added

- `Swhid`/`Kind`: mint/parse/render SWHIDs (ISO/IEC 18670:2025), VCS-agnostic
  core with a `path` qualifier.
- `git` backend (default feature, via `gix`): mint revision- and
  content-scoped SWHIDs from a working tree, and check whether an id still
  resolves.
- `test-support` feature: shared git fixtures for downstream tests.
