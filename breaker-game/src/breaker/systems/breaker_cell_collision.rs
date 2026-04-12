//! Breaker-cell collision detection.
//!
//! Detects when the breaker entity overlaps a cell entity and sends
//! [`BreakerImpactCell`] messages. Uses the spatial quadtree for
//! broad-phase filtering. Used by effect triggers to fire
//! `Impact(Cell)` / `Impacted(Breaker)` chains.

use bevy::prelude::*;
use rantzsoft_physics2d::resources::CollisionQuadtree;

use crate::{
    breaker::queries::BreakerSizeData,
    effect_v3::{effects::SizeBoostConfig, stacking::EffectStack},
    prelude::*,
    shared::{BREAKER_LAYER, CELL_LAYER},
};

/// Cell entity lookup for narrow-phase overlap verification.
type CellLookup<'w, 's> = Query<'w, 's, (&'static Position2D, &'static Aabb2D), With<Cell>>;

/// Detects breaker-cell collisions via quadtree AABB query.
///
/// For each breaker, queries the quadtree for nearby cell entities.
/// Broad-phase candidates are verified with a narrow-phase AABB overlap
/// check before sending [`BreakerImpactCell`].
pub(crate) fn breaker_cell_collision(
    quadtree: Res<CollisionQuadtree>,
    breaker_query: Query<BreakerSizeData, With<Breaker>>,
    cell_lookup: CellLookup,
    mut writer: MessageWriter<BreakerImpactCell>,
) {
    let Ok(breaker) = breaker_query.single() else {
        return;
    };

    let size_mult = breaker
        .size_boosts
        .map_or(1.0, EffectStack::<SizeBoostConfig>::aggregate);
    let scale = breaker.node_scale.map_or(1.0, |s| s.0);
    let half_w = breaker.base_width.half_width() * size_mult * scale;
    let half_h = breaker.base_height.half_height() * size_mult * scale;

    let breaker_aabb = Aabb2D::new(breaker.position.0, Vec2::new(half_w, half_h));
    let layers = CollisionLayers::new(BREAKER_LAYER, CELL_LAYER);
    let candidates = quadtree.quadtree.query_aabb_filtered(&breaker_aabb, layers);

    for cell_entity in candidates {
        let Ok((cell_pos, cell_aabb)) = cell_lookup.get(cell_entity) else {
            continue;
        };

        // Narrow-phase: verify actual AABB overlap
        let dx = (breaker.position.0.x - cell_pos.0.x).abs();
        let dy = (breaker.position.0.y - cell_pos.0.y).abs();
        if dx < half_w + cell_aabb.half_extents.x && dy < half_h + cell_aabb.half_extents.y {
            writer.write(BreakerImpactCell {
                breaker: breaker.entity,
                cell:    cell_entity,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;
    use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

    use super::*;
    use crate::{
        breaker::components::{BaseHeight, BaseWidth},
        effect_v3::{effects::SizeBoostConfig, stacking::EffectStack},
        shared::{GameDrawLayer, NodeScalingFactor, test_utils::TestAppBuilder},
    };

    fn size_stack(values: &[f32]) -> EffectStack<SizeBoostConfig> {
        let mut stack = EffectStack::default();
        for &v in values {
            stack.push(
                "test".into(),
                SizeBoostConfig {
                    multiplier: OrderedFloat(v),
                },
            );
        }
        stack
    }

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
        TestAppBuilder::new()
            .with_physics()
            .with_message::<BreakerImpactCell>()
            .insert_resource(BreakerCellHitMessages::default())
            .with_system(
                FixedUpdate,
                breaker_cell_collision
                    .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
            )
            .with_system(
                FixedUpdate,
                collect_breaker_cell_hits.after(breaker_cell_collision),
            )
            .build()
    }

    use crate::shared::test_utils::tick;

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

    // ── Bug Fix Regression: ActiveSizeBoosts in breaker AABB ────────
    //
    // The bug: breaker_cell_collision computes half_w = BaseWidth.half_width() * NodeScalingFactor
    // but ignores ActiveSizeBoosts. When size boosts are active, the collision AABB is too small.

    #[test]
    fn breaker_cell_collision_uses_active_size_boosts_in_size() {
        // Behavior 22: Breaker at (0,0) with ActiveSizeBoosts([2.0]).
        // Correct: half_w = 60.0 * 2.0 = 120.0. Cell at (100,0) half_extents (35,12).
        // dx = 100.0 < 120.0 + 35.0 = 155.0 => overlap => 1 message.
        // Bug: half_w = 60.0 (ignores boost), dx = 100.0 > 60.0 + 35.0 = 95.0 => miss.
        let mut app = test_app();

        let breaker_entity = spawn_breaker(&mut app, Vec2::new(0.0, 0.0));
        app.world_mut()
            .entity_mut(breaker_entity)
            .insert(size_stack(&[2.0]));

        let cell_entity = spawn_cell(&mut app, Vec2::new(100.0, 0.0), Vec2::new(35.0, 12.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerCellHitMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "breaker with ActiveSizeBoosts([2.0]) should overlap cell at dx=100 (threshold=155), got {} messages",
            msgs.0.len(),
        );
        assert_eq!(msgs.0[0].breaker, breaker_entity);
        assert_eq!(msgs.0[0].cell, cell_entity);
    }

    #[test]
    fn breaker_cell_collision_identity_boost_still_overlaps() {
        // Behavior 22 edge case: ActiveSizeBoosts([1.0]) => half_w = 60.0 * 1.0 = 60.0.
        // Cell at (90,0) half_extents (35,12). dx = 90.0 < 60.0 + 35.0 = 95.0 => overlap.
        let mut app = test_app();

        let breaker_entity = spawn_breaker(&mut app, Vec2::new(0.0, 0.0));
        app.world_mut()
            .entity_mut(breaker_entity)
            .insert(size_stack(&[1.0]));

        let cell_entity = spawn_cell(&mut app, Vec2::new(90.0, 0.0), Vec2::new(35.0, 12.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerCellHitMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "identity boost should still overlap at dx=90 (threshold=95), got {} messages",
            msgs.0.len(),
        );
        assert_eq!(msgs.0[0].breaker, breaker_entity);
        assert_eq!(msgs.0[0].cell, cell_entity);
    }

    #[test]
    fn breaker_cell_collision_boosted_does_not_overlap_beyond_range() {
        // Behavior 23: ActiveSizeBoosts([1.5]) => half_w = 60.0 * 1.5 = 90.0.
        // Cell at (130,0) half_extents (35,12). dx = 130.0 > 90.0 + 35.0 = 125.0 => no overlap.
        let mut app = test_app();

        let breaker_entity = spawn_breaker(&mut app, Vec2::new(0.0, 0.0));
        app.world_mut()
            .entity_mut(breaker_entity)
            .insert(size_stack(&[1.5]));

        spawn_cell(&mut app, Vec2::new(130.0, 0.0), Vec2::new(35.0, 12.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerCellHitMessages>();
        assert!(
            msgs.0.is_empty(),
            "boosted breaker should NOT overlap cell at dx=130 (threshold=125), got {} messages",
            msgs.0.len(),
        );
    }

    #[test]
    fn breaker_cell_collision_boosted_tangent_emits_no_message() {
        // Behavior 23 edge case: ActiveSizeBoosts([1.5]) => half_w = 90.0.
        // Cell at (125,0) half_extents (35,12). dx = 125.0, threshold = 90.0 + 35.0 = 125.0.
        // Strict inequality: 125.0 < 125.0 is false => 0 messages.
        let mut app = test_app();

        let breaker_entity = spawn_breaker(&mut app, Vec2::new(0.0, 0.0));
        app.world_mut()
            .entity_mut(breaker_entity)
            .insert(size_stack(&[1.5]));

        spawn_cell(&mut app, Vec2::new(125.0, 0.0), Vec2::new(35.0, 12.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerCellHitMessages>();
        assert!(
            msgs.0.is_empty(),
            "boosted breaker tangent to cell (dx == threshold=125) should emit 0 messages, got {}",
            msgs.0.len(),
        );
    }

    #[test]
    fn breaker_cell_collision_boost_and_node_scale_combined() {
        // Behavior 24: ActiveSizeBoosts([1.5]), NodeScalingFactor(2.0).
        // half_w = 60.0 * 1.5 * 2.0 = 180.0. Cell at (200,0) half_extents (35,12).
        // dx = 200.0 < 180.0 + 35.0 = 215.0 => overlap.
        let mut app = test_app();

        let breaker_entity = spawn_breaker(&mut app, Vec2::new(0.0, 0.0));
        app.world_mut()
            .entity_mut(breaker_entity)
            .insert((size_stack(&[1.5]), NodeScalingFactor(2.0)));

        let cell_entity = spawn_cell(&mut app, Vec2::new(200.0, 0.0), Vec2::new(35.0, 12.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerCellHitMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "breaker with boost+scale should overlap cell at dx=200 (threshold=215), got {} messages",
            msgs.0.len(),
        );
        assert_eq!(msgs.0[0].breaker, breaker_entity);
        assert_eq!(msgs.0[0].cell, cell_entity);
    }
}
