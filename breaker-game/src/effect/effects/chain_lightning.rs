//! Chain lightning effect handler — arcs lightning between nearby cells.
//!
//! Observes [`ChainLightningFired`] and damages nearby cells in an arc.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{collision_layers::CollisionLayers, resources::CollisionQuadtree};
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    cells::{components::Locked, messages::DamageCell},
    chips::components::DamageBoost,
    effect::definition::EffectTarget,
    shared::{BASE_BOLT_DAMAGE, CELL_LAYER},
};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a chain lightning effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct ChainLightningFired {
    /// Number of arcs from the origin cell.
    pub arcs: u32,
    /// Maximum arc range in world units.
    pub range: f32,
    /// Damage multiplier per arc.
    pub damage_mult: f32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Observer: handles chain lightning — arcs damage between nearby cells.
///
/// Starting from the bolt's position, finds the nearest unlocked cell within
/// `range`, damages it, then chains from that cell's position to the next
/// nearest unvisited unlocked cell, up to `arcs` total arcs.
pub(crate) fn handle_chain_lightning(
    trigger: On<ChainLightningFired>,
    quadtree: Res<CollisionQuadtree>,
    bolt_query: Query<(&Position2D, Option<&DamageBoost>)>,
    cell_query: Query<(&Position2D, Has<Locked>)>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let event = trigger.event();

    // Extract origin from targets
    let origin = event.targets.iter().find_map(|t| match t {
        EffectTarget::Entity(e) => bolt_query.get(*e).ok().map(|(pos, _)| (*e, pos.0)),
        EffectTarget::Location(pos) => Some((Entity::PLACEHOLDER, *pos)),
    });
    let Some((bolt_entity, origin_pos)) = origin else {
        return;
    };

    let damage_boost = bolt_query
        .get(bolt_entity)
        .ok()
        .and_then(|(_, db)| db)
        .map_or(0.0, |b| b.0);
    let damage = BASE_BOLT_DAMAGE * (1.0 + damage_boost) * event.damage_mult;

    let mut visited = HashSet::new();
    let mut current_pos = origin_pos;

    for _ in 0..event.arcs {
        let candidates = quadtree.quadtree.query_circle_filtered(
            current_pos,
            event.range,
            CollisionLayers::new(0, CELL_LAYER),
        );

        // Find nearest unvisited, unlocked cell
        let nearest = candidates
            .iter()
            .filter(|&&e| !visited.contains(&e))
            .filter(|&&e| cell_query.get(e).is_ok_and(|(_, locked)| !locked))
            .filter_map(|&e| cell_query.get(e).ok().map(|(pos, _)| (e, pos.0)))
            .min_by(|a, b| {
                a.1.distance_squared(current_pos)
                    .total_cmp(&b.1.distance_squared(current_pos))
            });

        let Some((cell_entity, cell_pos)) = nearest else {
            break;
        };

        damage_writer.write(DamageCell {
            cell: cell_entity,
            damage,
            source_bolt: Some(bolt_entity),
            source_chip: event.source_chip.clone(),
        });

        visited.insert(cell_entity);
        current_pos = cell_pos;
    }
}

/// Registers all observers and systems for the chain lightning effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_chain_lightning);
}

