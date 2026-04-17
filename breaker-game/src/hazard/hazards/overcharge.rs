//! Overcharge hazard — the Bolt gains speed per cell destroyed within a
//! single bump cycle (between consecutive [`BumpPerformed`] events), and
//! resets on the next bump. Per-kill multiplier scales linearly with
//! stack count. The effective multiplier is the per-kill multiplier
//! raised to the cycle's kill count, applied through the Bolt's
//! [`EffectStack<SpeedBoostConfig>`] under a dedicated source tag.

use bevy::{platform::collections::HashMap, prelude::*};
use ordered_float::OrderedFloat;

use crate::{
    bolt::components::Bolt,
    breaker::messages::BumpPerformed,
    cells::components::Cell,
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
    shared::death_pipeline::Destroyed,
};

/// Source tag for Overcharge's entry on the Bolt's
/// [`EffectStack<SpeedBoostConfig>`]. Dedicated so Overcharge can swap
/// its entry on kill/reset without disturbing other speed contributions.
const OVERCHARGE_SOURCE: &str = "hazard:overcharge";

/// Per-run tuning extracted from [`HazardTuning::Overcharge`] at activation.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct OverchargeConfig {
    /// Per-kill speed fraction at stack 1 (e.g. `0.05` → +5% per kill).
    pub(crate) base_frac:      f32,
    /// Additional per-kill speed fraction per stack beyond the first.
    pub(crate) per_level_frac: f32,
}

impl OverchargeConfig {
    /// Per-kill multiplier for the given stack count. Returns 1.0 when
    /// `stacks == 0` (no hazard active).
    #[must_use]
    pub(crate) fn per_kill_multiplier(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 1.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        1.0 + self.per_level_frac.mul_add(extra, self.base_frac)
    }
}

/// Tracks the number of cells a given bolt has destroyed since its last
/// bump. Reset to 0 on [`BumpPerformed`]. Missing entries are treated as 0.
#[derive(Component, Debug, Default, Clone, Copy)]
pub(crate) struct OverchargeKillCount(pub(crate) u32);

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Overcharge {
        base_frac,
        per_level_frac,
    } = *tuning
    else {
        warn!("overcharge::activate called with non-Overcharge tuning");
        return;
    };
    commands.insert_resource(OverchargeConfig {
        base_frac,
        per_level_frac,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            overcharge_count_kills,
            overcharge_reset_on_bump,
            overcharge_apply_speed,
        )
            .chain()
            .run_if(hazard_active(HazardKind::Overcharge))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// For each `Destroyed<Cell>` whose killer is a Bolt, increment that
/// bolt's [`OverchargeKillCount`]. Inserts the component on first kill.
fn overcharge_count_kills(
    mut reader: MessageReader<Destroyed<Cell>>,
    bolts: Query<Entity, With<Bolt>>,
    mut counts: Query<&mut OverchargeKillCount>,
    mut commands: Commands,
) {
    // Tally kills per bolt first so multiple kills in one tick don't
    // race against pending Commands inserts.
    let mut batch: HashMap<Entity, u32> = HashMap::default();
    for destroyed in reader.read() {
        let Some(killer) = destroyed.killer else {
            continue;
        };
        if !bolts.contains(killer) {
            continue;
        }
        *batch.entry(killer).or_insert(0) += 1;
    }

    for (bolt, kills) in batch {
        if let Ok(mut count) = counts.get_mut(bolt) {
            count.0 = count.0.saturating_add(kills);
        } else {
            commands.entity(bolt).insert(OverchargeKillCount(kills));
        }
    }
}

/// Resets a bolt's kill count on every `BumpPerformed` for that bolt.
/// Missing [`OverchargeKillCount`] is a no-op — zero is zero.
fn overcharge_reset_on_bump(
    mut reader: MessageReader<BumpPerformed>,
    mut counts: Query<&mut OverchargeKillCount>,
) {
    for bump in reader.read() {
        let Some(bolt) = bump.bolt else { continue };
        if let Ok(mut count) = counts.get_mut(bolt) {
            count.0 = 0;
        }
    }
}

/// Aggregated query over Bolts carrying an [`OverchargeKillCount`] and
/// (optionally) a [`SpeedBoostConfig`] effect stack.
type BoltSpeedQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        Option<&'static OverchargeKillCount>,
        Option<&'static mut EffectStack<SpeedBoostConfig>>,
    ),
    With<Bolt>,
>;

