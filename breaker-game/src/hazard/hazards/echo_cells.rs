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
    /// Ghost HP for the given stack count. 0.0 at stack 0.
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

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (echo_cells_track_deaths, echo_cells_spawn_ghosts)
            .chain()
            .run_if(hazard_active(HazardKind::EchoCells))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// For every `Destroyed<Cell>` whose victim is NOT a `GhostCell`,
/// spawns a `PendingGhost` marker with the death position and delay.
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
        commands.spawn(PendingGhost {
            position: destroyed.victim_pos,
            timer:    config.delay_secs,
        });
    }
}

/// Ticks each `PendingGhost`; on expiry spawns a ghost cell at the
/// stored position and despawns the marker.
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

    use bevy::prelude::Messages;

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
}
