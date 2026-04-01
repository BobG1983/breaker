//! Breaker-wall collision detection.
//!
//! Detects when the breaker entity overlaps a wall entity and sends
//! [`BreakerImpactWall`] messages. Uses the spatial quadtree for
//! broad-phase filtering. Used by effect triggers to fire
//! `Impact(Wall)` / `Impacted(Breaker)` chains.

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    breaker::{
        components::{BaseHeight, BaseWidth, Breaker},
        messages::BreakerImpactWall,
    },
    effect::effects::size_boost::ActiveSizeBoosts,
    shared::{BREAKER_LAYER, NodeScalingFactor, WALL_LAYER},
    wall::components::Wall,
};

/// Breaker query data for wall collision detection.
type BreakerWallCollisionQuery = (
    Entity,
    &'static Position2D,
    &'static BaseWidth,
    &'static BaseHeight,
    Option<&'static NodeScalingFactor>,
    Option<&'static ActiveSizeBoosts>,
);

/// Wall entity lookup for narrow-phase overlap verification.
type WallLookup<'w, 's> = Query<'w, 's, (&'static Position2D, &'static Aabb2D), With<Wall>>;

/// Detects breaker-wall collisions via quadtree AABB query.
///
/// For each breaker, queries the quadtree for nearby wall entities.
/// Broad-phase candidates are verified with a narrow-phase AABB overlap
/// check before sending [`BreakerImpactWall`]. The breaker already
/// clamps to playfield bounds in `move_breaker`, so this detects
/// edge-case overlaps for effect trigger chains.
pub(crate) fn breaker_wall_collision(
    quadtree: Res<CollisionQuadtree>,
    breaker_query: Query<BreakerWallCollisionQuery, With<Breaker>>,
    wall_lookup: WallLookup,
    mut writer: MessageWriter<BreakerImpactWall>,
) {
    let Ok((breaker_entity, breaker_pos, breaker_w, breaker_h, breaker_scale, size_boosts)) =
        breaker_query.single()
    else {
        return;
    };

    let size_mult = size_boosts.map_or(1.0, ActiveSizeBoosts::multiplier);
    let scale = breaker_scale.map_or(1.0, |s| s.0);
    let half_w = breaker_w.half_width() * size_mult * scale;
    let half_h = breaker_h.half_height() * size_mult * scale;

    let breaker_aabb = Aabb2D::new(breaker_pos.0, Vec2::new(half_w, half_h));
    let layers = CollisionLayers::new(BREAKER_LAYER, WALL_LAYER);
    let candidates = quadtree.quadtree.query_aabb_filtered(&breaker_aabb, layers);

    for wall_entity in candidates {
        let Ok((wall_pos, wall_aabb)) = wall_lookup.get(wall_entity) else {
            continue;
        };

        // Narrow-phase: verify actual AABB overlap
        let dx = (breaker_pos.0.x - wall_pos.0.x).abs();
        let dy = (breaker_pos.0.y - wall_pos.0.y).abs();
        if dx < half_w + wall_aabb.half_extents.x && dy < half_h + wall_aabb.half_extents.y {
            writer.write(BreakerImpactWall {
                breaker: breaker_entity,
                wall: wall_entity,
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
    struct BreakerWallHitMessages(Vec<BreakerImpactWall>);

    fn collect_breaker_wall_hits(
        mut reader: MessageReader<BreakerImpactWall>,
        mut msgs: ResMut<BreakerWallHitMessages>,
    ) {
        for msg in reader.read() {
            msgs.0.push(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(RantzPhysics2dPlugin)
            .add_message::<BreakerImpactWall>()
            .insert_resource(BreakerWallHitMessages::default())
            .add_systems(
                FixedUpdate,
                breaker_wall_collision
                    .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
            )
            .add_systems(
                FixedUpdate,
                collect_breaker_wall_hits.after(breaker_wall_collision),
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
                CollisionLayers::new(BREAKER_LAYER, WALL_LAYER),
                GameDrawLayer::Breaker,
            ))
            .id()
    }

    fn spawn_wall(app: &mut App, pos: Vec2, half_extents: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Wall,
                Aabb2D::new(Vec2::ZERO, half_extents),
                CollisionLayers::new(WALL_LAYER, BREAKER_LAYER),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
                GameDrawLayer::Wall,
            ))
            .id()
    }

    // ── B3: Breaker overlapping wall emits BreakerImpactWall ────────

    #[test]
    fn breaker_overlapping_wall_emits_impact_message() {
        // B3: Breaker at (-430,-250) half_w=60 half_h=10,
        // left wall at (-485,0) half_extents (5,300).
        // dx=|-430-(-485)|=55 < 60+5=65, dy=|-250-0|=250 < 10+300=310 => overlap => 1 message.
        let mut app = test_app();

        let breaker_entity = spawn_breaker(&mut app, Vec2::new(-430.0, -250.0));
        let wall_entity = spawn_wall(&mut app, Vec2::new(-485.0, 0.0), Vec2::new(5.0, 300.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerWallHitMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "breaker overlapping wall should emit exactly 1 BreakerImpactWall, got {}",
            msgs.0.len()
        );
        assert_eq!(
            msgs.0[0].breaker, breaker_entity,
            "BreakerImpactWall.breaker should match the breaker entity"
        );
        assert_eq!(
            msgs.0[0].wall, wall_entity,
            "BreakerImpactWall.wall should match the wall entity"
        );
    }

    #[test]
    fn breaker_with_entity_scale_overlaps_wall() {
        // B3 edge case: Breaker at (-460,-250) with NodeScalingFactor(2.0) => half_w=120 half_h=20.
        // Wall at (-485,0) half_x=5. dx=|-460-(-485)|=25 < 120+5=125 (overlap).
        // Verify 1 message.
        let mut app = test_app();

        let breaker_entity = spawn_breaker(&mut app, Vec2::new(-460.0, -250.0));
        app.world_mut()
            .entity_mut(breaker_entity)
            .insert(NodeScalingFactor(2.0));

        let wall_entity = spawn_wall(&mut app, Vec2::new(-485.0, 0.0), Vec2::new(5.0, 300.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerWallHitMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "scaled breaker should overlap wall, got {} messages",
            msgs.0.len()
        );
        assert_eq!(msgs.0[0].breaker, breaker_entity);
        assert_eq!(msgs.0[0].wall, wall_entity);
    }

    // ── B4: No message when breaker and wall do not overlap ─────────

    #[test]
    fn breaker_far_from_walls_emits_no_message() {
        // B4: Breaker at (0,-250) half_w=60, three walls far away.
        // Left wall at (-485,0): dx=485 >= 60+5=65 => no overlap.
        // Right wall at (485,0): dx=485 >= 60+5=65 => no overlap.
        // Ceiling at (0,310): dy=560 >= 10+5=15 => no overlap.
        let mut app = test_app();

        spawn_breaker(&mut app, Vec2::new(0.0, -250.0));
        spawn_wall(&mut app, Vec2::new(-485.0, 0.0), Vec2::new(5.0, 300.0));
        spawn_wall(&mut app, Vec2::new(485.0, 0.0), Vec2::new(5.0, 300.0));
        spawn_wall(&mut app, Vec2::new(0.0, 310.0), Vec2::new(500.0, 5.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerWallHitMessages>();
        assert!(
            msgs.0.is_empty(),
            "breaker far from all walls should emit 0 BreakerImpactWall, got {}",
            msgs.0.len()
        );
    }

    #[test]
    fn breaker_tangent_to_wall_emits_no_message() {
        // B4 edge case (tangent): Breaker at (-420,-250) half_w=60,
        // wall at (-485,0) half_x=5. dx=|-420-(-485)|=65, threshold=60+5=65.
        // Strict inequality: 65 < 65 is false => no message.
        let mut app = test_app();

        spawn_breaker(&mut app, Vec2::new(-420.0, -250.0));
        spawn_wall(&mut app, Vec2::new(-485.0, 0.0), Vec2::new(5.0, 300.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerWallHitMessages>();
        assert!(
            msgs.0.is_empty(),
            "breaker tangent to wall (dx == threshold) should emit 0 messages, got {}",
            msgs.0.len()
        );
    }

    #[test]
    fn breaker_with_entity_scale_shrink_tangent_to_wall_emits_no_message() {
        // B4 edge case (scale shrink): Breaker at (-450,-250) with NodeScalingFactor(0.5)
        // => half_w=30 half_h=5. Wall at (-485,0) half_x=5.
        // dx=|-450-(-485)|=35, threshold=30+5=35.
        // Strict inequality: 35 < 35 is false => no message.
        let mut app = test_app();

        let breaker_entity = spawn_breaker(&mut app, Vec2::new(-450.0, -250.0));
        app.world_mut()
            .entity_mut(breaker_entity)
            .insert(NodeScalingFactor(0.5));

        spawn_wall(&mut app, Vec2::new(-485.0, 0.0), Vec2::new(5.0, 300.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerWallHitMessages>();
        assert!(
            msgs.0.is_empty(),
            "scaled-down breaker tangent to wall should emit 0 messages, got {}",
            msgs.0.len()
        );
    }

    // ── Bug Fix Regression: ActiveSizeBoosts in breaker AABB ────────
    //
    // The bug: breaker_wall_collision computes half_w = BaseWidth.half_width() * NodeScalingFactor
    // but ignores ActiveSizeBoosts. When size boosts are active, the collision AABB is too small.

    #[test]
    fn breaker_wall_collision_uses_active_size_boosts_in_size() {
        // Behavior 25: Breaker at (-400,-250) with ActiveSizeBoosts([2.0]).
        // Correct: half_w = 60.0 * 2.0 = 120.0. Wall at (-485,0) half_extents (5,300).
        // dx = |-400 - (-485)| = 85.0. threshold = 120.0 + 5.0 = 125.0. 85.0 < 125.0 => overlap.
        // Bug: half_w = 60.0 (ignores boost), threshold = 65.0, 85.0 > 65.0 => miss.
        let mut app = test_app();

        let breaker_entity = spawn_breaker(&mut app, Vec2::new(-400.0, -250.0));
        app.world_mut().entity_mut(breaker_entity).insert(
            crate::effect::effects::size_boost::ActiveSizeBoosts(vec![2.0]),
        );

        let wall_entity = spawn_wall(&mut app, Vec2::new(-485.0, 0.0), Vec2::new(5.0, 300.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerWallHitMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "breaker with ActiveSizeBoosts([2.0]) should overlap wall at dx=85 (threshold=125), got {} messages",
            msgs.0.len(),
        );
        assert_eq!(msgs.0[0].breaker, breaker_entity);
        assert_eq!(msgs.0[0].wall, wall_entity);
    }

    #[test]
    fn breaker_wall_collision_identity_boost_still_overlaps() {
        // Behavior 25 edge case: ActiveSizeBoosts([1.0]) => half_w = 60.0 * 1.0 = 60.0.
        // Breaker at (-425,-250). Wall at (-485,0) half_x=5.
        // dx = |-425 - (-485)| = 60.0 < 60.0 + 5.0 = 65.0 => overlap.
        let mut app = test_app();

        let breaker_entity = spawn_breaker(&mut app, Vec2::new(-425.0, -250.0));
        app.world_mut().entity_mut(breaker_entity).insert(
            crate::effect::effects::size_boost::ActiveSizeBoosts(vec![1.0]),
        );

        let wall_entity = spawn_wall(&mut app, Vec2::new(-485.0, 0.0), Vec2::new(5.0, 300.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerWallHitMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "identity boost should still overlap at dx=60 (threshold=65), got {} messages",
            msgs.0.len(),
        );
        assert_eq!(msgs.0[0].breaker, breaker_entity);
        assert_eq!(msgs.0[0].wall, wall_entity);
    }

    #[test]
    fn breaker_wall_collision_boosted_does_not_overlap_beyond_range() {
        // Behavior 26: ActiveSizeBoosts([1.5]) => half_w = 60.0 * 1.5 = 90.0.
        // Breaker at (-300,-250). Wall at (-485,0) half_x=5.
        // dx = |-300 - (-485)| = 185.0 > 90.0 + 5.0 = 95.0 => no overlap.
        let mut app = test_app();

        let breaker_entity = spawn_breaker(&mut app, Vec2::new(-300.0, -250.0));
        app.world_mut().entity_mut(breaker_entity).insert(
            crate::effect::effects::size_boost::ActiveSizeBoosts(vec![1.5]),
        );

        spawn_wall(&mut app, Vec2::new(-485.0, 0.0), Vec2::new(5.0, 300.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerWallHitMessages>();
        assert!(
            msgs.0.is_empty(),
            "boosted breaker should NOT overlap wall at dx=185 (threshold=95), got {} messages",
            msgs.0.len(),
        );
    }

    #[test]
    fn breaker_wall_collision_boosted_tangent_emits_no_message() {
        // Behavior 26 edge case: ActiveSizeBoosts([1.5]) => half_w = 90.0.
        // Breaker at (-390,-250). Wall at (-485,0) half_x=5.
        // dx = |-390 - (-485)| = 95.0, threshold = 90.0 + 5.0 = 95.0.
        // Strict inequality: 95.0 < 95.0 is false => 0 messages.
        let mut app = test_app();

        let breaker_entity = spawn_breaker(&mut app, Vec2::new(-390.0, -250.0));
        app.world_mut().entity_mut(breaker_entity).insert(
            crate::effect::effects::size_boost::ActiveSizeBoosts(vec![1.5]),
        );

        spawn_wall(&mut app, Vec2::new(-485.0, 0.0), Vec2::new(5.0, 300.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerWallHitMessages>();
        assert!(
            msgs.0.is_empty(),
            "boosted breaker tangent to wall (dx == threshold=95) should emit 0 messages, got {}",
            msgs.0.len(),
        );
    }

    #[test]
    fn breaker_wall_collision_boost_and_node_scale_combined() {
        // Behavior 27: ActiveSizeBoosts([1.5]), NodeScalingFactor(2.0).
        // half_w = 60.0 * 1.5 * 2.0 = 180.0. Breaker at (-350,-250). Wall at (-485,0) half_x=5.
        // dx = |-350 - (-485)| = 135.0. threshold = 180.0 + 5.0 = 185.0. 135.0 < 185.0 => overlap.
        let mut app = test_app();

        let breaker_entity = spawn_breaker(&mut app, Vec2::new(-350.0, -250.0));
        app.world_mut().entity_mut(breaker_entity).insert((
            crate::effect::effects::size_boost::ActiveSizeBoosts(vec![1.5]),
            NodeScalingFactor(2.0),
        ));

        let wall_entity = spawn_wall(&mut app, Vec2::new(-485.0, 0.0), Vec2::new(5.0, 300.0));

        tick(&mut app);

        let msgs = app.world().resource::<BreakerWallHitMessages>();
        assert_eq!(
            msgs.0.len(),
            1,
            "breaker with boost+scale should overlap wall at dx=135 (threshold=185), got {} messages",
            msgs.0.len(),
        );
        assert_eq!(msgs.0[0].breaker, breaker_entity);
        assert_eq!(msgs.0[0].wall, wall_entity);
    }
}
