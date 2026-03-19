# Phase 4a: Seeded RNG & Run Seed

**Goal**: Every source of randomness in the game flows through a seeded, deterministic RNG. User-selectable seed on the RunSetup screen.

**Wave**: 1 (foundation, no dependencies) — parallel with 4b. **Session 1.**

## What Exists

- `GameRng` resource with `ChaCha8Rng` in `shared/mod.rs`
- `GameRng::from_seed(u64)` constructor
- `reset_run_state` reseeds with OS entropy at run start
- RunSetup screen exists with breaker selection

## What to Build

### Seed Input on RunSetup Screen
- Text input field for seed entry (numeric or string hash)
- "Random" button / empty field = OS entropy seed
- Display the active seed so the player can share it
- Seed stored in a `RunSeed(u64)` resource, set before transitioning to `Playing`

### Seed Propagation
- `reset_run_state` must use `RunSeed` instead of OS entropy
- `GameRng` reseeded from `RunSeed` at run start
- All future systems that need randomness take `&mut GameRng` — no `thread_rng()` or `random()` calls

### Scenario Runner Integration
- Scenarios can specify a seed in RON: `seed: Some(42)`
- Default: deterministic test seed (0)

## Scenario Coverage

- **New invariant**: `DeterministicSeed` — run two identical scenarios (same seed, same scripted input) and verify identical frame-by-frame state. This is a meta-invariant: rather than checking per-frame, it compares two runs post-hoc. Consider implementing as a scenario runner feature rather than a per-frame invariant.
- **Existing scenarios**: All existing chaos scenarios should be updated to use explicit seeds (instead of default 0) to verify seeded RNG propagation works.
- **New scenario**: `mechanic/seeded_determinism.scenario.ron` — scripted input, fixed seed, verifies the run produces expected state at key frames.

## Acceptance Criteria

1. Same seed + same breaker + same inputs = identical run (node sequence, chip offerings, everything)
2. Different seeds produce different runs
3. Seed is visible on RunSetup screen and can be entered manually
4. Existing tests pass — `GameRng::from_seed` behavior unchanged
