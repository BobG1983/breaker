//! `EchoCells` hazard — every destroyed cell leaves a ghost after
//! `delay_secs` seconds. Ghosts are low-HP `Cell` entities marked with
//! [`GhostCell`]; they carry no original cell rules. HP scales
//! geometrically per stack (`base_hp * multiplier^(stacks - 1)`).
//!
//! Ghost deaths do NOT spawn new ghosts — the `GhostCell` marker
//! filters them out in the tracker.

use bevy::prelude::*;

use crate::{
    cells::components::{Cell, CellHeight, CellWidth},
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
    shared::{
        collision_layers::{BOLT_LAYER, CELL_LAYER},
        death_pipeline::{Destroyed, Hp, KilledBy},
    },
};

/// Standard ghost dimensions — matches the default cell footprint used
/// across the project so ghosts collide like normal cells.
const GHOST_WIDTH: f32 = 70.0;
const GHOST_HEIGHT: f32 = 24.0;

/// Per-run tuning extracted from [`HazardTuning::EchoCells`].
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct EchoCellsConfig {
    /// Seconds between cell death and ghost spawn. Fixed — does not
    /// scale with stacks (intentional, per design).
    pub(crate) delay_secs:           f32,
    /// Ghost HP at stack 1.
    pub(crate) base_hp:              f32,
    /// Per-stack HP multiplier applied geometrically.
    pub(crate) per_level_multiplier: f32,
}

impl EchoCellsConfig {
    /// Ghost HP for the given stack count. Returns `0.0` when
    /// `stacks == 0` (hazard inactive / identity). For `stacks >= 1`,
    /// returns `base_hp * per_level_multiplier^(stacks - 1)` — geometric
    /// scaling in `stacks`, so stack 1 yields `base_hp`, stack 2 yields
    /// `base_hp * per_level_multiplier`, and so on. `saturating_sub` on
    /// the exponent prevents underflow when `stacks == 1` (extra = 0,
    /// so the multiplier term is `1.0` and the result is exactly
    /// `base_hp`). `#[must_use]` because callers must not drop the
    /// computed HP — it feeds directly into the ghost's `Hp`
    /// component. No upper cap — unlike Fracture's `splits_for`, ghost
    /// HP grows unboundedly with stack count, by design (the
    /// exponential curve is the steepest of all hazards).
    #[must_use]
    pub(crate) fn ghost_hp(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1).cast_signed();
        self.base_hp * self.per_level_multiplier.powi(extra)
    }
}

/// Marker entity spawned when a non-ghost cell dies. Carries the target
/// position and a countdown until the ghost materialises.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct PendingGhost {
    pub(crate) position: Vec2,
    pub(crate) timer:    f32,
}

/// Marks a spawned ghost cell. Prevents recursive ghost spawning and
/// lets other systems identify ghosts when needed.
#[derive(Component, Debug, Default, Clone, Copy)]
pub(crate) struct GhostCell;

/// Inserts `EchoCellsConfig` from `HazardTuning::EchoCells { delay_secs,
/// base_hp, per_level_multiplier }`. Called each time the player picks
/// Echo Cells from `hazards::activate`; last write wins (overwrites any
/// prior `EchoCellsConfig`; stack count is owned by `ActiveHazards`, not
/// the config; `PendingGhost` markers already in flight retain their
/// `timer` and `position` — the config only governs future ghost
/// spawns, i.e. the delay applied to new markers and the HP of future
/// ghost cells). Does not mutate `ActiveHazards`. Warns and no-ops on a
/// non-`EchoCells` tuning variant, leaving any existing
/// `EchoCellsConfig` intact.
pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::EchoCells {
        delay_secs,
        base_hp,
        per_level_multiplier,
    } = *tuning
    else {
        warn!("echo_cells::activate called with non-EchoCells tuning");
        return;
    };
    commands.insert_resource(EchoCellsConfig {
        delay_secs,
        base_hp,
        per_level_multiplier,
    });
}

/// Registers the two Echo Cells `FixedUpdate` systems
/// `echo_cells_track_deaths` and `echo_cells_spawn_ghosts`, chained in
/// that order via `.chain()`, both gated on
/// `hazard_active(HazardKind::EchoCells)` AND
/// `in_state(NodeState::Playing)`. `echo_cells_track_deaths` reads
/// `Destroyed<Cell>` and spawns `PendingGhost` markers for non-ghost
/// victims. `echo_cells_spawn_ghosts` ticks those markers' timers and
/// spawns ghost `Cell` entities (with `GhostCell` marker) directly via
/// `commands.spawn(...)`, not via a `SpawnGhostCell` message — matching
/// the shipped Fracture pattern of spawning debris directly. No
/// `DeathPipelineSystems` ordering is applied, because ghost spawns are
/// raw `commands.spawn` calls rather than death-pipeline emissions.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (echo_cells_track_deaths, echo_cells_spawn_ghosts)
            .chain()
            .run_if(hazard_active(HazardKind::EchoCells))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Reads `Destroyed<Cell>`; for each destroyed non-`GhostCell` victim,
/// spawns a `PendingGhost` marker carrying the victim's `victim_pos` and
/// `config.delay_secs` as the countdown. Victims that carry `GhostCell`
/// are skipped — ghost deaths must not recurse into new ghosts, so the
/// marker is not emitted for them. Early-returns (draining the message
/// reader via `reader.clear()`) when `EchoCellsConfig` is absent OR
/// `config.delay_secs <= 0.0` — the drain prevents stale
/// `Destroyed<Cell>` messages from spawning pending ghosts on a later
/// tick after the hazard activates or raises its delay.
fn echo_cells_track_deaths(
    mut reader: MessageReader<Destroyed<Cell>>,
    config: Option<Res<EchoCellsConfig>>,
    ghosts: Query<(), With<GhostCell>>,
    mut commands: Commands,
) {
    let Some(config) = config else {
        reader.clear();
        return;
    };
    if config.delay_secs <= 0.0 {
        reader.clear();
        return;
    }
    for destroyed in reader.read() {
        if ghosts.contains(destroyed.victim) {
            // Ghost deaths must not recurse.
            continue;
        }
        commands.spawn((
            PendingGhost {
                position: destroyed.victim_pos,
                timer:    config.delay_secs,
            },
            // Node-end cleanup: in-flight `PendingGhost` entities do NOT
            // carry `Cell`, so unlike spawned ghosts they do not inherit
            // `CleanupOnExit<NodeState>` via `#[require]`. Must be added
            // explicitly or they leak across `NodeState` transitions.
            CleanupOnExit::<NodeState>::default(),
        ));
    }
}

