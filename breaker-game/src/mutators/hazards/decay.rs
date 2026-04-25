//! Decay hazard — drains the node timer faster while active. Stacking adds
//! linear percentage on top of the base speedup.

use bevy::prelude::*;

use crate::{
    mutators::hazards::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
    state::run::node::{messages::ApplyTimePenalty, sets::NodeSystems},
};

/// Per-run tuning values extracted from [`HazardTuning::Decay`] at activation.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct DecayConfig {
    /// Percentage speedup at stack 1. `15.0` means the timer drains 15% faster.
    pub(crate) base_percent:      f32,
    /// Additional percentage speedup per stack beyond the first.
    pub(crate) per_level_percent: f32,
}

impl DecayConfig {
    /// Total speedup percentage for `stacks`. At `stacks == 0` the formula
    /// returns 0.0 — the hazard is inactive.
    #[must_use]
    pub(crate) const fn speedup_percent(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        self.per_level_percent.mul_add(extra, self.base_percent)
    }
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Decay {
        base_percent,
        per_level_percent,
    } = *tuning
    else {
        warn!("decay::activate called with non-Decay tuning");
        return;
    };
    commands.insert_resource(DecayConfig {
        base_percent,
        per_level_percent,
    });
}

pub(crate) fn wire(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        decay_tick
            .after(NodeSystems::TickTimer)
            .before(NodeSystems::ApplyTimePenalty)
            .run_if(hazard_active(HazardKind::Decay))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Converts the per-stack speedup percentage into an [`ApplyTimePenalty`]
/// message each tick. Skipped when [`DecayConfig`] is absent — the config is
/// only inserted once a Decay stack is selected.
fn decay_tick(
    time: Res<Time>,
    active: Res<ActiveHazards>,
    config: Option<Res<DecayConfig>>,
    mut writer: MessageWriter<ApplyTimePenalty>,
) {
    let Some(config) = config else { return };
    let stacks = active.stacks(HazardKind::Decay);
    let percent = config.speedup_percent(stacks);
    let extra_drain = time.delta_secs() * percent / 100.0;
    if extra_drain <= 0.0 {
        return;
    }
    writer.write(ApplyTimePenalty {
        seconds: extra_drain,
    });
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::ecs::message::Messages;

    use super::*;
    use crate::prelude::TestAppBuilder;

    fn test_app_playing() -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_resource::<ActiveHazards>()
            .with_message::<ApplyTimePenalty>()
            .build()
    }

    fn advance_fixed_update(app: &mut App, delta: Duration) {
        app.world_mut()
            .resource_mut::<Time<Virtual>>()
            .advance_by(delta);
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn penalties(app: &App) -> Vec<f32> {
        app.world()
            .resource::<Messages<ApplyTimePenalty>>()
            .iter_current_update_messages()
            .map(|m| m.seconds)
            .collect()
    }

    // ── speedup_percent formula ───────────────────────────────────────────

    #[test]
    fn speedup_percent_zero_stacks_is_zero() {
        let cfg = DecayConfig {
            base_percent:      15.0,
            per_level_percent: 5.0,
        };
        assert!((cfg.speedup_percent(0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn speedup_percent_stack_one_equals_base() {
        let cfg = DecayConfig {
            base_percent:      15.0,
            per_level_percent: 5.0,
        };
        assert!((cfg.speedup_percent(1) - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn speedup_percent_stack_three_adds_two_levels() {
        let cfg = DecayConfig {
            base_percent:      15.0,
            per_level_percent: 5.0,
        };
        // stack 3: base (15) + 2 * per_level (5) = 25.
        assert!((cfg.speedup_percent(3) - 25.0).abs() < f32::EPSILON);
    }

    // ── decay_tick — gating ────────────────────────────────────────────────

    #[test]
    fn decay_tick_does_nothing_when_config_absent() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, decay_tick);
        // Decay active but config was never inserted — system should skip.
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Decay);

        advance_fixed_update(&mut app, Duration::from_millis(100));

        assert!(penalties(&app).is_empty());
    }

    #[test]
    fn decay_tick_does_nothing_when_no_stacks() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, decay_tick);
        app.world_mut().insert_resource(DecayConfig {
            base_percent:      15.0,
            per_level_percent: 5.0,
        });
        // ActiveHazards stays empty.

        advance_fixed_update(&mut app, Duration::from_millis(100));

        assert!(penalties(&app).is_empty());
    }

    // ── decay_tick — numeric behaviour ─────────────────────────────────────

    #[test]
    fn decay_tick_stack_one_drains_base_percent_of_fixed_timestep() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, decay_tick);
        app.world_mut().insert_resource(DecayConfig {
            base_percent:      15.0,
            per_level_percent: 5.0,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Decay);

        advance_fixed_update(&mut app, Duration::from_millis(100));

        // Inside FixedUpdate, `Time::delta_secs()` is the fixed timestep
        // (Bevy default 1/64 s = 0.015625 s). With stack 1 at 15% the
        // per-tick penalty is 0.015625 * 0.15 = 0.00234375 s.
        let timestep = app
            .world()
            .resource::<Time<Fixed>>()
            .timestep()
            .as_secs_f32();
        let expected = timestep * 0.15;
        let msgs = penalties(&app);
        assert_eq!(msgs.len(), 1, "expected exactly one ApplyTimePenalty");
        assert!(
            (msgs[0] - expected).abs() < 1e-5,
            "stack 1, delta {timestep}s, 15% → {expected}s, got {}",
            msgs[0]
        );
    }

    #[test]
    fn decay_tick_stack_three_drains_additive_percent() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, decay_tick);
        app.world_mut().insert_resource(DecayConfig {
            base_percent:      15.0,
            per_level_percent: 5.0,
        });
        {
            let mut active = app.world_mut().resource_mut::<ActiveHazards>();
            active.add_stack(HazardKind::Decay);
            active.add_stack(HazardKind::Decay);
            active.add_stack(HazardKind::Decay);
        }

        advance_fixed_update(&mut app, Duration::from_millis(100));

        let timestep = app
            .world()
            .resource::<Time<Fixed>>()
            .timestep()
            .as_secs_f32();
        // stack 3 → base (15) + 2 * per_level (5) = 25%.
        let expected = timestep * 0.25;
        let msgs = penalties(&app);
        assert_eq!(msgs.len(), 1);
        assert!(
            (msgs[0] - expected).abs() < 1e-5,
            "stack 3, delta {timestep}s, 25% → {expected}s, got {}",
            msgs[0]
        );
    }

    #[test]
    fn decay_tick_zero_delta_sends_no_message() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, decay_tick);
        app.world_mut().insert_resource(DecayConfig {
            base_percent:      15.0,
            per_level_percent: 5.0,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Decay);

        // Only pump FixedUpdate, with zero accumulated time.
        app.update();

        assert!(
            penalties(&app).is_empty(),
            "no delta → no penalty (decay_tick early-returns on <= 0)"
        );
    }

    // ── activate — existing scaffold tests preserved ──────────────────────

    #[test]
    fn activate_with_matching_tuning_inserts_config() {
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

        let cfg = app.world().resource::<DecayConfig>();
        assert!((cfg.base_percent - 0.05).abs() < f32::EPSILON);
        assert!((cfg.per_level_percent - 0.03).abs() < f32::EPSILON);
    }

    #[test]
    fn activate_with_mismatched_tuning_does_nothing() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &HazardTuning::Haste {
                    base_percent:      0.1,
                    per_level_percent: 0.05,
                },
                &mut commands,
            );
        });
        app.update();
        assert!(app.world().get_resource::<DecayConfig>().is_none());
    }
}
