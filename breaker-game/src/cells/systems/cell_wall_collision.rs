//! Cell-wall collision detection.
//!
//! Detects when a cell entity overlaps a wall entity and sends
//! [`CellImpactWall`] messages. Uses the spatial quadtree for
//! broad-phase filtering. Used by effect triggers to fire
//! `Impact(Wall)` / `Impacted(Cell)` chains.

use bevy::prelude::*;
use rantzsoft_physics2d::resources::CollisionQuadtree;

use crate::{
    prelude::*,
    shared::{CELL_LAYER, WALL_LAYER},
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
    use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

    use super::*;
    use crate::{
        shared::{GameDrawLayer, test_utils::tick},
        walls::test_utils::{spawn_ceiling_wall, spawn_left_wall, spawn_right_wall},
    };

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
        use crate::shared::test_utils::TestAppBuilder;

        TestAppBuilder::new()
            .with_physics()
            .with_message::<CellImpactWall>()
            .insert_resource(CellWallHitMessages::default())
            .with_system(
                FixedUpdate,
                cell_wall_collision
                    .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
            )
            .with_system(
                FixedUpdate,
                collect_cell_wall_hits.after(cell_wall_collision),
            )
            .build()
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

    // ── B5: Cell overlapping wall emits CellImpactWall ──────────────

    #[test]
    fn cell_overlapping_wall_emits_impact_message() {
        // B5: Cell at (-450,50) half_extents (35,12), left wall at (-490,0) half_extents (90,300).
        // dx=|-450-(-490)|=40 < 35+90=125, dy=|50-0|=50 < 12+300=312 => overlap => 1 message.
        let mut app = test_app();

        let cell_entity = spawn_cell(&mut app, Vec2::new(-450.0, 50.0), Vec2::new(35.0, 12.0));
        let wall_entity = spawn_left_wall(&mut app);

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
        // Left wall at (-490,0) half_extents (90,300).
        // Cell1: dx=40 < 125, dy=50 < 312 => overlap.
        // Cell2: dx=35 < 125, dy=100 < 312 => overlap.
        let mut app = test_app();

        let cell1 = spawn_cell(&mut app, Vec2::new(-450.0, 50.0), Vec2::new(35.0, 12.0));
        let cell2 = spawn_cell(&mut app, Vec2::new(-455.0, 100.0), Vec2::new(35.0, 12.0));
        let wall_entity = spawn_left_wall(&mut app);

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
        // Left wall at (-490,0): dx=490 >= 35+90=125 => no overlap.
        // Right wall at (490,0): dx=490 >= 35+90=125 => no overlap.
        // Ceiling at (0,390): dy=290 >= 12+90=102 => no overlap.
        let mut app = test_app();

        spawn_cell(&mut app, Vec2::new(0.0, 100.0), Vec2::new(35.0, 12.0));
        spawn_left_wall(&mut app);
        spawn_right_wall(&mut app);
        spawn_ceiling_wall(&mut app);

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
        // B6 edge case: Cell at (-365,50) half_x=35, left wall at (-490,0) half_x=90.
        // dx=|-365-(-490)|=125, threshold=35+90=125. Strict inequality: 125 < 125 is false => no message.
        let mut app = test_app();

        spawn_cell(&mut app, Vec2::new(-365.0, 50.0), Vec2::new(35.0, 12.0));
        spawn_left_wall(&mut app);

        tick(&mut app);

        let msgs = app.world().resource::<CellWallHitMessages>();
        assert!(
            msgs.0.is_empty(),
            "cell tangent to wall (dx == threshold) should emit 0 messages, got {}",
            msgs.0.len()
        );
    }
}
