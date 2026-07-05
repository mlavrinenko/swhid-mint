# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-07-05

### Added

- `Swhid`/`Kind`: mint/parse/render SWHIDs (ISO/IEC 18670:2025), VCS-agnostic
  core with a `path` qualifier.
- `git` backend (default feature, via `gix`): mint revision- and
  content-scoped SWHIDs from a working tree, and check whether an id still
  resolves.
- `test-support` feature: shared git fixtures for downstream tests.
