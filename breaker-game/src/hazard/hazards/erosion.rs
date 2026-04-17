//! Erosion hazard — the Breaker slowly shrinks each tick; bumps restore
//! a fraction of the lost width. Shrink rate scales linearly with stack
//! count; restoration fractions stay fixed. Width is applied to the
//! Breaker's [`EffectStack<SizeBoostConfig>`] under a dedicated source
//! tag, mirroring the pattern used by Haste for bolt speed.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    breaker::{
        components::Breaker,
        messages::{BumpGrade, BumpPerformed},
    },
    effect_v3::{effects::SizeBoostConfig, stacking::EffectStack},
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

/// Source tag for Erosion's entry on the Breaker's
/// [`EffectStack<SizeBoostConfig>`]. Keeps Erosion's scaling isolated
/// from chip/protocol size boosts so each can be reconciled independently.
const EROSION_SOURCE: &str = "hazard:erosion";

/// Per-run tuning extracted from [`HazardTuning::Erosion`] at activation.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct ErosionConfig {
    /// Full-width fractions lost per second at stack 1. Scales linearly
    /// with stack count (stack 3 shrinks 3× as fast).
    pub(crate) shrink_rate:      f32,
    /// Floor on `width_fraction`; the breaker never shrinks below this.
    pub(crate) min_width_frac:   f32,
    /// Fraction of *lost* width restored on an Early/Late bump.
    pub(crate) restore_nonwhiff: f32,
    /// Fraction of *lost* width restored on a Perfect bump.
    pub(crate) restore_perfect:  f32,
}

/// Live per-run state — tracks the Breaker's current scale as a fraction
/// of its pristine width. 1.0 = full, `min_width_frac` = floor.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct ErosionState {
    pub(crate) width_fraction: f32,
}

impl Default for ErosionState {
    fn default() -> Self {
        Self {
            width_fraction: 1.0,
        }
    }
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Erosion {
        shrink_rate,
        min_width_frac,
        restore_nonwhiff,
        restore_perfect,
    } = *tuning
    else {
        warn!("erosion::activate called with non-Erosion tuning");
        return;
    };
    commands.insert_resource(ErosionConfig {
        shrink_rate,
        min_width_frac,
        restore_nonwhiff,
        restore_perfect,
    });
    commands.insert_resource(ErosionState::default());
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (erosion_shrink, erosion_restore, erosion_apply_width)
            .chain()
            .run_if(hazard_active(HazardKind::Erosion))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Shrinks `ErosionState.width_fraction` by `shrink_rate * stacks * dt`
/// each tick, clamped to `min_width_frac`. No-op at stack 0 or with no
/// config resource (defensive; the `run_if` should prevent that).
fn erosion_shrink(
    time: Res<Time<Fixed>>,
    active: Res<ActiveHazards>,
    config: Option<Res<ErosionConfig>>,
    state: Option<ResMut<ErosionState>>,
) {
    let (Some(config), Some(mut state)) = (config, state) else {
        return;
    };
    let stacks = active.stacks(HazardKind::Erosion);
    if stacks == 0 {
        return;
    }
    let delta = config.shrink_rate * stacks as f32 * time.delta_secs();
    if delta <= 0.0 {
        return;
    }
    state.width_fraction = (state.width_fraction - delta).max(config.min_width_frac);
}

/// Restores width when a bump lands. Perfect bumps restore
/// `restore_perfect × lost`; Early/Late bumps restore `restore_nonwhiff ×
/// lost`. Width is clamped to 1.0. Whiffs (absence of `BumpPerformed`)
/// contribute nothing.
fn erosion_restore(
    mut reader: MessageReader<BumpPerformed>,
    config: Option<Res<ErosionConfig>>,
    state: Option<ResMut<ErosionState>>,
) {
    let (Some(config), Some(mut state)) = (config, state) else {
        reader.clear();
        return;
    };
    for bump in reader.read() {
        let lost = (1.0 - state.width_fraction).max(0.0);
        if lost <= 0.0 {
            continue;
        }
        let fraction = match bump.grade {
            BumpGrade::Perfect => config.restore_perfect,
            BumpGrade::Early | BumpGrade::Late => config.restore_nonwhiff,
        };
        if fraction <= 0.0 {
            continue;
        }
        let restore = (lost * fraction).min(lost);
        state.width_fraction = (state.width_fraction + restore).min(1.0);
    }
}

/// Reconciles each Breaker's [`EffectStack<SizeBoostConfig>`] so that
/// exactly one entry with source [`EROSION_SOURCE`] exists, carrying the
/// current `width_fraction` multiplier. Bolts of the Haste pattern: new
/// stacks are inserted this tick. Idempotent across ticks.
fn erosion_apply_width(
    state: Option<Res<ErosionState>>,
    mut breakers: Query<(Entity, Option<&mut EffectStack<SizeBoostConfig>>), With<Breaker>>,
    mut commands: Commands,
) {
    let Some(state) = state else { return };
    let entry = SizeBoostConfig {
        multiplier: OrderedFloat(state.width_fraction),
    };

    for (entity, stack) in &mut breakers {
        if let Some(mut stack) = stack {
            stack.retain_by_source(EROSION_SOURCE);
            stack.push(EROSION_SOURCE.to_owned(), entry.clone());
        } else {
            let mut fresh = EffectStack::<SizeBoostConfig>::default();
            fresh.push(EROSION_SOURCE.to_owned(), entry.clone());
            commands.entity(entity).insert(fresh);
        }
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

    fn spawn_breaker(app: &mut App) -> Entity {
        app.world_mut().spawn(Breaker).id()
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

    // ── erosion_shrink ────────────────────────────────────────────────────

    #[test]
    fn shrink_stack_one_reduces_width_fraction() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, erosion_shrink);
        app.world_mut().insert_resource(ErosionConfig {
            shrink_rate:      0.05,
            min_width_frac:   0.35,
            restore_nonwhiff: 0.25,
            restore_perfect:  0.50,
        });
        app.world_mut().insert_resource(ErosionState::default());
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Erosion);

        tick_with_dt(&mut app, Duration::from_secs(1));

        let state = app.world().resource::<ErosionState>();
        assert!(
            (state.width_fraction - 0.95).abs() < 1e-5,
            "1s at stack 1 with 0.05 rate → width 0.95, got {}",
            state.width_fraction
        );
    }

