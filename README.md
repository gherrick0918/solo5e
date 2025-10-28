# solo5e

![Rust CI](https://github.com/gherrick0918/solo5e/actions/workflows/rust-ci.yml/badge.svg)

Portable, deterministic 5e solo engine with a Rust core and multiple front-ends (CLI → Web/WASM → Android).

## Repo layout
- `engine/` — headless rules engine (Rust)
- `cli/` — command-line harness for seeds, rolls, and smoke tests
- `schema/` — JSON Schemas for content files
- `content/` — example content (e.g., characters)
- `.github/workflows/` — CI for fmt, clippy, and tests

## Quick start
```bash
cargo run -p cli -- --help
```

## CLI & JSON

### Dump a sample character (UTF-8, no BOM)
```bash
cargo run -p cli -- actor-dump --out content/characters/fighter.json
```

### Load a character and run three demo checks
```bash
cargo run -p cli -- actor-load --file content/characters/fighter.json --dc 13
```

#### Windows / PowerShell note
PowerShell redirection can write UTF-16 or add a UTF-8 BOM, which breaks JSON parsing. Use the `--out` flag as above, or ensure UTF-8 without BOM. The `actor-load` command is BOM-aware and will accept UTF-8/UTF-16 files.

## Schema
Actor JSON Schema: `schema/actor.schema.json` reflects the serde layout in `engine`:

- `abilities`: `{ str, dex, con, int, wis, cha }` (integers)
- `proficiency_bonus`: integer
- `save_proficiencies`: array of abilities (e.g., `str`, `con`)
- `skill_proficiencies`: array of skills (e.g., `athletics`, `perception`)

Example: `content/characters/sample_fighter.json`.

## Status
- ✅ Toolchain & CI
- ✅ Basic engine: dice, checks, abilities/skills, actor
- ✅ CLI: roll/check/actor-demo, JSON dump/load
- 🚧 Content format, more rules systems, and platform front-ends

---

*This project aims to implement SRD 5.1-compliant systems. Non-SRD content will be kept out of the core.*
### Attack demo with weapon presets

```bash
# default: longsword, STR, proficient
cargo run -p cli -- attack-demo --ac 15 --seed 999

# pick a weapon preset explicitly
cargo run -p cli -- attack-demo --weapon longsword --ac 15 --adv advantage

# finesse defaults to DEX when ability=auto
cargo run -p cli -- attack-demo --weapon shortsword --ac 14

# ranged uses DEX (auto)
cargo run -p cli -- attack-demo --weapon longbow --ac 14

# force STR on a finesse weapon
cargo run -p cli -- attack-demo --weapon dagger --ability str --ac 13

# simulate non-proficiency
cargo run -p cli -- attack-demo --weapon longbow --no-prof --ac 15

# override damage dice explicitly
cargo run -p cli -- attack-demo --weapon longsword --dice 1d10 --ac 15
```

### Attack a target from JSON

```bash
# Single round vs goblin
cargo run -p cli -- attack-vs --target content/targets/goblin.json --weapon longsword

# Multi-round (stop early if target hits 0 HP)
cargo run -p cli -- attack-vs --target content/targets/goblin.json --rounds 5 --weapon longbow --weapons content/weapons/basic.json --adv advantage --seed 999
```


---

## Verify locally

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# demos
cargo run -p cli -- attack-demo --ac 15 --seed 999
cargo run -p cli -- attack-demo --weapon shortsword --ac 14
cargo run -p cli -- attack-demo --weapon longbow --no-prof --ac 15
cargo run -p cli -- attack-demo --weapon dagger --ability str --ac 13
cargo run -p cli -- attack-demo --weapon longsword --dice 1d10 --ac 15
```
