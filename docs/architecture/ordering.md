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
| `PhysicsSystems::BreakerCollision` | `physics/sets.rs` | `bolt_breaker_collision` |
| `PhysicsSystems::BoltLost` | `physics/sets.rs` | `bolt_lost` |
| `BehaviorSystems::Bridge` | `behaviors/sets.rs` | `bridge_bump`, `bridge_bolt_lost`, `bridge_bump_whiff` |
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
apply_archetype_config_overrides       [behaviors domain]
  .before(BreakerSystems::InitParams)
    BreakerSystems::InitParams
    (init_breaker_params)              [breaker domain]
      <- init_archetype .after(BreakerSystems::InitParams)   [behaviors domain]
      <- reset_breaker .after(BreakerSystems::InitParams)
         BreakerSystems::Reset                                [breaker domain]
      <- UiSystems::SpawnTimerHud
         (spawn_timer_hud)             [ui domain]
           <- spawn_lives_display .after(init_archetype)
                                  .after(UiSystems::SpawnTimerHud)  [behaviors domain]

spawn_bolt → init_bolt_params          [bolt domain, .after(spawn_bolt)]
  BoltSystems::InitParams
    <- reset_bolt .after(BoltSystems::InitParams)
                  .after(BreakerSystems::Reset)
       BoltSystems::Reset              [bolt domain]
```

Note: `spawn_breaker` → `ApplyDeferred` → `init_breaker_params` are chained inside the breaker plugin. `spawn_side_panels` + `ApplyDeferred` + `spawn_timer_hud` are chained inside the UI plugin, so `UiSystems::SpawnTimerHud` is the externally-visible anchor. `reset_bolt` is the last OnEnter system — it waits for both breaker reset and bolt init.

### FixedUpdate

```
BreakerSystems::Move
  <- (hover_bolt, prepare_bolt_velocity) .after(BreakerSystems::Move)
    BoltSystems::PrepareVelocity
      <- bolt_cell_collision .after(BoltSystems::PrepareVelocity)
        <- bolt_breaker_collision .after(bolt_cell_collision)
          PhysicsSystems::BreakerCollision
            <- apply_bump_velocity .after(PhysicsSystems::BreakerCollision)
                                   .before(PhysicsSystems::BoltLost)
            <- grade_bump .after(update_bump)
                          .after(PhysicsSystems::BreakerCollision)
              BreakerSystems::GradeBump
                <- (perfect_bump_dash_cancel, spawn_bump_grade_text, spawn_whiff_text) .after(grade_bump)
                <- bridge_bump .after(BreakerSystems::GradeBump)
                   .in_set(BehaviorSystems::Bridge)              [behaviors domain]
                <- bridge_bump_whiff .after(BreakerSystems::GradeBump)
                   .in_set(BehaviorSystems::Bridge)              [behaviors domain]
            <- clamp_bolt_to_playfield .after(bolt_breaker_collision)
            <- bolt_lost .after(clamp_bolt_to_playfield)
              PhysicsSystems::BoltLost
                <- bridge_bolt_lost .after(PhysicsSystems::BoltLost)
                   .in_set(BehaviorSystems::Bridge)          [behaviors domain]
            <- spawn_additional_bolt .after(BehaviorSystems::Bridge)  [bolt domain]
```

Reading: breaker moves first, then bolt velocity is prepared, then cell collisions run, then breaker collision, then bump grading (`BreakerSystems::GradeBump`) and velocity application, then bolt-lost detection. All three behavior bridge systems (`bridge_bump`, `bridge_bump_whiff`, `bridge_bolt_lost`) run in `BehaviorSystems::Bridge` (exported from `behaviors/sets.rs`) — downstream consumers order `.after(BehaviorSystems::Bridge)`.

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

### OnExit(GameState::MainMenu)

```
reset_run_state                                             [run domain]
  <- generate_node_sequence_system .after(reset_run_state) [run domain]
```

Reading: run state is reset and RNG is reseeded first, then the node sequence is generated from the freshly seeded `GameRng`.

**Intra-domain constraints (breaker):** `update_bump` → `move_breaker` → `update_breaker_state` (one chain); `grade_bump` runs `.after(update_bump).after(PhysicsSystems::BreakerCollision)` — it is NOT after `update_breaker_state`. `trigger_bump_visual` also runs `.after(update_bump)`.

## Schedule Placement

- **FixedUpdate** — all gameplay and physics systems. Required for deterministic, seed-reproducible behavior.
- **Update** — visual-only systems (interpolation, UI rendering, shader updates). No gameplay state mutation.
- **OnEnter / OnExit** — state transition setup and cleanup (spawning, despawning, resource initialization).
