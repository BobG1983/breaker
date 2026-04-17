//! Haste hazard — multiplies bolt speed via the `SpeedBoostConfig` stack.
//! Stacking adds linear percentage on top of the base. Reconciles every
//! tick so the bolt stack always reflects the current hazard stack count.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    bolt::components::Bolt,
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

/// Source tag used on the bolt's `EffectStack<SpeedBoostConfig>`. Shared
/// between the apply system and teardown so Haste can reconcile its own
/// entries without disturbing chip- or protocol-owned boosts.
const HASTE_SOURCE: &str = "hazard:haste";

/// Per-run tuning extracted from [`HazardTuning::Haste`] at activation.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct HasteConfig {
    /// Added speed percentage at stack 1 (e.g. `20.0` → +20%).
    pub(crate) base_percent:      f32,
    /// Additional speed percentage per stack beyond the first.
    pub(crate) per_level_percent: f32,
}

impl HasteConfig {
    /// Multiplicative speed factor for the given stack count. Returns 1.0
    /// when `stacks == 0` (hazard inactive).
    #[must_use]
    pub(crate) fn multiplier(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 1.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        let percent = self.per_level_percent.mul_add(extra, self.base_percent);
        1.0 + percent / 100.0
    }
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Haste {
        base_percent,
        per_level_percent,
    } = *tuning
    else {
        warn!("haste::activate called with non-Haste tuning");
        return;
    };
    commands.insert_resource(HasteConfig {
        base_percent,
        per_level_percent,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        haste_apply_speed
            .run_if(hazard_active(HazardKind::Haste))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Reconciles each Bolt's `EffectStack<SpeedBoostConfig>` so that exactly
/// one entry with source [`HASTE_SOURCE`] exists, carrying the current
/// multiplier. Bolts without a stack yet are given one this tick; the
/// entry is pushed on the same tick. Idempotent across ticks.
fn haste_apply_speed(
    active: Res<ActiveHazards>,
    config: Option<Res<HasteConfig>>,
    mut bolts: Query<(Entity, Option<&mut EffectStack<SpeedBoostConfig>>), With<Bolt>>,
    mut commands: Commands,
) {
    let Some(config) = config else { return };
    let stacks = active.stacks(HazardKind::Haste);
    let multiplier = config.multiplier(stacks);
    let entry = SpeedBoostConfig {
        multiplier: OrderedFloat(multiplier),
    };

    for (entity, stack) in &mut bolts {
        if let Some(mut stack) = stack {
            stack.retain_by_source(HASTE_SOURCE);
            stack.push(HASTE_SOURCE.to_owned(), entry.clone());
        } else {
            let mut fresh = EffectStack::<SpeedBoostConfig>::default();
            fresh.push(HASTE_SOURCE.to_owned(), entry.clone());
            commands.entity(entity).insert(fresh);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::TestAppBuilder;

    fn test_app_playing() -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_resource::<ActiveHazards>()
            .build()
    }

    fn spawn_bolt(app: &mut App) -> Entity {
        app.world_mut().spawn(Bolt).id()
    }

    fn run_fixed_update(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── multiplier formula ────────────────────────────────────────────────

    #[test]
    fn multiplier_zero_stacks_is_one() {
        let cfg = HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        };
        assert!((cfg.multiplier(0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn multiplier_stack_one_is_base_plus_one() {
        let cfg = HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        };
        assert!((cfg.multiplier(1) - 1.20).abs() < 1e-6);
    }

    #[test]
    fn multiplier_stack_three_adds_two_levels() {
        let cfg = HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        };
        // 1 + (20 + 10*2)/100 = 1.40
        assert!((cfg.multiplier(3) - 1.40).abs() < 1e-6);
    }

    // ── haste_apply_speed behaviour ───────────────────────────────────────

    #[test]
    fn bolt_without_stack_gets_one_with_haste_entry() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, haste_apply_speed);
        app.world_mut().insert_resource(HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Haste);
        let bolt = spawn_bolt(&mut app);

        run_fixed_update(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .expect("EffectStack inserted");
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 1.20).abs() < 1e-6);
    }

    #[test]
    fn reconciles_to_single_haste_entry_across_ticks() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, haste_apply_speed);
        app.world_mut().insert_resource(HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Haste);
        let bolt = spawn_bolt(&mut app);

        run_fixed_update(&mut app);
        run_fixed_update(&mut app);
        run_fixed_update(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .unwrap();
        assert_eq!(stack.len(), 1, "must not accumulate duplicate entries");
        assert!((stack.aggregate() - 1.20).abs() < 1e-6);
    }

    #[test]
    fn updates_multiplier_when_stacks_increase() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, haste_apply_speed);
        app.world_mut().insert_resource(HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Haste);
        let bolt = spawn_bolt(&mut app);

        run_fixed_update(&mut app);
        // Bump stacks to 3
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Haste);
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Haste);
        run_fixed_update(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .unwrap();
        assert_eq!(stack.len(), 1);
        assert!((stack.aggregate() - 1.40).abs() < 1e-6);
    }

    #[test]
    fn preserves_non_haste_entries_on_existing_stack() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, haste_apply_speed);
        app.world_mut().insert_resource(HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Haste);

        // Spawn bolt with a pre-existing chip-owned boost.
        let mut seed = EffectStack::<SpeedBoostConfig>::default();
        seed.push(
            "chip:overclock".to_owned(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        );
        let bolt = app.world_mut().spawn((Bolt, seed)).id();

        run_fixed_update(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .unwrap();
        // 1.5 (chip) * 1.20 (haste) = 1.80
        assert_eq!(stack.len(), 2);
        assert!((stack.aggregate() - 1.80).abs() < 1e-5);
    }

    #[test]
    fn no_apply_when_config_absent() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, haste_apply_speed);
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Haste);
        let bolt = spawn_bolt(&mut app);

        run_fixed_update(&mut app);

        assert!(
            app.world()
                .get::<EffectStack<SpeedBoostConfig>>(bolt)
                .is_none(),
            "no stack should be inserted without HasteConfig"
        );
    }

    // ── activate — preserved scaffold tests ────────────────────────────────

    #[test]
    fn activate_with_matching_tuning_inserts_config() {
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

        let cfg = app.world().resource::<HasteConfig>();
        assert!((cfg.base_percent - 0.1).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<HasteConfig>().is_none());
    }
}
