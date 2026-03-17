# Phase 3d: Scenario Runner

**Goal**: Automated gameplay testing — specify a breaker, layout, and input strategy, run the game, catch runtime failures.

---

## Architecture

Separate workspace member (`breaker-scenario-runner/`) with its own RON scenario files. Uses `argh` for CLI.

```
breaker-scenario-runner/
├── Cargo.toml          # depends on breaker (game lib), argh, ron, serde, rand
├── src/
│   ├── main.rs         # CLI entry point
│   ├── lib.rs          # module declarations + re-exports
│   ├── types.rs        # ScenarioDefinition, InputStrategy, InvariantKind, etc.
│   ├── lifecycle.rs    # ScenarioLifecycle plugin, ScenarioConfig, frame counter
│   ├── invariants.rs   # assertion systems, ViolationLog
│   ├── input.rs        # ChaosMonkey, ScriptedInput strategies
│   ├── log_capture.rs  # tracing Layer for WARN/ERROR capture
│   └── runner.rs       # App construction, multi-scenario execution, evaluate_pass
└── scenarios/          # RON scenario files (crate-local, never shipped)
    ├── aegis_chaos.scenario.ron
    ├── aegis_bolt_stress.scenario.ron
    ├── chrono_scripted.scenario.ron
    ├── prism_stress.scenario.ron
    ├── self_tests/
    │   ├── bolt_oob_detection.scenario.ron
    │   ├── breaker_oob_detection.scenario.ron
    │   └── nan_detection.scenario.ron
    └── regressions/    # empty — populated as regressions are caught
```

---

## CLI

```
cargo dscenario -- -s aegis_chaos              # run one, visual (with window)
cargo dscenario -- -s aegis_chaos --headless   # run one, headless (no window)
cargo dscenario -- --all --headless            # run all, headless (CI)
```

---

## Scenario Format (RON)

```ron
(
    breaker: "aegis",
    layout: "corridor",
    input: Chaos((seed: 42, action_prob: 0.3)),
    max_frames: 10000,
    invariants: [BoltInBounds, NoEntityLeaks, NoNaN, ValidStateTransitions],
    expected_violations: None,
    debug_setup: None,
)
```

`expected_violations: Some([BoltInBounds])` — used in self-test scenarios to assert the invariant checker fires.

---

## How It Works

1. Parse CLI args with argh
2. Build the app:
   - **Visual**: full `DefaultPlugins` with a window — watch the scenario play
   - **Headless**: `DefaultPlugins` with winit disabled + `ScheduleRunnerPlugin`
3. Add the game's `Game` plugin group (DebugPlugin is not added by Game when feature = "dev" is absent)
4. Add `ScenarioLifecycle` plugin which:
   - Loads the scenario RON file via `ScenarioConfig` resource
   - Auto-navigates `MainMenu` → `Playing` with the specified breaker and layout override
   - Auto-skips `ChipSelect` → `NodeTransition`
   - Counts fixed-update frames via `ScenarioFrame`
   - Exits when `max_frames` is reached or `RunEnd` state is entered
5. Invariant systems run each FixedUpdate, appending to `ViolationLog`
6. Custom `tracing::Layer` captures WARN/ERROR from `breaker` targets into `CapturedLogs`
7. `evaluate_pass` checks violations and logs against `expected_violations` — returns pass/fail

---

## Input Strategies

- **Chaos**: seeded random actions each frame, state-aware (gameplay actions during Playing, menu actions during menus)
- **Scripted**: `Vec<ScriptedFrame>` — deterministic input at specific frames
- **Hybrid**: scripted actions up to `scripted_frames`, then chaos monkey takes over

---

## Invariant Systems

Collect violations without panicking (report all at end):

- **BoltInBounds** — bolt position within playfield (with margin)
- **BreakerInBounds** — breaker within horizontal bounds
- **NoEntityLeaks** — entity count doesn't grow unbounded
- **NoNaN** — no NaN in any Transform
- **ValidStateTransitions** — no impossible state jumps

---

## Log Capture

Custom `tracing::Layer` on `LogPlugin::custom_layer` captures WARN/ERROR from `breaker` and `breaker_scenario_runner` targets. Any captured log fails the scenario (unless it matches `expected_violations`).

---

## CI Integration

`.github/workflows/ci.yml` has two parallel jobs:
- `test` — fmt + clippy + tests on Linux/macOS/Windows
- `scenarios` — headless scenario runner on Linux (`cargo run -p breaker_scenario_runner -- --all --headless`)

---

## Checklist

- [x] Create `breaker-scenario-runner/` workspace member with argh dependency
- [x] `cargo dscenario` alias in `.cargo/config.toml`
- [x] Scenario RON format (breaker, layout, input strategy, max_frames, invariants, expected_violations, debug_setup)
- [x] ScenarioLifecycle plugin: loader, lifecycle, auto-navigation, frame counter
- [x] Visual mode (full window)
- [x] Headless mode (no winit, ScheduleRunnerPlugin)
- [x] Chaos monkey input strategy (seeded, state-aware)
- [x] Scripted input strategy
- [x] Hybrid input strategy (scripted then chaos)
- [x] Invariant systems (bolt bounds, breaker bounds, entity leaks, NaN, state transitions)
- [x] Custom tracing Layer for WARN/ERROR capture
- [x] Frame-limited exit with pass/fail exit code
- [x] 4 stress scenario RON files + 3 self-test scenarios
- [x] CI workflow (`.github/workflows/ci.yml`) with test (3 platforms) + scenarios (headless) jobs
- [x] `NodeLayoutRegistry::get_by_name()` + `ScenarioLayoutOverride` resource added to main crate
- [x] `expected_violations` field for self-test scenarios that intentionally trigger invariants
- [x] Input recording sub-domain (`debug/recording/`) + `--record` CLI flag for capturing scripted inputs
