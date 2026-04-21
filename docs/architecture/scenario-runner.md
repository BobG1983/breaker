# Scenario Runner

## What it is

A headless harness that executes game code under controllable input, capturing frame-by-frame state and checking **invariants** — per-frame truth properties — at every tick. Violations surface as concrete failures with the exact frame, entity, and state that tripped them.

It is not a test runner in the unit-test sense. Unit tests exercise one function in isolation against a handwritten assertion. The scenario runner exercises the *whole integrated game* against automated oracles across thousands of frames and many simultaneous subsystems.

## Why it exists

Three distinct purposes, all important:

### 1. Automated exploration of the game state space

Chaos and stress input drive the game through state combinations no unit test (and no human tester) can practically enumerate. Invariants ride along as oracles. Silent bugs that would accumulate over hours of play become immediate localized failures with frame numbers and entity IDs.

Without invariants, "the game didn't crash for 5000 frames" is not a signal. With them, every tick is an assertion.

### 2. Deterministic regression guards derived from real bugs

A chaos run that trips an invariant produces a recording. That recording becomes a **scripted scenario** — an exact replay of the input sequence that caused the failure. Once the bug is fixed, the scripted scenario lives in the suite forever. Future regressions trip immediately instead of waiting for chaos to randomly rediscover them.

This is the force multiplier: every bug caught once becomes a permanent guard. Scenarios accumulate faster than bugs.

### 3. Bug-report instrument for live players

When the game ships, players report bugs in natural language: *"I was playing Chrono, had Siphon and Echo Strike stacked three times, and after the fourth node the game started double-counting damage."* Without automation, developers try to reproduce the state from that description, usually fail, and close the ticket.

With session recording + the scenario runner, the player's session (or a summary of it) becomes a scripted scenario. The runner replays it. Invariants fire at the exact frame something went wrong, producing a concrete diagnostic instead of a guess.

**This is the pipeline the whole system is designed around.** Chaos finds bugs. Scripted scenarios pin them. Player recordings feed the loop. Invariants are the oracles that make all three work.

## Input modes

Scenarios specify an `InputStrategy`:

- **Chaos** — pseudo-random input. Parameters like `action_prob` control density. Used for exploration.
- **Scripted** — explicit action sequences. Used for regression pins and targeted edge-case coverage.
- **Perfect / Early / Late / Whiff / Random** — bump-grade drivers. Force a specific bump outcome every frame. Used to pin timing-specific behavior.
- **Hybrid** — combination strategies. Mix scripted setup with chaos execution, or vice versa.

Any input mode can be combined with **frame mutations** that force game state at specific frames: `InjectProtocol`, `InjectHazardStack`, `SpawnExtraPrimaryBolts`, resource overrides, etc. Mutations turn scripted scenarios into precise edge-case drivers.

## Scenario types

- **Smoke** — minimal scenario, typically `disable_physics: true` with no actions, 500 frames. Verifies the plugin wiring doesn't crash on build.
- **Chaos** — broad exploration with chaos input, 3000–5000 frames, typically a protocol/hazard injected at frame 10.
- **Stress** — `stress: (runs: N, parallelism: M)` in the RON. Runs N copies in parallel. Fails if ANY copy trips a violation. High-N stress amplifies rare-race bug signals.
- **Scripted** — deterministic action sequences. Either hand-authored or derived from a recording.
- **Self-test** — intentionally triggers a specific invariant to verify the checker fires. `expected_violations: Some([InvariantKind::Foo])`. Every invariant should have one.

## Invariants

