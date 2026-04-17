//! Volatility hazard — cells gain HP over time, capped at a multiple of
//! their starting HP. Stacking accelerates the per-second growth.

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

/// Per-run tuning extracted from [`HazardTuning::Volatility`] at activation.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct VolatilityConfig {
    /// HP gained per 5 seconds at stack 1 — converted to per-second internally.
    pub(crate) hp_per_5s:                f32,
    /// Multiplier on starting HP that caps growth.
    pub(crate) cap_multiplier:           f32,
    /// Each stack beyond 1 multiplies the per-second rate by `(1 + frac * (stack - 1))`.
    pub(crate) per_level_reduction_frac: f32,
}

impl VolatilityConfig {
    /// Per-second HP growth rate for the given stack count. Returns 0.0
    /// when `stacks == 0` (hazard inactive).
    #[must_use]
    pub(crate) fn rate_per_sec(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let base_per_sec = self.hp_per_5s / 5.0;
        let extra = stacks.saturating_sub(1) as f32;
        base_per_sec * self.per_level_reduction_frac.mul_add(extra, 1.0)
    }
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Volatility {
        hp_per_5s,
        cap_multiplier,
        per_level_reduction_frac,
    } = *tuning
    else {
        warn!("volatility::activate called with non-Volatility tuning");
        return;
    };
    commands.insert_resource(VolatilityConfig {
        hp_per_5s,
        cap_multiplier,
        per_level_reduction_frac,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        volatility_grow_cell_hp
            .run_if(hazard_active(HazardKind::Volatility))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Each tick, grows every living cell's HP by `rate_per_sec(stacks) * dt`,
/// capped at `cap_multiplier * starting`. Skips cells already at 0 HP
/// (death pipeline owns them).
fn volatility_grow_cell_hp(
    time: Res<Time<Fixed>>,
    active: Res<ActiveHazards>,
    config: Option<Res<VolatilityConfig>>,
    mut cells: Query<&mut Hp, With<Cell>>,
) {
    let Some(config) = config else { return };
    let stacks = active.stacks(HazardKind::Volatility);
    let rate = config.rate_per_sec(stacks);
    if rate <= 0.0 {
        return;
    }
    let delta = rate * time.delta_secs();
    for mut hp in &mut cells {
        if hp.current <= 0.0 {
            continue;
        }
        let cap = hp.starting * config.cap_multiplier;
        hp.current = (hp.current + delta).min(cap);
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

    // ── rate_per_sec formula ──────────────────────────────────────────────

    #[test]
    fn rate_zero_stacks_is_zero() {
        let cfg = VolatilityConfig {
            hp_per_5s:                1.0,
            cap_multiplier:           3.0,
            per_level_reduction_frac: 0.1,
        };
        assert!((cfg.rate_per_sec(0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn rate_stack_one_is_base_per_sec() {
        let cfg = VolatilityConfig {
            hp_per_5s:                1.0,
            cap_multiplier:           3.0,
            per_level_reduction_frac: 0.1,
        };
        // 1.0 hp / 5.0 s = 0.2 hp/s.
        assert!((cfg.rate_per_sec(1) - 0.2).abs() < 1e-6);
    }

    #[test]
    fn rate_stack_three_scales_with_extra_levels() {
        let cfg = VolatilityConfig {
            hp_per_5s:                1.0,
            cap_multiplier:           3.0,
            per_level_reduction_frac: 0.1,
        };
        // base 0.2 * (1 + 0.1 * 2) = 0.2 * 1.2 = 0.24.
        assert!((cfg.rate_per_sec(3) - 0.24).abs() < 1e-6);
    }

    // ── volatility_grow_cell_hp behaviour ─────────────────────────────────

    #[test]
    fn cell_hp_grows_at_per_second_rate() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, volatility_grow_cell_hp);
        app.world_mut().insert_resource(VolatilityConfig {
            hp_per_5s:                1.0,
            cap_multiplier:           3.0,
            per_level_reduction_frac: 0.1,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Volatility);
        let cell = spawn_cell(&mut app, 5.0, 5.0);

        // 1 second tick at stack 1 (0.2 hp/s) → 5.2.
        tick_with_dt(&mut app, Duration::from_secs(1));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert!(
            (hp.current - 5.2).abs() < 1e-5,
            "after 1s at stack 1, expected 5.2 hp, got {}",
            hp.current
        );
    }

    #[test]
    fn cell_hp_caps_at_starting_times_multiplier() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, volatility_grow_cell_hp);
        app.world_mut().insert_resource(VolatilityConfig {
            hp_per_5s:                10.0,
            cap_multiplier:           3.0,
            per_level_reduction_frac: 0.0,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Volatility);
        let cell = spawn_cell(&mut app, 14.0, 5.0);

        // Cap = 5 * 3 = 15. With rate = 2.0 hp/s, 1s adds 2 hp → would
        // reach 16 but should clamp to 15.
        tick_with_dt(&mut app, Duration::from_secs(1));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert!(
            (hp.current - 15.0).abs() < f32::EPSILON,
            "growth must clamp to starting * cap_multiplier (15.0), got {}",
            hp.current
        );
    }

    #[test]
    fn dead_cells_are_not_grown() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, volatility_grow_cell_hp);
        app.world_mut().insert_resource(VolatilityConfig {
            hp_per_5s:                1.0,
            cap_multiplier:           3.0,
            per_level_reduction_frac: 0.1,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Volatility);
        let cell = spawn_cell(&mut app, 0.0, 5.0);

        tick_with_dt(&mut app, Duration::from_secs(1));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert!(
            hp.current.abs() < f32::EPSILON,
            "dead cell (0 hp) must stay at 0, got {}",
            hp.current
        );
    }

    #[test]
    fn no_growth_when_config_absent() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, volatility_grow_cell_hp);
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Volatility);
        let cell = spawn_cell(&mut app, 5.0, 5.0);

        tick_with_dt(&mut app, Duration::from_secs(1));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert!((hp.current - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn no_growth_when_stacks_zero() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, volatility_grow_cell_hp);
        app.world_mut().insert_resource(VolatilityConfig {
            hp_per_5s:                1.0,
            cap_multiplier:           3.0,
            per_level_reduction_frac: 0.1,
        });
        // ActiveHazards stays empty.
        let cell = spawn_cell(&mut app, 5.0, 5.0);

        tick_with_dt(&mut app, Duration::from_secs(1));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert!((hp.current - 5.0).abs() < f32::EPSILON);
    }

    // ── activate — preserved scaffold tests ────────────────────────────────

    #[test]
    fn activate_with_matching_tuning_inserts_config() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &HazardTuning::Volatility {
                    hp_per_5s:                1.0,
                    cap_multiplier:           3.0,
                    per_level_reduction_frac: 0.1,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<VolatilityConfig>();
        assert!((cfg.hp_per_5s - 1.0).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<VolatilityConfig>().is_none());
    }
}
