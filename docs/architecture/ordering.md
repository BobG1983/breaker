# System Ordering — Loose with Key Constraints

No named phase sets or global pipeline. Only add `.before()` / `.after()` where actual data dependencies exist. Let Bevy parallelize everything else.

Ordering is added when systems have proven data dependencies, not speculatively. If a system doesn't read another system's output, it runs freely.

## SystemSet Convention

Domains MAY define a `pub enum {Domain}Systems` with `#[derive(SystemSet)]` in `sets.rs` to expose ordering points for cross-domain use.

**Rules:**

- Each variant names one **pivotal system** that other domains depend on.
- The owning domain tags its system with `.in_set(DomainSystems::Variant)`.
- Consuming domains order with `.after(DomainSystems::Variant)`.
- **Never reference bare system function names across domain boundaries** — always use SystemSet enums. This keeps cross-domain ordering stable even if the underlying system is renamed or split.
- Only create a SystemSet variant when another domain actually needs to order against it. Don't pre-create sets "just in case".
- **Group systems sharing a constraint** with tuple syntax: `(sys_a, sys_b).after(Target)` rather than repeating `.after(Target)` on each system individually. Keeps the shared dependency visible in one place.

**Defined sets:**

| Set | Domain | Tags |
|-----|--------|------|
| `BreakerSystems::Move` | `breaker/sets.rs` | `move_breaker` |
| `BreakerSystems::InitParams` | `breaker/sets.rs` | `init_breaker_params` |
| `BreakerSystems::Reset` | `breaker/sets.rs` | `reset_breaker` (intra-domain only — no cross-domain consumers yet) |
| `BreakerSystems::GradeBump` | `breaker/sets.rs` | `grade_bump` |
| `BoltSystems::InitParams` | `bolt/sets.rs` | `init_bolt_params` |
| `BoltSystems::PrepareVelocity` | `bolt/sets.rs` | `prepare_bolt_velocity` |
| `BoltSystems::Reset` | `bolt/sets.rs` | `reset_bolt` (intra-domain only — no cross-domain consumers yet) |
| `BoltSystems::BreakerCollision` | `bolt/sets.rs` | `bolt_breaker_collision` |
| `BoltSystems::BoltLost` | `bolt/sets.rs` | `bolt_lost` |
| `rantzsoft_physics2d::PhysicsSystems::MaintainQuadtree` | `rantzsoft_physics2d/src/plugin.rs` | `maintain_quadtree` (incremental quadtree update — game collision systems order `.after` this) |
| `rantzsoft_physics2d::PhysicsSystems::EnforceDistanceConstraints` | `rantzsoft_physics2d/src/plugin.rs` | `enforce_distance_constraints` in external crate (game-level `bolt::enforce_distance_constraints` also uses this name — intra-domain) |
| `rantzsoft_spatial2d::SpatialSystems::SavePrevious` | `rantzsoft_spatial2d/src/plugin.rs` | `save_previous` (FixedFirst — snapshots Position2D/Rotation2D into Previous* before physics tick) |
| `rantzsoft_spatial2d::SpatialSystems::ApplyVelocity` | `rantzsoft_spatial2d/src/plugin.rs` | `apply_velocity` (FixedUpdate — advances Position2D by Velocity2D * dt for entities with ApplyVelocity marker) |
| `rantzsoft_spatial2d::SpatialSystems::ComputeGlobals` | `rantzsoft_spatial2d/src/plugin.rs` | `compute_globals` (RunFixedMainLoop AfterFixedMainLoop — resolves parent/child hierarchy into Global* components) |
| `rantzsoft_spatial2d::SpatialSystems::DeriveTransform` | `rantzsoft_spatial2d/src/plugin.rs` | `derive_transform` (RunFixedMainLoop AfterFixedMainLoop — writes Transform from Global* + DrawLayer Z; runs after ComputeGlobals) |
| `BoltSystems::CellCollision` | `bolt/sets.rs` | `bolt_cell_collision` (bolt-cell CCD sweep — fires before BreakerCollision) |
| `BreakerSystems::UpdateState` | `breaker/sets.rs` | `update_breaker_state` (intra-domain only — no cross-domain consumers yet) |
| `EffectSystems::Bridge` | `effect/sets.rs` | `bridge_bump`, `bridge_bolt_lost`, `bridge_bump_whiff`, `bridge_no_bump`, `bridge_cell_impact`, `bridge_breaker_impact`, `bridge_wall_impact`, `bridge_cell_death`, `bridge_bolt_death`, `bridge_timer_threshold` |
| `EffectSystems::Recalculate` | `effect/sets.rs` | `recalculate_speed`, `recalculate_damage`, `recalculate_piercing`, `recalculate_size`, `recalculate_bump_force`, `recalculate_quick_stop` — unordered relative to gameplay chain; `run_if(PlayingState::Active)` |
| `UiSystems::SpawnTimerHud` | `ui/sets.rs` | `spawn_timer_hud` |
| `NodeSystems::TrackCompletion` | `run/node/sets.rs` | `track_node_completion` |
| `NodeSystems::TickTimer` | `run/node/sets.rs` | `tick_node_timer` |
| `NodeSystems::ApplyTimePenalty` | `run/node/sets.rs` | `apply_time_penalty` |
| `NodeSystems::Spawn` | `run/node/sets.rs` | `spawn_cells_from_layout` (OnEnter) |
| `NodeSystems::InitTimer` | `run/node/sets.rs` | `init_node_timer` (OnEnter) |

