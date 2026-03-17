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
├── breaker-game/           # [package] name = "breaker" (lib) / "brickbreaker" (bin)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   └── main.rs
│   └── assets/
├── breaker-derive/         # [package] name = "brickbreaker_derive"
│   ├── Cargo.toml
│   └── src/
└── breaker-scenario-runner/ # added in 3d
```

## What Moves

| From | To |
|------|----|
| `src/` | `breaker-game/src/` |
| `assets/` | `breaker-game/assets/` |
| `brickbreaker_derive/` | `breaker-derive/` |
| `Cargo.toml` [package] section | `breaker-game/Cargo.toml` |

## What Updates

- Root `Cargo.toml` — becomes `[workspace]` only, members = `["breaker-game", "breaker-derive", "breaker-scenario-runner"]`
- `.cargo/config.toml` — aliases use workspace members
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
