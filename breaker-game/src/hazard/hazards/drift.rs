//! Drift hazard — ambient wind pushes bolts in a telegraphed direction.
//! Direction changes every `period_secs`; magnitude scales linearly with
//! stack count. The hazard writes directly to each Bolt's `Velocity2D`
//! each tick (force × dt), keeping the implementation self-contained.

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::components::Bolt,
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

/// Per-run tuning extracted from [`HazardTuning::Drift`] at activation.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct DriftConfig {
    /// Force magnitude (world-space units / second²) at stack 1.
    pub(crate) force:           f32,
    /// Seconds between direction changes. Does not scale with stacks.
    pub(crate) period_secs:     f32,
    /// Additional force magnitude per stack beyond the first.
    pub(crate) per_level_force: f32,
}

impl DriftConfig {
    /// Effective force magnitude for the given stack count. Returns 0.0
    /// when `stacks == 0`.
    #[must_use]
    pub(crate) const fn force_magnitude(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        self.per_level_force.mul_add(extra, self.force)
    }
}

/// Current wind state. There is one global wind direction affecting all
/// bolts; individual bolts experience different outcomes only via their
/// existing velocities.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct DriftWind {
    pub(crate) direction: Vec2,
    /// Seconds remaining until the next direction change.
    pub(crate) timer:     f32,
}