**Example:**

```rust
// In breaker/sets.rs
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BreakerSystems {
    Move,        // The move_breaker system
    InitParams,  // The init_breaker_params system
}

// In breaker/plugin.rs — tag the system
move_breaker.in_set(BreakerSystems::Move)
init_breaker_params.in_set(BreakerSystems::InitParams)

// In bolt/plugin.rs — order against it
(hover_bolt, prepare_bolt_velocity.in_set(BoltSystems::PrepareVelocity))
    .after(BreakerSystems::Move)
```

## Current Ordering Chain

The actual cross-domain ordering constraints in the codebase:

### OnEnter(GameState::Playing)

```
apply_breaker_config_overrides         [effect domain]
  .before(BreakerSystems::InitParams)
    BreakerSystems::InitParams
    (init_breaker_params)              [breaker domain]
      <- init_breaker .after(BreakerSystems::InitParams)     [effect domain]
      <- reset_breaker .after(BreakerSystems::InitParams)
         BreakerSystems::Reset                                [breaker domain]
      <- UiSystems::SpawnTimerHud
         (spawn_timer_hud)             [ui domain]
           <- spawn_lives_display .after(init_breaker)
                                  .after(UiSystems::SpawnTimerHud)  [effect domain]

spawn_bolt → init_bolt_params          [bolt domain, .after(spawn_bolt)]
  BoltSystems::InitParams
    <- reset_bolt .after(BoltSystems::InitParams)
                  .after(BreakerSystems::Reset)
       BoltSystems::Reset              [bolt domain]
```

Note: `spawn_breaker` → `ApplyDeferred` → `init_breaker_params` are chained inside the breaker plugin. `spawn_side_panels` + `ApplyDeferred` + `spawn_timer_hud` are chained inside the UI plugin, so `UiSystems::SpawnTimerHud` is the externally-visible anchor. `reset_bolt` is the last OnEnter system — it waits for both breaker reset and bolt init.

### FixedUpdate