    #[test]
    fn shrink_stack_three_scales_linearly() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, erosion_shrink);
        app.world_mut().insert_resource(ErosionConfig {
            shrink_rate:      0.05,
            min_width_frac:   0.35,
            restore_nonwhiff: 0.25,
            restore_perfect:  0.50,
        });
        app.world_mut().insert_resource(ErosionState::default());
        for _ in 0..3 {
            app.world_mut()
                .resource_mut::<ActiveHazards>()
                .add_stack(HazardKind::Erosion);
        }

        tick_with_dt(&mut app, Duration::from_secs(1));

        let state = app.world().resource::<ErosionState>();
        // 1.0 - 0.05 * 3 = 0.85
        assert!((state.width_fraction - 0.85).abs() < 1e-5);
    }

    #[test]
    fn shrink_clamps_to_min_width_frac() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, erosion_shrink);
        app.world_mut().insert_resource(ErosionConfig {
            shrink_rate:      1.0, // huge — would go to 0 in one sec
            min_width_frac:   0.35,
            restore_nonwhiff: 0.25,
            restore_perfect:  0.50,
        });
        app.world_mut().insert_resource(ErosionState {
            width_fraction: 0.40,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Erosion);

        tick_with_dt(&mut app, Duration::from_secs(1));

        let state = app.world().resource::<ErosionState>();
        assert!((state.width_fraction - 0.35).abs() < f32::EPSILON);
    }

    // ── erosion_restore ───────────────────────────────────────────────────

    #[test]
    fn perfect_bump_restores_half_of_lost_width() {
        let mut app = test_app_playing();
        app.add_message::<BumpPerformed>();
        app.add_systems(FixedUpdate, erosion_restore);
        app.world_mut().insert_resource(ErosionConfig {
            shrink_rate:      0.05,
            min_width_frac:   0.35,
            restore_nonwhiff: 0.25,
            restore_perfect:  0.50,
        });
        app.world_mut().insert_resource(ErosionState {
            width_fraction: 0.60,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Erosion);

        app.world_mut().write_message(BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    None,
            breaker: Entity::PLACEHOLDER,
        });
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let state = app.world().resource::<ErosionState>();
        // lost = 0.40, restore = 0.40 * 0.50 = 0.20, width = 0.80
        assert!(
            (state.width_fraction - 0.80).abs() < 1e-5,
            "perfect bump from 0.60 with 50% restore → 0.80, got {}",
            state.width_fraction
        );
    }

    #[test]
    fn early_bump_restores_quarter_of_lost_width() {
        let mut app = test_app_playing();
        app.add_message::<BumpPerformed>();
        app.add_systems(FixedUpdate, erosion_restore);
        app.world_mut().insert_resource(ErosionConfig {
            shrink_rate:      0.05,
            min_width_frac:   0.35,
            restore_nonwhiff: 0.25,
            restore_perfect:  0.50,
        });
        app.world_mut().insert_resource(ErosionState {
            width_fraction: 0.60,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Erosion);

        app.world_mut().write_message(BumpPerformed {
            grade:   BumpGrade::Early,
            bolt:    None,
            breaker: Entity::PLACEHOLDER,
        });
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let state = app.world().resource::<ErosionState>();
        // lost = 0.40, restore = 0.40 * 0.25 = 0.10, width = 0.70
        assert!((state.width_fraction - 0.70).abs() < 1e-5);
    }

    #[test]
    fn late_bump_uses_nonwhiff_fraction() {
        let mut app = test_app_playing();
        app.add_message::<BumpPerformed>();
        app.add_systems(FixedUpdate, erosion_restore);
        app.world_mut().insert_resource(ErosionConfig {
            shrink_rate:      0.05,
            min_width_frac:   0.35,
            restore_nonwhiff: 0.25,
            restore_perfect:  0.50,
        });
        app.world_mut().insert_resource(ErosionState {
            width_fraction: 0.60,
        });

        app.world_mut().write_message(BumpPerformed {
            grade:   BumpGrade::Late,
            bolt:    None,
            breaker: Entity::PLACEHOLDER,
        });
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let state = app.world().resource::<ErosionState>();
        assert!((state.width_fraction - 0.70).abs() < 1e-5);
    }

    #[test]
    fn restore_clamps_to_full_width() {
        let mut app = test_app_playing();
        app.add_message::<BumpPerformed>();
        app.add_systems(FixedUpdate, erosion_restore);
        app.world_mut().insert_resource(ErosionConfig {
            shrink_rate:      0.05,
            min_width_frac:   0.35,
            restore_nonwhiff: 0.25,
            restore_perfect:  1.0, // 100% → restore to full
        });
        app.world_mut().insert_resource(ErosionState {
            width_fraction: 0.50,
        });

        app.world_mut().write_message(BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    None,
            breaker: Entity::PLACEHOLDER,
        });
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let state = app.world().resource::<ErosionState>();
        assert!((state.width_fraction - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn no_restore_when_already_at_full_width() {
        let mut app = test_app_playing();
        app.add_message::<BumpPerformed>();
        app.add_systems(FixedUpdate, erosion_restore);
        app.world_mut().insert_resource(ErosionConfig {
            shrink_rate:      0.05,
            min_width_frac:   0.35,
            restore_nonwhiff: 0.25,
            restore_perfect:  0.50,
        });
        app.world_mut().insert_resource(ErosionState {
            width_fraction: 1.0,
        });

        app.world_mut().write_message(BumpPerformed {
            grade:   BumpGrade::Perfect,
            bolt:    None,
            breaker: Entity::PLACEHOLDER,
        });
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let state = app.world().resource::<ErosionState>();
        assert!((state.width_fraction - 1.0).abs() < f32::EPSILON);
    }

    // ── erosion_apply_width ───────────────────────────────────────────────

    #[test]
    fn apply_width_inserts_stack_and_pushes_entry() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, erosion_apply_width);
        app.world_mut().insert_resource(ErosionState {
            width_fraction: 0.75,
        });
        let breaker = spawn_breaker(&mut app);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .expect("EffectStack inserted");
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 0.75).abs() < 1e-6);
    }

    #[test]
    fn apply_width_reconciles_to_single_entry_across_ticks() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, erosion_apply_width);
        app.world_mut().insert_resource(ErosionState {
            width_fraction: 0.80,
        });
        let breaker = spawn_breaker(&mut app);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 0.80).abs() < 1e-6);
    }

    #[test]
    fn apply_width_preserves_non_erosion_entries() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, erosion_apply_width);
        app.world_mut().insert_resource(ErosionState {
            width_fraction: 0.80,
        });

        let mut seed = EffectStack::<SizeBoostConfig>::default();
        seed.push(
            "chip:heavy".to_owned(),
            SizeBoostConfig {
                multiplier: OrderedFloat(1.25),
            },
        );
        let breaker = app.world_mut().spawn((Breaker, seed)).id();

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let stack = app
            .world()
            .get::<EffectStack<SizeBoostConfig>>(breaker)
            .unwrap();
        // 1.25 (chip) * 0.80 (erosion) = 1.0 aggregate
        assert_eq!(stack.len(), 2);
        assert!((stack.aggregate() - 1.0).abs() < 1e-5);
    }

    // ── activate — preserved scaffold tests ────────────────────────────────

    #[test]
    fn activate_with_matching_tuning_inserts_config_and_state() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &HazardTuning::Erosion {
                    shrink_rate:      0.05,
                    min_width_frac:   0.35,
                    restore_nonwhiff: 0.25,
                    restore_perfect:  0.5,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<ErosionConfig>();
        assert!((cfg.shrink_rate - 0.05).abs() < f32::EPSILON);
        let state = app.world().resource::<ErosionState>();
        assert!((state.width_fraction - 1.0).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<ErosionConfig>().is_none());
        assert!(app.world().get_resource::<ErosionState>().is_none());
    }
}
