# Protocol: Iron Curtain

## Category
`custom-system`

## Game Design
Bolt-lost becomes an offensive event, not just a penalty.

- On bolt-lost: a damage wave spreads outward from breaker position across the playfield.
- Wave damage at origin = `bolt.base_damage * damage_fraction` (default 0.5).
- `|x|`-symmetric linear falloff with distance from breaker: a flat full-damage zone (`falloff_start`) surrounds the breaker, then falloff begins beyond. Cells at distance `|cell.y - breaker.y|` within the radius are damaged symmetrically — above and below.
- Wave visual dims through its life (VFX layer).
- Does NOT prevent bolt-lost penalties (life loss / time loss per `BoltLossBehavior`).

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct IronCurtainConfig {
    pub damage_fraction: f32,       // 0.5
    pub falloff_start: f32,         // world units; flat full-damage zone
}
```

## Components
None — the wave is a one-shot event that iterates cells and emits `DamageDealt<Cell>` messages. No persistent wave entity for gameplay logic.

## Messages
**Reads**: `BoltLost { bolt }` (bolt-lifecycle message, not part of the death pipeline).
**Sends**: `DamageDealt<Cell> { target, damage, source }` in `DmgSystems::EmitDamage` — one per cell within the falloff radius. Source: `"protocol:iron_curtain"`.

## Systems

### `iron_curtain_on_bolt_lost`
- **Schedule**: `FixedUpdate`, in `DmgSystems::EmitDamage`.
- **run_if**: `protocol_active(ProtocolKind::IronCurtain)` + `in_state(NodeState::Playing)`.
- **Behavior**:
  1. Reads `BoltLost`.
  2. Gets the breaker position (`Query<&Position2D, With<Breaker>>`).
  3. Gets bolt base damage from the bolt entity (if still alive) or falls back to `DEFAULT_BOLT_BASE_DAMAGE`.
  4. Computes `wave_origin_damage = bolt_base_damage * config.damage_fraction`.
  5. Iterates alive cells (`Query<(Entity, &Position2D), With<Cell>>`). For each, computes `|cell.y - breaker.y|`.
  6. If distance `≤ falloff_start`: full damage.
  7. Else: linear falloff. `let falloff_distance = distance - falloff_start; let max_distance = playfield_extent - falloff_start; let factor = 1.0 - (falloff_distance / max_distance).clamp(0.0, 1.0); damage = wave_origin_damage * factor;`
  8. Emits `DamageDealt<Cell>` for each cell with `damage > 0`.

### Damage falloff formula (spec pin)

```
distance    = |cell.y - breaker.y|
max_distance = playfield_extent - falloff_start

if distance <= falloff_start {
    damage = wave_origin_damage
} else {
    factor = 1.0 - ((distance - falloff_start) / max_distance).clamp(0.0, 1.0)
    damage = wave_origin_damage * factor
}
```

`|cell.y - breaker.y|` is abs-symmetric. Cells below the breaker receive damage consistent with the symmetric formula — the wave does not explicitly mask them.

## Pipeline position (dmg crate)

- **Trigger**: `BoltLost` (bolt-lifecycle — NOT part of `DeathPipelineSystems`).
- **Emits**: `DamageDealt<Cell>` in `DmgSystems::EmitDamage` for each cell in range.
- **Source**: `"protocol:iron_curtain"`.
- **No** `DamageBoostStack` / `VulnerableStack` interaction — the emitted damage flows through the standard chain (boosts → mutate → vulnerable → apply) like any other source.

## Cross-Domain Dependencies
- **bolt**: Reads `BoltLost`. Reads bolt base damage.
- **breaker**: Reads breaker position.
- **cells**: Reads cell positions. Emits `DamageDealt<Cell>`.
- **shared**: Reads `PlayfieldConfig` for `playfield_extent` in the falloff formula.

## Expected Behaviors (for test specs)

1. **Bolt-lost triggers damage wave from breaker position** — `damage_fraction: 0.5`, bolt base damage 20.0, breaker at (0, -200): origin damage 10.0.
2. **Cells within `falloff_start` take full damage** — `falloff_start: 50.0`, breaker at (0, -200), cell at (30, -170) (|Δy| = 30): full damage 10.0.
3. **Cells beyond `falloff_start` take reduced damage** — `falloff_start: 50.0`, `playfield_extent: 400.0`, breaker at (0, -200), cell at (0, 0) (|Δy| = 200): `factor = 1 - (150/350) = 0.571`; damage ≈ 5.71.
4. **Cells at max distance take near-zero damage** — cell at edge (|Δy| = 400): `factor = 0`; no `DamageDealt<Cell>` emitted.
5. **Abs-symmetric falloff** — a cell below the breaker at |Δy| = 30 takes full damage, same as a cell above at |Δy| = 30.
6. **Bolt-lost penalties still apply** — standard bolt-lost handler runs alongside (life/time loss). Iron Curtain's damage wave is additive, not a replacement.
7. **Multiple bolt-lost events each trigger a wave** — two bolts lost same frame: two independent waves.

## Edge Cases
- **Bolt entity already despawned**: fallback reads `DEFAULT_BOLT_BASE_DAMAGE`.
- **No cells alive**: wave fires but emits nothing.
- **Breaker at edge of playfield**: falloff still computed from breaker position; abs-symmetric wave reaches however far it can.
- **Multiple bolts lost simultaneously**: cells can take damage from multiple waves in the same tick.
- **Interaction with Debt Collector**: Debt Collector's stack is lost on bolt-lost (separate system); Iron Curtain's wave fires independently and does not benefit from Debt Collector's stack.