```
rantzsoft_physics2d::PhysicsSystems::MaintainQuadtree
  (maintain_quadtree)           [rantzsoft_physics2d — incremental spatial index update]
    <- bolt_cell_collision .after(BoltSystems::PrepareVelocity)
                           .after(rantzsoft_physics2d::PhysicsSystems::MaintainQuadtree)
    <- shockwave_collision .after(tick_shockwave)
                           .after(rantzsoft_physics2d::PhysicsSystems::MaintainQuadtree)

EffectSystems::Recalculate
  (recalculate_speed, recalculate_damage, recalculate_piercing,
   recalculate_size, recalculate_bump_force, recalculate_quick_stop)
  [effect domain — unordered relative to gameplay chain; reads Active* from previous frame]

move_breaker .after(update_bump)
  BreakerSystems::Move
    <- (hover_bolt, prepare_bolt_velocity) .after(BreakerSystems::Move)
      BoltSystems::PrepareVelocity
            <- bolt_cell_collision .after(BoltSystems::PrepareVelocity)
                                   .after(rantzsoft_physics2d::PhysicsSystems::MaintainQuadtree)
              BoltSystems::CellCollision
                <- bolt_breaker_collision .after(BoltSystems::CellCollision)
                  BoltSystems::BreakerCollision
            <- grade_bump .after(update_bump)
                          .after(BoltSystems::BreakerCollision)
              BreakerSystems::GradeBump
                <- (perfect_bump_dash_cancel, spawn_bump_grade_text, spawn_whiff_text) .after(grade_bump)
                <- bridge_bump .after(BreakerSystems::GradeBump)
                   .in_set(EffectSystems::Bridge)              [effect domain]
                <- bridge_bump_whiff .after(BreakerSystems::GradeBump)
                   .in_set(EffectSystems::Bridge)              [effect domain]
                <- bridge_no_bump .after(bridge_breaker_impact).after(bridge_bump)
                   .in_set(EffectSystems::Bridge)              [effect domain]
                <- bridge_cell_impact .after(BoltSystems::BreakerCollision)
                   .in_set(EffectSystems::Bridge)              [effect domain]
                <- bridge_breaker_impact .after(BoltSystems::BreakerCollision)
                   .in_set(EffectSystems::Bridge)              [effect domain]
                <- bridge_wall_impact .after(BoltSystems::BreakerCollision)
                   .in_set(EffectSystems::Bridge)              [effect domain]
            <- clamp_bolt_to_playfield .after(bolt_breaker_collision)
            <- enforce_distance_constraints .after(clamp_bolt_to_playfield)  [bolt domain]
            <- bolt_lost .after(enforce_distance_constraints)
              BoltSystems::BoltLost
                <- bridge_bolt_lost .after(BoltSystems::BoltLost)
                   .in_set(EffectSystems::Bridge)          [effect domain]
                <- break_chain_on_bolt_lost .after(BoltSystems::BoltLost)  [bolt domain]
            <- bridge_cell_death .in_set(EffectSystems::Bridge)
               [effect domain, unordered relative to physics chain]
            <- bridge_bolt_death .in_set(EffectSystems::Bridge)
               [effect domain, unordered relative to physics chain]
            <- bridge_timer_threshold .in_set(EffectSystems::Bridge)
               [effect domain, unordered relative to physics chain]
```

Reading: the quadtree is maintained first (incremental — only changed entities re-inserted). `EffectSystems::Recalculate` converts `Active*` stacks into `Effective*` scalars (`EffectiveSpeedMultiplier`, `EffectiveDamageMultiplier`, `EffectivePiercing`, `EffectiveSizeMultiplier`, `EffectiveBumpForce`, `EffectiveQuickStop`) — it runs unordered relative to the gameplay chain, reading Active* state that is stable during gameplay (Active* vecs only change at chip selection between nodes). Then breaker moves, bolt velocity is prepared, cell collisions run (reading quadtree for broad-phase, tagged `BoltSystems::CellCollision`), then breaker collision (`BoltSystems::BreakerCollision`), then bump grading (`BreakerSystems::GradeBump`), then distance constraints enforced (chain bolts), then bolt-lost detection (`BoltSystems::BoltLost`). All effect bridge systems run in `EffectSystems::Bridge` — downstream consumers order `.after(EffectSystems::Bridge)`.

