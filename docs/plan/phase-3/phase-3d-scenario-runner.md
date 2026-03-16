# Phase 3d: Scenario Runner

**Goal**: Automated gameplay testing — specify a breaker, layout, and input strategy, run the game, catch runtime failures.

---

## Architecture

Separate workspace member (`scenario-runner/`) with its own RON scenario files. Uses `argh` for CLI.

```
scenario-runner/
├── Cargo.toml          # depends on brickbreaker, argh
├── src/
│   ├── main.rs         # CLI entry point
│   ├── lib.rs          # scenario types, runner lifecycle
│   ├── invariants.rs   # assertion systems
│   └── input.rs        # chaos monkey, scripted sequences
└── scenarios/          # RON scenario files (crate-local, never shipped)
    ├── aegis_chaos.scenario.ron
    ├── prism_stress.scenario.ron
    └── ...
```

---

## CLI

```
cargo dscenario -- -s aegis_chaos              # run one, visual (with window)
cargo dscenario -- -s aegis_chaos --headless   # run one, headless (no window)
cargo dscenario -- --all --headless            # run all, headless (CI)
```

```rust
#[derive(FromArgs)]
/// Scenario runner for brickbreaker
struct Args {
    /// run a specific scenario by name
    #[argh(option, short = 's')]
    scenario: Option<String>,

    /// run without a window (for CI)
    #[argh(switch)]
    headless: bool,

    /// run all scenarios in the scenarios/ directory
    #[argh(switch)]
    all: bool,
}
```

---

## Scenario Format (RON)

```ron
(
    breaker: "aegis",
    layout: "node_03",
    input: Chaos(seed: 42, action_prob: 0.3),
    max_frames: 10000,
    invariants: [BoltInBounds, NoEntityLeaks, NoNaN, ValidStateTransitions],
)
```

---

## How It Works

1. Parse CLI args with argh
2. Build the app:
   - **Visual**: full `DefaultPlugins` with a window — watch the scenario play
   - **Headless**: `DefaultPlugins` with `backends: None`, disable winit, add `ScheduleRunnerPlugin`
3. Add the game's plugin group (minus DebugPlugin)
4. Add a `ScenarioPlugin` which:
   - Loads the scenario RON file
   - Auto-navigates Loading → MainMenu → Playing with the specified breaker and layout
   - Injects inputs each frame based on the configured strategy
   - Runs invariant systems every frame, collecting violations
   - Captures `warn!()`/`error!()` via a custom `tracing::Layer`
   - Exits after `max_frames` with pass/fail exit code

---

## Input Strategies

- **Chaos**: seeded random actions each frame, state-aware (gameplay actions during Playing, menu actions during menus)
- **Scripted**: `Vec<(frame, Vec<GameAction>)>` — deterministic input at specific frames
- **Hybrid**: scripted navigation to Playing, then chaos monkey takes over

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

Custom `tracing::Layer` on `LogPlugin::custom_layer` captures WARN/ERROR from `brickbreaker` targets. Any captured log fails the scenario.

---

## Headless Considerations

Systems that may not work headless (investigate at implementation time):
- Anything reading `Window` or `PrimaryWindow` — guard or mock
- Visual-only Update systems (bump_visual, tilt_visual) — harmless, just animate unseen transforms
- `bevy_egui` (DebugPlugin) — disabled, not a concern
- Asset loading — works headless, `file_path` must point to `game/assets/`

---

## CI Integration

```yaml
scenario-test:
  runs-on: ubuntu-latest
  timeout-minutes: 10
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - name: Run all scenarios headless
      run: cargo dscenario -- --all --headless
      timeout-minutes: 5
```

No GPU needed (`backends: None`). No display server needed (winit disabled).

---

## Checklist

- [ ] Create `scenario-runner/` workspace member with argh dependency
- [ ] `cargo dscenario` alias in `.cargo/config.toml`
- [ ] Scenario RON format (breaker, layout, input strategy, max_frames, invariants)
- [ ] ScenarioPlugin: loader, lifecycle, auto-navigation
- [ ] Visual mode (full window)
- [ ] Headless mode (no winit, backends: None, ScheduleRunnerPlugin)
- [ ] Chaos monkey input strategy (seeded, state-aware)
- [ ] Scripted input strategy
- [ ] Invariant systems (bolt bounds, entity leaks, NaN, state transitions)
- [ ] Custom tracing Layer for WARN/ERROR capture
- [ ] Frame-limited exit with pass/fail exit code
- [ ] 3+ initial scenario RON files
