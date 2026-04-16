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
- **Phase sets — exception to pivotal-system rule.** A SystemSet variant that represents a pipeline *phase* (not a single pivotal system) may be used as a tag target by other plugins. The owning plugin is still responsible for `configure_sets`; other plugins contribute systems via `.in_set(PhaseSet::Variant)`. Currently only `DeathPipelineSystems::{ApplyDamage, DetectDeaths, HandleKill}` qualify — each phase legitimately hosts multiple systems across plugins (e.g. `handle_kill::<Cell>` + `handle_kill::<Bolt>` + `handle_kill::<Wall>` from `DeathPipelinePlugin`, and `handle_breaker_death` from `RunPlugin`, all in `HandleKill`). Create a phase set only when multiple plugins legitimately need to contribute to the same pipeline stage — don't invent them speculatively.

**Defined sets:**

| Set | Domain | Tags |
|-----|--------|------|
| `BreakerSystems::Move` | `breaker/sets.rs` | `move_breaker` |
| `BreakerSystems::Reset` | `breaker/sets.rs` | `reset_breaker` (intra-domain only — no cross-domain consumers yet) |
| `BreakerSystems::GradeBump` | `breaker/sets.rs` | `grade_bump` |
| `BoltSystems::Reset` | `bolt/sets.rs` | `reset_bolt` |
| `BoltSystems::BreakerCollision` | `bolt/sets.rs` | `bolt_breaker_collision` |
| `BoltSystems::BoltLost` | `bolt/sets.rs` | `bolt_lost` |
| `rantzsoft_physics2d::PhysicsSystems::MaintainQuadtree` | `rantzsoft_physics2d/src/plugin.rs` | `maintain_quadtree` (incremental quadtree update — game collision systems order `.after` this) |
| `rantzsoft_physics2d::PhysicsSystems::EnforceDistanceConstraints` | `rantzsoft_physics2d/src/plugin.rs` | `enforce_distance_constraints` in external crate (game-level `bolt::enforce_distance_constraints` also uses this name — intra-domain) |
| `rantzsoft_spatial2d::SpatialSystems::SavePrevious` | `rantzsoft_spatial2d/src/plugin.rs` | `save_previous` (FixedFirst — snapshots Position2D/Rotation2D into Previous* before physics tick) |
| `rantzsoft_spatial2d::SpatialSystems::ApplyVelocity` | `rantzsoft_spatial2d/src/plugin.rs` | `apply_velocity` (FixedUpdate — advances Position2D by Velocity2D * dt for entities with ApplyVelocity marker) |
| `rantzsoft_spatial2d::SpatialSystems::ComputeGlobals` | `rantzsoft_spatial2d/src/plugin.rs` | `compute_globals` (RunFixedMainLoop AfterFixedMainLoop — resolves parent/child hierarchy into Global* components) |
| `rantzsoft_spatial2d::SpatialSystems::DeriveTransform` | `rantzsoft_spatial2d/src/plugin.rs` | `derive_transform` (RunFixedMainLoop AfterFixedMainLoop — writes Transform from Global* + DrawLayer Z; runs after ComputeGlobals) |
| `BoltSystems::CellCollision` | `bolt/sets.rs` | `bolt_cell_collision` (bolt-cell CCD sweep — fires before WallCollision and BreakerCollision) |
| `BoltSystems::WallCollision` | `bolt/sets.rs` | `bolt_wall_collision` (bolt-wall reflection — runs `.after(BoltSystems::CellCollision)`) |
| `BreakerSystems::UpdateState` | `breaker/sets.rs` | `update_breaker_state` (intra-domain only — no cross-domain consumers yet) |
| `EffectV3Systems::Bridge` | `effect_v3/sets.rs` | `bridge_bump`, `bridge_bolt_lost`, `bridge_bump_whiff`, `bridge_no_bump`, `bridge_cell_impact`, `bridge_breaker_impact`, `bridge_wall_impact`, `on_cell_destroyed`, `on_bolt_destroyed`, `on_wall_destroyed`, `on_breaker_destroyed`, `bridge_timer_threshold` |
| `EffectV3Systems::Tick` | `effect_v3/sets.rs` | tick systems for active effects (e.g. `tick_shockwave`, `tick_chain_lightning`) — runs after `Bridge` in FixedUpdate |
| `EffectV3Systems::Conditions` | `effect_v3/sets.rs` | condition evaluation systems (e.g. `evaluate_conditions`) — runs after `Tick` in FixedUpdate |
| `EffectV3Systems::Reset` | `effect_v3/sets.rs` | effect state reset on `OnEnter(NodeState::Loading)` — not in FixedUpdate chain |
| `DeathPipelineSystems::ApplyDamage` | `shared/death_pipeline/sets.rs` | `apply_damage::<Cell>`, `apply_damage::<Bolt>`, `apply_damage::<Wall>`, `apply_damage::<Breaker>` — phase set (see note) |
| `DeathPipelineSystems::DetectDeaths` | `shared/death_pipeline/sets.rs` | `detect_deaths::<Cell>`, `detect_deaths::<Bolt>`, `detect_deaths::<Wall>`, `detect_deaths::<Breaker>` — phase set |
| `DeathPipelineSystems::HandleKill` | `shared/death_pipeline/sets.rs` | `handle_kill::<Cell>`, `handle_kill::<Bolt>`, `handle_kill::<Wall>` from `DeathPipelinePlugin`; `handle_breaker_death` from `RunPlugin` — phase set |
| `UiSystems::SpawnTimerHud` | `state/run/node/hud/sets.rs` | `spawn_timer_hud` |
| `NodeSystems::TrackCompletion` | `state/run/node/sets.rs` | `track_node_completion` |
| `NodeSystems::TickTimer` | `state/run/node/sets.rs` | `tick_node_timer` |
| `NodeSystems::ApplyTimePenalty` | `state/run/node/sets.rs` | `apply_time_penalty` |
| `NodeSystems::Spawn` | `state/run/node/sets.rs` | `spawn_cells_from_layout` (OnEnter) |
| `NodeSystems::InitTimer` | `state/run/node/sets.rs` | `init_node_timer` (OnEnter) |
| `NodeSystems::Cleanup` | `state/run/node/sets.rs` | `cleanup_on_exit::<NodeState>` (OnEnter Teardown — effect bridges that need a cleaned-up world order `.after(NodeSystems::Cleanup)`) |