An invariant is a per-frame property that must always be true in a correctly-functioning game. The runner checks every registered invariant every tick (or every N ticks, depending on the check's cost).

### What makes a good invariant

A good invariant pins a **contract** — a property the game's design says must always hold. Examples of contract shapes:

- **Ownership / lifecycle**: state X only exists while system Y owns it (e.g., `GreedStacks` is zero or absent whenever `ActiveProtocols` doesn't contain `Greed`).
- **State isolation**: state X resets at boundary Y (e.g., `SiphonStreak` clears on node exit).
- **Bounded mutation paths**: value X only changes through function Y (e.g., `BurnoutHeat` stays in `[0, 1]` because `burnout_update_heat` is the sole mutator).
- **Entity integrity**: no dangling references (e.g., `BoundEffects` references point to live entities).
- **Counter monotonicity / reset**: counter X grows then resets on event Y, never goes negative.
- **Conservation / non-leak**: total entities of kind X never exceeds Y.

### What makes a bad invariant

- **Arbitrary numeric bounds**: `counter < 1000` is a magic number, not an invariant. If the counter is unbounded by design, any upper limit produces false positives on legitimate play.
- **Redundant with unit tests**: if a clamp is already unit-tested, the invariant's value comes from catching *future* paths that violate the clamp, not verifying the current clamp works. Still useful, but framed as a regression guard for code that doesn't exist yet.
- **Symptom-specific**: invariants should pin contracts, not reproduce single bugs. A one-off assertion about a specific state combination belongs in a unit test.
- **Checking things the type system already guarantees**: `Option` is not None when set to Some — the compiler already knows.

### Why contracts > clamps

The value of an invariant is what it catches that *nothing else does*. Unit tests catch the clamp. Invariants catch the *next* path that mutates state outside the clamp. That's why ownership/lifecycle invariants are the highest-leverage: they catch state leaks that span features, surface only in long sessions, and never show up in isolated unit tests.

### Self-tests

Every `InvariantKind` must have a self-test scenario that intentionally triggers the violation. Without this, a broken checker (always returns `None`) provides false safety. The self-test harness uses `expected_violations: Some([Kind])` and a `FrameMutation` that forces the violating state.

## The recording-and-replay loop

This is the multiplier that makes everything else worth doing:

1. **Chaos, scripted, or live player input** drives the game.
2. **The runner records** frame-by-frame input + mutations.
3. **An invariant trips** (or a player reports a bug).
4. **The recording becomes a scripted scenario** — either auto-generated or hand-extracted.
5. **Developers replay it locally.** Invariants fire at the exact frame. Diagnosis is concrete.
6. **Fix applied.** The scripted scenario stays as a permanent regression pin.
7. **Next release** — any reintroduction of the bug trips the scenario immediately.

Recording is not just a debug tool — it's the input side of the feedback loop. Without it, chaos finds bugs that then vanish. With it, every bug found is a bug captured.

## Invariant coverage as a release gate

All invariants should have self-tests. All mechanics should have at least one scenario exercising them (chaos or scripted). Coverage parity is verified by `cargo scenario -- --coverage`.

The coverage report is a release-readiness signal: gaps are places where bugs could ship unnoticed because nothing in the suite is watching. A release with gaps is a release where "didn't crash during chaos" is the only oracle — which is the state without invariants, and the state this whole system exists to move beyond.

## What this means for development

- **When fixing a bug**, ask: what contract was violated? Is there an invariant that would have caught it? If not, add one. If yes but no scenario drives to the failing state, add a scripted scenario.
- **When adding a mechanic**, add a chaos scenario (exploration) and — where contracts matter — invariants that ride along in every scenario touching that mechanic. Write the scripted scenarios that drive to the contract edges.
- **When reviewing player bug reports**, first action is "is there a session recording?" If yes, replay it. If no, the bug's resolution cost is dominated by reproducing it manually — which is precisely what this system exists to eliminate.
- **Invariants are the compounding asset.** Scenarios multiply them. Recordings feed them. The suite gets stronger with every bug found, because the bug becomes permanent coverage.

## Related files

- `breaker-scenario-runner/src/types/definitions/invariants.rs` — `InvariantKind` enum and check dispatch.
- `breaker-scenario-runner/src/invariants/checkers/` — one checker per invariant kind.
- `breaker-scenario-runner/scenarios/self_tests/` — one per invariant kind.
- `breaker-scenario-runner/scenarios/stress/` — chaos and stress scenarios.
- `breaker-scenario-runner/scenarios/mechanic/` — smoke and scripted mechanic scenarios.
- `docs/architecture/testing.md` — testing philosophy and tier strategy.
