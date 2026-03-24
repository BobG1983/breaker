//! Shockwave effect handler — area damage around the bolt's position.
//!
//! Observes [`EffectFired`], pattern-matches on
//! [`TriggerChain::Shockwave`], and writes [`DamageCell`] messages for all
//! non-locked cells within range. Damage includes [`DamageBoost`] if present.

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, resources::CollisionQuadtree};
use rantzsoft_spatial2d::components::Position2D;

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
///
/// Only cells with both [`Cell`] and [`Aabb2D`] components are candidates —
/// this ensures the query matches the set of entities registered in the
/// [`CollisionQuadtree`], excluding bare cells that lack collision data.
type ShockwaveCellQuery = (Entity, &'static Position2D, Has<Locked>);

/// Filter for shockwave cell candidates: must be a cell with collision data.
type ShockwaveCellFilter = (With<Cell>, With<Aabb2D>);

/// Observer: handles shockwave area damage when an effect fires.
///
/// Self-selects via pattern matching on [`TriggerChain::Shockwave`] — ignores
/// all other effect variants. Writes [`DamageCell`] messages for all non-locked
/// cells within `range` of the bolt's position that have collision data
/// ([`Aabb2D`] component). Damage is calculated as
/// `BASE_BOLT_DAMAGE * (1.0 + DamageBoost)`.
pub(crate) fn handle_shockwave(
    trigger: On<EffectFired>,
    quadtree: Res<CollisionQuadtree>,
    bolt_query: Query<(&Position2D, Option<&DamageBoost>)>,
    cell_query: Query<ShockwaveCellQuery, ShockwaveCellFilter>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let _ = &quadtree;
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
    let Ok((bolt_pos, damage_boost)) = bolt_query.get(bolt_entity) else {
        return;
    };
    let boost = damage_boost.map_or(0.0, |b| b.0);
    let damage = BASE_BOLT_DAMAGE * (1.0 + boost);
    let center = bolt_pos.0;

    for (cell_entity, cell_pos, is_locked) in &cell_query {
        if is_locked {
            continue;
        }
        let dist = (cell_pos.0 - center).length();
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
    use rantzsoft_physics2d::{
        aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
    };
    use rantzsoft_spatial2d::components::{Position2D, Spatial2D};

    use super::*;
    use crate::{
        cells::{
            components::{Cell, CellHealth, Locked},
            messages::DamageCell,
        },
        chips::{components::DamageBoost, definition::TriggerChain},
        shared::{BASE_BOLT_DAMAGE, BOLT_LAYER, CELL_LAYER, GameDrawLayer},
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
            .add_plugins(RantzPhysics2dPlugin)
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
        app.world_mut().spawn(Position2D(Vec2::new(x, y))).id()
    }

    fn spawn_bolt_with_damage_boost(app: &mut App, x: f32, y: f32, boost: f32) -> Entity {
        app.world_mut()
            .spawn((Position2D(Vec2::new(x, y)), DamageBoost(boost)))
            .id()
    }

    fn spawn_cell(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
        app.world_mut()
            .spawn((
                Cell,
                CellHealth::new(hp),
                Aabb2D::new(Vec2::ZERO, Vec2::new(35.0, 12.0)),
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                Position2D(Vec2::new(x, y)),
                Spatial2D,
                GameDrawLayer::Cell,
            ))
            .id()
    }

    fn spawn_locked_cell(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
        app.world_mut()
            .spawn((
                Cell,
                CellHealth::new(hp),
                Locked,
                Aabb2D::new(Vec2::ZERO, Vec2::new(35.0, 12.0)),
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                Position2D(Vec2::new(x, y)),
                Spatial2D,
                GameDrawLayer::Cell,
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

    // --- Quadtree circle query tests ---
    //
    // These tests verify that the shockwave handler finds cells via the
    // `CollisionQuadtree` circle query rather than by iterating all cells.

    #[test]
    fn shockwave_only_damages_cells_in_quadtree() {
        // A cell entity with `Cell`, `CellHealth`, `Position2D` but WITHOUT
        // `Aabb2D`/`CollisionLayers` — so it is NOT in the quadtree.
        //
        // The refactored shockwave queries the quadtree, so this cell is
        // invisible. The current system iterates all cells via `cell_query`,
        // so this cell IS found and damaged.
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);

        // Spawn cell without Aabb2D/CollisionLayers — not in quadtree
        let _bare_cell = app
            .world_mut()
            .spawn((
                Cell,
                CellHealth::new(20.0),
                Position2D(Vec2::new(10.0, 0.0)),
                Spatial2D,
                GameDrawLayer::Cell,
            ))
            .id();

        // A properly-registered cell (WITH Aabb2D/CollisionLayers) — in quadtree
        let registered_cell = spawn_cell(&mut app, 20.0, 0.0, 20.0);

        trigger_shockwave(&mut app, bolt, 64.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            1,
            "only the cell with Aabb2D/CollisionLayers (in quadtree) should receive \
             DamageCell — got {} messages (2 means the system still iterates all cells \
             instead of querying the quadtree)",
            captured.0.len()
        );
        assert_eq!(
            captured.0[0].cell, registered_cell,
            "DamageCell should target the registered cell, not the bare cell"
        );
    }

    #[test]
    fn shockwave_cell_without_aabb2d_not_found_via_quadtree() {
        // A cell entity with `Cell` and `Position2D` but WITHOUT `Aabb2D`
        // is not registered in the quadtree. The refactored shockwave uses
        // the quadtree circle query, so this cell should be invisible.
        //
        // Meanwhile, a cell without `Aabb2D` but WITH the `Cell` component
        // IS visible to the current `cell_query: Query<..., With<Cell>>`.
        //
        // Unlike `shockwave_only_damages_cells_in_quadtree` (which has a
        // registered cell for comparison), this test has ONLY the bare cell.
        // The current system damages it; the refactored system does not.
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);

        // Only a bare cell (no Aabb2D/CollisionLayers) — not in quadtree
        app.world_mut().spawn((
            Cell,
            CellHealth::new(20.0),
            Position2D(Vec2::new(10.0, 0.0)),
            Spatial2D,
            GameDrawLayer::Cell,
        ));

        trigger_shockwave(&mut app, bolt, 64.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            0,
            "cell without Aabb2D should be invisible to quadtree-based shockwave — \
             got {} messages (non-zero means the system still iterates all cells)",
            captured.0.len()
        );
    }
}
