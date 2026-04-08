# Protocol: Iron Curtain

## Category
`custom-system`

## Game Design
Bolt-lost becomes an offensive event, not just a penalty.

- On bolt-lost: damage wave spreads upward from breaker position across the full playfield.
- Wave damage = 0.5x bolt base damage at origin.
- Linear falloff with distance from breaker (small "no falloff" window near breaker so close hits feel impactful).
- Wave dims visually through its life.
- Does NOT prevent bolt-lost penalties (life loss, time penalty, etc.).

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct IronCurtainConfig {
    /// Fraction of bolt base damage dealt at the wave origin (breaker position).
    /// e.g., 0.5 means 50% of bolt base damage.
    pub damage_fraction: f32,
    /// Distance from breaker within which damage is NOT reduced (flat full-damage zone).
    /// Beyond this distance, linear falloff begins.
    pub falloff_start: f32,
}
```

## Components
No per-entity components needed. The damage wave is a one-shot event that scans all cells and sends `DamageDealt<Cell>` messages immediately. No persistent wave entity is required for the gameplay logic (visual FX may spawn a transient entity, but that is FX domain scope).

## Messages
**Reads**: `BoltLost`, `BumpPerformed` (to get bolt base damage — or read from bolt component before destruction)
**Sends**: `DamageDealt<Cell> { cell, damage, source_chip }` for each cell hit by the wave

**Note on bolt entity lifetime**: `BoltLost` fires before the bolt entity is destroyed. The system must read the bolt's base damage from the bolt entity (or from a resource like `BoltConfig`) before the entity is cleaned up. If the bolt entity is already despawned by the time this system runs, fall back to `BoltConfig.base_damage`.

## Systems

### `iron_curtain_on_bolt_lost`
- **Schedule**: `FixedUpdate`
- **Run if**: `protocol_active(ProtocolKind::IronCurtain)` + `in_state(NodeState::Playing)`
- **What it does**:
  1. Reads `BoltLost` messages.
  2. Gets the breaker position (from `Breaker` entity query).
  3. Gets bolt base damage (from bolt entity or `BoltConfig`).
  4. Computes wave damage at origin: `bolt_base_damage * config.damage_fraction`.
  5. Iterates all alive cells, computing distance from breaker to cell.
  6. For cells within `config.falloff_start`: full wave damage.
  7. For cells beyond `config.falloff_start`: linear falloff to 0 at the top of the playfield.
  8. Sends `DamageDealt<Cell> { cell, damage, source_chip: None }` for each cell that takes > 0 damage.
- **Ordering**: After bolt-lost detection, before cell cleanup. Must run while cell entities still exist.

### Damage Falloff Formula
```
let max_distance = playfield_height - falloff_start;
let cell_distance = (cell_position.y - breaker_position.y).abs();

if cell_distance <= falloff_start {
    damage = wave_origin_damage;  // full damage, no falloff
} else {
    let falloff_distance = cell_distance - falloff_start;
    let falloff_factor = 1.0 - (falloff_distance / max_distance).clamp(0.0, 1.0);
    damage = wave_origin_damage * falloff_factor;
}
```

## Cross-Domain Dependencies
- **bolt**: Reads `BoltLost` message. Reads bolt base damage from bolt entity or `BoltConfig`.
- **breaker**: Reads breaker entity position.
- **cells**: Sends `DamageDealt<Cell>` message to all cells in range. Reads cell positions via query.
- **shared**: Reads `PlayfieldConfig` for playfield dimensions (needed for falloff calculation).

## Expected Behaviors (for test specs)

1. **Bolt-lost triggers damage wave from breaker position**
   - Given: Iron Curtain active, `damage_fraction: 0.5`, bolt base damage = 20.0, breaker at (0.0, -200.0)
   - When: `BoltLost` is sent
   - Then: Wave originates at breaker position with origin damage = 10.0 (20.0 * 0.5)

2. **Cells within falloff_start take full damage**
   - Given: `falloff_start: 50.0`, breaker at (0.0, -200.0), cell A at (30.0, -170.0) (distance 30.0)
   - When: Damage wave fires
   - Then: Cell A receives `DamageDealt<Cell> { damage: 10.0 }` (full damage, within falloff_start)

3. **Cells beyond falloff_start take reduced damage (linear falloff)**
   - Given: `falloff_start: 50.0`, playfield height = 400.0, breaker at (0.0, -200.0), cell B at (0.0, 0.0) (distance 200.0)
   - When: Damage wave fires with origin damage 10.0
   - Then: `falloff_distance = 200.0 - 50.0 = 150.0`, `max_distance = 400.0 - 50.0 = 350.0`, `factor = 1.0 - 150.0/350.0 = 0.571`, Cell B receives `DamageDealt<Cell> { damage: 5.71 }`

4. **Cells at maximum distance take near-zero damage**
   - Given: `falloff_start: 50.0`, playfield height = 400.0, cell at top of playfield (distance = 400.0)
   - When: Damage wave fires with origin damage 10.0
   - Then: `falloff_factor = 1.0 - (350.0/350.0) = 0.0`, Cell receives 0 damage, no `DamageDealt<Cell>` sent

5. **Bolt-lost penalties still apply**
   - Given: Iron Curtain active
   - When: `BoltLost` is sent
   - Then: Normal bolt-lost consequences (life loss, time penalty) still fire. Iron Curtain's damage wave is additive, not a replacement.

6. **No wave when no bolt-lost**
   - Given: Iron Curtain active, normal gameplay (no bolt-lost)
   - When: Cells are hit by normal bolt impacts
   - Then: No damage wave fires. Iron Curtain has no effect on normal hits.

7. **Multiple bolt-lost events in quick succession each trigger a wave**
   - Given: Two bolts lost in the same frame
   - When: Two `BoltLost` messages are sent
   - Then: Two independent damage waves fire (one per bolt-lost)

## Edge Cases
- **Bolt entity already despawned**: If the bolt is cleaned up before this system runs, read base damage from `BoltConfig` resource (fallback). This depends on system ordering — must run before bolt cleanup.
- **No cells alive**: Wave fires but hits nothing. No `DamageDealt<Cell>` messages sent.
- **Breaker at edge of playfield**: Falloff still computed from breaker position. Cells behind the breaker (below it) are not damaged — wave spreads upward only.
- **Multiple bolts lost simultaneously**: Each generates its own wave. Cells can take damage from multiple waves in the same frame.
- **Interaction with other protocols**: Debt Collector stack is lost on bolt-lost (separate system). Iron Curtain wave fires independently of Debt Collector's stack loss. The wave damage does not benefit from Debt Collector's stack (different trigger path).