**Example:**

```rust
// In breaker/sets.rs
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BreakerSystems {
    Move,   // The move_breaker system
    Reset,  // The reset_breaker system
}

// In breaker/plugin.rs — tag the system
move_breaker.in_set(BreakerSystems::Move)
reset_breaker.in_set(BreakerSystems::Reset)

// In bolt/plugin.rs — order against it
hover_bolt.after(BreakerSystems::Move)
```

## Current Ordering Chain

The actual cross-domain ordering constraints in the codebase:

### OnEnter(NodeState::AnimateIn)

```
begin_node_birthing                      [bolt domain — inserts Birthing on all bolt entities, zeroes Scale2D/PreviousScale/CollisionLayers]
```

### OnEnter(NodeState::Loading)

```
spawn_or_reuse_breaker                   [breaker domain — Breaker::builder() via registry]
  <- apply_node_scale_to_breaker
       .after(spawn_or_reuse_breaker)
       .after(NodeSystems::Spawn)        [breaker domain]
  <- reset_breaker .after(spawn_or_reuse_breaker)
       BreakerSystems::Reset             [breaker domain]
  <- UiSystems::SpawnTimerHud
       (spawn_timer_hud)                 [state/run/node/hud domain]

NodeSystems::Spawn
  (spawn_cells_from_layout)             [state/run/node domain — OnEnter]
    <- dispatch_cell_effects .after(NodeSystems::Spawn)      [cells domain]

spawn_walls                                                  [walls domain — Wall::builder() via WallRegistry]

spawn_bolt                               [bolt domain — uses Bolt::builder() + BoltRegistry]
    <- apply_node_scale_to_bolt
             .after(spawn_bolt)
             .after(NodeSystems::Spawn)  [bolt domain]
    <- reset_bolt .after(spawn_bolt)
                  .after(BreakerSystems::Reset)
       BoltSystems::Reset              [bolt domain]
```

