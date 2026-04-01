---
name: violation-log-output-trace
description: End-to-end trace of ViolationLog entry flow from checker push through to stdout, including verbose flag path
type: project
---

# ViolationLog Output Flow Trace

Traced on `feature/scenario-coverage`, Bevy 0.18.

## Key finding: --verbose flag exists and prints full ViolationEntry.message

`cargo scenario -- -s <name> -v` (or `-v`/`--verbose`) is the switch to see
actual speed values, entity IDs, and bounds from BoltSpeedInRange violations.

## Data types

- `ViolationEntry` (in `breaker-scenario-runner/src/invariants/types.rs`):
  - `frame: u32` — FixedUpdate frame
  - `invariant: InvariantKind`
  - `entity: Option<Entity>`
  - `message: String` — human-readable with concrete values (speed, bounds, mult)

- `ViolationLog(Vec<ViolationEntry>)` — Bevy Resource, accumulated during run.

## How message is constructed (BoltSpeedInRange example)

`check_bolt_speed_in_range` in `breaker-scenario-runner/src/invariants/checkers/bolt_speed_in_range.rs`:
```
message: format!(
    "BoltSpeedInRange FAIL frame={} entity={entity:?} speed={speed:.1} bounds=[{:.1}, {:.1}] mult={mult:.2}",
    frame.0, effective_min, effective_max,
)
```
Contains: frame, entity ID, actual speed, effective min/max bounds, multiplier.

## Evaluation: ViolationEntry.message is NOT in compact output

`collect_and_evaluate` in `breaker-scenario-runner/src/runner/app.rs` calls
`ScenarioVerdict::evaluate()` which produces generic reasons from
`InvariantKind::fail_reason()` — e.g. "bolt speed outside configured min/max".
This string has NO concrete values — no speed, no bounds.

In compact mode (`print_compact_failures` in `src/runner/output.rs`):
- Groups violations by `InvariantKind`, prints count + frame range only
- Does NOT print `ViolationEntry.message` at all

Example compact output:
```
  BoltSpeedInRange               x47    frames 15..892
```

## Verbose mode: full ViolationEntry.message IS printed

`print_verbose_failures` in `src/runner/output.rs` (called when `--verbose`/`-v`):
```rust
for v in violations {
    println!(
        "  VIOLATION frame={} {:?} entity={:?}: {}",
        v.frame, v.invariant, v.entity, v.message
    );
}
```
This prints the full `message` field — actual speed, bounds, mult, entity ID.

## How to enable

Single scenario:
```
cargo scenario -- -s <scenario_name> -v
```
All scenarios:
```
cargo scenario -- --all -v
```
Serial (all in-process, full output without subprocess buffering):
```
cargo scenario -- --all --serial -v
```

## Parallel mode note

In parallel mode (`run_all_parallel`), each scenario is a subprocess. The parent
process captures stdout/stderr and reprints them indented under `[scenario_name]`.
The `-v` flag is forwarded to each subprocess via `cmd.arg("-v")` in `spawn_batched`.
So verbose output still works in parallel mode, but it appears after ALL batch processes
complete, interleaved in scenario order.

## Why: stale memory note (updated 2026-03-31)

Prior memory entry `scenario-failure-trace.md` said BoltSpeedInRange checker was missing
`EffectiveSpeedMultiplier`. The checker was subsequently renamed to `bolt_speed_accurate.rs`
and rewritten to use `ActiveSpeedBoosts::multiplier()` + `BaseSpeed`/`MinSpeed`/`MaxSpeed`
from `rantzsoft_spatial2d`. `EffectiveSpeedMultiplier` no longer exists (Effective* cache removal).
The checker now verifies exact speed `(base * mult).clamp(min, max)` rather than a range check.
