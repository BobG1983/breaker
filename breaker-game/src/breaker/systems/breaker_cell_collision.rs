//! Breaker-cell collision detection.
//!
//! Detects when the breaker entity overlaps a cell entity and sends
//! [`BreakerImpactCell`] messages. Uses the spatial quadtree for
//! broad-phase filtering. Used by effect triggers to fire
//! `Impact(Cell)` / `Impacted(Breaker)` chains.

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    breaker::{
        components::{BaseHeight, BaseWidth, Breaker},
        messages::BreakerImpactCell,
    },
    cells::components::Cell,
    shared::{BREAKER_LAYER, CELL_LAYER, NodeScalingFactor},
};

/// Breaker query data for cell collision detection.
type BreakerCellCollisionQuery = (
    Entity,
    &'static Position2D,
    &'static BaseWidth,
    &'static BaseHeight,
    Option<&'static NodeScalingFactor>,
);

/// Cell entity lookup for narrow-phase overlap verification.
type CellLookup<'w, 's> = Query<'w, 's, (&'static Position2D, &'static Aabb2D), With<Cell>>;

/// Detects breaker-cell collisions via quadtree AABB query.
///
/// For each breaker, queries the quadtree for nearby cell entities.
/// Broad-phase candidates are verified with a narrow-phase AABB overlap
/// check before sending [`BreakerImpactCell`].
pub(crate) fn breaker_cell_collision(
    quadtree: Res<CollisionQuadtree>,
    breaker_query: Query<BreakerCellCollisionQuery, With<Breaker>>,
    cell_lookup: CellLookup,
    mut writer: MessageWriter<BreakerImpactCell>,
) {
    let Ok((breaker_entity, breaker_pos, breaker_w, breaker_h, breaker_scale)) =
        breaker_query.single()
    else {
        return;
    };

    let scale = breaker_scale.map_or(1.0, |s| s.0);
    let half_w = breaker_w.half_width() * scale;
    let half_h = breaker_h.half_height() * scale;

    let breaker_aabb = Aabb2D::new(breaker_pos.0, Vec2::new(half_w, half_h));
    let layers = CollisionLayers::new(BREAKER_LAYER, CELL_LAYER);
    let candidates = quadtree.quadtree.query_aabb_filtered(&breaker_aabb, layers);

    for cell_entity in candidates {
        let Ok((cell_pos, cell_aabb)) = cell_lookup.get(cell_entity) else {
            continue;
        };

        // Narrow-phase: verify actual AABB overlap
        let dx = (breaker_pos.0.x - cell_pos.0.x).abs();
        let dy = (breaker_pos.0.y - cell_pos.0.y).abs();
        if dx < half_w + cell_aabb.half_extents.x && dy < half_h + cell_aabb.half_extents.y {
            writer.write(BreakerImpactCell {
                breaker: breaker_entity,
                cell: cell_entity,
            });
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
    struct BreakerCellHitMessages(Vec<BreakerImpactCell>);

    fn collect_breaker_cell_hits(
        mut reader: MessageReader<BreakerImpactCell>,
        mut msgs: ResMut<BreakerCellHitMessages>,
    ) {
        for msg in reader.read() {
            msgs.0.push(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(RantzPhysics2dPlugin)
            .add_message::<BreakerImpactCell>()
            .insert_resource(BreakerCellHitMessages::default())
            .add_systems(
                FixedUpdate,
                breaker_cell_collision
                    .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
            )
            .add_systems(
                FixedUpdate,
                collect_breaker_cell_hits.after(breaker_cell_collision),
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

    fn spawn_breaker(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Breaker,
                BaseWidth(120.0),
                BaseHeight(20.0),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
                Aabb2D::new(Vec2::ZERO, Vec2::new(60.0, 10.0)),
                CollisionLayers::new(BREAKER_LAYER, CELL_LAYER),
                GameDrawLayer::Breaker,
            ))
            .id()
    }

    fn spawn_cell(app: &mut App, pos: Vec2, half_extents: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Cell,
                Aabb2D::new(Vec2::ZERO, half_extents),
                CollisionLayers::new(CELL_LAYER, BREAKER_LAYER),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
                GameDrawLayer::Cell,
            ))
            .id()
    }

    // ── B1: Breaker overlapping cell emits BreakerImpactCell ────────

    #[test]
    fn breaker_overlapping_cell_emits_impact_message() {
        // B1: Breaker at (0,0) half_w=60 half_h=10, cell at (50,5) half_extents (35,12).
        // dx=50 < 60+35=95, dy=5 < 10+12=22 => overlap => 1 message.
        let mut app = test_app();

        let breaker_entity = spawn_breaker(&mut app, Vec2::new(0.0, 0.0));
        let cell_entity = spawn_cell(&mut app, Vec2::new(50.0, 5.0), Vec2::new(35.0, 12.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerCellHitMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "breaker overlapping cell should emit exactly 1 BreakerImpactCell, got {}",
            msgs.0.len()
        );
        assert_eq!(
            msgs.0[0].breaker, breaker_entity,
            "BreakerImpactCell.breaker should match the breaker entity"
        );
        assert_eq!(
            msgs.0[0].cell, cell_entity,
            "BreakerImpactCell.cell should match the cell entity"
        );
    }

    #[test]
    fn breaker_with_entity_scale_expands_collision_area() {
        // B1 edge case: Breaker at (0,0) with NodeScalingFactor(2.0) => half_w=120, half_h=20.
        // Cell at (110,0) half_extents (35,12). At scale=1.0: dx=110 > 60+35=95 (no overlap).
        // At scale=2.0: dx=110 < 120+35=155 (overlap). Verify 1 message.
        let mut app = test_app();

        let breaker_entity = spawn_breaker(&mut app, Vec2::new(0.0, 0.0));
        app.world_mut()
            .entity_mut(breaker_entity)
            .insert(NodeScalingFactor(2.0));

        let cell_entity = spawn_cell(&mut app, Vec2::new(110.0, 0.0), Vec2::new(35.0, 12.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerCellHitMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "scaled breaker should overlap cell that is out of range at scale=1.0, got {} messages",
            msgs.0.len()
        );
        assert_eq!(msgs.0[0].breaker, breaker_entity);
        assert_eq!(msgs.0[0].cell, cell_entity);
    }

    // ── B2: No message when breaker and cell do not overlap ─────────

    #[test]
    fn breaker_far_from_cell_emits_no_message() {
        // B2: Breaker at (0,-250) half_h=10, cell at (0,100) half_h=12.
        // dy=350 >= 10+12=22 => no overlap => 0 messages.
        let mut app = test_app();

        spawn_breaker(&mut app, Vec2::new(0.0, -250.0));
        spawn_cell(&mut app, Vec2::new(0.0, 100.0), Vec2::new(35.0, 12.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerCellHitMessages>();
        assert!(
            msgs.0.is_empty(),
            "breaker far from cell should emit 0 BreakerImpactCell, got {}",
            msgs.0.len()
        );
    }

    #[test]
    fn breaker_tangent_to_cell_emits_no_message() {
        // B2 edge case: Breaker at (95,0) half_w=60, cell at (0,0) half_x=35.
        // dx=95, threshold=60+35=95. Strict inequality: 95 < 95 is false => no message.
        let mut app = test_app();

        spawn_breaker(&mut app, Vec2::new(95.0, 0.0));
        spawn_cell(&mut app, Vec2::new(0.0, 0.0), Vec2::new(35.0, 12.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerCellHitMessages>();
        assert!(
            msgs.0.is_empty(),
            "breaker tangent to cell (dx == threshold) should emit 0 messages, got {}",
            msgs.0.len()
        );
    }
}
