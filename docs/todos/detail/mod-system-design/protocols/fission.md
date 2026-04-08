# Protocol: Fission

## Category
`custom-system`

## Game Design
You WANT to maximize destruction volume for bolt splits.

- Every 8th cell destroyed permanently splits one bolt into two.
- New bolt inherits parent's effects.
- Persists across nodes.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct FissionConfig {
    /// Number of cell kills required to trigger a split (default 8).
    pub kills_per_split: u32,
}
```

Populated from `ProtocolTuning::Fission { kills_per_split }`.

## Components
```rust
/// Tracks the running cell-kill counter toward the next Fission split.
/// Inserted as a resource (not per-entity) because it is a global counter
/// that persists across nodes.
#[derive(Resource, Debug, Default)]
pub(crate) struct FissionCounter {
    /// Kills since last split.
    pub kills: u32,
}
```

Note: `FissionCounter` is a `Resource`, not a `Component`, because it tracks a global kill count across all bolts and all nodes. It is NOT cleared between nodes (persists across the run).

## Messages
**Reads**: `CellDestroyedAt { position, was_required_to_clear }`
**Sends**: None directly — spawns a new bolt entity via `Commands` (or via `Bolt::builder()`)

## Systems

### `fission_count_kills`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Fission)`, `in_state(NodeState::Playing)`
- **Behavior**: Reads `CellDestroyedAt` messages. For each cell destroyed: increments `FissionCounter.kills` by 1. If `kills >= config.kills_per_split`: resets `kills` to 0, triggers a bolt split.
- **Ordering**: After cells `cleanup_cell` (needs `CellDestroyedAt` populated).

### `fission_split_bolt`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Fission)`, `in_state(NodeState::Playing)`
- **Behavior**: When `fission_count_kills` determines a split is needed: selects a bolt to split (the bolt that last destroyed a cell, or if ambiguous, any active bolt). Spawns a new bolt entity at the same position as the parent bolt. The new bolt's velocity is the parent's velocity rotated by a small angle (e.g., 15-30 degrees) so the two bolts diverge. The new bolt inherits the parent bolt's `BoundEffects` / `StagedEffects` (chip effects). The new bolt is permanent — it does NOT despawn after a timer.
- **Spawn method**: Must use `Bolt::builder()` per project conventions. Clone relevant effect components from parent bolt.
- **Ordering**: Immediately after `fission_count_kills` (same system or chained).

**Implementation note**: `fission_count_kills` and `fission_split_bolt` can be a single system if cleaner. The split needs deferred commands (spawning entity) so may need `Commands`. Alternatively, a single system reads messages, counts, and spawns when threshold is met.

## Cross-Domain Dependencies
- **cells domain**: Reads `CellDestroyedAt` message (to count kills).
- **bolt domain**: Spawns new bolt entities via `Bolt::builder()`. Reads parent bolt's `Transform`, `Velocity2D`, `BoundEffects`, `StagedEffects` (to clone into the new bolt).
- **effect domain**: New bolt inherits parent's `BoundEffects` / `StagedEffects`. The effect system should treat the new bolt like any other bolt for future effect dispatch.
- **run domain**: The new bolt is a permanent member of the bolt pool. It should be tracked by whatever system counts active bolts (for bolt-lost logic, node completion, etc.).

## Expected Behaviors (for test specs)

1. **Kill counter increments on cell destruction**
   - Given: `FissionCounter { kills: 0 }`, `kills_per_split = 8`.
   - When: A cell is destroyed (`CellDestroyedAt` received).
   - Then: `FissionCounter.kills = 1`.

2. **8th kill triggers a split**
   - Given: `FissionCounter { kills: 7 }`, `kills_per_split = 8`.
   - When: A cell is destroyed.
   - Then: `FissionCounter.kills` resets to 0. A new bolt entity is spawned.

3. **New bolt spawns at parent position**
   - Given: Parent bolt at position (200.0, 300.0) with velocity (150.0, 400.0).
   - When: Fission split triggers.
   - Then: New bolt spawns at (200.0, 300.0). New bolt velocity has same magnitude as parent but rotated by a small divergence angle.

4. **New bolt inherits parent's chip effects**
   - Given: Parent bolt has `BoundEffects` containing Piercing and Chain Lightning.
   - When: Fission split triggers.
   - Then: New bolt has `BoundEffects` containing Piercing and Chain Lightning (cloned from parent).

5. **Counter persists across nodes**
   - Given: `FissionCounter { kills: 5 }` at end of node. New node begins.
   - When: 3 cells destroyed in new node.
   - Then: Split triggers on the 3rd kill (5 + 3 = 8). Counter resets to 0.

6. **Counter resets only on split, not on node transition**
   - Given: `FissionCounter { kills: 3 }`. Node ends.
   - When: New node starts.
   - Then: `FissionCounter.kills` still 3.

7. **New bolt is permanent**
   - Given: Fission split creates a new bolt.
   - When: 30 seconds elapse.
   - Then: New bolt still exists (no despawn timer). Only destroyed by bolt-lost.

8. **Multiple splits in rapid succession**
   - Given: `FissionCounter { kills: 7 }`, `kills_per_split = 8`. 9 cells destroyed in quick succession (e.g., shockwave).
   - When: `fission_count_kills` processes all 9 `CellDestroyedAt` messages.
   - Then: First split at kill 8 (counter resets to 0). Counter reaches 1 after processing the 9th kill. Only 1 split occurs (not 2).

## Edge Cases
- All cell kill sources count: bolt impact, shockwave, explosion, chain lightning, piercing beam — any `CellDestroyedAt` message increments the counter.
- Which bolt to split: if the kill came from a specific bolt (traceable via `DamageDealt<Cell>.source_chip` or bolt entity), split that bolt. If kill source is ambiguous (AoE), split any active bolt (e.g., the first in the query).
- New bolt divergence angle: should be consistent (always same direction offset) or alternating. Exact angle is a tuning value but not in RON config for now — hardcoded as a small constant (15-20 degrees). Can be promoted to config later if needed.
- `FissionCounter` cleared on run end (not node end). Cleanup in protocol cleanup system alongside `FissionConfig`.
- Interaction with Conductor: new bolt from Fission does NOT automatically become Conducted. Player must Perfect Bump it to make it primary.
- Bolt cap: no explicit cap on bolt count from Fission. Over a long run with many kills, bolt count grows. If performance becomes an issue, a max-bolt cap can be added later.
- Parent bolt velocity unchanged by the split — only the new bolt gets the rotated velocity.
