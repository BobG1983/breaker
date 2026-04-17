//! `GravitySurge` hazard — destroyed cells spawn short-lived gravity wells
//! that pull the bolt. Stacking increases both the duration of each well
//! and its pull strength (linear in duration, fractional in strength).
//! Forces are applied directly to each Bolt's `Velocity2D` each tick and
//! are clamped at short range to prevent numeric blow-up.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use crate::{
    bolt::components::Bolt,
    cells::components::Cell,
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
    shared::death_pipeline::Destroyed,
};

/// Distance floor (world units) below which the pull force magnitude is
/// clamped. Prevents divide-by-zero when a bolt sits on top of a well.
const MIN_PULL_DISTANCE: f32 = 20.0;

/// Per-run tuning extracted from [`HazardTuning::GravitySurge`].
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct GravitySurgeConfig {
    /// Seconds each well lives at stack 1.
    pub(crate) base_duration_secs:      f32,
    /// Additional seconds per stack beyond the first.
    pub(crate) per_level_duration_secs: f32,
    /// Pull strength (force magnitude at `MIN_PULL_DISTANCE`) at stack 1.
    pub(crate) base_strength:           f32,
    /// Fractional per-stack increase applied multiplicatively to strength
    /// with sqrt diminishing returns: `strength * (1 + frac * sqrt(s-1))`.
    pub(crate) per_level_strength_frac: f32,
}

impl GravitySurgeConfig {
    /// Well lifetime for the given stack count. Returns 0.0 at stack 0.
    #[must_use]
    pub(crate) const fn duration_secs(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        self.per_level_duration_secs
            .mul_add(extra, self.base_duration_secs)
    }

    /// Pull strength for the given stack count. Square-root-in-stack
    /// diminishing returns.
    #[must_use]
    pub(crate) fn strength(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        let multiplier = self.per_level_strength_frac.mul_add(extra.sqrt(), 1.0);
        self.base_strength * multiplier
    }
}