impl Default for DriftWind {
    fn default() -> Self {
        Self {
            direction: Vec2::X,
            timer:     0.0,
        }
    }
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Drift {
        force,
        period_secs,
        per_level_force,
    } = *tuning
    else {
        warn!("drift::activate called with non-Drift tuning");
        return;
    };
    commands.insert_resource(DriftConfig {
        force,
        period_secs,
        per_level_force,
    });
    commands.insert_resource(DriftWind {
        direction: Vec2::X,
        // Timer starts at 0 so the first tick re-rolls direction via the
        // seeded RNG — deterministic initial wind for replay/scenarios.
        timer:     0.0,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (drift_update_wind, drift_apply_force)
            .chain()
            .run_if(hazard_active(HazardKind::Drift))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Ticks the wind timer; when it expires, rolls a new random unit vector
/// via `GameRng` (seeded for deterministic replay) and resets the timer
/// to `period_secs`.
fn drift_update_wind(
    time: Res<Time<Fixed>>,
    config: Option<Res<DriftConfig>>,
    wind: Option<ResMut<DriftWind>>,
    rng: Option<ResMut<GameRng>>,
) {
    let (Some(config), Some(mut wind), Some(mut rng)) = (config, wind, rng) else {
        return;
    };
    wind.timer -= time.delta_secs();
    if wind.timer > 0.0 {
        return;
    }
    let angle = rng.0.random_range(0.0..std::f32::consts::TAU);
    wind.direction = Vec2::new(angle.cos(), angle.sin());
    wind.timer = config.period_secs;
}

/// Adds `direction × force × dt` to every Bolt's `Velocity2D` each tick.
fn drift_apply_force(
    time: Res<Time<Fixed>>,
    active: Res<ActiveHazards>,
    config: Option<Res<DriftConfig>>,
    wind: Option<Res<DriftWind>>,
    mut bolts: Query<&mut Velocity2D, With<Bolt>>,
) {
    let (Some(config), Some(wind)) = (config, wind) else {
        return;
    };
    let stacks = active.stacks(HazardKind::Drift);
    let force = config.force_magnitude(stacks);
    if force <= 0.0 {
        return;
    }
    let impulse = wind.direction * force * time.delta_secs();
    for mut velocity in &mut bolts {
        velocity.0 += impulse;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    use super::*;
    use crate::prelude::TestAppBuilder;

    fn test_app_playing() -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_resource::<ActiveHazards>()
            .build()
    }

    fn insert_rng(app: &mut App, seed: u64) {
        app.world_mut()
            .insert_resource(GameRng(ChaCha8Rng::seed_from_u64(seed)));
    }

    fn spawn_bolt(app: &mut App, velocity: Vec2) -> Entity {
        app.world_mut().spawn((Bolt, Velocity2D(velocity))).id()
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

    // ── force_magnitude formula ───────────────────────────────────────────

    #[test]
    fn force_zero_stacks_is_zero() {
        let cfg = DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        };
        assert!((cfg.force_magnitude(0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn force_stack_one_equals_base() {
        let cfg = DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        };
        assert!((cfg.force_magnitude(1) - 100.0).abs() < 1e-4);
    }

    #[test]
    fn force_stack_three_adds_two_levels() {
        let cfg = DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        };
        // 100 + 33.3 * 2 = 166.6
        assert!((cfg.force_magnitude(3) - 166.6).abs() < 1e-4);
    }

    // ── drift_update_wind ─────────────────────────────────────────────────

    #[test]
    fn wind_rolls_new_direction_when_timer_expires() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, drift_update_wind);
        insert_rng(&mut app, 42);
        app.world_mut().insert_resource(DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        });
        app.world_mut().insert_resource(DriftWind {
            direction: Vec2::X,
            timer:     0.0,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let wind = app.world().resource::<DriftWind>();
        assert!(
            (wind.direction.length() - 1.0).abs() < 1e-5,
            "direction should remain a unit vector, got len={}",
            wind.direction.length()
        );
        assert!(
            (wind.timer - 8.0).abs() < 1e-5,
            "timer should reset to period_secs, got {}",
            wind.timer
        );
    }

    #[test]
    fn wind_direction_stays_constant_within_interval() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, drift_update_wind);
        insert_rng(&mut app, 42);
        app.world_mut().insert_resource(DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        });
        let initial = Vec2::new(0.5, 0.5).normalize();
        app.world_mut().insert_resource(DriftWind {
            direction: initial,
            timer:     4.0,
        });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let wind = app.world().resource::<DriftWind>();
        assert!((wind.direction - initial).length() < 1e-5);
        assert!((wind.timer - 3.9).abs() < 1e-5);
    }

    // ── drift_apply_force ─────────────────────────────────────────────────

    #[test]
    fn apply_force_accelerates_bolt_in_wind_direction() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, drift_apply_force);
        app.world_mut().insert_resource(DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        });
        app.world_mut().insert_resource(DriftWind {
            direction: Vec2::new(1.0, 0.0),
            timer:     8.0,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Drift);
        let bolt = spawn_bolt(&mut app, Vec2::ZERO);

        tick_with_dt(&mut app, Duration::from_secs(1));

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        // 100 units/sec² × 1s = +100 in X
        assert!(
            (velocity.0.x - 100.0).abs() < 1e-3,
            "expected +100 in X, got {:?}",
            velocity.0
        );
        assert!(velocity.0.y.abs() < 1e-5);
    }

    #[test]
    fn apply_force_scales_with_stacks() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, drift_apply_force);
        app.world_mut().insert_resource(DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.0,
        });
        app.world_mut().insert_resource(DriftWind {
            direction: Vec2::new(0.0, -1.0),
            timer:     8.0,
        });
        for _ in 0..3 {
            app.world_mut()
                .resource_mut::<ActiveHazards>()
                .add_stack(HazardKind::Drift);
        }
        let bolt = spawn_bolt(&mut app, Vec2::ZERO);

        tick_with_dt(&mut app, Duration::from_secs(1));

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        // 100 + 33 * 2 = 166 → applied in -Y for 1 second
        assert!(velocity.0.x.abs() < 1e-5);
        assert!((velocity.0.y - -166.0).abs() < 1e-3);
    }

    #[test]
    fn apply_force_affects_all_bolts() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, drift_apply_force);
        app.world_mut().insert_resource(DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        });
        app.world_mut().insert_resource(DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Drift);
        let bolt_a = spawn_bolt(&mut app, Vec2::ZERO);
        let bolt_b = spawn_bolt(&mut app, Vec2::new(50.0, 0.0));

        tick_with_dt(&mut app, Duration::from_secs(1));

        let vel_a = app.world().get::<Velocity2D>(bolt_a).unwrap();
        let vel_b = app.world().get::<Velocity2D>(bolt_b).unwrap();
        assert!((vel_a.0.x - 100.0).abs() < 1e-3);
        assert!((vel_b.0.x - 150.0).abs() < 1e-3);
    }

    #[test]
    fn apply_force_noop_at_zero_stacks() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, drift_apply_force);
        app.world_mut().insert_resource(DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        });
        app.world_mut().insert_resource(DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        });
        let bolt = spawn_bolt(&mut app, Vec2::ZERO);

        tick_with_dt(&mut app, Duration::from_secs(1));

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        assert!(velocity.0.length() < f32::EPSILON);
    }

    // ── activate — preserved scaffold tests ────────────────────────────────

    #[test]
    fn activate_with_matching_tuning_inserts_config_and_wind() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &HazardTuning::Drift {
                    force:           100.0,
                    period_secs:     8.0,
                    per_level_force: 33.3,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<DriftConfig>();
        assert!((cfg.force - 100.0).abs() < f32::EPSILON);
        let wind = app.world().resource::<DriftWind>();
        assert!((wind.timer - 0.0).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<DriftConfig>().is_none());
        assert!(app.world().get_resource::<DriftWind>().is_none());
    }
}
