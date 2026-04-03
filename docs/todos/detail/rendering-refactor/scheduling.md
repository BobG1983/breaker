# System Scheduling

How `rantzsoft_vfx` exposes its system ordering to the game, and the internal schedule structure.

## VfxSet Enum

The crate defines a `VfxSet` system set enum. The game orders its gameplay sets before `VfxSet::ProcessMessages`.

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum VfxSet {
    /// Read incoming messages (AttachVisuals, ExecuteRecipe, Set/Add/RemoveModifier, primitives).
    /// Game systems that send VFX messages must run BEFORE this set.
    ProcessMessages,

    /// Recipe dispatch: advance phase timers, evaluate triggers, emit per-primitive messages.
    RecipeDispatch,

    /// Spawn VFX entities from dispatched primitive messages.
    PrimitiveSpawn,

    /// Compute modifier stacks, update shader uniforms from computed modifiers.
    ModifierCompute,

    /// Advance active VFX (particle update, ring expansion, beam shrink, anchored tracking).
    Tick,

    /// Despawn expired VFX entities, remove timed modifiers, poll anchored source entities.
    Cleanup,
}
```

## Internal Ordering (FixedUpdate)

```
VfxSet::ProcessMessages
    → VfxSet::RecipeDispatch
    → VfxSet::PrimitiveSpawn
    → VfxSet::ModifierCompute
    → VfxSet::Tick
    → VfxSet::Cleanup
```

All sets are chained in this order within `FixedUpdate`. The crate configures this in its plugin `build()`.

## Game Integration

The game orders its gameplay systems before `VfxSet::ProcessMessages`:

```rust
// In game plugin setup:
app.configure_sets(
    FixedUpdate,
    GameplaySet::PostPhysics.before(VfxSet::ProcessMessages),
);
```

The game is free to choose which of its sets runs "last before VFX." The only contract is: all VFX messages must be sent before `VfxSet::ProcessMessages` runs.

## Update Schedule Systems

Some VFX systems run in `Update` (not `FixedUpdate`) for smooth interpolation:

| System | Schedule | Set | Why |
|--------|----------|-----|-----|
| `update_shader_uniforms` | Update | — | Interpolate material uniforms between FixedUpdate ticks |
| `update_particles` | Update | — | Smooth particle motion (visual-only, no gameplay interaction) |
| `apply_dilation_ramp` | Update | — | Slow-motion ramp reads `Time<Real>`, writes `Time<Virtual>` |
| `update_screen_shake` | Update | — | Camera offset decays per-frame for smooth shake |

These do not use `VfxSet` — they are standalone Update systems with no ordering relationship to gameplay.

## PostUpdate Systems

| System | Schedule | After | Why |
|--------|----------|-------|-----|
| `sample_trail_positions` | PostUpdate | `TransformSystems::Propagate` | Trail entities sample `GlobalTransform` after transform propagation |
| `update_anchored_positions` | PostUpdate | `TransformSystems::Propagate` | Anchored primitives read final entity positions |

## Headless Mode

`RantzVfxPlugin::headless()` registers `VfxSet` (so game ordering constraints compile) but only registers `ProcessMessages` handlers as no-ops that accept and discard messages. All other sets are empty. See [headless.md](headless.md).

## System-to-Set Mapping

| System | VfxSet | Purpose |
|--------|--------|---------|
| `handle_attach_visuals` | ProcessMessages | Create mesh/material/aura/trail for entities |
| `handle_set_modifier` | ProcessMessages | Overwrite modifier by source key |
| `handle_add_modifier` | ProcessMessages | Add stacking modifier with DR |
| `handle_remove_modifier` | ProcessMessages | Remove modifier by source key |
| `handle_execute_recipe` | ProcessMessages | Spawn RecipeExecution, emit RecipeStarted |
| `handle_cancel_recipe` | ProcessMessages | Despawn RecipeExecution and children |
| `handle_spawn_*` | ProcessMessages | Accept direct primitive messages |
| `handle_trigger_*` | ProcessMessages | Accept direct screen effect messages |
| `advance_recipe_phases` | RecipeDispatch | Evaluate phase triggers, emit per-primitive messages |
| `spawn_primitive_entities` | PrimitiveSpawn | Create VFX entities from dispatched primitive data |
| `compute_modifier_stacks` | ModifierCompute | Combine Set + DR-scaled Add values per entity |
| `update_entity_glow_uniforms` | ModifierCompute | Push computed modifiers to EntityGlowMaterial |
| `tick_expanding_rings` | Tick | Advance ring radius |
| `tick_beams` | Tick | Advance beam shrink/afterimage |
| `tick_emitters` | Tick | Spawn particles from emitters |
| `tick_electric_arcs` | Tick | Regenerate jitter vertices |
| `check_phase_completion` | Tick | Poll PhaseGroup children, trigger dependent phases |
| `cleanup_expired_primitives` | Cleanup | Despawn VFX entities past lifetime |
| `cleanup_timed_modifiers` | Cleanup | Remove modifiers past duration |
| `cleanup_trail_sources` | Cleanup | Despawn trails whose source entity is gone |
| `cleanup_anchored_orphans` | Cleanup | Despawn anchored primitives whose tracked entity is gone |
| `cleanup_recipe_executions` | Cleanup | Despawn RecipeExecution when source entity gone |
| `emit_recipe_lifecycle` | Cleanup | Emit PhaseComplete, RecipeComplete messages |
