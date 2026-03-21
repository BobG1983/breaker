//! Shockwave effect handler — area damage around the bolt's position.
//!
//! Observes [`EffectFired`], pattern-matches on
//! [`TriggerChain::Shockwave`], and writes [`DamageCell`] messages for all
//! non-locked cells within range. Damage includes [`DamageBoost`] if present.

use bevy::prelude::*;

use crate::{
    behaviors::events::EffectFired,
    cells::{
        components::{Cell, Locked},
        messages::DamageCell,
    },
    chips::{components::DamageBoost, definition::TriggerChain},
    shared::BASE_BOLT_DAMAGE,
};

/// Cell data needed by the shockwave effect handler.
type ShockwaveCellQuery = (Entity, &'static Transform, Has<Locked>);

/// Observer: handles shockwave area damage when an effect fires.
///
/// Self-selects via pattern matching on [`TriggerChain::Shockwave`] — ignores
/// all other effect variants. Writes [`DamageCell`] messages for all non-locked
/// cells within `range` of the bolt's position. Damage is calculated as
/// `BASE_BOLT_DAMAGE * (1.0 + DamageBoost)`.
pub(crate) fn handle_shockwave(
    trigger: On<EffectFired>,
    bolt_query: Query<(&Transform, Option<&DamageBoost>)>,
    cell_query: Query<ShockwaveCellQuery, With<Cell>>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let TriggerChain::Shockwave {
        base_range,
        range_per_level,
        stacks,
    } = &trigger.event().effect
    else {
        return;
    };
    #[expect(
        clippy::cast_precision_loss,
        reason = "stacks is always small (< max_stacks ≈ 5)"
    )]
    let extra_stacks = (*stacks).saturating_sub(1) as f32;
    let range = extra_stacks.mul_add(*range_per_level, *base_range);
    let Some(bolt_entity) = trigger.event().bolt else {
        return;
    };
    let Ok((bolt_tf, damage_boost)) = bolt_query.get(bolt_entity) else {
        return;
    };
    let boost = damage_boost.map_or(0.0, |b| b.0);
    let damage = BASE_BOLT_DAMAGE * (1.0 + boost);
    let center = bolt_tf.translation.truncate();

    for (cell_entity, cell_tf, is_locked) in &cell_query {
        if is_locked {
            continue;
        }
        let dist = (cell_tf.translation.truncate() - center).length();
        if dist <= range {
            damage_writer.write(DamageCell {
                cell: cell_entity,
                damage,
                source_bolt: bolt_entity,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cells::{
            components::{Cell, CellHealth, Locked},
            messages::DamageCell,
        },
        chips::{components::DamageBoost, definition::TriggerChain},
        shared::BASE_BOLT_DAMAGE,
    };

    // --- Test infrastructure ---

    /// Captured `DamageCell` messages written by the shockwave observer.
    #[derive(Resource, Default)]
    struct CapturedDamage(Vec<DamageCell>);

    fn capture_damage(mut reader: MessageReader<DamageCell>, mut captured: ResMut<CapturedDamage>) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<DamageCell>()
            .init_resource::<CapturedDamage>()
            .add_systems(FixedUpdate, capture_damage)
            .add_observer(handle_shockwave);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn spawn_bolt(app: &mut App, x: f32, y: f32) -> Entity {
        app.world_mut().spawn(Transform::from_xyz(x, y, 0.0)).id()
    }

    fn spawn_bolt_with_damage_boost(app: &mut App, x: f32, y: f32, boost: f32) -> Entity {
        app.world_mut()
            .spawn((Transform::from_xyz(x, y, 0.0), DamageBoost(boost)))
            .id()
    }

    fn spawn_cell(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
        app.world_mut()
            .spawn((Cell, CellHealth::new(hp), Transform::from_xyz(x, y, 0.0)))
            .id()
    }

    fn spawn_locked_cell(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
        app.world_mut()
            .spawn((
                Cell,
                CellHealth::new(hp),
                Locked,
                Transform::from_xyz(x, y, 0.0),
            ))
            .id()
    }

    fn trigger_shockwave(app: &mut App, bolt: Entity, range: f32) {
        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::Shockwave {
                base_range: range,
                range_per_level: 0.0,
                stacks: 1,
            },
            bolt: Some(bolt),
        });
        app.world_mut().flush();
        tick(app);
    }

    fn trigger_shockwave_stacked(
        app: &mut App,
        bolt: Entity,
        base_range: f32,
        range_per_level: f32,
        stacks: u32,
    ) {
        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::Shockwave {
                base_range,
                range_per_level,
                stacks,
            },
            bolt: Some(bolt),
        });
        app.world_mut().flush();
        tick(app);
    }

    // --- Tests ---

    #[test]
    fn shockwave_writes_damage_cell_for_in_range_cells() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let cell_a = spawn_cell(&mut app, 30.0, 0.0, 20.0);
        let cell_b = spawn_cell(&mut app, 50.0, 0.0, 20.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            2,
            "shockwave with two in-range cells should write two DamageCell messages, got {}",
            captured.0.len()
        );

        let msg_a = captured
            .0
            .iter()
            .find(|m| m.cell == cell_a)
            .expect("should have a DamageCell for cell A at (30, 0)");
        assert!(
            (msg_a.damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
            "DamageCell.damage should be {}, got {}",
            BASE_BOLT_DAMAGE,
            msg_a.damage
        );
        assert_eq!(msg_a.source_bolt, bolt);

        let msg_b = captured
            .0
            .iter()
            .find(|m| m.cell == cell_b)
            .expect("should have a DamageCell for cell B at (50, 0)");
        assert!(
            (msg_b.damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
            "DamageCell.damage should be {}, got {}",
            BASE_BOLT_DAMAGE,
            msg_b.damage
        );
    }

    #[test]
    fn shockwave_skips_locked_cells() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let _locked = spawn_locked_cell(&mut app, 10.0, 0.0, 10.0);
        let unlocked = spawn_cell(&mut app, 20.0, 0.0, 20.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            1,
            "only one DamageCell for the unlocked cell, got {}",
            captured.0.len()
        );
        assert_eq!(
            captured.0[0].cell, unlocked,
            "DamageCell should target the unlocked cell, not the locked one"
        );
    }

    #[test]
    fn shockwave_applies_damage_boost() {
        let mut app = test_app();
        let bolt = spawn_bolt_with_damage_boost(&mut app, 0.0, 0.0, 0.5);
        let _cell = spawn_cell(&mut app, 10.0, 0.0, 20.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(captured.0.len(), 1, "one DamageCell should be written");
        // damage = BASE_BOLT_DAMAGE * (1.0 + 0.5) = 10.0 * 1.5 = 15.0
        assert!(
            (captured.0[0].damage - 15.0).abs() < f32::EPSILON,
            "DamageCell.damage with DamageBoost(0.5) should be 15.0, got {}",
            captured.0[0].damage
        );
    }

    #[test]
    fn shockwave_no_op_when_bolt_is_none() {
        let mut app = test_app();
        let _cell = spawn_cell(&mut app, 10.0, 0.0, 10.0);

        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::test_shockwave(64.0),
            bolt: None,
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            0,
            "no DamageCell messages when bolt is None, got {}",
            captured.0.len()
        );
    }

    #[test]
    fn shockwave_ignores_non_shockwave_effects() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let _cell = spawn_cell(&mut app, 10.0, 0.0, 20.0);

        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::MultiBolt {
                base_count: 3,
                count_per_level: 0,
                stacks: 1,
            },
            bolt: Some(bolt),
        });
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            0,
            "MultiBolt effect should not produce any DamageCell messages, got {}",
            captured.0.len()
        );
    }

    #[test]
    fn shockwave_uses_stacked_range() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let cell_30 = spawn_cell(&mut app, 30.0, 0.0, 20.0);
        let cell_50 = spawn_cell(&mut app, 50.0, 0.0, 20.0);
        let cell_70 = spawn_cell(&mut app, 70.0, 0.0, 20.0);
        let _cell_100 = spawn_cell(&mut app, 100.0, 0.0, 20.0);

        // Shockwave { base_range: 64.0, range_per_level: 32.0, stacks: 2 }
        // effective range = 64.0 + (2-1)*32.0 = 96.0
        trigger_shockwave_stacked(&mut app, bolt, 64.0, 32.0, 2);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            3,
            "stacks=2 effective range 96.0: cells at 30, 50, 70 should be hit, not cell at 100; got {} hits",
            captured.0.len()
        );
        let hit_cells: Vec<Entity> = captured.0.iter().map(|m| m.cell).collect();
        assert!(
            hit_cells.contains(&cell_30),
            "cell at 30 should be within effective range 96.0"
        );
        assert!(
            hit_cells.contains(&cell_50),
            "cell at 50 should be within effective range 96.0"
        );
        assert!(
            hit_cells.contains(&cell_70),
            "cell at 70 should be within effective range 96.0"
        );
    }
}
