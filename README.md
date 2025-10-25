# solo5e

![Rust CI](https://github.com/gherrick0918/solo5e/actions/workflows/rust-ci.yml/badge.svg)

Portable, deterministic 5e solo engine with a Rust core and multiple front-ends (CLI → Web/WASM → Android).

## Repo layout
- `engine/` — headless rules engine (Rust)
- `cli/` — command-line harness for seeds, rolls, and smoke tests
- `.github/workflows/` — CI for fmt, clippy, and tests

## Quick start
```bash
cargo run -p cli -- --help
```

## Status
- ✅ Toolchain set up
- ✅ CI in PR (#3)
- 🚧 Engine scaffolding and CLI features coming next

---

*This project aims to implement SRD 5.1-compliant systems. Non-SRD content will be kept out of the core.*
