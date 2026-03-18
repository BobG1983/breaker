# Cargo Commands

**NEVER** use bare `cargo build`, `cargo check`, `cargo clippy`, or `cargo test` — these produce non-dynamic-linked artifacts that conflict with the dynamic-linked build and cause slow rebuilds.

## Game crate (`breaker-game`)

| Task | Alias |
|------|-------|
| Run | `cargo dev` |
| Build | `cargo dbuild` |
| Type check | `cargo dcheck` |
| Lint | `cargo dclippy` |
| Test | `cargo dtest` |

## Scenario runner (`breaker-scenario-runner`)

| Task | Alias | When |
|------|-------|------|
| Run scenarios | `cargo scenario` | Normal use — release build, fast and quiet |
| Run scenarios (dev) | `cargo dscenario` | Developing/debugging the runner itself |
| Type check | `cargo dscheck` | |
| Lint | `cargo dsclippy` | |
| Test | `cargo dstest` | |

## Exceptions

- `cargo fmt` — no dev alias; covers the whole workspace
- `cargo build --release` — release CI only; do NOT add `bevy/dynamic_linking` in release builds