/// Reconciles each Bolt's [`EffectStack<SpeedBoostConfig>`] so that
/// exactly one entry with source [`OVERCHARGE_SOURCE`] carries
/// `per_kill_multiplier ^ kill_count`. Bolts with zero kills get no
/// entry (the reconcile retains no stale entry either). Idempotent.
fn overcharge_apply_speed(
    active: Res<ActiveHazards>,
    config: Option<Res<OverchargeConfig>>,
    mut bolts: BoltSpeedQuery,
    mut commands: Commands,
) {
    let Some(config) = config else { return };
    let stacks = active.stacks(HazardKind::Overcharge);
    let per_kill = config.per_kill_multiplier(stacks);

    for (entity, count, stack) in &mut bolts {
        let kills = count.map_or(0, |c| c.0);
        let multiplier = per_kill.powi(kills.cast_signed());

        if let Some(mut stack) = stack {
            stack.retain_by_source(OVERCHARGE_SOURCE);
            if kills > 0 {
                stack.push(
                    OVERCHARGE_SOURCE.to_owned(),
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(multiplier),
                    },
                );
            }
        } else if kills > 0 {
            let mut fresh = EffectStack::<SpeedBoostConfig>::default();
            fresh.push(
                OVERCHARGE_SOURCE.to_owned(),
                SpeedBoostConfig {
                    multiplier: OrderedFloat(multiplier),
                },
            );
            commands.entity(entity).insert(fresh);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use bevy::prelude::Messages;

    use super::*;
    use crate::prelude::TestAppBuilder;

    fn test_app_playing() -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_resource::<ActiveHazards>()
            .with_message::<Destroyed<Cell>>()
            .with_message::<BumpPerformed>()
            .build()
    }

    fn spawn_bolt(app: &mut App) -> Entity {
        app.world_mut().spawn(Bolt).id()
    }

    fn spawn_cell(app: &mut App) -> Entity {
        app.world_mut().spawn(Cell).id()
    }

    fn write_cell_destroyed(app: &mut App, victim: Entity, killer: Option<Entity>) {
        app.world_mut()
            .resource_mut::<Messages<Destroyed<Cell>>>()
            .write(Destroyed::<Cell> {
                victim,
                killer,
                victim_pos: Vec2::ZERO,
                killer_pos: None,
                _marker: PhantomData,
            });
    }

    fn run_fixed_update(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── per_kill_multiplier formula ───────────────────────────────────────

    #[test]
    fn per_kill_zero_stacks_is_one() {
        let cfg = OverchargeConfig {
            base_frac:      0.05,
            per_level_frac: 0.03,
        };
        assert!((cfg.per_kill_multiplier(0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn per_kill_stack_one_is_base_plus_one() {
        let cfg = OverchargeConfig {
            base_frac:      0.05,
            per_level_frac: 0.03,
        };
        assert!((cfg.per_kill_multiplier(1) - 1.05).abs() < 1e-6);
    }

    #[test]
    fn per_kill_stack_three_adds_two_levels() {
        let cfg = OverchargeConfig {
            base_frac:      0.05,
            per_level_frac: 0.03,
        };
        // 1 + 0.05 + 0.03*2 = 1.11
        assert!((cfg.per_kill_multiplier(3) - 1.11).abs() < 1e-6);
    }

    // ── overcharge_count_kills ────────────────────────────────────────────

    #[test]
    fn first_kill_inserts_count_of_one() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, overcharge_count_kills);
        app.world_mut().insert_resource(OverchargeConfig {
            base_frac:      0.05,
            per_level_frac: 0.03,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Overcharge);
        let bolt = spawn_bolt(&mut app);
        let cell = spawn_cell(&mut app);
        write_cell_destroyed(&mut app, cell, Some(bolt));

        run_fixed_update(&mut app);

        let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
        assert_eq!(count.0, 1);
    }

    #[test]
    fn subsequent_kills_accumulate() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, overcharge_count_kills);
        app.world_mut().insert_resource(OverchargeConfig {
            base_frac:      0.05,
            per_level_frac: 0.03,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Overcharge);
        let bolt = spawn_bolt(&mut app);
        let cell_a = spawn_cell(&mut app);
        let cell_b = spawn_cell(&mut app);
        let cell_c = spawn_cell(&mut app);
        write_cell_destroyed(&mut app, cell_a, Some(bolt));
        write_cell_destroyed(&mut app, cell_b, Some(bolt));
        write_cell_destroyed(&mut app, cell_c, Some(bolt));
        run_fixed_update(&mut app);

        let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
        assert_eq!(count.0, 3);
    }

    #[test]
    fn ignores_kills_with_no_killer() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, overcharge_count_kills);
        app.world_mut().insert_resource(OverchargeConfig {
            base_frac:      0.05,
            per_level_frac: 0.03,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Overcharge);
        let bolt = spawn_bolt(&mut app);
        let cell = spawn_cell(&mut app);
        write_cell_destroyed(&mut app, cell, None);

        run_fixed_update(&mut app);

        assert!(
            app.world().get::<OverchargeKillCount>(bolt).is_none(),
            "no count should be inserted for environmental kills"
        );
    }

    #[test]
    fn ignores_kills_by_non_bolt_killers() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, overcharge_count_kills);
        app.world_mut().insert_resource(OverchargeConfig {
            base_frac:      0.05,
            per_level_frac: 0.03,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Overcharge);
        let not_bolt = app.world_mut().spawn_empty().id();
        let cell = spawn_cell(&mut app);
        write_cell_destroyed(&mut app, cell, Some(not_bolt));

        run_fixed_update(&mut app);

        assert!(
            app.world().get::<OverchargeKillCount>(not_bolt).is_none(),
            "killer must be a Bolt to accumulate"
        );
    }

    // ── overcharge_reset_on_bump ──────────────────────────────────────────

    #[test]
    fn bump_resets_kill_count_for_bumping_bolt() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, overcharge_reset_on_bump);
        let bolt = app.world_mut().spawn((Bolt, OverchargeKillCount(5))).id();

        app.world_mut().write_message(BumpPerformed {
            grade:   crate::breaker::messages::BumpGrade::Perfect,
            bolt:    Some(bolt),
            breaker: Entity::PLACEHOLDER,
        });
        run_fixed_update(&mut app);

        let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
        assert_eq!(count.0, 0);
    }

    #[test]
    fn bump_without_bolt_is_noop() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, overcharge_reset_on_bump);
        let bolt = app.world_mut().spawn((Bolt, OverchargeKillCount(3))).id();

        app.world_mut().write_message(BumpPerformed {
            grade:   crate::breaker::messages::BumpGrade::Perfect,
            bolt:    None,
            breaker: Entity::PLACEHOLDER,
        });
        run_fixed_update(&mut app);

        let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
        assert_eq!(count.0, 3, "no bolt identified on the bump → no reset");
    }

    // ── overcharge_apply_speed ────────────────────────────────────────────

    #[test]
    fn apply_speed_pushes_compounding_multiplier() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, overcharge_apply_speed);
        app.world_mut().insert_resource(OverchargeConfig {
            base_frac:      0.05,
            per_level_frac: 0.03,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Overcharge);
        let bolt = app.world_mut().spawn((Bolt, OverchargeKillCount(3))).id();

        run_fixed_update(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .expect("EffectStack inserted");
        assert_eq!(stack.len(), 1);
        // (1.05)^3 ≈ 1.157625
        assert!(
            (stack.aggregate() - 1.05_f32.powi(3)).abs() < 1e-5,
            "expected 1.05^3 = {}, got {}",
            1.05_f32.powi(3),
            stack.aggregate()
        );
    }

    #[test]
    fn apply_speed_with_zero_kills_inserts_no_entry() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, overcharge_apply_speed);
        app.world_mut().insert_resource(OverchargeConfig {
            base_frac:      0.05,
            per_level_frac: 0.03,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Overcharge);
        let bolt = spawn_bolt(&mut app);

        run_fixed_update(&mut app);

        assert!(
            app.world()
                .get::<EffectStack<SpeedBoostConfig>>(bolt)
                .is_none(),
            "bolts with zero kills should not receive an Overcharge entry"
        );
    }

    #[test]
    fn apply_speed_reconciles_on_reset() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, overcharge_apply_speed);
        app.world_mut().insert_resource(OverchargeConfig {
            base_frac:      0.05,
            per_level_frac: 0.03,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Overcharge);
        let bolt = app.world_mut().spawn((Bolt, OverchargeKillCount(4))).id();

        run_fixed_update(&mut app);
        // Simulate a bump reset: kill count goes to 0
        *app.world_mut()
            .get_mut::<OverchargeKillCount>(bolt)
            .unwrap() = OverchargeKillCount(0);
        run_fixed_update(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .unwrap();
        assert_eq!(stack.len(), 0, "Overcharge entry must be removed on reset");
    }

    #[test]
    fn apply_speed_preserves_non_overcharge_entries() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, overcharge_apply_speed);
        app.world_mut().insert_resource(OverchargeConfig {
            base_frac:      0.05,
            per_level_frac: 0.03,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Overcharge);

        let mut seed = EffectStack::<SpeedBoostConfig>::default();
        seed.push(
            "chip:overclock".to_owned(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        );
        let bolt = app
            .world_mut()
            .spawn((Bolt, OverchargeKillCount(2), seed))
            .id();

        run_fixed_update(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<SpeedBoostConfig>>(bolt)
            .unwrap();
        assert_eq!(stack.len(), 2);
        // 1.5 (chip) * 1.05^2 (overcharge) = 1.65375
        let expected = 1.5 * 1.05_f32.powi(2);
        assert!((stack.aggregate() - expected).abs() < 1e-5);
    }

    // ── activate — preserved scaffold tests ────────────────────────────────

    #[test]
    fn activate_with_matching_tuning_inserts_config() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &HazardTuning::Overcharge {
                    base_frac:      0.1,
                    per_level_frac: 0.05,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<OverchargeConfig>();
        assert!((cfg.base_frac - 0.1).abs() < f32::EPSILON);
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
        assert!(app.world().get_resource::<OverchargeConfig>().is_none());
    }
}
