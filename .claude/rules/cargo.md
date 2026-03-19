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
```

`--serial` and `--parallel` are mutually exclusive.

### Stress testing via RON

Scenarios can declare a `stress` field to automatically run multiple copies under `--all`:

```ron
stress: Some(()),                           // 32 runs, 32 parallelism (defaults)
stress: Some((runs: 64)),                   // 64 runs, 32 parallelism
stress: Some((runs: 64, parallelism: 16)),  // explicit both
```

When `stress` is `Some(...)`, `cargo scenario -- --all` spawns multiple subprocess copies and aggregates results. A stress scenario passes only if ALL copies pass. Use `-s name` to run a stress scenario individually (it still spawns copies). See `scenarios/stress/breaker_oob_stress.scenario.ron` for an example.

## Exceptions

- `cargo fmt` — no dev alias; covers the whole workspace
- `cargo build --release` — release CI only; do NOT add `bevy/dynamic_linking` in release builds
