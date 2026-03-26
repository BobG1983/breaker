---
name: Infrastructure sentinel patterns for scenario runner
description: godmode breaker, quick_clear layout, chip_selections, initial_chips, and Perfect input — how each is wired and when to use them
type: reference
---

## Sentinel Identifiers

| RON field value | What it does | Source |
|-----------------|--------------|--------|
| `breaker: "godmode"` | Inserts synthetic `BreakerDefinition` with `life_pool: None` — bolt_lost never depletes lives | `lifecycle/mod.rs::bypass_menu_to_playing` |
| `layout: "quick_clear"` | Inserts synthetic 1x1 `NodeLayout` with a single `'S'` cell — clears very quickly | same |

Both sentinels are inserted at `OnEnter(MainMenu)` before `Playing` is entered. They do not exist in the RON registry on disk.

## chip_selections vs initial_chips

These are two different injection mechanisms:

| Field | When triggered | Mechanism |
|-------|---------------|-----------|
| `initial_chips: Some([...])` | At run start (bypass_menu_to_playing) | Dispatches `ChipSelected` messages immediately via `chip_writer` |
| `chip_selections: Some([...])` | At each `ChipSelect` state entry | Consumed one-at-a-time by `auto_skip_chip_select` per node clear |

Use `chip_selections` to test the ChipSelect→TransitionIn path (requires a node clear to trigger).
Use `initial_chips` to pre-load chips before any node starts.

## Perfect Input Strategies

`Perfect(BumpMode)` — position-tracks bolt each frame and injects Bump at computed timing:
- `AlwaysPerfect` — always perfectly timed, guaranteed BumpGrade::Perfect
- `NeverBump` — tracks but never bumps; bolt falls through breaker
- `AlwaysEarly` / `AlwaysLate` / `AlwaysWhiff` — deterministic bad timing
- `Random` — picks a random BumpMode each frame from the RNG seed

Useful combinations:
- `godmode` + `Perfect(AlwaysPerfect)` — sustained infinite play, good for stress testing effects
- `godmode` + `Perfect(NeverBump)` — bolt always falls, good for bolt_lost path testing
- `godmode` + `Perfect(Random)` — varied bump grades, exercises full bump grade distribution

## allow_early_end Semantics

- `allow_early_end: true` (default) — RunEnd state triggers `AppExit::Success`; scenario exits early
- `allow_early_end: false` — RunEnd triggers restart (`MainMenu -> Playing` cycle); only `max_frames` exits

For smoke tests of infrastructure (godmode, quick_clear, Perfect strategies):
- Use `allow_early_end: false` + long `max_frames` to stress the mechanism across multiple runs
- Use `allow_early_end: true` + shorter `max_frames` when testing a single-run path (e.g., NeverBump)

## Smoke Test vs Self-Test Distinction

**Self-test** (invariant violation): uses `expected_violations: Some([...])` + `debug_setup` or `frame_mutations` to intentionally trigger an invariant and assert it fires.

**Smoke test** (infrastructure): uses `expected_violations: None` + `allow_early_end: false`. No debug_setup needed — just runs cleanly for the full frame budget to confirm no crash.

Both categories live in `scenarios/self_tests/`.