Note: `spawn_or_reuse_breaker` is a single system that replaces the old 4-system chain (`spawn_breaker` → `init_breaker_params` → `init_breaker` → `dispatch_breaker_effects`). All components are emitted by `Breaker::builder()` in one call; effects are dispatched via `dispatch_initial_effects` command. `reset_bolt` is the last OnEnter system — it waits for both breaker reset and bolt init. `dispatch_cell_effects` runs after `NodeSystems::Spawn` to ensure cells are present before effects are dispatched. `spawn_walls` reads from `WallRegistry`, calls `Wall::builder()` three times (left, right, ceiling), and dispatches effects via `commands.stamp_effect` (one call per `RootNode::Stamp` in the wall definition's `effects: Vec<RootNode>`) — the old `dispatch_wall_effects` stub was removed. `dispatch_bolt_effects` runs in FixedUpdate (not OnEnter) — it processes `Added<BoltDefinitionRef>` each tick, so it first fires the frame after the bolt spawns.

### FixedUpdate

```
tick_birthing                            [bolt domain — lerps Scale2D from zero → target; restores CollisionLayers on completion; removes Birthing]
  .run_if(in_state(NodeState::AnimateIn).or(in_state(NodeState::Playing)))
  [unordered — runs independently of physics chain]

dispatch_bolt_effects .before(EffectV3Systems::Bridge)
  [bolt domain — processes Added<BoltDefinitionRef> each FixedUpdate tick]

rantzsoft_physics2d::PhysicsSystems::MaintainQuadtree
  (maintain_quadtree)           [rantzsoft_physics2d — incremental spatial index update]
    <- bolt_cell_collision .after(rantzsoft_physics2d::PhysicsSystems::MaintainQuadtree)
                           .after(rantzsoft_physics2d::PhysicsSystems::EnforceDistanceConstraints)
                           .after(BreakerSystems::Move)
                           .before(EffectV3Systems::Bridge)
    <- shockwave_collision .after(tick_shockwave)
                           .after(rantzsoft_physics2d::PhysicsSystems::MaintainQuadtree)

move_breaker .after(update_bump)
  BreakerSystems::Move
    <- hover_bolt .after(BreakerSystems::Move)
    <- normalize_bolt_speed_after_constraints                [bolt domain — normalizes speed post-constraint enforcement]
            <- bolt_cell_collision .after(rantzsoft_physics2d::PhysicsSystems::MaintainQuadtree)
                                   .after(normalize_bolt_speed_after_constraints)
                                   .before(EffectV3Systems::Bridge)
              BoltSystems::CellCollision
                <- bolt_wall_collision .after(BoltSystems::CellCollision)
                                       .before(EffectV3Systems::Bridge)
                  BoltSystems::WallCollision
                <- bolt_breaker_collision .after(BoltSystems::CellCollision)
                                          .before(EffectV3Systems::Bridge)
                  BoltSystems::BreakerCollision
            <- grade_bump .after(update_bump)
                          .after(BoltSystems::BreakerCollision)
              BreakerSystems::GradeBump
                <- (perfect_bump_dash_cancel, spawn_bump_grade_text, spawn_whiff_text) .after(grade_bump)
                <- bridge_bump .after(BreakerSystems::GradeBump)
                   .in_set(EffectV3Systems::Bridge)              [effect domain]
                <- bridge_bump_whiff .after(BreakerSystems::GradeBump)
                   .in_set(EffectV3Systems::Bridge)              [effect domain]
                <- bridge_no_bump .after(bridge_breaker_impact).after(bridge_bump)
                   .in_set(EffectV3Systems::Bridge)              [effect domain]
                <- bridge_cell_impact .after(BoltSystems::BreakerCollision)
                   .in_set(EffectV3Systems::Bridge)              [effect domain]
                <- bridge_breaker_impact .after(BoltSystems::BreakerCollision)
                   .in_set(EffectV3Systems::Bridge)              [effect domain]
                <- bridge_wall_impact .after(BoltSystems::BreakerCollision)
                   .in_set(EffectV3Systems::Bridge)              [effect domain]
            <- clamp_bolt_to_playfield .after(bolt_breaker_collision)
            <- enforce_distance_constraints .after(clamp_bolt_to_playfield)  [bolt domain]
            <- bolt_lost .after(enforce_distance_constraints)
                         .before(EffectV3Systems::Bridge)
              BoltSystems::BoltLost
                <- bridge_bolt_lost .after(BoltSystems::BoltLost)
                   .in_set(EffectV3Systems::Bridge)          [effect domain]
                <- break_chain_on_bolt_lost .after(BoltSystems::BoltLost)  [bolt domain]
            <- on_cell_destroyed .in_set(EffectV3Systems::Bridge)
               [effect domain, unordered relative to physics chain]
            <- on_bolt_destroyed .in_set(EffectV3Systems::Bridge)
               [effect domain, unordered relative to physics chain]
            <- on_wall_destroyed .in_set(EffectV3Systems::Bridge)
               [effect domain, unordered relative to physics chain]
            <- on_breaker_destroyed .in_set(EffectV3Systems::Bridge)
               [effect domain, unordered relative to physics chain]
            <- bridge_timer_threshold .in_set(EffectV3Systems::Bridge)
               [effect domain, unordered relative to physics chain]

DeathPipelineSystems::ApplyDamage
  .after(EffectV3Systems::Tick)                              [shared/death_pipeline domain]
  (apply_damage::<Cell>, apply_damage::<Bolt>, apply_damage::<Wall>, apply_damage::<Breaker>)
    <- update_cell_damage_visuals .after(DeathPipelineSystems::ApplyDamage)
                                  .before(DeathPipelineSystems::HandleKill) [cells domain]
    <- track_evolution_damage .after(DeathPipelineSystems::ApplyDamage)     [run domain]
    <- DeathPipelineSystems::DetectDeaths .after(DeathPipelineSystems::ApplyDamage)
       (detect_deaths::<Cell>, detect_deaths::<Bolt>, detect_deaths::<Wall>, detect_deaths::<Breaker>)
         <- DeathPipelineSystems::HandleKill .after(DeathPipelineSystems::DetectDeaths)
            (handle_kill::<Cell>, handle_kill::<Bolt>, handle_kill::<Wall>, handle_breaker_death)
              <- track_cells_destroyed .after(DeathPipelineSystems::HandleKill)  [run domain]
              <- detect_mass_destruction .after(DeathPipelineSystems::HandleKill) [run/node domain]
              <- detect_combo_king .after(DeathPipelineSystems::HandleKill)       [run/node domain]
              <- check_lock_release .after(DeathPipelineSystems::HandleKill)      [cells domain]
              <- NodeSystems::TrackCompletion .after(DeathPipelineSystems::HandleKill)
                 (track_node_completion)                                          [run/node domain]

tick_bolt_lifespan .before(BoltSystems::BoltLost)
                   .before(DeathPipelineSystems::HandleKill)  [bolt domain — writes KillYourself<Bolt> on timer expiry]
```

Reading: the quadtree is maintained first (incremental — only changed entities re-inserted). Consumers read `Active*` components directly via `.multiplier()` / `.total()` methods. Then breaker moves, speed is normalized post-constraint (`normalize_bolt_speed_after_constraints`), cell collisions run (tagged `BoltSystems::CellCollision`), wall collision (`BoltSystems::WallCollision`), breaker collision (`BoltSystems::BreakerCollision`), bump grading (`BreakerSystems::GradeBump`), distance constraints enforced (chain bolts), bolt-lost detection (`BoltSystems::BoltLost`). All collision systems run `.before(EffectV3Systems::Bridge)` so damage messages are present when bridges evaluate. Velocity is enforced by `apply_velocity_formula` at each collision/steering site — there is no separate velocity preparation step. All effect bridge systems run in `EffectV3Systems::Bridge`. After Bridge, the death pipeline runs: `DeathPipelineSystems::ApplyDamage` (reads `DamageDealt<T>`, reduces `Hp`) → `DetectDeaths` (reads `Hp`, writes `KillYourself<T>`) → `HandleKill` (marks `Dead`, writes `Destroyed<T>` + `DespawnEntity`). `update_cell_damage_visuals` runs after `ApplyDamage` and before `HandleKill` to update color feedback on still-living cells. Consumers of kill events (track_cells_destroyed, detect_mass_destruction, detect_combo_king, check_lock_release, track_node_completion) order `.after(DeathPipelineSystems::HandleKill)`. `process_despawn_requests` runs in `FixedPostUpdate` — sole despawn site. The full effect pipeline order within FixedUpdate is: `EffectV3Systems::Bridge` → `EffectV3Systems::Tick` → `EffectV3Systems::Conditions`. `EffectV3Systems::Reset` runs on `OnEnter(NodeState::Loading)` — not in the FixedUpdate chain.

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

### FixedPostUpdate

```
process_despawn_requests   [shared/death_pipeline — sole despawn site; reads DespawnEntity messages written by handle_kill::<T> in FixedUpdate]
```

Reading: `DespawnEntity` messages are written during `HandleKill` in `FixedUpdate`. Deferring despawn to `FixedPostUpdate` ensures all `FixedUpdate` consumers (kill-event readers, effect bridges, node tracking) see the entity before it is removed from the world.

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

### Transition Lifecycle (rantzsoft_stateflow)

`TransitionOut` and `TransitionIn` are no longer game states. Screen transitions are managed entirely by the `rantzsoft_stateflow` crate. Routes declare transition effects via `.with_transition(TransitionType::Out(...))` etc. The lifecycle crate's `orchestrate_transitions` system (runs in `Update`) drives the start/run/end phase progression using `StartingTransition<T>` / `RunningTransition<T>` / `EndingTransition<T>` resources as gating conditions.

The `fx` domain's `spawn_transition_out`, `spawn_transition_in`, `cleanup_transition`, and `animate_transition` systems have been removed. No game-side ordering constraints exist for transitions — they are internal to `rantzsoft_stateflow`.

### OnExit(MenuState::Main)

```
reset_run_state                                             [state/run domain]
  <- generate_node_sequence_system .after(reset_run_state) [state/run domain]
```

Reading: run state is reset and RNG is reseeded first, then the node sequence is generated from the freshly seeded `GameRng`.

**Intra-domain constraints (breaker):** `update_bump` → `move_breaker` (`BreakerSystems::Move`) → `update_breaker_state` (`BreakerSystems::UpdateState`, with `(perfect_bump_dash_cancel, spawn_bump_grade_text, spawn_whiff_text)` also before `UpdateState`); `grade_bump` runs `.after(update_bump).after(BoltSystems::BreakerCollision)` — it is NOT after `update_breaker_state`. `trigger_bump_visual` also runs `.after(update_bump)`.

## Schedule Placement

- **FixedUpdate** — all gameplay and physics systems. Required for deterministic, seed-reproducible behavior.
- **Update** — visual-only systems (interpolation, UI rendering, shader updates). No gameplay state mutation.
- **OnEnter / OnExit** — state transition setup and cleanup (spawning, despawning, resource initialization).
