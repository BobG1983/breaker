---
name: StagedEffects and bridge query patterns (Waves C/D/E/G2)
description: Option<&StagedEffects> in all 6 bridge queries; clone-before-walk; walk_staged_effects early exit; RemoveStagedEffectCommand O(n); watcher nested loop — all confirmed acceptable
type: project
---

## Patterns confirmed acceptable (Waves C/D/E/G2)

### Option<&StagedEffects> archetype fragmentation (concern 1)
All 6 bridge queries widened to `(&BoundEffects, Option<&StagedEffects>)`. This splits BoundEffects carriers into two archetypes (with/without StagedEffects). Primary carriers are 1 Breaker + 1 Bolt + ~50-200 Cells. Two archetypes instead of one is negligible overhead. No action needed until Phase 3 if cells universally carry StagedEffects.

### walk_staged_effects empty-slice early exit (concern 2)
When `staged` is `None`, callers produce `staged.map(|s| s.0.clone()).unwrap_or_default()` — an empty `Vec` with no heap allocation (capacity 0). The `walk_staged_effects` for loop over an empty slice executes zero iterations. Confirmed zero-cost path.

### RemoveStagedEffectCommand O(n) scan+shift (concern 3)
`iter().position()` + `Vec::remove(pos)` inside `Command::apply`. Called only when a staged entry matches and fires — at most a handful of entries per entity per frame. O(n) at n=few is negligible. Not a hot path.

### ArmedFiredParticipants::track HashMap mutation (concern 4)
`entry().or_default().push()` called only when an armed `On` entry fires on a participant. Fires happen on bump events. At 1 bolt + 1 breaker as participants, HashMap contains at most 2 entries per key. No allocation concern.

### reverse_effect loop in evaluate_conditions drain (concern 5)
`commands.reverse_effect(participant, ...)` inside `for participant in tracked` where `tracked` is drained from ArmedFiredParticipants. N = number of tracked participants. At current scale N ≤ 2 (bolt + breaker). Negligible. Watch at Phase 3 if many participants are possible.

### 4 watcher systems with Query<Entity, Added<T>> nested loop (concern 6)
`stamp_spawned_bolts`, `stamp_spawned_breakers`, `stamp_spawned_cells`, `stamp_spawned_walls` — all guard with `registry.entries.is_empty()` early return. Inner loop is over registry entries (few chips at most). `Added<T>` filter means the query only yields entities on their first frame. Spawn events are rare (node start). Zero per-frame cost when no entities spawn. Correct and efficient.

### tick_entropy_engine Option<&EffectSourceChip> (concern 7)
Now a normal Bevy system. `Option<&EffectSourceChip>` on EntropyCounter entities produces 2 archetypes max. At 1-few counter entities this is negligible. Early `bump_count == 0` return keeps per-frame cost near zero between bumps.

### walk_staged_effects O(n) position scan per consumed entry (concern 8)
Same as concern 3 — deferred via `remove_staged_effect` command. Called only on match. n=few. Acceptable.
