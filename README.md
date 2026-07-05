# swhid-mint

[![CI](https://github.com/mlavrinenko/swhid-mint/actions/workflows/ci.yml/badge.svg)](https://github.com/mlavrinenko/swhid-mint/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/swhid-mint.svg)](https://crates.io/crates/swhid-mint)
[![License: MIT](https://img.shields.io/crates/l/swhid-mint.svg)](LICENSE-MIT)

Mint and resolve Software Heritage identifiers (SWHID, ISO/IEC 18670:2025). VCS-agnostic core; a git backend derives ids via gix

## Install

```bash
cargo add swhid-mint
```

## Usage

```rust
use swhid_mint::{Kind, Swhid};

let text = "swh:1:rev:0000000000000000000000000000000000000000;path=a/b.typ";
let id = Swhid::parse(text).expect("valid");
assert_eq!(id.kind, Kind::Revision);
assert_eq!(id.path.as_deref(), Some("a/b.typ"));
assert_eq!(id.render(), text);
```

The core (`Swhid::render`/`Swhid::parse`) is VCS-agnostic. Deriving an id from a
working tree, or checking one still resolves, lives behind the `git` backend
feature (on by default, via the `gix` crate).

## Development

Prerequisites: [Nix](https://nixos.org/) with flakes enabled.

```bash
direnv allow         # or: nix develop

just check           # fmt + clippy + tests + file-size + drift check
just build
just test
just cover           # code coverage (70% minimum)
just fmt             # format code
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for coding conventions.

## License

MIT
