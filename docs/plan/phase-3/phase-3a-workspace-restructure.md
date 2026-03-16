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

- [ ] Create `game/` directory, move `src/` and `assets/` into it
- [ ] Create `game/Cargo.toml` with game's [package] + [dependencies]
- [ ] Rename `brickbreaker_derive/` to `derive/`
- [ ] Root `Cargo.toml` becomes workspace-only
- [ ] Update `.cargo/config.toml` aliases
- [ ] Update `CLAUDE.md`
- [ ] Update architecture docs (plugins.md, standards.md)
- [ ] Verify all tests pass (`cargo dtest`)
- [ ] Verify `cargo dev` still works
