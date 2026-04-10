# Name
Shockwave

# Enum Variant
- `EffectType::Shockwave(ShockwaveConfig)`

# Config
`ShockwaveConfig { base_range: f32, range_per_level: f32, stacks: u32, speed: f32 }`

# Fire
1. Calculate effective range: `base_range + range_per_level * (stacks - 1)`.
2. Read the source entity's position.
3. Snapshot the source entity's damage multiplier (from `EffectStack<DamageBoostConfig>` aggregate) and base damage (from `BoltBaseDamage` component).
4. Spawn a shockwave entity at the source position with:
   - `ShockwaveRadius(0.0)`
   - `ShockwaveMaxRadius(effective_range)`
   - `ShockwaveSpeed(config.speed)`
   - `ShockwaveBaseDamage(base_damage)`
   - `ShockwaveDamageMultiplier(snapshotted_multiplier)`
   - `EffectSourceChip(source_chip_attribution)`
   - `ShockwaveDamaged(HashSet::new())`
   - `ShockwaveSource` marker
   - `CleanupOnExit<NodeState>` — despawn on node teardown as safety net
5. Fire does NOT deal damage -- the `apply_shockwave_damage` system does.
6. Fire does NOT check if cells are in range -- the damage system does.

# Reverse
Not reversible.

# Source Location
`src/effect/effects/shockwave/config.rs`

# New Types
- `ShockwaveSource` -- marker component identifying shockwave entities
- `ShockwaveRadius(f32)` -- current expansion radius
- `ShockwaveMaxRadius(f32)` -- radius at which the shockwave despawns
- `ShockwaveSpeed(f32)` -- expansion rate in world units per second
- `ShockwaveDamaged(HashSet<Entity>)` -- tracks which cells have already been damaged
- `ShockwaveBaseDamage(f32)` -- snapshotted base damage at spawn time
- `ShockwaveDamageMultiplier(f32)` -- snapshotted damage multiplier at spawn time
- `EffectSourceChip(Option<String>)` -- which chip produced this effect (shared across effects)

# New Systems

## tick_shockwave
- **What it does**: For each entity with `ShockwaveSource`, increase `ShockwaveRadius` by `ShockwaveSpeed * dt`.
- **What it does NOT do**: Does not deal damage. Does not despawn shockwaves. Does not check cells.
- **Schedule**: FixedUpdate, in `EffectSystems::Tick`, with `run_if(in_state(NodeState::Playing))`.

## sync_shockwave_visual
- **What it does**: For each entity with `ShockwaveSource`, set `Scale2D` to match the current `ShockwaveRadius`.
- **What it does NOT do**: Does not modify `ShockwaveRadius`. Does not deal damage.
- **Schedule**: FixedUpdate, in `EffectSystems::Tick`, chained after `tick_shockwave`, with `run_if(in_state(NodeState::Playing))`.

## apply_shockwave_damage
- **What it does**: For each entity with `ShockwaveSource`, query the quadtree for cells within `ShockwaveRadius`. For each cell not already in `ShockwaveDamaged`, send `DamageDealt<Cell>` with `ShockwaveBaseDamage * ShockwaveDamageMultiplier` and add the cell to the `ShockwaveDamaged` set.
- **What it does NOT do**: Does not deal damage directly -- sends the message. Does not modify `ShockwaveRadius`.
- **Schedule**: FixedUpdate, in `EffectSystems::Tick`, chained after `sync_shockwave_visual`, with `run_if(in_state(NodeState::Playing))`.

## despawn_finished_shockwave
- **What it does**: For each entity with `ShockwaveSource`, if `ShockwaveRadius >= ShockwaveMaxRadius`, despawn the entity.
- **What it does NOT do**: Does not deal damage. Does not modify radius.
- **Schedule**: FixedUpdate, in `EffectSystems::Tick`, chained after `apply_shockwave_damage`, with `run_if(in_state(NodeState::Playing))`.
