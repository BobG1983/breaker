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
| Run scenarios | `cargo scenario` | **Always use this** — release build, fast |
| Run scenarios (dev) | `cargo dscenario` | **Only** when debugging a bug in the runner itself |
| Type check | `cargo dscheck` | |
| Lint | `cargo dsclippy` | |
| Test | `cargo dstest` | |

`cargo scenario` is the release build and MUST be used for all scenario validation. `cargo dscenario` is a dev build with debug symbols — only use it when you suspect a bug in the scenario runner code itself (not in the game code) and need additional debug output.

### Scenario runner options

All options go after `--`:

```
cargo scenario -- --all              # All scenarios, parallel (default 32 jobs)
cargo scenario -- --all -p 4         # All scenarios, 4 parallel jobs
cargo scenario -- --all -p all       # All scenarios, unlimited parallelism
cargo scenario -- --all --serial     # All scenarios, in-process sequential
cargo scenario -- --all --loop 10    # All scenarios 10 times
cargo scenario -- -s aegis_chaos     # Single scenario, in-process
cargo scenario -- -s prism_scatter -p 10      # Stress test: 10 parallel copies
cargo scenario -- -s prism_scatter -p 10 -l 3 # 30 total: 3 rounds of 10
```

`--serial` and `--parallel` are mutually exclusive.

## Exceptions

- `cargo fmt` — no dev alias; covers the whole workspace
- `cargo build --release` — release CI only; do NOT add `bevy/dynamic_linking` in release builds
