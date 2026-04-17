//! Renewal hazard — every living cell carries a countdown timer; on
//! expiry the cell heals back to its `starting` HP and the timer resets.
//! Each stack shortens the timer multiplicatively with diminishing
//! returns: `duration = base * (1 - frac)^(stacks - 1)`.

use bevy::prelude::*;

use crate::{
    cells::components::Cell,
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
    shared::death_pipeline::Hp,
};

/// Per-run tuning extracted from [`HazardTuning::Renewal`].
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct RenewalConfig {
    /// Countdown duration at stack 1 (seconds).
    pub(crate) base_period_secs:         f32,
    /// Per-stack fractional reduction applied multiplicatively (diminishing).
    pub(crate) per_level_reduction_frac: f32,
}

impl RenewalConfig {
    /// Timer duration for the given stack count. `base * (1 - frac) ^
    /// (stacks - 1)`. Returns 0.0 at stack 0.
    #[must_use]
    pub(crate) fn duration_secs(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1).cast_signed();
        let factor = (1.0 - self.per_level_reduction_frac).max(0.0);
        self.base_period_secs * factor.powi(extra)
    }
}

/// Per-cell countdown for Renewal. Ticks down each tick; on expiry the
/// cell heals to its `starting` HP and the timer resets using the
/// *current* stack count's duration.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct RenewalTimer {
    pub(crate) remaining: f32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Renewal {
        base_period_secs,
        per_level_reduction_frac,
    } = *tuning
    else {
        warn!("renewal::activate called with non-Renewal tuning");
        return;
    };
    commands.insert_resource(RenewalConfig {
        base_period_secs,
        per_level_reduction_frac,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (renewal_attach_timers, renewal_tick)
            .chain()
            .run_if(hazard_active(HazardKind::Renewal))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Attaches a `RenewalTimer` to every Cell that lacks one. Handles both
/// node-start (cells spawned first) and dynamically-spawned cells (e.g.
/// Echo Cells / Fracture).
fn renewal_attach_timers(
    active: Res<ActiveHazards>,
    config: Option<Res<RenewalConfig>>,
    cells: Query<Entity, (With<Cell>, Without<RenewalTimer>)>,
    mut commands: Commands,
) {
    let Some(config) = config else { return };
    let stacks = active.stacks(HazardKind::Renewal);
    let duration = config.duration_secs(stacks);
    if duration <= 0.0 {
        return;
    }
    for entity in &cells {
        commands.entity(entity).insert(RenewalTimer {
            remaining: duration,
        });
    }
}

/// Ticks each cell's `RenewalTimer` down; on expiry heals the cell back
/// to `starting` HP and resets `remaining` to the current duration.
fn renewal_tick(
    time: Res<Time<Fixed>>,
    active: Res<ActiveHazards>,
    config: Option<Res<RenewalConfig>>,
    mut cells: Query<(&mut RenewalTimer, &mut Hp), With<Cell>>,
) {
    let Some(config) = config else { return };
    let stacks = active.stacks(HazardKind::Renewal);
    let duration = config.duration_secs(stacks);
    let dt = time.delta_secs();

    for (mut timer, mut hp) in &mut cells {
        if hp.current <= 0.0 {
            continue;
        }
        timer.remaining -= dt;
        if timer.remaining > 0.0 {
            continue;
        }
        if hp.current < hp.starting {
            hp.current = hp.starting;
        }
        timer.remaining = duration;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::prelude::TestAppBuilder;

    fn test_app_playing() -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_resource::<ActiveHazards>()
            .build()
    }

    fn spawn_cell(app: &mut App, current: f32, starting: f32) -> Entity {
        app.world_mut()
            .spawn((
                Cell,
                Hp {
                    current,
                    starting,
                    max: None,
                },
            ))
            .id()
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

    // ── duration_secs formula ─────────────────────────────────────────────

    #[test]
    fn duration_zero_stacks_is_zero() {
        let cfg = RenewalConfig {
            base_period_secs:         10.0,
            per_level_reduction_frac: 0.2,
        };
        assert!((cfg.duration_secs(0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn duration_stack_one_equals_base() {
        let cfg = RenewalConfig {
            base_period_secs:         10.0,
            per_level_reduction_frac: 0.2,
        };
        assert!((cfg.duration_secs(1) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn duration_stack_three_uses_diminishing_returns() {
        let cfg = RenewalConfig {
            base_period_secs:         10.0,
            per_level_reduction_frac: 0.2,
        };
        // 10 * 0.8^2 = 6.4
        assert!((cfg.duration_secs(3) - 6.4).abs() < 1e-4);
    }

    // ── renewal_attach_timers ─────────────────────────────────────────────

    #[test]
    fn attach_adds_timer_to_cells_without_one() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, renewal_attach_timers);
        app.world_mut().insert_resource(RenewalConfig {
            base_period_secs:         10.0,
            per_level_reduction_frac: 0.2,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Renewal);
        let cell = spawn_cell(&mut app, 50.0, 100.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let timer = app.world().get::<RenewalTimer>(cell).unwrap();
        assert!((timer.remaining - 10.0).abs() < 1e-5);
    }

    #[test]
    fn attach_does_not_duplicate_existing_timers() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, renewal_attach_timers);
        app.world_mut().insert_resource(RenewalConfig {
            base_period_secs:         10.0,
            per_level_reduction_frac: 0.2,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Renewal);
        let cell = spawn_cell(&mut app, 50.0, 100.0);
        app.world_mut()
            .entity_mut(cell)
            .insert(RenewalTimer { remaining: 2.0 });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let timer = app.world().get::<RenewalTimer>(cell).unwrap();
        assert!((timer.remaining - 2.0).abs() < 1e-5);
    }

    // ── renewal_tick ──────────────────────────────────────────────────────

    #[test]
    fn timer_expiry_heals_damaged_cell_to_starting() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, renewal_tick);
        app.world_mut().insert_resource(RenewalConfig {
            base_period_secs:         10.0,
            per_level_reduction_frac: 0.2,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Renewal);
        let cell = spawn_cell(&mut app, 30.0, 100.0);
        app.world_mut()
            .entity_mut(cell)
            .insert(RenewalTimer { remaining: 0.05 });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert!(
            (hp.current - 100.0).abs() < 1e-5,
            "cell should heal to starting HP, got {}",
            hp.current
        );
        let timer = app.world().get::<RenewalTimer>(cell).unwrap();
        assert!(
            timer.remaining > 9.0,
            "timer should reset, got {}",
            timer.remaining
        );
    }

    #[test]
    fn timer_stays_within_period() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, renewal_tick);
        app.world_mut().insert_resource(RenewalConfig {
            base_period_secs:         10.0,
            per_level_reduction_frac: 0.2,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Renewal);
        let cell = spawn_cell(&mut app, 30.0, 100.0);
        app.world_mut()
            .entity_mut(cell)
            .insert(RenewalTimer { remaining: 5.0 });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert!((hp.current - 30.0).abs() < f32::EPSILON);
        let timer = app.world().get::<RenewalTimer>(cell).unwrap();
        assert!((timer.remaining - 4.9).abs() < 1e-5);
    }

    #[test]
    fn full_hp_cell_does_not_over_heal_but_resets_timer() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, renewal_tick);
        app.world_mut().insert_resource(RenewalConfig {
            base_period_secs:         10.0,
            per_level_reduction_frac: 0.2,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Renewal);
        let cell = spawn_cell(&mut app, 100.0, 100.0);
        app.world_mut()
            .entity_mut(cell)
            .insert(RenewalTimer { remaining: 0.05 });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert!((hp.current - 100.0).abs() < f32::EPSILON);
        let timer = app.world().get::<RenewalTimer>(cell).unwrap();
        assert!(timer.remaining > 9.0);
    }

    #[test]
    fn dead_cell_is_skipped() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, renewal_tick);
        app.world_mut().insert_resource(RenewalConfig {
            base_period_secs:         10.0,
            per_level_reduction_frac: 0.2,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Renewal);
        let cell = spawn_cell(&mut app, 0.0, 100.0);
        app.world_mut()
            .entity_mut(cell)
            .insert(RenewalTimer { remaining: 0.05 });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert!(hp.current.abs() < f32::EPSILON);
    }

    #[test]
    fn timer_reset_uses_current_stack_duration() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, renewal_tick);
        app.world_mut().insert_resource(RenewalConfig {
            base_period_secs:         10.0,
            per_level_reduction_frac: 0.2,
        });
        for _ in 0..3 {
            app.world_mut()
                .resource_mut::<ActiveHazards>()
                .add_stack(HazardKind::Renewal);
        }
        let cell = spawn_cell(&mut app, 30.0, 100.0);
        app.world_mut()
            .entity_mut(cell)
            .insert(RenewalTimer { remaining: 0.05 });

        tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

        let timer = app.world().get::<RenewalTimer>(cell).unwrap();
        // Stack 3 → 10 * 0.8^2 = 6.4
        assert!(
            (timer.remaining - 6.4).abs() < 1e-4,
            "reset should use stack-3 duration (6.4), got {}",
            timer.remaining
        );
    }

    // ── activate — preserved scaffold tests ────────────────────────────────

    #[test]
    fn activate_with_matching_tuning_inserts_config() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &HazardTuning::Renewal {
                    base_period_secs:         10.0,
                    per_level_reduction_frac: 0.2,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<RenewalConfig>();
        assert!((cfg.base_period_secs - 10.0).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<RenewalConfig>().is_none());
    }
}
