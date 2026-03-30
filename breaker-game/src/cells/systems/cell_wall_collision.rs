//! Cell-wall collision detection.
//!
//! Detects when a cell entity overlaps a wall entity and sends
//! [`CellImpactWall`] messages. Uses the spatial quadtree for
//! broad-phase filtering. Used by effect triggers to fire
//! `Impact(Wall)` / `Impacted(Cell)` chains.

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    cells::{components::Cell, messages::CellImpactWall},
    shared::{CELL_LAYER, WALL_LAYER},
    wall::components::Wall,
};

/// Wall entity lookup for narrow-phase overlap verification.
type WallLookup<'w, 's> = Query<'w, 's, (&'static Position2D, &'static Aabb2D), With<Wall>>;

/// Detects cell-wall collisions via quadtree AABB query.
///
/// For each cell, queries the quadtree for nearby wall entities.
/// Broad-phase candidates are verified with a narrow-phase AABB overlap
/// check before sending [`CellImpactWall`].
pub(crate) fn cell_wall_collision(
    quadtree: Res<CollisionQuadtree>,
    cell_query: Query<(Entity, &Position2D, &Aabb2D), With<Cell>>,
    wall_lookup: WallLookup,
    mut writer: MessageWriter<CellImpactWall>,
) {
    let layers = CollisionLayers::new(CELL_LAYER, WALL_LAYER);

    for (cell_entity, cell_pos, cell_aabb) in &cell_query {
        let cell_aabb_query = Aabb2D::new(cell_pos.0, cell_aabb.half_extents);
        let candidates = quadtree
            .quadtree
            .query_aabb_filtered(&cell_aabb_query, layers);

        for wall_entity in candidates {
            let Ok((wall_pos, wall_aabb)) = wall_lookup.get(wall_entity) else {
                continue;
            };

            // Narrow-phase: verify actual AABB overlap
            let dx = (cell_pos.0.x - wall_pos.0.x).abs();
            let dy = (cell_pos.0.y - wall_pos.0.y).abs();
            if dx < cell_aabb.half_extents.x + wall_aabb.half_extents.x
                && dy < cell_aabb.half_extents.y + wall_aabb.half_extents.y
            {
                writer.write(CellImpactWall {
                    cell: cell_entity,
                    wall: wall_entity,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use rantzsoft_physics2d::plugin::RantzPhysics2dPlugin;
    use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

    use super::*;
    use crate::shared::GameDrawLayer;

    // ── Helpers ──────────────────────────────────────────────────────

    #[derive(Resource, Default)]
    struct CellWallHitMessages(Vec<CellImpactWall>);

    fn collect_cell_wall_hits(
        mut reader: MessageReader<CellImpactWall>,
        mut msgs: ResMut<CellWallHitMessages>,
    ) {
        for msg in reader.read() {
            msgs.0.push(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(RantzPhysics2dPlugin)
            .add_message::<CellImpactWall>()
            .insert_resource(CellWallHitMessages::default())
            .add_systems(
                FixedUpdate,
                cell_wall_collision
                    .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
            )
            .add_systems(
                FixedUpdate,
                collect_cell_wall_hits.after(cell_wall_collision),
            );
        app
    }

    /// Accumulates one fixed timestep then runs one update.
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn spawn_cell(app: &mut App, pos: Vec2, half_extents: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Cell,
                Aabb2D::new(Vec2::ZERO, half_extents),
                CollisionLayers::new(CELL_LAYER, WALL_LAYER),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
                GameDrawLayer::Cell,
            ))
            .id()
    }

    fn spawn_wall(app: &mut App, pos: Vec2, half_extents: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Wall,
                Aabb2D::new(Vec2::ZERO, half_extents),
                CollisionLayers::new(WALL_LAYER, CELL_LAYER),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
                GameDrawLayer::Wall,
            ))
            .id()
    }

    // ── B5: Cell overlapping wall emits CellImpactWall ──────────────

    #[test]
    fn cell_overlapping_wall_emits_impact_message() {
        // B5: Cell at (-450,50) half_extents (35,12), left wall at (-485,0) half_extents (5,300).
        // dx=|-450-(-485)|=35 < 35+5=40, dy=|50-0|=50 < 12+300=312 => overlap => 1 message.
        let mut app = test_app();

        let cell_entity = spawn_cell(&mut app, Vec2::new(-450.0, 50.0), Vec2::new(35.0, 12.0));
        let wall_entity = spawn_wall(&mut app, Vec2::new(-485.0, 0.0), Vec2::new(5.0, 300.0));

        tick(&mut app);

        let msgs = app.world().resource::<CellWallHitMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "cell overlapping wall should emit exactly 1 CellImpactWall, got {}",
            msgs.0.len()
        );
        assert_eq!(
            msgs.0[0].cell, cell_entity,
            "CellImpactWall.cell should match the cell entity"
        );
        assert_eq!(
            msgs.0[0].wall, wall_entity,
            "CellImpactWall.wall should match the wall entity"
        );
    }

    #[test]
    fn multiple_cells_overlapping_same_wall_emit_multiple_messages() {
        // B5 edge case: Two cells overlapping the same wall => exactly 2 messages.
        // Cell1 at (-450,50), Cell2 at (-455,100), both with half_extents (35,12).
        // Wall at (-485,0) half_extents (5,300).
        // Cell1: dx=35 < 40, dy=50 < 312 => overlap.
        // Cell2: dx=30 < 40, dy=100 < 312 => overlap.
        let mut app = test_app();

        let cell1 = spawn_cell(&mut app, Vec2::new(-450.0, 50.0), Vec2::new(35.0, 12.0));
        let cell2 = spawn_cell(&mut app, Vec2::new(-455.0, 100.0), Vec2::new(35.0, 12.0));
        let wall_entity = spawn_wall(&mut app, Vec2::new(-485.0, 0.0), Vec2::new(5.0, 300.0));

        tick(&mut app);

        let msgs = app.world().resource::<CellWallHitMessages>();
        assert_eq!(
            msgs.0.len(),
            2,
            "two cells overlapping one wall should emit 2 CellImpactWall, got {}",
            msgs.0.len()
        );

        let cell_entities: Vec<Entity> = msgs.0.iter().map(|m| m.cell).collect();
        assert!(
            cell_entities.contains(&cell1),
            "messages should include cell1"
        );
        assert!(
            cell_entities.contains(&cell2),
            "messages should include cell2"
        );
        for msg in &msgs.0 {
            assert_eq!(
                msg.wall, wall_entity,
                "all CellImpactWall.wall fields should match the wall entity"
            );
        }
    }

    // ── B6: No message when cell and wall do not overlap ────────────

    #[test]
    fn cell_far_from_walls_emits_no_message() {
        // B6: Cell at (0,100) half_extents (35,12), three walls far away.
        // Left wall at (-485,0): dx=485 >= 35+5=40 => no overlap.
        // Right wall at (485,0): dx=485 >= 35+5=40 => no overlap.
        // Ceiling at (0,310): dy=210 >= 12+5=17 => no overlap.
        let mut app = test_app();

        spawn_cell(&mut app, Vec2::new(0.0, 100.0), Vec2::new(35.0, 12.0));
        spawn_wall(&mut app, Vec2::new(-485.0, 0.0), Vec2::new(5.0, 300.0));
        spawn_wall(&mut app, Vec2::new(485.0, 0.0), Vec2::new(5.0, 300.0));
        spawn_wall(&mut app, Vec2::new(0.0, 310.0), Vec2::new(500.0, 5.0));

        tick(&mut app);

        let msgs = app.world().resource::<CellWallHitMessages>();
        assert!(
            msgs.0.is_empty(),
            "cell far from all walls should emit 0 CellImpactWall, got {}",
            msgs.0.len()
        );
    }

    #[test]
    fn cell_tangent_to_wall_emits_no_message() {
        // B6 edge case: Cell at (-445,50) half_x=35, wall at (-485,0) half_x=5.
        // dx=|-445-(-485)|=40, threshold=35+5=40. Strict inequality: 40 < 40 is false => no message.
        let mut app = test_app();

        spawn_cell(&mut app, Vec2::new(-445.0, 50.0), Vec2::new(35.0, 12.0));
        spawn_wall(&mut app, Vec2::new(-485.0, 0.0), Vec2::new(5.0, 300.0));

        tick(&mut app);

        let msgs = app.world().resource::<CellWallHitMessages>();
        assert!(
            msgs.0.is_empty(),
            "cell tangent to wall (dx == threshold) should emit 0 messages, got {}",
            msgs.0.len()
        );
    }
}
