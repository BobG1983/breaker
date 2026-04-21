# Run Recording and Replay — Investigation

## Goal

Close the recording side of the recording-and-replay loop described in `docs/architecture/scenario-runner.md`. The scenario runner already *executes* scripted scenarios. The game has no counterpart that *produces* them. Today, bugs found in live play cannot be turned into deterministic regression pins without a developer manually hand-reconstructing the state.

## Status

**Phase 1 complete.** Runner-side chaos auto-capture is implemented: `collect_and_evaluate` in `breaker-scenario-runner/src/runner/app/evaluate.rs` calls `write_chaos_regression` on any chaos scenario violation, writing a deterministic scripted replay to `scenarios/regressions/<timestamp>-chaos-<name>.scenario.ron`. Phase 2 (in-game dev-mode recorder) design questions remain open.

## Motivating scenarios

1. **Crash in dev/playtest**: Write `/tmp/<gamename>/<gameversion>/<timestamp>-crash.log` plus `<timestamp>-crash.proposed.scenario.ron` alongside it. The scenario file seeds the state (breaker, node, chips, protocols, hazards) and replays the input leading up to the crash. Developer opens the RON, trims/edits as needed, commits as a regression pin.
2. **Failed chaos scenario**: When the scenario runner hits a violation, automatically emit the recorded input + state so the exact chaos seed that tripped the invariant becomes a scripted scenario. Turns every chaos find into a deterministic pin.
3. **Shipped-game bug report**: Player's session (or a summary) becomes a scenario. Invariants replay against it to localize the bug.

Goals 1 and 2 are dev-time; goal 3 is release-time and likely a later phase.

## Open design questions

### 1. What granularity gets recorded?

**Option A — full-run log.** Every input event, every chip pick, every RNG call. Faithful but large; a long run might be MB per session.

**Option B — state snapshots + diff input.** Snapshot at node boundaries (breaker/chip/protocol/hazard state), then log inputs within the node. Much smaller; relies on determinism to replay.

**Option C — "slice to crash point."** Snapshot lets the runner inject the state directly at the failure frame; no need to replay the whole run. Requires reliable snapshot serialization.

**Option D — hybrid.** Snapshot at node start (A) + input log within the current node (B). Crash scenario seeds the node-boundary state then replays input. Minimizes file size while keeping determinism cheap.

Provisional lean: **D**. Lets the runner skip straight to the failing node, which was the user's instinct ("scenario.ron is the breaker, the node, the chips, protocols, hazards... and then the input").

### 2. Dev-mode-only or always-on?

Recording is cheap if it's an in-memory ring buffer. Writing the buffer to disk only on crash or on explicit save. That makes always-on reasonable.

Gates to consider: feature flag, debug-console toggle, environment variable. Decision deferred until the buffer's memory footprint is measured.

### 3. Does the runner need to snapshot *exactly* the game's internal state?

Trade-off: snapshot everything the scenario runner would need to resume from frame N, vs. snapshot a smaller canonical state (breaker id, node id, chips, protocols, hazards) and let the runner rebuild from scratch using the game's own node-start construction path.

Latter is simpler and matches the existing mutation vocabulary (`InjectProtocol`, `InjectHazardStack`, `InjectChip`, etc.). Former is heavier but more robust to game-code drift.

Provisional lean: **latter** — reuse `FrameMutation` as the snapshot vocabulary. If the canonical-state approach turns out lossy, revisit.

### 4. How does RNG reproduce?

Either the recording captures the seed used and the runner plays with the same seed, or every RNG call is logged explicitly. The former depends on every system using the same seeded RNG and nothing else. Needs `researcher-system-dependencies` sweep to verify.

### 5. Is chaos-failure auto-capture in the runner a separate concern?

Goal 2 doesn't need in-game recording — the runner already knows the input sequence it's driving. On failure, it could just dump its own input buffer to `scenarios/regressions/<timestamp>-chaos-<name>.scenario.ron`.

This is low-hanging and probably the **first** deliverable: "on chaos failure, dump the recorded chaos input as a scripted scenario." That alone closes part of the loop and validates the format before the in-game recorder is built.

### 6. Minimal repro and `--slice`?

The user suggested binary-search over a failing scenario's input to produce a minimal repro. Property-testing territory (`proptest`, `quickcheck`). Evaluate: is it worth replicating shrinking logic, or should the runner just integrate `proptest` for chaos?

Not urgent. Out of scope for the first recorder delivery.

## Recommended phasing

1. **Phase 1 — Runner-side chaos auto-capture.** On a chaos scenario violation, the runner dumps its current frame-level input buffer to `scenarios/regressions/<timestamp>-chaos-<name>.scenario.ron` with the original InjectProtocol/InjectHazard mutations preserved. Zero new game-side code. This validates the `RecordedInput` format against the existing scenario schema.
2. **Phase 2 — In-game dev-mode recorder.** Ring-buffer of inputs + state snapshots at node boundaries. Write on crash (panic hook) and on explicit debug-console `/record save` command. Uses the mutation vocabulary as snapshot output.
3. **Phase 3 — Release-mode recorder + bug-report ingestion.** Write to user-writable save location. Optional: serialize + upload with bug reports.
4. **Phase 4 (optional) — Shrinking / minimal repro.** Evaluate `proptest` integration vs. hand-rolled binary search over the recorded input.

## Acceptance (for Phase 1 alone)

- Chaos scenario failure produces a replayable regression scenario on disk
- That regression scenario reproduces the violation deterministically when replayed
- Format matches existing `.scenario.ron` schema

## Risk notes

- RNG determinism is load-bearing. If it's not already solid, fix that before building this.
- Format churn: the scenario schema and mutation vocabulary are still evolving. Recording-side code will need to track schema changes.
- "Full log" temptation: if we capture every input every frame, file sizes grow fast on long runs. Prefer the snapshot-plus-input-since-snapshot shape.

## Out of scope

- Shipping the recorder to release builds (revisit after Phase 2 proves the in-dev path)
- Automated bug-report submission UI
- Property-test integration (Phase 4, optional)