#[cfg(test)]
mod tests {
    use rantzsoft_physics2d::{
        aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
    };
    use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D};

    use super::*;
    use crate::{
        cells::{
            components::{Cell, CellHealth, Locked},
            messages::DamageCell,
        },
        chips::components::DamageBoost,
        shared::{BASE_BOLT_DAMAGE, BOLT_LAYER, CELL_LAYER, GameDrawLayer},
    };

    // --- Test infrastructure ---

    /// Captured `DamageCell` messages written by the chain lightning handler.
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
            .add_systems(FixedPostUpdate, capture_damage)
            .add_observer(handle_chain_lightning);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn spawn_cell(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
        let pos = Vec2::new(x, y);
        app.world_mut()
            .spawn((
                Cell,
                CellHealth::new(hp),
                Aabb2D::new(Vec2::ZERO, Vec2::new(35.0, 12.0)),
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
                GameDrawLayer::Cell,
            ))
            .id()
    }

    fn spawn_locked_cell(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
        let pos = Vec2::new(x, y);
        app.world_mut()
            .spawn((
                Cell,
                CellHealth::new(hp),
                Locked,
                Aabb2D::new(Vec2::ZERO, Vec2::new(35.0, 12.0)),
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
                GameDrawLayer::Cell,
            ))
            .id()
    }

    fn spawn_bolt(app: &mut App, x: f32, y: f32) -> Entity {
        app.world_mut().spawn(Position2D(Vec2::new(x, y))).id()
    }

    fn spawn_bolt_with_damage_boost(app: &mut App, x: f32, y: f32, boost: f32) -> Entity {
        app.world_mut()
            .spawn((Position2D(Vec2::new(x, y)), DamageBoost(boost)))
            .id()
    }

    fn trigger_chain_lightning(
        app: &mut App,
        bolt: Entity,
        arcs: u32,
        range: f32,
        damage_mult: f32,
    ) {
        use crate::effect::typed_events::ChainLightningFired;

        app.world_mut().commands().trigger(ChainLightningFired {
            arcs,
            range,
            damage_mult,
            targets: vec![EffectTarget::Entity(bolt)],
            source_chip: None,
        });
        app.world_mut().flush();
        tick(app);
    }

    // --- Tests ---

    #[test]
    fn handle_chain_lightning_does_not_panic() {
        use crate::effect::typed_events::ChainLightningFired;

        let mut app = test_app();

        app.world_mut().commands().trigger(ChainLightningFired {
            arcs: 3,
            range: 100.0,
            damage_mult: 1.0,
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();

        // Stub handler should not panic when receiving its typed event.
    }

    /// Chain lightning with arcs:1, range:100. Bolt at (0,200), Cell at (30,210).
    /// Distance ~32 units, within range 100. Should produce 1 `DamageCell` with
    /// damage = `BASE_BOLT_DAMAGE` * 1.0 = 10.0.
    #[test]
    fn chain_lightning_damages_nearest_cell() {
        let mut app = test_app();

        // Spawn the cell first, let quadtree update
        let cell = spawn_cell(&mut app, 30.0, 210.0, 50.0);
        tick(&mut app);

        // Now trigger chain lightning from a bolt at (0, 200)
        let bolt = spawn_bolt(&mut app, 0.0, 200.0);
        trigger_chain_lightning(&mut app, bolt, 1, 100.0, 1.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            1,
            "chain lightning arcs:1, cell at ~32 units should produce 1 DamageCell, got {}",
            captured.0.len()
        );
        assert_eq!(
            captured.0[0].cell, cell,
            "DamageCell should target the nearby cell"
        );
        let expected_damage = BASE_BOLT_DAMAGE * 1.0;
        assert!(
            (captured.0[0].damage - expected_damage).abs() < f32::EPSILON,
            "damage should be {expected_damage} (BASE_BOLT_DAMAGE * damage_mult 1.0), got {}",
            captured.0[0].damage
        );
    }

    /// Chain lightning with arcs:3, range:60. Bolt at (0,0), cells at (50,0),
    /// (100,0), (200,0). A->B is 50 units (in range), B->C is 100 units
    /// (beyond range 60). Should produce 2 `DamageCell` messages for A and B.
    #[test]
    fn chain_lightning_chains_between_cells() {
        let mut app = test_app();

        let cell_a = spawn_cell(&mut app, 50.0, 0.0, 50.0);
        let cell_b = spawn_cell(&mut app, 100.0, 0.0, 50.0);
        let _cell_c = spawn_cell(&mut app, 200.0, 0.0, 50.0);
        tick(&mut app);

        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        trigger_chain_lightning(&mut app, bolt, 3, 60.0, 1.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            2,
            "chain lightning arcs:3, range:60 should hit A (50 from bolt) and B (50 from A), \
             but not C (100 from B, beyond range 60); got {} hits",
            captured.0.len()
        );
        let hit_entities: Vec<Entity> = captured.0.iter().map(|d| d.cell).collect();
        assert!(
            hit_entities.contains(&cell_a),
            "cell A at (50,0) should be hit, got {hit_entities:?}"
        );
        assert!(
            hit_entities.contains(&cell_b),
            "cell B at (100,0) should be hit, got {hit_entities:?}"
        );
    }

    /// Chain lightning with arcs:2, range:70. Locked cell A at (30,0),
    /// normal cell B at (60,0). Locked cells should be skipped.
    /// Only B should receive `DamageCell`.
    #[test]
    fn chain_lightning_skips_locked_cells() {
        let mut app = test_app();

        let _locked_a = spawn_locked_cell(&mut app, 30.0, 0.0, 50.0);
        let cell_b = spawn_cell(&mut app, 60.0, 0.0, 50.0);
        tick(&mut app);

        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        trigger_chain_lightning(&mut app, bolt, 2, 70.0, 1.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            1,
            "chain lightning should skip locked cell A, only hit cell B; got {} hits",
            captured.0.len()
        );
        assert_eq!(
            captured.0[0].cell, cell_b,
            "only unlocked cell B should receive DamageCell"
        );
    }

    /// Chain lightning with range:50, no cells within range. Should produce
    /// zero `DamageCell` messages (no-op).
    #[test]
    fn chain_lightning_no_cells_in_range_is_noop() {
        let mut app = test_app();

        // Cell at (200, 0) — well beyond range 50
        let _far_cell = spawn_cell(&mut app, 200.0, 0.0, 50.0);
        tick(&mut app);

        let bolt = spawn_bolt(&mut app, 0.0, 0.0);
        trigger_chain_lightning(&mut app, bolt, 3, 50.0, 1.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            0,
            "no cells within range 50 should produce 0 DamageCell, got {}",
            captured.0.len()
        );
    }

    /// Chain lightning with `DamageBoost(0.5)` on the bolt, `damage_mult:1.0`.
    /// Damage = `BASE_BOLT_DAMAGE` * (1.0 + 0.5) * `damage_mult` = 10.0 * 1.5 * 1.0 = 15.0.
    #[test]
    fn chain_lightning_respects_damage_boost() {
        let mut app = test_app();

        let cell = spawn_cell(&mut app, 30.0, 0.0, 50.0);
        tick(&mut app);

        let bolt = spawn_bolt_with_damage_boost(&mut app, 0.0, 0.0, 0.5);
        trigger_chain_lightning(&mut app, bolt, 1, 100.0, 1.0);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            1,
            "should produce 1 DamageCell, got {}",
            captured.0.len()
        );
        assert_eq!(captured.0[0].cell, cell);
        // damage = BASE_BOLT_DAMAGE * (1.0 + 0.5) * damage_mult(1.0) = 15.0
        assert!(
            (captured.0[0].damage - 15.0).abs() < f32::EPSILON,
            "with DamageBoost(0.5), damage should be 15.0 (10.0 * 1.5), got {}",
            captured.0[0].damage
        );
    }
}
