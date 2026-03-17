# Phase 3a: Workspace Restructure

**Goal**: Convert to Axum-style workspace with all crates as peer directories under the root.

---

## Target Layout

```
brickbreaker/
├── Cargo.toml              # [workspace] only — no [package]
├── .cargo/
├── .claude/
├── docs/
├── game/                   # [package] name = "brickbreaker"
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   └── main.rs
│   └── assets/
├── derive/                 # [package] name = "brickbreaker_derive"
│   ├── Cargo.toml
│   └── src/
└── scenario-runner/        # added in 3d (empty placeholder or created then)
```

## What Moves

| From | To |
|------|----|
| `src/` | `game/src/` |
| `assets/` | `game/assets/` |
| `brickbreaker_derive/` | `derive/` |
| `Cargo.toml` [package] section | `game/Cargo.toml` |

## What Updates

- Root `Cargo.toml` — becomes `[workspace]` only, members = `["game", "derive"]`
- `.cargo/config.toml` — aliases use `workspace.default-members` or explicit `-p brickbreaker`
- `CLAUDE.md` — build commands
- `docs/architecture/plugins.md` — folder tree
- CI workflows — paths
- `.claude/` config — working directory references

---

## Checklist

- [x] Create `breaker-game/` directory, move `src/` and `assets/` into it
- [x] Create `breaker-game/Cargo.toml` with game's [package] + [dependencies]
- [x] Rename `brickbreaker_derive/` to `breaker-derive/`
- [x] Root `Cargo.toml` becomes workspace-only
- [x] Update `.cargo/config.toml` aliases
- [x] Update `CLAUDE.md`
- [x] Update architecture docs (plugins.md, standards.md)
- [x] Verify all tests pass (`cargo dtest`)
- [x] Verify `cargo dev` still works

> **Note**: Final directory names follow the `breaker-<name>` convention (`breaker-game/`, `breaker-derive/`) rather than the original plan's `game/`, `derive/`.
