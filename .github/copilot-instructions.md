# Solo5e Copilot Instructions

## Architecture Overview

This is a **Rust-based D&D 5e solo combat engine** with a deterministic, seed-driven design pattern:

- **`engine/`**: Headless rules engine with pure functions for dice rolling, ability checks, attacks, and damage
- **`cli/`**: Command-line harness for testing engine functionality with reproducible results via seeds
- **`schema/`**: JSON Schema definitions that match Rust serde layouts exactly
- **`content/`**: Example JSON data files for characters, weapons, and targets

## Key Design Patterns

### Deterministic RNG System
All randomness uses `ChaCha8Rng` with explicit seeds. The `Dice` struct owns the RNG state:
```rust
let mut dice = Dice::from_seed(42);
let roll = dice.d20(AdMode::Normal);
```

### Typed Enums for Game Mechanics
- `Ability` enum maps to JSON strings: `"str"`, `"dex"`, etc.
- `Skill` enum uses snake_case: `"sleight_of_hand"`, `"animal_handling"`
- `AdMode` for advantage/disadvantage: `Normal`, `Advantage`, `Disadvantage`

### JSON-First Data Contract
All game objects serialize/deserialize via serde with specific conventions:
- Abilities use short names: `"str"`, `"int"` (not `"strength"`, `"intelligence"`)
- Collections use `HashSet` for uniqueness (proficiencies, resistances)
- The engine matches JSON Schema exactly—see `schema/actor.schema.json`

### Attack Resolution Pipeline
1. **Attack roll**: `attack()` function handles nat20/nat1 edge cases
2. **Damage calculation**: `damage()` function with critical hit doubling
3. **Damage type adjustment**: resistance/vulnerability/immunity via `adjust_damage_by_type()`

## Development Workflow

### Core Commands
```powershell
# Format, lint, test (CI requirements)
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Quick engine demos
cargo run -p cli -- actor-demo --ac 15 --seed 999
cargo run -p cli -- attack-demo --weapon longsword --ac 15 --adv advantage
cargo run -p cli -- attack-vs --target content/targets/goblin.json --rounds 5
```

### CLI Architecture Patterns
- **Subcommands**: Each major engine feature has a CLI wrapper (roll, check, attack-demo, etc.)
- **Seed consistency**: All commands accept `--seed` for reproducible results
- **JSON I/O**: Use `--out` for UTF-8 output (avoid PowerShell UTF-16/BOM issues)
- **File loading**: CLI handles BOM detection for cross-platform JSON files

### Testing Strategy
- **Property-based tests**: Uses `proptest` for edge case discovery
- **Snapshot testing**: Uses `insta` for regression testing of complex outputs  
- **Deterministic assertions**: All tests use fixed seeds for consistent results

## Content Management

### Weapon System
Weapons support finesse (DEX/STR choice), ranged attacks, and versatile dice:
```json
{
  "name": "longsword",
  "dice": {"count": 1, "sides": 8},
  "versatile": {"count": 1, "sides": 10},
  "finesse": false,
  "damage_type": "slashing"
}
```

### Actor Proficiencies
Characters have separate proficiency arrays for saves and skills:
```json
{
  "save_proficiencies": ["str", "con"],
  "skill_proficiencies": ["athletics", "perception"]
}
```

## Cross-Platform Considerations

- **Windows**: PowerShell redirection adds BOM—use CLI `--out` flag instead
- **File encoding**: CLI accepts UTF-8 and UTF-16 files with BOM detection
- **Path handling**: Use `PathBuf` and absolute paths in tools
- **CI coverage**: Tests run on Ubuntu + Windows to catch platform issues

## SRD Compliance

This project implements **SRD 5.1-compliant systems only**. Non-SRD content stays out of the engine core. When adding new mechanics, verify against the official SRD before implementation.