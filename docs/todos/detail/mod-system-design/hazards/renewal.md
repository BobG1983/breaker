# Hazard: Renewal

## Game Design

Cells have a countdown timer and regenerate to full HP on expiry, then the timer resets shorter. 10s base timer, -20%/level (diminishing returns). This creates a "beat the clock per cell" mechanic — the player must finish off damaged cells before they heal. Partially damaged cells that survive long enough snap back to full HP. At higher stacks the regen timer is shorter, giving less time to finish cells off. The diminishing returns on timer reduction prevent it from becoming instant.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct RenewalConfig {
    /// Base timer duration in seconds before first regen.
    pub base_timer: f32,
    /// Percentage reduction per stack beyond the first (applied with diminishing returns).
    pub per_level_reduction_percent: f32,
}
```

Extracted from `HazardTuning::Renewal { base_timer, per_level_reduction_percent }` at activation time.

## Components

### `RenewalTimer`

```rust
/// Per-cell countdown timer for Renewal hazard regen.
/// When the timer expires, the cell regenerates to full HP and the timer resets
/// with the current (possibly shorter) duration.
#[derive(Component, Debug)]
pub(crate) struct RenewalTimer {
    /// Time remaining until next regen.
    pub remaining: f32,
    /// The timer duration used for this cycle (gets shorter on resets at higher stacks).
    pub duration: f32,
}
```

Inserted on every cell entity when Renewal activates. The timer ticks down; on expiry, the cell heals to full and the timer resets with the current computed duration.

## Messages

**Reads**: `ActiveHazards` for stack count (to compute current timer duration on reset)
**Sends**: `HealCell { cell: Entity, amount: f32 }` — new message, owned by `cells` domain. Sent with `amount` equal to the cell's missing HP (heal to full).

Note: Renewal needs to know how much HP the cell is missing. Options:
1. Read cell HP via a cross-domain query (allowed for reads), compute `max_hp - current_hp`, send as `amount`.
2. Send a `HealCellToFull { cell: Entity }` variant or use a sentinel value (e.g., `f32::MAX` meaning "heal to full").

Option 1 is cleaner and avoids new message variants.

## Systems

### `renewal_init_timers`

- **Schedule**: Runs once when Renewal is first activated (on `HazardSelected { kind: Renewal }` or at node start if Renewal is active)
- **Behavior**: For every living cell entity, insert a `RenewalTimer` component with `remaining` and `duration` set to the computed timer for the current stack count.
- **Timer formula (diminishing returns)**: `timer = base_timer * (1.0 - per_level_reduction_percent / 100.0) ^ (stack - 1)`

Also needs to run on `OnEnter(NodeState::Playing)` for subsequent nodes within the same run, to attach timers to the new node's cells.

### `renewal_tick`

- **Schedule**: `FixedUpdate`
- **Run condition**: `hazard_active(HazardKind::Renewal).and(in_state(NodeState::Playing))`
- **Ordering**: Standard — no specific ordering relative to other hazard systems
- **Behavior**: For each cell with a `RenewalTimer`:
  1. Tick `remaining` down by `delta_secs`.
  2. If `remaining <= 0.0`:
     a. Read the cell's current HP and max HP.
     b. If current HP < max HP, send `HealCell { cell, amount: max_hp - current_hp }`.
     c. Recompute timer duration for current stack count (stack may have increased since last reset).
     d. Reset `remaining` to the new duration.

### `renewal_reset_on_stack_change`

- **Schedule**: Runs when `HazardSelected { kind: Renewal }` is received for stacks 2+
- **Behavior**: Recomputes `duration` for all existing `RenewalTimer` components to reflect the new stack count. Does NOT reset `remaining` — in-progress timers continue but future resets will use the shorter duration.

## Stacking Behavior

Diminishing returns on timer reduction: `timer = base_timer * (1.0 - reduction_percent / 100.0) ^ (stack - 1)`

With `base_timer=10.0` and `per_level_reduction_percent=20.0`:

| Stack | Timer duration | Reduction from base |
|-------|---------------|-------------------|
| 1 | 10.0s | 0% |
| 2 | 8.0s | 20% |
| 3 | 6.4s | 36% |
| 5 | 4.096s | ~59% |
| 10 | 1.342s | ~87% |

The diminishing returns curve means early stacks have a large impact (10s -> 8s -> 6.4s) but later stacks have diminishing effect. The timer asymptotically approaches 0 but never reaches it.

## Cross-Domain Dependencies

| Domain | Direction | Message/Query |
|--------|-----------|--------------|
| `cells` | sends to | `HealCell` — heals cell to full HP |
| `cells` | reads from | Cell HP query — needs current HP and max HP to compute heal amount |

Renewal reads cell HP (cross-domain read, allowed) but never writes cell HP directly. The cells domain applies the heal via `HealCell`.

## Expected Behaviors (for test specs)

1. **Cell regens to full after timer expires at stack 1**
   - Given: Renewal active at stack 1, `base_timer=10.0`, cell with 30/100 HP, `RenewalTimer { remaining: 0.05, duration: 10.0 }`
   - When: `renewal_tick` runs with `delta_secs=0.1`
   - Then: Timer expires, `HealCell { cell, amount: 70.0 }` sent, timer resets to `remaining=10.0`

2. **Timer is shorter at stack 3**
   - Given: Renewal active at stack 3, `base_timer=10.0`, `per_level_reduction_percent=20.0`
   - When: Timer resets after expiry
   - Then: New `duration` and `remaining` set to 6.4s (10.0 * 0.8^2)

3. **Full HP cell still resets timer but sends no heal**
   - Given: Renewal active, cell at 100/100 HP, timer expires
   - When: `renewal_tick` processes the expiry
   - Then: No `HealCell` sent (missing HP is 0), timer resets normally

4. **Timer continues ticking between regens**
   - Given: `RenewalTimer { remaining: 5.0 }`, `delta_secs=0.1`
   - When: `renewal_tick` runs
   - Then: `remaining` becomes 4.9, no heal sent

5. **New cells get timers on node start**
   - Given: Renewal active at stack 2, new node starts with fresh cells
   - When: `renewal_init_timers` runs on `OnEnter(NodeState::Playing)`
   - Then: All cells receive `RenewalTimer { remaining: 8.0, duration: 8.0 }`

## Edge Cases

- **Renewal + Decay synergy**: Double time pressure — the node timer is ticking faster (Decay) while cells are regenerating on their own countdown (Renewal). The player races against both clocks. No special interaction code needed — the pressure is emergent.
- **Renewal + Cascade interaction**: If Cascade heals a cell above its pre-damage HP (overheal to cap), does Renewal still fire? Yes — Renewal always ticks. If the cell is already at full HP when the timer expires, no heal is sent, but the timer still resets. Renewal doesn't "know" about Cascade.
- **Node-end cleanup**: `RenewalConfig` resource removed at run end. `RenewalTimer` components are on cell entities, which are despawned with the node.
- **Destroyed cells**: When a cell is destroyed, its entity is despawned, taking the `RenewalTimer` component with it. No stale timer cleanup needed.
- **Ghost cells (Echo Cells)**: Ghost cells spawned by Echo Cells should also receive `RenewalTimer` components if Renewal is active. The `renewal_init_timers` system (or a variant that runs on cell spawn) needs to handle dynamically spawned cells, not just cells present at node start.
- **Stack increase mid-node**: When the player picks another Renewal stack, existing timers keep their current `remaining` but `duration` is updated for the next reset cycle. This prevents jarring mid-countdown changes.
- **Diminishing returns floor**: The timer approaches 0 asymptotically. At very high stacks (e.g., stack 20), the timer is ~0.115s, meaning cells regen almost instantly. In practice, the player is overwhelmed long before this.
