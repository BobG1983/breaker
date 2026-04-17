//! Resonance hazard — narrows the bump perfect window. Stacking adds
//! linear seconds on top of the base narrowing.

use bevy::prelude::*;

use crate::{
    breaker::components::{Breaker, BumpPerfectWindow},
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

/// Per-run tuning extracted from [`HazardTuning::Resonance`] at activation.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct ResonanceConfig {
    /// Seconds shaved off the perfect window at stack 1.
    pub(crate) base_window_secs:      f32,
    /// Additional seconds shaved per stack beyond the first.
    pub(crate) per_level_window_secs: f32,
}

impl ResonanceConfig {
    /// Total seconds to narrow the perfect window for `stacks` stacks.
    /// Returns 0.0 when `stacks == 0`.
    #[must_use]
    pub(crate) const fn narrowing_secs(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        self.per_level_window_secs
            .mul_add(extra, self.base_window_secs)
    }
}

/// Stores each Breaker's pristine `BumpPerfectWindow` value so the live
/// component can be restored when Resonance deactivates and to use as the
/// baseline against which the hazard subtracts.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct ResonanceBaselineWindow(pub(crate) f32);

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Resonance {
        base_window_secs,
        per_level_window_secs,
    } = *tuning
    else {
        warn!("resonance::activate called with non-Resonance tuning");
        return;
    };
    commands.insert_resource(ResonanceConfig {
        base_window_secs,
        per_level_window_secs,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        resonance_apply
            .run_if(hazard_active(HazardKind::Resonance))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Snapshots the baseline `BumpPerfectWindow` on each Breaker the first
/// time Resonance sees it, then writes
/// `BumpPerfectWindow = max(0.0, baseline - narrowing)`. Idempotent — the
/// system can run every tick without drift because it always recomputes
/// from the stored baseline.
fn resonance_apply(
    active: Res<ActiveHazards>,
    config: Option<Res<ResonanceConfig>>,
    mut breakers: Query<
        (
            Entity,
            &mut BumpPerfectWindow,
            Option<&ResonanceBaselineWindow>,
        ),
        With<Breaker>,
    >,
    mut commands: Commands,
) {
    let Some(config) = config else { return };
    let stacks = active.stacks(HazardKind::Resonance);
    let narrowing = config.narrowing_secs(stacks);

    for (entity, mut window, baseline) in &mut breakers {
        let baseline_value = if let Some(b) = baseline {
            b.0
        } else {
            let snapped = window.0;
            commands
                .entity(entity)
                .insert(ResonanceBaselineWindow(snapped));
            snapped
        };
        let target = (baseline_value - narrowing).max(0.0);
        window.0 = target;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::TestAppBuilder;

    fn spawn_breaker(app: &mut App, perfect_window: f32) -> Entity {
        app.world_mut()
            .spawn((Breaker, BumpPerfectWindow(perfect_window)))
            .id()
    }

    fn test_app_playing() -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_resource::<ActiveHazards>()
            .build()
    }

    fn run_fixed_update(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── narrowing_secs formula ─────────────────────────────────────────────

    #[test]
    fn narrowing_zero_stacks_is_zero() {
        let cfg = ResonanceConfig {
            base_window_secs:      0.05,
            per_level_window_secs: 0.01,
        };
        assert!((cfg.narrowing_secs(0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn narrowing_stack_one_equals_base() {
        let cfg = ResonanceConfig {
            base_window_secs:      0.05,
            per_level_window_secs: 0.01,
        };
        assert!((cfg.narrowing_secs(1) - 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn narrowing_stack_three_adds_two_levels() {
        let cfg = ResonanceConfig {
            base_window_secs:      0.05,
            per_level_window_secs: 0.01,
        };
        assert!((cfg.narrowing_secs(3) - 0.07).abs() < 1e-6);
    }

    // ── resonance_apply — baseline + narrowing ────────────────────────────

    #[test]
    fn resonance_apply_snapshots_baseline_and_narrows_window() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, resonance_apply);
        app.world_mut().insert_resource(ResonanceConfig {
            base_window_secs:      0.05,
            per_level_window_secs: 0.01,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Resonance);
        let breaker = spawn_breaker(&mut app, 0.20);

        run_fixed_update(&mut app);

        let window = app.world().get::<BumpPerfectWindow>(breaker).unwrap();
        assert!(
            (window.0 - 0.15).abs() < 1e-5,
            "stack 1 narrows by 0.05 from 0.20 baseline → 0.15, got {}",
            window.0
        );
        let baseline = app
            .world()
            .get::<ResonanceBaselineWindow>(breaker)
            .expect("baseline should be inserted on first apply");
        assert!((baseline.0 - 0.20).abs() < f32::EPSILON);
    }

    #[test]
    fn resonance_apply_uses_stored_baseline_on_second_tick() {
        // Regression: the second tick must NOT subtract from the *already
        // narrowed* live value — it must always recompute from the snapshot.
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, resonance_apply);
        app.world_mut().insert_resource(ResonanceConfig {
            base_window_secs:      0.05,
            per_level_window_secs: 0.01,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Resonance);
        let breaker = spawn_breaker(&mut app, 0.20);

        run_fixed_update(&mut app);
        run_fixed_update(&mut app);
        run_fixed_update(&mut app);

        let window = app.world().get::<BumpPerfectWindow>(breaker).unwrap();
        assert!(
            (window.0 - 0.15).abs() < 1e-5,
            "stable narrow at 0.15 across multiple ticks, got {}",
            window.0
        );
    }

    #[test]
    fn resonance_apply_clamps_to_zero_when_narrowing_exceeds_baseline() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, resonance_apply);
        app.world_mut().insert_resource(ResonanceConfig {
            base_window_secs:      0.30,
            per_level_window_secs: 0.10,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Resonance);
        let breaker = spawn_breaker(&mut app, 0.10);

        run_fixed_update(&mut app);

        let window = app.world().get::<BumpPerfectWindow>(breaker).unwrap();
        assert!(
            window.0.abs() < f32::EPSILON,
            "narrowing 0.30 vs baseline 0.10 → clamped to 0, got {}",
            window.0
        );
    }

    #[test]
    fn resonance_apply_does_nothing_when_config_absent() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, resonance_apply);
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Resonance);
        let breaker = spawn_breaker(&mut app, 0.20);

        run_fixed_update(&mut app);

        let window = app.world().get::<BumpPerfectWindow>(breaker).unwrap();
        assert!((window.0 - 0.20).abs() < f32::EPSILON);
        assert!(
            app.world()
                .get::<ResonanceBaselineWindow>(breaker)
                .is_none(),
            "no baseline should be inserted when config is absent"
        );
    }

    // ── activate — preserved scaffold tests ────────────────────────────────

    #[test]
    fn activate_with_matching_tuning_inserts_config() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &HazardTuning::Resonance {
                    base_window_secs:      0.01,
                    per_level_window_secs: 0.005,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<ResonanceConfig>();
        assert!((cfg.base_window_secs - 0.01).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<ResonanceConfig>().is_none());
    }
}