/// Ticks every `PendingGhost.timer` by `time.delta_secs()` each
/// `FixedUpdate` frame. When a timer reaches `<= 0.0`, computes
/// `hp = config.ghost_hp(active.stacks(HazardKind::EchoCells))` once
/// before iterating and, if `hp > 0.0`, spawns a new entity with
/// `Cell`, `GhostCell`, `Position2D` at the marker's stored position,
/// `Scale2D { x: GHOST_WIDTH, y: GHOST_HEIGHT }`,
/// `Aabb2D::new(Vec2::ZERO, Vec2::new(GHOST_WIDTH / 2.0,
/// GHOST_HEIGHT / 2.0))`, `CollisionLayers::new(CELL_LAYER,
/// BOLT_LAYER)`, `CellWidth::new(GHOST_WIDTH)`,
/// `CellHeight::new(GHOST_HEIGHT)`, `Hp::new(hp)`, and
/// `KilledBy::default()`. The `PendingGhost` marker is despawned in
/// either branch (even when `hp == 0.0`) — the marker is always
/// consumed on expiry so it does not retry on the next frame.
/// Early-returns when `EchoCellsConfig` is absent; no reader drain is
/// needed here, unlike the tracker, because this system does not read
/// a `MessageReader`. `CleanupOnExit<NodeState>` is NOT listed in the
/// spawn tuple — `Cell` carries `#[require(Spatial2D,
/// CleanupOnExit<NodeState>)]`, so ghosts inherit node-exit cleanup
/// automatically.
fn echo_cells_spawn_ghosts(
    time: Res<Time<Fixed>>,
    active: Res<ActiveHazards>,
    config: Option<Res<EchoCellsConfig>>,
    mut pending: Query<(Entity, &mut PendingGhost)>,
    mut commands: Commands,
) {
    let Some(config) = config else { return };
    let stacks = active.stacks(HazardKind::EchoCells);
    let hp = config.ghost_hp(stacks);
    let dt = time.delta_secs();

    for (entity, mut p) in &mut pending {
        p.timer -= dt;
        if p.timer > 0.0 {
            continue;
        }
        if hp > 0.0 {
            commands.spawn((
                Cell,
                GhostCell,
                Position2D(p.position),
                Scale2D {
                    x: GHOST_WIDTH,
                    y: GHOST_HEIGHT,
                },
                Aabb2D::new(Vec2::ZERO, Vec2::new(GHOST_WIDTH / 2.0, GHOST_HEIGHT / 2.0)),
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                CellWidth::new(GHOST_WIDTH),
                CellHeight::new(GHOST_HEIGHT),
                Hp::new(hp),
                KilledBy::default(),
            ));
        }
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use std::{marker::PhantomData, time::Duration};

    use bevy::{ecs::world::CommandQueue, prelude::Messages};
    use rantzsoft_stateflow::CleanupOnExit;

    use super::*;
    use crate::prelude::TestAppBuilder;

    fn test_app_playing() -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_resource::<ActiveHazards>()
            .with_message::<Destroyed<Cell>>()
            .build()
    }

    fn write_destroyed(app: &mut App, victim: Entity, pos: Vec2) {
        app.world_mut()
            .resource_mut::<Messages<Destroyed<Cell>>>()
            .write(Destroyed::<Cell> {
                victim,
                killer: None,
                victim_pos: pos,
                killer_pos: None,
                _marker: PhantomData,
            });
    }

    fn tick_with_dt(app: &mut App, dt: Duration) {
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .set_timestep(dt);
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(dt);
        app.update();
    }

    // ── New shared helpers (Fracture precedent) ───────────────────────────

    fn test_app_not_playing() -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .with_resource::<ActiveHazards>()
            .with_message::<Destroyed<Cell>>()
            .build()
    }

    const fn canonical_config() -> EchoCellsConfig {
        EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        }
    }

    fn install_echo_cells_config(app: &mut App, cfg: EchoCellsConfig) {
        app.world_mut().insert_resource(cfg);
    }

    fn add_echo_cells_stacks(app: &mut App, count: u32) {
        let mut active = app.world_mut().resource_mut::<ActiveHazards>();
        for _ in 0..count {
            active.add_stack(HazardKind::EchoCells);
        }
    }

    fn activate_now(app: &mut App, tuning: &HazardTuning) {
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, app.world());
            activate(tuning, &mut commands);
        }
        queue.apply(app.world_mut());
    }

    // ── ghost_hp formula ──────────────────────────────────────────────────

    #[test]
    fn ghost_hp_zero_stacks_is_zero() {
        let cfg = EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        };
        assert!((cfg.ghost_hp(0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ghost_hp_stack_one_is_base() {
        let cfg = EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        };
        assert!((cfg.ghost_hp(1) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn ghost_hp_doubles_per_stack() {
        let cfg = EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        };
        assert!((cfg.ghost_hp(3) - 4.0).abs() < 1e-5);
        assert!((cfg.ghost_hp(5) - 16.0).abs() < 1e-4);
    }

    // ── echo_cells_track_deaths ───────────────────────────────────────────

    #[test]
    fn death_creates_pending_ghost() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        app.world_mut().insert_resource(EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        });
        let victim = app.world_mut().spawn(Cell).id();
        write_destroyed(&mut app, victim, Vec2::new(100.0, 200.0));

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&PendingGhost>();
        let pendings: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(pendings.len(), 1);
        assert!((pendings[0].position - Vec2::new(100.0, 200.0)).length() < 1e-5);
        assert!((pendings[0].timer - 1.5).abs() < 1e-5);
    }

    #[test]
    fn ghost_death_does_not_create_pending() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        app.world_mut().insert_resource(EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        });
        let ghost_victim = app.world_mut().spawn((Cell, GhostCell)).id();
        write_destroyed(&mut app, ghost_victim, Vec2::ZERO);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&PendingGhost>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    #[test]
    fn multiple_deaths_create_multiple_pendings() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        app.world_mut().insert_resource(EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        });
        for i in 0..3 {
            let victim = app.world_mut().spawn(Cell).id();
            write_destroyed(&mut app, victim, Vec2::new(i as f32 * 100.0, 0.0));
        }
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&PendingGhost>();
        assert_eq!(query.iter(app.world()).count(), 3);
    }

    // ── echo_cells_spawn_ghosts ───────────────────────────────────────────

    #[test]
    fn ghost_spawns_after_delay() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        app.world_mut().insert_resource(EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::EchoCells);
        let pending = app
            .world_mut()
            .spawn(PendingGhost {
                position: Vec2::new(50.0, 75.0),
                timer:    0.05,
            })
            .id();

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        // PendingGhost is despawned.
        assert!(app.world().get_entity(pending).is_err());
        // Ghost cell spawned at the recorded position.
        let mut query = app.world_mut().query::<(&GhostCell, &Position2D, &Hp)>();
        let ghosts: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(ghosts.len(), 1);
        let (_, pos, hp) = ghosts[0];
        assert!((pos.0 - Vec2::new(50.0, 75.0)).length() < 1e-4);
        assert!((hp.current - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn ghost_hp_reflects_current_stacks() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        app.world_mut().insert_resource(EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        });
        for _ in 0..3 {
            app.world_mut()
                .resource_mut::<ActiveHazards>()
                .add_stack(HazardKind::EchoCells);
        }
        app.world_mut().spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    0.05,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let mut query = app.world_mut().query::<(&GhostCell, &Hp)>();
        let ghosts: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(ghosts.len(), 1);
        // 1 * 2^2 = 4
        assert!((ghosts[0].1.current - 4.0).abs() < 1e-5);
    }

    #[test]
    fn ghost_does_not_spawn_before_timer_expires() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        app.world_mut().insert_resource(EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              1.0,
            per_level_multiplier: 2.0,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::EchoCells);
        app.world_mut().spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    1.0,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let mut query = app.world_mut().query::<&GhostCell>();
        assert_eq!(query.iter(app.world()).count(), 0);
        let mut q = app.world_mut().query::<&PendingGhost>();
        let remaining = q.iter(app.world()).next().unwrap().timer;
        assert!((remaining - 0.9).abs() < 1e-5);
    }

    // ── activate — preserved scaffold tests ────────────────────────────────

    #[test]
    fn activate_with_matching_tuning_inserts_config() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &HazardTuning::EchoCells {
                    delay_secs:           2.0,
                    base_hp:              1.0,
                    per_level_multiplier: 2.0,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<EchoCellsConfig>();
        assert!((cfg.delay_secs - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn activate_with_mismatched_tuning_does_nothing() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &HazardTuning::Decay {
                    base_percent:      0.05,
                    per_level_percent: 0.03,
                },
                &mut commands,
            );
        });
        app.update();
        assert!(app.world().get_resource::<EchoCellsConfig>().is_none());
    }

    // ── A. ghost_hp formula — extended coverage ───────────────────────────

    // Behavior 1 — stack 2 is exactly base * multiplier.
    #[test]
    fn ghost_hp_stack_two_is_exactly_base_times_multiplier() {
        let cfg = canonical_config();
        assert!((cfg.ghost_hp(2) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn ghost_hp_stack_two_with_base_five_multiplier_two_is_ten() {
        let cfg = EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              5.0,
            per_level_multiplier: 2.0,
        };
        assert!((cfg.ghost_hp(2) - 10.0).abs() < 1e-5);
    }

    // Behavior 2 — non-doubling multiplier scales geometrically.
    #[test]
    fn ghost_hp_with_non_doubling_multiplier_scales_correctly() {
        let cfg = EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              2.0,
            per_level_multiplier: 1.5,
        };
        // 2.0 * 1.5^2 = 4.5
        assert!((cfg.ghost_hp(3) - 4.5).abs() < 1e-4);
    }

    #[test]
    fn ghost_hp_with_identity_multiplier_is_constant_across_stacks() {
        let cfg = EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              3.0,
            per_level_multiplier: 1.0,
        };
        for k in [1u32, 2, 3, 5, 10] {
            assert!((cfg.ghost_hp(k) - 3.0).abs() < 1e-5, "k={k}");
        }
    }

    // Behavior 3 — sub-one multiplier decays.
    #[test]
    fn ghost_hp_with_sub_one_multiplier_decays() {
        let cfg = EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              4.0,
            per_level_multiplier: 0.5,
        };
        // 4.0 * 0.5^2 = 1.0
        assert!((cfg.ghost_hp(3) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn ghost_hp_with_zero_multiplier_at_stack_two_is_zero() {
        let cfg = EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              4.0,
            per_level_multiplier: 0.0,
        };
        // 4.0 * 0.0^1 = 0.0
        assert!(cfg.ghost_hp(2).abs() < f32::EPSILON);
    }

    // Behavior 4 — zero base HP is always zero.
    #[test]
    fn ghost_hp_zero_base_is_always_zero() {
        let cfg = EchoCellsConfig {
            delay_secs:           1.5,
            base_hp:              0.0,
            per_level_multiplier: 2.0,
        };
        for k in [0u32, 1, 2, 3, 5, 10] {
            assert!(cfg.ghost_hp(k).abs() < f32::EPSILON, "k={k}");
        }
    }

    // Behavior 5 — u32::MAX wraps via .cast_signed() to a finite value.
    #[test]
    fn ghost_hp_u32_max_wraps_via_cast_signed() {
        // stacks.saturating_sub(1) = u32::MAX-1 = 4294967294u32;
        // .cast_signed() bit-reinterprets to -2i32; 2.0.powi(-2) = 0.25.
        let cfg = canonical_config();
        let r = cfg.ghost_hp(u32::MAX);
        assert!(r.is_finite());
        assert!((r - 0.25).abs() < 1e-4);
    }

    // Behavior 5a — overflow pin: beyond the f32 threshold, powi saturates to +INF.
    #[test]
    fn ghost_hp_overflows_to_infinity_beyond_f32_max() {
        let cfg = canonical_config();
        // 2.0^128 ≈ 3.4e38 is above f32::MAX → powi saturates to +INF.
        let r = cfg.ghost_hp(129);
        assert!(r.is_infinite());
        assert!(r > 0.0);
    }

    // Behavior 5b — paired finite bracket: one stack below the threshold stays in-range.
    #[test]
    fn ghost_hp_at_overflow_threshold_minus_one_is_finite() {
        let cfg = canonical_config();
        // 2.0^127 ≈ 1.7e38 is within f32::MAX.
        let r = cfg.ghost_hp(128);
        assert!(r.is_finite());
        assert!(r > 0.0);
    }

    // Behavior 6 — Copy semantics allow repeated calls.
    #[test]
    fn ghost_hp_copy_semantics_allow_repeated_call() {
        let cfg = canonical_config();
        let a = cfg.ghost_hp(3);
        let b = cfg.ghost_hp(3);
        assert!((a - 4.0).abs() < 1e-5);
        assert!((b - 4.0).abs() < 1e-5);
        // cfg still usable afterwards — pins #[derive(Clone, Copy)].
        assert!((cfg.ghost_hp(1) - 1.0).abs() < 1e-5);
    }

    // ── B. echo_cells_track_deaths — extended coverage ────────────────────

    // Behavior 7 — no config → no pendings; reader drained.
    #[test]
    fn track_deaths_no_pending_when_config_absent() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        // NO EchoCellsConfig inserted.
        let victim = app.world_mut().spawn(Cell).id();
        write_destroyed(&mut app, victim, Vec2::new(10.0, 20.0));

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&PendingGhost>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    #[test]
    fn track_deaths_reader_drained_when_config_absent_then_config_installed_no_new_message() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        let victim = app.world_mut().spawn(Cell).id();
        write_destroyed(&mut app, victim, Vec2::new(10.0, 20.0));

        // First tick: no config, reader drained via reader.clear().
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        // Install config and tick with NO new message — reader was drained.
        install_echo_cells_config(&mut app, canonical_config());
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&PendingGhost>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    // Behavior 8 — delay_secs == 0.0 short-circuits.
    #[test]
    fn track_deaths_no_pending_when_delay_secs_is_zero() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        install_echo_cells_config(
            &mut app,
            EchoCellsConfig {
                delay_secs:           0.0,
                base_hp:              1.0,
                per_level_multiplier: 2.0,
            },
        );
        let victim = app.world_mut().spawn(Cell).id();
        write_destroyed(&mut app, victim, Vec2::ZERO);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&PendingGhost>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    #[test]
    fn track_deaths_reader_drained_when_delay_zero_then_delay_raised_no_new_message() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        install_echo_cells_config(
            &mut app,
            EchoCellsConfig {
                delay_secs:           0.0,
                base_hp:              1.0,
                per_level_multiplier: 2.0,
            },
        );
        let victim = app.world_mut().spawn(Cell).id();
        write_destroyed(&mut app, victim, Vec2::ZERO);

        // First tick: delay is 0, reader drained.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        // Raise the delay; tick with NO new message.
        install_echo_cells_config(&mut app, canonical_config());
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&PendingGhost>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    // Behavior 9 — negative delay short-circuits (pins <= 0.0 predicate).
    #[test]
    fn track_deaths_no_pending_when_delay_secs_is_negative() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        install_echo_cells_config(
            &mut app,
            EchoCellsConfig {
                delay_secs:           -0.5,
                base_hp:              1.0,
                per_level_multiplier: 2.0,
            },
        );
        let victim = app.world_mut().spawn(Cell).id();
        write_destroyed(&mut app, victim, Vec2::ZERO);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&PendingGhost>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    // Behavior 10 — pending timer seeded from config.delay_secs exactly.
    #[test]
    fn track_deaths_pending_timer_equals_delay_secs_exactly() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        install_echo_cells_config(
            &mut app,
            EchoCellsConfig {
                delay_secs:           2.5,
                base_hp:              1.0,
                per_level_multiplier: 2.0,
            },
        );
        let victim = app.world_mut().spawn(Cell).id();
        write_destroyed(&mut app, victim, Vec2::ZERO);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&PendingGhost>();
        let pendings: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(pendings.len(), 1);
        assert!((pendings[0].timer - 2.5).abs() < 1e-5);
    }

    // Behavior 11 — pending position matches victim_pos.
    #[test]
    fn track_deaths_pending_position_matches_victim_pos() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        install_echo_cells_config(&mut app, canonical_config());
        let victim = app.world_mut().spawn(Cell).id();
        write_destroyed(&mut app, victim, Vec2::new(-123.5, 456.25));

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&PendingGhost>();
        let pendings: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(pendings.len(), 1);
        assert!((pendings[0].position - Vec2::new(-123.5, 456.25)).length() < 1e-4);
    }

    #[test]
    fn track_deaths_pending_position_stays_finite_at_boundary_magnitude() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        install_echo_cells_config(&mut app, canonical_config());
        let victim = app.world_mut().spawn(Cell).id();
        let big = Vec2::new(1e6, -1e6);
        write_destroyed(&mut app, victim, big);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&PendingGhost>();
        let pendings: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(pendings.len(), 1);
        assert!(pendings[0].position.x.is_finite());
        assert!(pendings[0].position.y.is_finite());
        assert!((pendings[0].position - big).length() < 1e-1);
    }

    // Behavior 12 — three distinct positions yield three distinct pendings.
    #[test]
    fn track_deaths_three_distinct_positions_yield_three_distinct_pendings() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        install_echo_cells_config(&mut app, canonical_config());
        let expected = [
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(0.0, 100.0),
        ];
        for pos in expected {
            let victim = app.world_mut().spawn(Cell).id();
            write_destroyed(&mut app, victim, pos);
        }

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&PendingGhost>();
        let positions: Vec<Vec2> = query.iter(app.world()).map(|p| p.position).collect();
        assert_eq!(positions.len(), 3);
        for exp in &expected {
            assert!(
                positions.iter().any(|p| (*p - *exp).length() < 1e-4),
                "missing expected position {exp:?}"
            );
        }
    }

    // Behavior 13 — despawned victim entity: no panic, creates pending.
    // Known edge: a ghost whose entity has been reaped before the Destroyed
    // message is read loses its recursion-filter protection (see Behavior 16).
    // This test pins the "no panic + pending created" half of the contract
    // using a plain Cell (not a ghost).
    #[test]
    fn track_deaths_does_not_panic_when_victim_entity_is_despawned() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        install_echo_cells_config(&mut app, canonical_config());
        let victim = app.world_mut().spawn(Cell).id();
        write_destroyed(&mut app, victim, Vec2::ZERO);
        // Despawn the victim AFTER writing the message but BEFORE the tracker reads it.
        app.world_mut().despawn(victim);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        // ghosts.contains(stale_entity) returns false → PendingGhost created.
        let mut query = app.world_mut().query::<&PendingGhost>();
        let pendings: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(pendings.len(), 1);
        assert!((pendings[0].position - Vec2::ZERO).length() < 1e-4);
    }

    // Behavior 14 — PendingGhost marker entity carries no GhostCell.
    #[test]
    fn track_deaths_pending_ghost_carries_no_ghost_cell_marker() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        install_echo_cells_config(&mut app, canonical_config());
        let victim = app.world_mut().spawn(Cell).id();
        write_destroyed(&mut app, victim, Vec2::ZERO);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut pending_without_ghost = app
            .world_mut()
            .query_filtered::<Entity, (With<PendingGhost>, Without<GhostCell>)>();
        assert_eq!(pending_without_ghost.iter(app.world()).count(), 1);
        let mut ghosts = app.world_mut().query_filtered::<Entity, With<GhostCell>>();
        assert_eq!(ghosts.iter(app.world()).count(), 0);
    }

    // Behavior 15 — mixed ghost + plain deaths spawn only the plain one.
    #[test]
    fn track_deaths_mixed_ghost_and_plain_deaths_spawns_only_plain() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        install_echo_cells_config(&mut app, canonical_config());
        let plain = app.world_mut().spawn(Cell).id();
        let ghost = app.world_mut().spawn((Cell, GhostCell)).id();
        write_destroyed(&mut app, plain, Vec2::new(10.0, 0.0));
        write_destroyed(&mut app, ghost, Vec2::new(20.0, 0.0));

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&PendingGhost>();
        let pendings: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(pendings.len(), 1);
        assert!((pendings[0].position - Vec2::new(10.0, 0.0)).length() < 1e-4);
    }

    // Behavior 16 — despawned ghost entity loses its recursion-filter protection.
    // Known edge: Query::contains(despawned_entity) returns false, so the
    // tracker cannot tell the victim was a GhostCell → a PendingGhost is
    // created. This pins shipped behavior; see Open Question #2 in the test
    // spec for the follow-up design discussion.
    #[test]
    fn track_deaths_despawned_ghost_entity_still_filtered_as_non_ghost() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        install_echo_cells_config(&mut app, canonical_config());
        let ghost = app.world_mut().spawn((Cell, GhostCell)).id();
        write_destroyed(&mut app, ghost, Vec2::new(42.0, 0.0));
        // Despawn the ghost BEFORE the tracker runs — the Query::contains
        // recursion filter can no longer see it.
        app.world_mut().despawn(ghost);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&PendingGhost>();
        let pendings: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(pendings.len(), 1);
        assert!((pendings[0].position - Vec2::new(42.0, 0.0)).length() < 1e-4);
    }

    // ── C. echo_cells_spawn_ghosts — extended coverage ────────────────────

    // Behavior 17 — no config: early return leaves PendingGhost untouched.
    #[test]
    fn spawn_ghosts_no_spawn_when_config_absent() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        // NO EchoCellsConfig.
        add_echo_cells_stacks(&mut app, 1);
        let pending = app
            .world_mut()
            .spawn(PendingGhost {
                position: Vec2::ZERO,
                timer:    0.05,
            })
            .id();

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        // Pending entity preserved (unlike the hp <= 0.0 path which despawns it).
        assert!(app.world().get_entity(pending).is_ok());
        let mut query = app.world_mut().query::<&GhostCell>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    // Behavior 18 — zero stacks despawns pending without spawning ghost.
    #[test]
    fn spawn_ghosts_zero_stacks_despawns_pending_without_spawning_ghost() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(&mut app, canonical_config());
        // Zero stacks → ghost_hp(0) == 0.0 → hp <= 0.0 branch, pending despawned.
        app.world_mut().spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    0.05,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let mut ghosts = app.world_mut().query::<&GhostCell>();
        assert_eq!(ghosts.iter(app.world()).count(), 0);
        let mut pendings = app.world_mut().query::<&PendingGhost>();
        assert_eq!(pendings.iter(app.world()).count(), 0);
    }

    // Behavior 19 — zero base_hp despawns pending without spawning ghost.
    #[test]
    fn spawn_ghosts_zero_base_hp_despawns_pending_without_spawning_ghost() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(
            &mut app,
            EchoCellsConfig {
                delay_secs:           1.5,
                base_hp:              0.0,
                per_level_multiplier: 2.0,
            },
        );
        add_echo_cells_stacks(&mut app, 3);
        app.world_mut().spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    0.05,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let mut ghosts = app.world_mut().query::<&GhostCell>();
        assert_eq!(ghosts.iter(app.world()).count(), 0);
        let mut pendings = app.world_mut().query::<&PendingGhost>();
        assert_eq!(pendings.iter(app.world()).count(), 0);
    }

    // Behavior 20 — ghost cell carries the full component suite.
    #[test]
    fn spawn_ghosts_ghost_cell_carries_full_component_suite() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        app.world_mut().spawn(PendingGhost {
            position: Vec2::new(100.0, 50.0),
            timer:    0.05,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let mut query = app.world_mut().query::<(
            &Cell,
            &GhostCell,
            &Position2D,
            &Scale2D,
            &Aabb2D,
            &CollisionLayers,
            &CellWidth,
            &CellHeight,
            &Hp,
            &KilledBy,
        )>();
        assert_eq!(query.iter(app.world()).count(), 1);
    }

    // Behavior 21 — ghost CellWidth / CellHeight pinned to GHOST_* constants.
    #[test]
    fn spawn_ghosts_ghost_cell_width_and_height_are_seventy_and_twenty_four() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        app.world_mut().spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    0.05,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let mut query = app
            .world_mut()
            .query::<(&GhostCell, &CellWidth, &CellHeight)>();
        let dims: Vec<(f32, f32)> = query
            .iter(app.world())
            .map(|(_, cw, ch)| (cw.value, ch.value))
            .collect();
        assert_eq!(dims.len(), 1);
        assert!(
            (dims[0].0 - 70.0).abs() < f32::EPSILON,
            "width = {}",
            dims[0].0
        );
        assert!(
            (dims[0].1 - 24.0).abs() < f32::EPSILON,
            "height = {}",
            dims[0].1
        );
    }

    // Behavior 22 — ghost Scale2D matches dimensions.
    #[test]
    fn spawn_ghosts_ghost_scale_matches_dimensions() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        app.world_mut().spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    0.05,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let mut query = app.world_mut().query::<(&GhostCell, &Scale2D)>();
        let scales: Vec<Scale2D> = query.iter(app.world()).map(|(_, s)| *s).collect();
        assert_eq!(scales.len(), 1);
        assert!(
            (scales[0].x - 70.0).abs() < f32::EPSILON,
            "x = {}",
            scales[0].x
        );
        assert!(
            (scales[0].y - 24.0).abs() < f32::EPSILON,
            "y = {}",
            scales[0].y
        );
    }

    // Behavior 23 — ghost Aabb2D equals half-dimensions centered at Vec2::ZERO.
    #[test]
    fn spawn_ghosts_ghost_aabb_is_half_dimensions_centered() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        app.world_mut().spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    0.05,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let mut query = app.world_mut().query::<(&GhostCell, &Aabb2D)>();
        let aabbs: Vec<Aabb2D> = query.iter(app.world()).map(|(_, a)| *a).collect();
        assert_eq!(aabbs.len(), 1);
        assert_eq!(aabbs[0], Aabb2D::new(Vec2::ZERO, Vec2::new(35.0, 12.0)));
    }

    // Behavior 24 — ghost CollisionLayers == (CELL_LAYER, BOLT_LAYER).
    #[test]
    fn spawn_ghosts_ghost_collision_layers_are_cell_x_bolt() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        app.world_mut().spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    0.05,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let mut query = app.world_mut().query::<(&GhostCell, &CollisionLayers)>();
        let layers: Vec<CollisionLayers> = query.iter(app.world()).map(|(_, l)| *l).collect();
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0], CollisionLayers::new(CELL_LAYER, BOLT_LAYER));
    }

    // Behavior 25 — ghost Hp.current == Hp.starting; max.is_none().
    #[test]
    fn spawn_ghosts_ghost_hp_current_equals_starting() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 3);
        app.world_mut().spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    0.05,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let mut query = app.world_mut().query::<(&GhostCell, &Hp)>();
        let ghosts: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(ghosts.len(), 1);
        let hp = ghosts[0].1;
        assert!((hp.current - 4.0).abs() < 1e-5);
        assert!((hp.starting - 4.0).abs() < 1e-5);
        assert!(hp.max.is_none());
    }

    // Behavior 26 — ghost carries CleanupOnExit<NodeState> at stack 1.
    // Forward-compat pin: Cell carries #[require(Spatial2D,
    // CleanupOnExit<NodeState>)], so ghosts inherit it automatically. This
    // regression-guards against anyone silently removing the #[require].
    #[test]
    fn spawn_ghosts_ghost_carries_cleanup_on_exit_node_state_at_stack_one() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        app.world_mut().spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    0.05,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let mut query = app
            .world_mut()
            .query::<(&GhostCell, &CleanupOnExit<NodeState>)>();
        assert_eq!(query.iter(app.world()).count(), 1);
    }

    // Behavior 27 — ghost carries CleanupOnExit<NodeState> at stack 3.
    #[test]
    fn spawn_ghosts_ghost_carries_cleanup_on_exit_node_state_at_stack_three() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 3);
        app.world_mut().spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    0.05,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let mut query = app
            .world_mut()
            .query::<(&GhostCell, &CleanupOnExit<NodeState>)>();
        assert_eq!(query.iter(app.world()).count(), 1);
    }

    // Behavior 27b — PendingGhost carries CleanupOnExit<NodeState>
    // explicitly. PendingGhost is a standalone marker entity — it does
    // NOT carry `Cell`, so it does NOT inherit cleanup via `#[require]`.
    // The track system must attach `CleanupOnExit::<NodeState>::default()`
    // to each spawn, or in-flight pendings leak when `NodeState` exits.
    #[test]
    fn track_deaths_pending_ghost_carries_cleanup_on_exit_node_state() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_track_deaths);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::ZERO);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app
            .world_mut()
            .query::<(&PendingGhost, &CleanupOnExit<NodeState>)>();
        assert_eq!(
            query.iter(app.world()).count(),
            1,
            "PendingGhost must carry CleanupOnExit<NodeState> — it does not\
             inherit it via Cell's #[require] because it does not carry Cell"
        );
    }

    // Behavior 28 — multiple pendings all expire on the same frame.
    #[test]
    fn spawn_ghosts_multiple_pendings_all_expire_same_frame() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        let expected = [
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(200.0, 0.0),
        ];
        for pos in expected {
            app.world_mut().spawn(PendingGhost {
                position: pos,
                timer:    0.05,
            });
        }

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let mut query = app.world_mut().query::<(&GhostCell, &Position2D)>();
        let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
        assert_eq!(positions.len(), 3);
        for exp in &expected {
            assert!(
                positions.iter().any(|p| (*p - *exp).length() < 1e-4),
                "missing expected position {exp:?}"
            );
        }
        let mut pendings = app.world_mut().query::<&PendingGhost>();
        assert_eq!(pendings.iter(app.world()).count(), 0);
    }

    // Behavior 29 — timer decrements by delta_seconds.
    #[test]
    fn spawn_ghosts_timer_decrements_by_delta_seconds() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        app.world_mut().spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    1.5,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.4));

        let mut pendings = app.world_mut().query::<&PendingGhost>();
        let ps: Vec<_> = pendings.iter(app.world()).collect();
        assert_eq!(ps.len(), 1);
        assert!((ps[0].timer - 1.1).abs() < 1e-5);
        let mut ghosts = app.world_mut().query::<&GhostCell>();
        assert_eq!(ghosts.iter(app.world()).count(), 0);
    }

    #[test]
    fn spawn_ghosts_timer_decrements_on_two_consecutive_ticks() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        app.world_mut().spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    1.5,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.4));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.4));

        let mut pendings = app.world_mut().query::<&PendingGhost>();
        let ps: Vec<_> = pendings.iter(app.world()).collect();
        assert_eq!(ps.len(), 1);
        assert!((ps[0].timer - 0.7).abs() < 1e-5);
        let mut ghosts = app.world_mut().query::<&GhostCell>();
        assert_eq!(ghosts.iter(app.world()).count(), 0);
    }

    // Behavior 30 — timer expires exactly at the 0.0 boundary (strict > 0.0 check).
    #[test]
    fn spawn_ghosts_timer_expires_exactly_at_zero_boundary() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        app.world_mut().spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    0.016,
        });

        // timer -= 0.016 → 0.0 exactly; 0.0 > 0.0 is false → expiry branch fires.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut ghosts = app.world_mut().query::<&GhostCell>();
        assert_eq!(ghosts.iter(app.world()).count(), 1);
        let mut pendings = app.world_mut().query::<&PendingGhost>();
        assert_eq!(pendings.iter(app.world()).count(), 0);
    }

    // Behavior 31 — overshoot expiry still spawns exactly once (no make-up loop).
    #[test]
    fn spawn_ghosts_overshoot_expiry_still_spawns_once() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        app.world_mut().spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    0.1,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(10.0));

        let mut ghosts = app.world_mut().query::<&GhostCell>();
        assert_eq!(ghosts.iter(app.world()).count(), 1);
        let mut pendings = app.world_mut().query::<&PendingGhost>();
        assert_eq!(pendings.iter(app.world()).count(), 0);
    }

    // Behavior 32 — mid-flight stack change affects the already-pending ghost's HP.
    #[test]
    fn spawn_ghosts_mid_flight_stack_change_affects_pending() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        app.world_mut().spawn(PendingGhost {
            position: Vec2::ZERO,
            timer:    1.0,
        });

        // First tick: 1.0 - 0.5 = 0.5, no ghost yet.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.5));
        {
            let mut ghosts = app.world_mut().query::<&GhostCell>();
            assert_eq!(ghosts.iter(app.world()).count(), 0);
        }

        // Increase stacks from 1 → 3 mid-flight.
        add_echo_cells_stacks(&mut app, 2);

        // Second tick: 0.5 - 0.6 = -0.1 → expires; HP computed at 3 stacks.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.6));

        let mut query = app.world_mut().query::<(&GhostCell, &Hp)>();
        let ghosts: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(ghosts.len(), 1);
        // 1.0 * 2.0^2 = 4.0 (3 stacks at expiry, not the 1 stack at creation).
        assert!((ghosts[0].1.current - 4.0).abs() < 1e-5);
    }

    // Behavior 33 — spawned ghost's Position2D matches the PendingGhost position.
    #[test]
    fn spawn_ghosts_ghost_position_matches_pending_position() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, echo_cells_spawn_ghosts);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        app.world_mut().spawn(PendingGhost {
            position: Vec2::new(-777.0, 888.5),
            timer:    0.05,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let mut query = app.world_mut().query::<(&GhostCell, &Position2D)>();
        let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
        assert_eq!(positions.len(), 1);
        assert!((positions[0] - Vec2::new(-777.0, 888.5)).length() < 1e-4);
    }

    // ── D. register(app) — full chained pipeline ──────────────────────────

    // Behavior 34 — chained pipeline: track phase → spawn phase across ticks.
    #[test]
    fn register_chains_track_then_spawn() {
        let mut app = test_app_playing();
        register(&mut app);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::new(50.0, 50.0));

        // Tick 1 — tracker materializes PendingGhost { timer: 1.5, pos: (50, 50) }.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
        {
            let mut pendings = app.world_mut().query::<&PendingGhost>();
            let ps: Vec<_> = pendings.iter(app.world()).collect();
            assert_eq!(ps.len(), 1);
            assert!((ps[0].position - Vec2::new(50.0, 50.0)).length() < 1e-4);
        }

        // Tick 2 — timer 1.5 - 1.5 = 0.0; 0.0 > 0.0 is false → ghost fires.
        tick_with_dt(&mut app, Duration::from_secs_f32(1.5));
        {
            let mut pendings = app.world_mut().query::<&PendingGhost>();
            assert_eq!(pendings.iter(app.world()).count(), 0);
        }
        let mut query = app.world_mut().query::<(&GhostCell, &Position2D)>();
        let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
        assert_eq!(positions.len(), 1);
        assert!((positions[0] - Vec2::new(50.0, 50.0)).length() < 1e-4);
    }

    // Behavior 34a — strict > 0.0 boundary pin via full register() pipeline.
    // Pin strict > 0.0 boundary. timer arrives at exactly 0.0 in the second
    // tick's spawn system; 0.0 > 0.0 is false → ghost fires. If
    // echo_cells_spawn_ghosts ever changes the comparison to >= 0.0, this
    // test fails and flags the regression.
    #[test]
    fn register_ghost_fires_at_exactly_zero_timer_strict_gt_check() {
        let mut app = test_app_playing();
        register(&mut app);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::new(50.0, 50.0));

        // First tick — track phase materializes PendingGhost { timer: 1.5 }.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
        // Second tick — spawn phase decrements 1.5 - 1.5 = 0.0; strict > 0.0 → false.
        tick_with_dt(&mut app, Duration::from_secs_f32(1.5));

        let mut query = app.world_mut().query::<(&GhostCell, &Position2D)>();
        let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
        assert_eq!(positions.len(), 1);
        assert!((positions[0] - Vec2::new(50.0, 50.0)).length() < 1e-4);
        let mut pendings = app.world_mut().query::<&PendingGhost>();
        assert_eq!(pendings.iter(app.world()).count(), 0);
    }

    // Behavior 34b — multi-ghost queue-and-fire test OMITTED.
    // The test spec (section D, #34b) marks this as optional and
    // authorizes deletion if Bevy 0.18 .chain() auto-apply-deferred
    // semantics make the expected outcome non-deterministic. Single-ghost
    // coverage in #34 and #34a pins the pipeline and strict-boundary
    // behavior without dependence on cross-system Commands flushing.

    // Behavior 35 — gate off (NodeState != Playing) → nothing tracked.
    #[test]
    fn register_gate_off_not_playing_does_not_track() {
        let mut app = test_app_not_playing();
        register(&mut app);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::ZERO);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut pendings = app.world_mut().query::<&PendingGhost>();
        assert_eq!(pendings.iter(app.world()).count(), 0);
        let mut ghosts = app.world_mut().query::<&GhostCell>();
        assert_eq!(ghosts.iter(app.world()).count(), 0);
    }

    // Behavior 36 — positive control: identical setup except for Playing state.
    // Divergence between #35 and #36 proves the NodeState gate is the discriminator.
    #[test]
    fn register_positive_control_state_playing_tracks() {
        let mut app = test_app_playing();
        register(&mut app);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);
        write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::ZERO);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut pendings = app.world_mut().query::<&PendingGhost>();
        let ps: Vec<_> = pendings.iter(app.world()).collect();
        assert_eq!(ps.len(), 1);
        assert!((ps[0].position - Vec2::ZERO).length() < 1e-4);
    }

    // Behavior 37 — gate off (zero stacks) → nothing tracked / spawned.
    #[test]
    fn register_gate_off_zero_stacks_does_not_track() {
        let mut app = test_app_playing();
        register(&mut app);
        install_echo_cells_config(&mut app, canonical_config());
        // NO stacks added — hazard_active(EchoCells) returns false.
        write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::ZERO);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut pendings = app.world_mut().query::<&PendingGhost>();
        assert_eq!(pendings.iter(app.world()).count(), 0);
        let mut ghosts = app.world_mut().query::<&GhostCell>();
        assert_eq!(ghosts.iter(app.world()).count(), 0);
    }

    // Behavior 38 — pre-gate messages accumulate until the gate opens.
    #[test]
    fn register_pregate_messages_accumulate_until_gate_opens() {
        // Pin Bevy MessageReader semantics under a run_if gate: when the
        // gate is closed, the system does not consume messages; the
        // reader's cursor stays put. Opening the gate on a later tick lets
        // the system read ALL unread messages — including the pre-gate
        // ones. Shared pattern with fracture / overcharge / drift /
        // gravity_surge.
        //
        // Setup: gate starts CLOSED (no Echo Cells stack). Write one
        // Destroyed<Cell>. Tick (gate off → no-op). Toggle gate ON without
        // writing a new message. Tick again. The pre-gate message must now
        // be consumed → one PendingGhost at the death position.
        let mut app = test_app_playing();
        register(&mut app);
        install_echo_cells_config(&mut app, canonical_config());

        // Tick 1 — gate off, pre-gate death written.
        write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::new(5.0, 5.0));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
        {
            let mut q = app.world_mut().query::<&PendingGhost>();
            assert_eq!(
                q.iter(app.world()).count(),
                0,
                "gate closed → no pending this tick"
            );
        }

        // Tick 2 — open gate, write no new message. Pre-gate message
        // retained by Bevy's double-buffered Messages<T> is consumed now.
        add_echo_cells_stacks(&mut app, 1);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&PendingGhost>();
        let pendings: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(
            pendings.len(),
            1,
            "gate open → retained pre-gate message consumed → 1 pending"
        );
        assert!((pendings[0].position - Vec2::new(5.0, 5.0)).length() < 1e-4);
    }

    // Behavior 39 — gate reopens when a stack is added; new message processed.
    #[test]
    fn register_gate_reopens_when_stack_added_processes_new_messages() {
        let mut app = test_app_playing();
        register(&mut app);
        install_echo_cells_config(&mut app, canonical_config());

        // Tick 1 — gate off, no message written.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        // Open the gate, THEN write the message.
        add_echo_cells_stacks(&mut app, 1);
        write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&PendingGhost>();
        let pendings: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(pendings.len(), 1);
        assert!((pendings[0].position - Vec2::ZERO).length() < 1e-4);
    }

    // Behavior 40 — gate reopens when NodeState enters Playing.
    // TODO(recovery): needs in-test NodeState transition helper — see
    // TestAppBuilder::in_state_node_playing pattern at
    // breaker-game/src/shared/test_utils/builder.rs:80-107. Skipped per
    // test spec Open Question #1; covered indirectly by the stack-add gate
    // test (Behavior 39) and the positive-control pair (Behaviors 35/36).

    // Behavior 41 — second tick without a new message does not spawn more.
    #[test]
    fn register_second_tick_without_message_does_not_spawn_more() {
        let mut app = test_app_playing();
        register(&mut app);
        install_echo_cells_config(&mut app, canonical_config());
        add_echo_cells_stacks(&mut app, 1);

        write_destroyed(&mut app, Entity::PLACEHOLDER, Vec2::new(50.0, 50.0));
        // Tick 1 — track phase materializes pending.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
        // Tick 2 — timer 1.5 - 1.5 = 0.0 → ghost fires.
        tick_with_dt(&mut app, Duration::from_secs_f32(1.5));

        // Sanity: one ghost, zero pendings after materialization.
        {
            let mut ghosts = app.world_mut().query::<&GhostCell>();
            assert_eq!(ghosts.iter(app.world()).count(), 1);
            let mut pendings = app.world_mut().query::<&PendingGhost>();
            assert_eq!(pendings.iter(app.world()).count(), 0);
        }

        // Two more ticks with NO new message → no additional ghosts.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut ghosts = app.world_mut().query::<&GhostCell>();
        assert_eq!(ghosts.iter(app.world()).count(), 1);
        let mut pendings = app.world_mut().query::<&PendingGhost>();
        assert_eq!(pendings.iter(app.world()).count(), 0);
    }

    // ── E. activate — extended coverage ───────────────────────────────────

    // Behavior 42 — matching EchoCells tuning pins all three fields.
    #[test]
    fn activate_now_with_matching_tuning_pins_all_three_fields() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::EchoCells {
                delay_secs:           3.0,
                base_hp:              7.5,
                per_level_multiplier: 1.25,
            },
        );

        let cfg = app.world().resource::<EchoCellsConfig>();
        assert!((cfg.delay_secs - 3.0).abs() < f32::EPSILON);
        assert!((cfg.base_hp - 7.5).abs() < f32::EPSILON);
        assert!((cfg.per_level_multiplier - 1.25).abs() < f32::EPSILON);
    }

    // Behavior 43 — mismatched Drift tuning inserts nothing.
    #[test]
    fn activate_now_with_mismatched_drift_does_nothing() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Drift {
                force:           100.0,
                period_secs:     8.0,
                per_level_force: 33.3,
            },
        );
        assert!(app.world().get_resource::<EchoCellsConfig>().is_none());
    }

    // Behavior 44 — mismatched Fracture tuning inserts nothing.
    #[test]
    fn activate_now_with_mismatched_fracture_does_nothing() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Fracture {
                base_splits:      2,
                per_level_splits: 1,
            },
        );
        assert!(app.world().get_resource::<EchoCellsConfig>().is_none());
    }

    // Behavior 45 — mismatched Overcharge tuning inserts nothing.
    #[test]
    fn activate_now_with_mismatched_overcharge_does_nothing() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Overcharge {
                base_frac:      0.05,
                per_level_frac: 0.03,
            },
        );
        assert!(app.world().get_resource::<EchoCellsConfig>().is_none());
    }

    // Behavior 46 — mismatched Cascade with NaN does not panic.
    #[test]
    fn activate_now_with_mismatched_cascade_nan_does_not_panic() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Cascade {
                base_heal:      f32::NAN,
                per_level_heal: f32::NAN,
            },
        );
        assert!(app.world().get_resource::<EchoCellsConfig>().is_none());
    }

    // Behavior 47 — second activate overwrites (last-write-wins).
    #[test]
    fn second_activate_overwrites_echo_cells_config() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::EchoCells {
                delay_secs:           1.5,
                base_hp:              1.0,
                per_level_multiplier: 2.0,
            },
        );
        activate_now(
            &mut app,
            &HazardTuning::EchoCells {
                delay_secs:           3.0,
                base_hp:              5.0,
                per_level_multiplier: 1.5,
            },
        );

        let cfg = app.world().resource::<EchoCellsConfig>();
        assert!((cfg.delay_secs - 3.0).abs() < f32::EPSILON);
        assert!((cfg.base_hp - 5.0).abs() < f32::EPSILON);
        assert!((cfg.per_level_multiplier - 1.5).abs() < f32::EPSILON);
    }

    // Behavior 48 — third activate overwrites to boundary zeros.
    #[test]
    fn third_activate_overwrites_to_boundary_values() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::EchoCells {
                delay_secs:           1.5,
                base_hp:              1.0,
                per_level_multiplier: 2.0,
            },
        );
        activate_now(
            &mut app,
            &HazardTuning::EchoCells {
                delay_secs:           3.0,
                base_hp:              5.0,
                per_level_multiplier: 1.5,
            },
        );
        activate_now(
            &mut app,
            &HazardTuning::EchoCells {
                delay_secs:           0.0,
                base_hp:              0.0,
                per_level_multiplier: 0.0,
            },
        );

        let cfg = app.world().resource::<EchoCellsConfig>();
        assert!((cfg.delay_secs - 0.0).abs() < f32::EPSILON);
        assert!((cfg.base_hp - 0.0).abs() < f32::EPSILON);
        assert!((cfg.per_level_multiplier - 0.0).abs() < f32::EPSILON);
    }

    // Behavior 49 — mismatch after match preserves the existing config.
    #[test]
    fn activate_now_mismatch_after_match_preserves_existing_config() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::EchoCells {
                delay_secs:           2.0,
                base_hp:              4.0,
                per_level_multiplier: 2.0,
            },
        );
        activate_now(
            &mut app,
            &HazardTuning::Haste {
                base_percent:      0.1,
                per_level_percent: 0.05,
            },
        );

        let cfg = app.world().resource::<EchoCellsConfig>();
        assert!((cfg.delay_secs - 2.0).abs() < f32::EPSILON);
        assert!((cfg.base_hp - 4.0).abs() < f32::EPSILON);
        assert!((cfg.per_level_multiplier - 2.0).abs() < f32::EPSILON);
    }

    // Behavior 50 — activate on an empty world does not panic.
    #[test]
    fn activate_now_on_empty_world_does_not_panic() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::EchoCells {
                delay_secs:           1.5,
                base_hp:              1.0,
                per_level_multiplier: 2.0,
            },
        );

        let cfg = app.world().resource::<EchoCellsConfig>();
        assert!((cfg.delay_secs - 1.5).abs() < f32::EPSILON);
        assert!((cfg.base_hp - 1.0).abs() < f32::EPSILON);
        assert!((cfg.per_level_multiplier - 2.0).abs() < f32::EPSILON);
    }

    // ── F. Component derive pins ──────────────────────────────────────────

    // Static trait-bound assertions — compile-time pins for Clone/Copy/Default
    // without triggering clippy::clone_on_copy (which fires on `.clone()` of
    // Copy types). Calling the zero-body fn is a no-op at runtime; the trait
    // bound is checked at monomorphization.
    const fn assert_clone<T: Clone>() {}
    const fn assert_copy<T: Copy>() {}
    const fn assert_default<T: Default>() {}

    // Behavior 51 — PendingGhost derives Copy + Clone.
    #[test]
    fn pending_ghost_is_clone_copy() {
        assert_clone::<PendingGhost>();
        assert_copy::<PendingGhost>();
        let a = PendingGhost {
            position: Vec2::new(1.0, 2.0),
            timer:    0.5,
        };
        let b = a; // copy, not move — a remains usable below.
        let c = a;
        assert_eq!(a.position, b.position);
        assert!((a.timer - c.timer).abs() < f32::EPSILON);
    }

    // Behavior 52 — GhostCell derives Default + Clone + Copy.
    #[test]
    fn ghost_cell_is_clone_copy_default() {
        assert_clone::<GhostCell>();
        assert_copy::<GhostCell>();
        assert_default::<GhostCell>();
        // Copy semantics pin: `a` remains usable after being copied twice.
        let a = GhostCell;
        let (b, c) = (a, a);
        let _ = b;
        let _ = c;
        // Default-constructor pin: `GhostCell::default()` returns the
        // canonical unit value (use the bare struct to avoid
        // `clippy::default_constructed_unit_structs`).
        let _: GhostCell = GhostCell;
    }

    // Behavior 53 — EchoCellsConfig derives Clone + Copy.
    #[test]
    fn echo_cells_config_is_clone_copy() {
        assert_clone::<EchoCellsConfig>();
        assert_copy::<EchoCellsConfig>();
        let cfg = canonical_config();
        let dup = cfg; // copy, not move — cfg remains usable below.
        assert!((dup.delay_secs - cfg.delay_secs).abs() < f32::EPSILON);
        assert!((cfg.base_hp - 1.0).abs() < f32::EPSILON);
    }
}