```
NodeSystems::TrackCompletion
  (track_node_completion)             [run/node domain]
NodeSystems::TickTimer
  (tick_node_timer)                   [run/node domain]
    NodeSystems::ApplyTimePenalty
      (apply_time_penalty)            [run/node domain]

handle_node_cleared .after(NodeSystems::TrackCompletion)   [run domain]
handle_timer_expired .after(NodeSystems::ApplyTimePenalty)
                     .after(handle_node_cleared)            [run domain]
handle_run_lost .after(handle_node_cleared)
                .after(handle_timer_expired)                [run domain]
```

Reading: completion tracking runs first (cells consumed → NodeCleared sent), then run handles it. Timer ticks, then time penalties apply. Run checks for timer expiry after penalties and after node-cleared (clear beats a simultaneous timer expiry — player wins tie-frame). Run-lost is checked last.

### FixedFirst

```
save_previous         [rantzsoft_spatial2d — snapshots Position2D/Rotation2D/Scale2D/Velocity2D into Previous* for interpolation]
```

### AfterFixedMainLoop (RunFixedMainLoop schedule, AfterFixedMainLoop system set)

```
compute_globals → derive_transform → propagate_position → propagate_rotation → propagate_scale
  [rantzsoft_spatial2d — chained in sequence, runs once per visual frame after all FixedUpdate ticks]
```

Reading: after all FixedUpdate ticks complete for the current visual frame, `compute_globals` resolves the parent/child hierarchy and writes `GlobalPosition2D`/`GlobalRotation2D`/`GlobalScale2D`; `derive_transform` reads Global* plus the entity's `DrawLayer` Z value and `InterpolateTransform2D` flag to write the final `Transform` (interpolated or direct); propagation systems distribute parent global values to children. Transform is derived — game systems NEVER write `Transform` directly.

### OnEnter(GameState::TransitionOut) / OnEnter(GameState::TransitionIn)

```
OnEnter(TransitionOut):
  spawn_transition_out        [fx domain — spawns TransitionOverlay with TransitionTimer]
OnExit(TransitionOut):
  cleanup_transition          [fx domain — despawns all TransitionOverlay entities]

OnEnter(TransitionIn):
  advance_node                [run domain — increments node index in RunState]
  spawn_transition_in         [fx domain — spawns TransitionOverlay with TransitionTimer]
OnExit(TransitionIn):
  cleanup_transition          [fx domain — despawns all TransitionOverlay entities]
```

Note: `advance_node` and `spawn_transition_in` are unordered relative to each other (no data dependency). The `animate_transition` system runs in `Update` conditioned on `in_state(TransitionOut).or(in_state(TransitionIn))` — it ticks the `TransitionTimer` and sets `NextState` to `ChipSelect` (on `TransitionOut` completion) or `Playing` (on `TransitionIn` completion).

### OnExit(GameState::MainMenu)

```
reset_run_state                                             [run domain]
  <- generate_node_sequence_system .after(reset_run_state) [run domain]
```

Reading: run state is reset and RNG is reseeded first, then the node sequence is generated from the freshly seeded `GameRng`.

**Intra-domain constraints (breaker):** `update_bump` → `move_breaker` (`BreakerSystems::Move`) → `update_breaker_state` (`BreakerSystems::UpdateState`, with `(perfect_bump_dash_cancel, spawn_bump_grade_text, spawn_whiff_text)` also before `UpdateState`); `grade_bump` runs `.after(update_bump).after(BoltSystems::BreakerCollision)` — it is NOT after `update_breaker_state`. `trigger_bump_visual` also runs `.after(update_bump)`.

## Schedule Placement

- **FixedUpdate** — all gameplay and physics systems. Required for deterministic, seed-reproducible behavior.
- **Update** — visual-only systems (interpolation, UI rendering, shader updates). No gameplay state mutation.
- **OnEnter / OnExit** — state transition setup and cleanup (spawning, despawning, resource initialization).