/// A short-lived gravity well entity. Spawned at the position of a
/// destroyed cell. The `Position2D` on the same entity stores the world
/// location; `strength` is baked in at spawn time so post-spawn stack
/// changes don't retroactively alter older wells.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct GravityWell {
    pub(crate) strength:  f32,
    pub(crate) remaining: f32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::GravitySurge {
        base_duration_secs,
        per_level_duration_secs,
        base_strength,
        per_level_strength_frac,
    } = *tuning
    else {
        warn!("gravity_surge::activate called with non-GravitySurge tuning");
        return;
    };
    commands.insert_resource(GravitySurgeConfig {
        base_duration_secs,
        per_level_duration_secs,
        base_strength,
        per_level_strength_frac,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            spawn_gravity_wells,
            gravity_well_pull,
            despawn_expired_gravity_wells,
        )
            .chain()
            .run_if(hazard_active(HazardKind::GravitySurge))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Spawns a `GravityWell` at every `Destroyed<Cell>` position. Duration
/// and strength are snapshotted at spawn time from current stack count.
fn spawn_gravity_wells(
    mut reader: MessageReader<Destroyed<Cell>>,
    active: Res<ActiveHazards>,
    config: Option<Res<GravitySurgeConfig>>,
    mut commands: Commands,
) {
    let Some(config) = config else {
        reader.clear();
        return;
    };
    let stacks = active.stacks(HazardKind::GravitySurge);
    let duration = config.duration_secs(stacks);
    let strength = config.strength(stacks);
    if duration <= 0.0 || strength <= 0.0 {
        reader.clear();
        return;
    }
    for destroyed in reader.read() {
        commands.spawn((
            GravityWell {
                strength,
                remaining: duration,
            },
            Position2D(destroyed.victim_pos),
        ));
    }
}

/// Applies the summed pull from every active well to each bolt's
/// `Velocity2D`. Also ticks each well's `remaining` down.
fn gravity_well_pull(
    time: Res<Time<Fixed>>,
    mut wells: Query<(&mut GravityWell, &Position2D)>,
    mut bolts: Query<(&Position2D, &mut Velocity2D), With<Bolt>>,
) {
    let dt = time.delta_secs();
    // Collect well snapshots and tick remaining.
    let mut snapshots: Vec<(Vec2, f32)> = Vec::new();
    for (mut well, pos) in &mut wells {
        well.remaining -= dt;
        if well.remaining > 0.0 {
            snapshots.push((pos.0, well.strength));
        }
    }
    if snapshots.is_empty() {
        return;
    }
    for (bolt_pos, mut vel) in &mut bolts {
        let mut accel = Vec2::ZERO;
        for (well_pos, strength) in &snapshots {
            let delta = *well_pos - bolt_pos.0;
            let distance = delta.length().max(MIN_PULL_DISTANCE);
            let direction = delta / distance;
            // Inverse-linear falloff, clamped at the distance floor.
            accel += direction * (*strength / distance);
        }
        vel.0 += accel * dt;
    }
}

/// Despawns wells whose `remaining` has dropped to or below zero.
fn despawn_expired_gravity_wells(wells: Query<(Entity, &GravityWell)>, mut commands: Commands) {
    for (entity, well) in &wells {
        if well.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
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

    fn spawn_bolt(app: &mut App, position: Vec2, velocity: Vec2) -> Entity {
        app.world_mut()
            .spawn((Bolt, Position2D(position), Velocity2D(velocity)))
            .id()
    }

    fn spawn_well(app: &mut App, position: Vec2, strength: f32, remaining: f32) -> Entity {
        app.world_mut()
            .spawn((
                GravityWell {
                    strength,
                    remaining,
                },
                Position2D(position),
            ))
            .id()
    }

    fn write_cell_destroyed(app: &mut App, pos: Vec2) {
        app.world_mut()
            .resource_mut::<Messages<Destroyed<Cell>>>()
            .write(Destroyed::<Cell> {
                victim:     Entity::PLACEHOLDER,
                killer:     None,
                victim_pos: pos,
                killer_pos: None,
                _marker:    PhantomData,
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

    // ── duration_secs / strength formulas ─────────────────────────────────

    #[test]
    fn duration_zero_stacks_is_zero() {
        let cfg = GravitySurgeConfig {
            base_duration_secs:      2.0,
            per_level_duration_secs: 1.0,
            base_strength:           500.0,
            per_level_strength_frac: 0.5,
        };
        assert!((cfg.duration_secs(0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn duration_scales_linearly_with_stacks() {
        let cfg = GravitySurgeConfig {
            base_duration_secs:      2.0,
            per_level_duration_secs: 1.0,
            base_strength:           500.0,
            per_level_strength_frac: 0.5,
        };
        assert!((cfg.duration_secs(1) - 2.0).abs() < 1e-6);
        assert!((cfg.duration_secs(3) - 4.0).abs() < 1e-6);
    }

    #[test]
    fn strength_zero_stacks_is_zero() {
        let cfg = GravitySurgeConfig {
            base_duration_secs:      2.0,
            per_level_duration_secs: 1.0,
            base_strength:           500.0,
            per_level_strength_frac: 0.5,
        };
        assert!((cfg.strength(0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn strength_has_sqrt_diminishing_returns() {
        let cfg = GravitySurgeConfig {
            base_duration_secs:      2.0,
            per_level_duration_secs: 1.0,
            base_strength:           500.0,
            per_level_strength_frac: 0.5,
        };
        // Stack 1: 500 * (1 + 0.5 * sqrt(0)) = 500
        assert!((cfg.strength(1) - 500.0).abs() < 1e-4);
        // Stack 5: 500 * (1 + 0.5 * sqrt(4)) = 500 * 2 = 1000
        assert!((cfg.strength(5) - 1000.0).abs() < 1e-4);
    }

    // ── spawn_gravity_wells ────────────────────────────────────────────────

    #[test]
    fn destroyed_cell_spawns_well_at_position() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, spawn_gravity_wells);
        app.world_mut().insert_resource(GravitySurgeConfig {
            base_duration_secs:      2.0,
            per_level_duration_secs: 1.0,
            base_strength:           500.0,
            per_level_strength_frac: 0.5,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::GravitySurge);

        write_cell_destroyed(&mut app, Vec2::new(100.0, 200.0));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(&GravityWell, &Position2D)>();
        let wells: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(wells.len(), 1);
        let (well, pos) = wells[0];
        assert!((pos.0 - Vec2::new(100.0, 200.0)).length() < f32::EPSILON);
        assert!((well.remaining - 2.0).abs() < 1e-5);
        assert!((well.strength - 500.0).abs() < 1e-4);
    }

    #[test]
    fn spawn_noop_without_config() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, spawn_gravity_wells);
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::GravitySurge);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&GravityWell>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    // ── gravity_well_pull ─────────────────────────────────────────────────

    #[test]
    fn single_well_pulls_bolt_toward_it() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, gravity_well_pull);
        let _well = spawn_well(&mut app, Vec2::new(0.0, 0.0), 500.0, 2.0);
        let bolt = spawn_bolt(&mut app, Vec2::new(100.0, 0.0), Vec2::ZERO);

        tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        // Well is to the left → accel in -X. Distance = 100, force = 500/100 = 5 u/s²
        // After 1s, velocity = 5 u/s in -X.
        assert!(
            vel.0.x < 0.0,
            "expected negative X velocity, got {:?}",
            vel.0
        );
        assert!(vel.0.y.abs() < 1e-5);
    }

    #[test]
    fn multiple_wells_compound_force() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, gravity_well_pull);
        // Two wells equidistant on X axis → forces cancel in X.
        spawn_well(&mut app, Vec2::new(-100.0, 0.0), 500.0, 2.0);
        spawn_well(&mut app, Vec2::new(100.0, 0.0), 500.0, 2.0);
        // Third well above the bolt → pull in +Y.
        spawn_well(&mut app, Vec2::new(0.0, 100.0), 500.0, 2.0);
        let bolt = spawn_bolt(&mut app, Vec2::ZERO, Vec2::ZERO);

        tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        assert!(
            vel.0.x.abs() < 1e-4,
            "X-axis wells should cancel, got {}",
            vel.0.x
        );
        assert!(vel.0.y > 0.0, "+Y well should pull up, got {}", vel.0.y);
    }

    #[test]
    fn pull_is_clamped_at_min_distance() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, gravity_well_pull);
        spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);
        let bolt = spawn_bolt(&mut app, Vec2::new(1.0, 0.0), Vec2::ZERO);

        tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

        let vel = app.world().get::<Velocity2D>(bolt).unwrap();
        // Distance floor = 20.0. Pull = 500 / 20 = 25 u/s². After 1s = 25 u/s.
        // Without the floor, pull at distance 1 would be ~500, which would be
        // a runaway.
        assert!(
            vel.0.length() < 100.0,
            "pull should be clamped (<100), got {}",
            vel.0.length()
        );
    }

    #[test]
    fn well_remaining_ticks_down() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, gravity_well_pull);
        let well = spawn_well(&mut app, Vec2::ZERO, 500.0, 2.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.5));

        let remaining = app.world().get::<GravityWell>(well).unwrap().remaining;
        assert!((remaining - 1.5).abs() < 1e-5);
    }

    // ── despawn_expired_gravity_wells ─────────────────────────────────────

    #[test]
    fn expired_well_is_despawned() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, despawn_expired_gravity_wells);
        let well = spawn_well(&mut app, Vec2::ZERO, 500.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        assert!(
            app.world().get_entity(well).is_err(),
            "well with remaining=0 should be despawned"
        );
    }

    #[test]
    fn active_well_is_not_despawned() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, despawn_expired_gravity_wells);
        let well = spawn_well(&mut app, Vec2::ZERO, 500.0, 1.5);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        assert!(app.world().get_entity(well).is_ok());
    }

    // ── activate — preserved scaffold tests ────────────────────────────────

    #[test]
    fn activate_with_matching_tuning_inserts_config() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &HazardTuning::GravitySurge {
                    base_duration_secs:      2.0,
                    per_level_duration_secs: 1.0,
                    base_strength:           100.0,
                    per_level_strength_frac: 0.2,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<GravitySurgeConfig>();
        assert!((cfg.base_strength - 100.0).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<GravitySurgeConfig>().is_none());
    }
}
