# Siphon

Assumes: #2 (`mutators/protocols/siphon/`), #4 (bolt-loss behavior — `ReverseTimePenalty` renamed to `IncreaseNodeTimer { delta }`, field `seconds` → `delta`).

## What's broken

`SiphonStreak.kill_count` is tracked and incremented but never consumed by gameplay. Each kill adds a FLAT `time_per_kill` regardless of streak length. The design intent for chains/streaks is that longer chains reward more — but today kill 2 and kill 20 both add the same flat amount. The `kill_count` field has no gameplay purpose.

## Fix

### Escalating reward formula

Kill N within the streak window adds `time_per_kill * (N - 1)` seconds:

| Kill | `kill_count` after | Time added |
|------|--------------------|------------|
| 1    | 1                  | 0 (streak starts) |
| 2    | 2                  | `time_per_kill * 1` |
| 3    | 3                  | `time_per_kill * 2` |
| 4    | 4                  | `time_per_kill * 3` |
| N    | N                  | `time_per_kill * (N - 1)` |

First kill still adds no time (design §Expected Behavior 1 unchanged). Kill 2 matches the original flat value, preserving the base experience. Kills 3+ escalate linearly.

### Code change

`mutators/protocols/siphon/system.rs` — `siphon_on_cell_destroyed`:

```rust
// Before (flat)
penalty_writer.write(ReverseTimePenalty {
    seconds: config.time_per_kill,
});

// After (scaled, using the #4-renamed message)
let delta = config.time_per_kill * (streak.kill_count - 1) as f32;
writer.write(IncreaseNodeTimer { delta });
```

Both renames from #4 apply:
- Message type `ReverseTimePenalty` → `IncreaseNodeTimer`
- Field `seconds` → `delta`

`kill_count` is incremented BEFORE this write (confirmed in the existing code — it's already at the correct line). At the second kill `kill_count == 2` so multiplier `= 1.0` → delta `= time_per_kill` (matches old flat behavior for kill 2). At the third kill `kill_count == 3` → multiplier `2.0` → delta `= 2 * time_per_kill`.

The first-kill branch stays as-is: no message write, just `kill_count = 1`.

### Design-doc update

`docs/design/protocols/siphon.md`:

§Game Design — replace:

> Each subsequent kill within the window: +0.5s added to node timer. Window resets to 2s.

with:

> Each subsequent kill within the window adds time that SCALES with streak length: kill N contributes `time_per_kill * (N - 1)` seconds. Longer chains feel progressively more valuable. Window resets to `streak_window` on each kill.

§Expected Behaviors 2 and 3 — rewrite to show the escalating values with concrete numbers (kill 2 → `time_per_kill * 1`, kill 3 → `time_per_kill * 2`, etc.).

§Expected Behaviors 7 (same-frame multi-kill) — "Three cells destroyed in same frame: first starts streak (no time), second adds `time_per_kill * 1`, third adds `time_per_kill * 2`."

## Tests

`mutators/protocols/siphon/tests/escalating_reward.rs`:

1. **`kill_sequence_produces_linear_schedule`** — activate Siphon with `time_per_kill = 0.5`, `streak_window = 2.0`. Drive 5 `Destroyed<Cell>` messages back-to-back (all within the window). Collect emitted `IncreaseNodeTimer` messages. Assert sequence is `[0.5, 1.0, 1.5, 2.0]` (4 messages — first kill emits none). Assert final `streak.kill_count == 5`.
2. **`first_kill_emits_no_message`** — activate Siphon with empty streak state. Drive one `Destroyed<Cell>`. Assert zero `IncreaseNodeTimer` messages; `streak.kill_count == 1`.
3. **`window_expiry_resets_chain_to_one`** — drive two kills quickly (`kill_count` reaches 2); advance time past `streak_window`; drive another kill. Assert the post-expiry kill emits zero messages (starts a new streak at `kill_count == 1`).
4. **`same_frame_multi_kill_uses_sequential_counts`** — in one FixedUpdate tick, emit 3 `Destroyed<Cell>` messages. Tick. Assert two `IncreaseNodeTimer` messages emitted: `[0.5, 1.0]` (kill 2 and kill 3). Assert `streak.kill_count == 3`.

### Tests to REWRITE

Existing `tests/on_cell_destroyed.rs` pins specific `ReverseTimePenalty.seconds` values under the old flat formula. Every assertion about kills 3+ changes under the new formula:

| Old (flat) | New (scaled) |
|------------|--------------|
| Kill 2: `0.5` | `0.5` (unchanged) |
| Kill 3: `0.5` | `1.0` |
| Kill 4: `0.5` | `1.5` |
| Same-frame 3 kills: `0.5, 0.5` total `1.0` | `0.5, 1.0` total `1.5` |

Also rename `ReverseTimePenalty` → `IncreaseNodeTimer` and `seconds` → `delta` in every assertion (per #4's rename, which should be landed before this TODO).

## Code changes summary

| File | Change |
|------|--------|
| `mutators/protocols/siphon/system.rs` | Change `seconds: config.time_per_kill` to `delta: config.time_per_kill * (streak.kill_count - 1) as f32`; rename `ReverseTimePenalty` → `IncreaseNodeTimer` (already renamed by #4 — just use the new name) |
| `mutators/protocols/siphon/tests/escalating_reward.rs` | NEW — tests 1-4 |
| `mutators/protocols/siphon/tests/on_cell_destroyed.rs` | REWRITE assertions to match scaled formula |
| `docs/design/protocols/siphon.md` | Design-doc updates per Fix section |

## Out of scope

- HUD indicator showing `Streak: N (next kill +Xs)` — Phase 5q HUD work. Escalating reward makes the indicator high-value but it is not required for the mechanic itself.
- Changing `streak_window` semantics — unchanged; resets per kill as before.
- First-kill-in-node-adds-time variant — out of scope; first kill always adds zero.
