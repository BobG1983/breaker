//! `MirrorConfig` — fire-and-forget bolt duplication.

use bevy::prelude::*;
use rantzsoft_physics2d::collision_layers::CollisionLayers;
use rantzsoft_spatial2d::components::{Position2D, Scale2D, Velocity2D};
use serde::{Deserialize, Serialize};

use crate::{
    bolt::components::{Bolt, ExtraBolt},
    effect_v3::{storage::BoundEffects, traits::Fireable},
    shared::birthing::Birthing,
};

/// Duplicates a bolt with a mirrored velocity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MirrorConfig {
    /// Whether the mirrored bolt copies the source bolt's effect trees.
    pub inherit: bool,
}

impl Fireable for MirrorConfig {
    fn fire(&self, entity: Entity, _source: &str, world: &mut World) {
        // Read source state
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);
        let vel = world.get::<Velocity2D>(entity).map_or(Vec2::ZERO, |v| v.0);

        // Read source effects for inheritance
        let inherited_effects = if self.inherit {
            world.get::<BoundEffects>(entity).cloned()
        } else {
            None
        };

        // Compute mirrored velocity (negate x, keep y)
        let mirrored_vel = Vec2::new(-vel.x, vel.y);

        let birthing = Birthing::new(Scale2D { x: 8.0, y: 8.0 }, CollisionLayers::default());

        // Always spawn, even with zero velocity
        let mut bolt_entity = world.spawn((
            Bolt,
            ExtraBolt,
            Position2D(pos),
            Velocity2D(mirrored_vel),
            birthing,
        ));

        if let Some(effects) = inherited_effects {
            bolt_entity.insert(effects);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;
    use rantzsoft_spatial2d::components::{BaseSpeed, Position2D, Velocity2D};

    use super::*;
    use crate::{
        bolt::components::{Bolt, ExtraBolt},
        effect_v3::{
            effects::DamageBoostConfig,
            storage::BoundEffects,
            traits::Fireable,
            types::{EffectType, Tree},
        },
        shared::birthing::Birthing,
    };

    fn spawn_source(world: &mut World, pos: Vec2, vel: Vec2) -> Entity {
        world
            .spawn((Bolt, Position2D(pos), Velocity2D(vel), BaseSpeed(400.0)))
            .id()
    }

    #[test]
    fn fire_spawns_one_extra_bolt() {
        let mut world = World::new();
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(150.0, 350.0));

        let config = MirrorConfig { inherit: false };
        config.fire(source, "mirror_protocol", &mut world);
        world.flush();

        let extra_count = world
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(&world)
            .count();
        assert_eq!(extra_count, 1, "should spawn exactly 1 ExtraBolt entity");
    }

    #[test]
    fn spawned_bolt_is_at_source_position() {
        let mut world = World::new();
        let source = spawn_source(&mut world, Vec2::new(55.0, 120.0), Vec2::new(150.0, 350.0));

        let config = MirrorConfig { inherit: false };
        config.fire(source, "mirror_protocol", &mut world);
        world.flush();

        let positions: Vec<Vec2> = world
            .query_filtered::<&Position2D, With<ExtraBolt>>()
            .iter(&world)
            .map(|p| p.0)
            .collect();
        assert_eq!(positions.len(), 1);
        assert!(
            (positions[0] - Vec2::new(55.0, 120.0)).length() < 1e-3,
            "spawned bolt should be at source position, got {:?}",
            positions[0],
        );
    }

    #[test]
    fn spawned_bolt_has_negated_x_same_y_velocity() {
        let mut world = World::new();
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(150.0, 350.0));

        let config = MirrorConfig { inherit: false };
        config.fire(source, "mirror_protocol", &mut world);
        world.flush();

        let velocities: Vec<Vec2> = world
            .query_filtered::<&Velocity2D, With<ExtraBolt>>()
            .iter(&world)
            .map(|v| v.0)
            .collect();
        assert_eq!(velocities.len(), 1);
        let expected = Vec2::new(-150.0, 350.0);
        assert!(
            (velocities[0] - expected).length() < 1e-3,
            "mirror velocity should be ({expected:?}), got {:?}",
            velocities[0],
        );
    }

    #[test]
    fn negative_x_velocity_negates_to_positive() {
        let mut world = World::new();
        let source = spawn_source(
            &mut world,
            Vec2::new(100.0, 200.0),
            Vec2::new(-200.0, 300.0),
        );

        let config = MirrorConfig { inherit: false };
        config.fire(source, "mirror_protocol", &mut world);
        world.flush();

        let velocities: Vec<Vec2> = world
            .query_filtered::<&Velocity2D, With<ExtraBolt>>()
            .iter(&world)
            .map(|v| v.0)
            .collect();
        assert_eq!(velocities.len(), 1);
        let expected = Vec2::new(200.0, 300.0);
        assert!(
            (velocities[0] - expected).length() < 1e-3,
            "negated -200 x should be 200, got {:?}",
            velocities[0],
        );
    }

    #[test]
    fn zero_x_velocity_produces_same_velocity() {
        let mut world = World::new();
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        let config = MirrorConfig { inherit: false };
        config.fire(source, "mirror_protocol", &mut world);
        world.flush();

        let velocities: Vec<Vec2> = world
            .query_filtered::<&Velocity2D, With<ExtraBolt>>()
            .iter(&world)
            .map(|v| v.0)
            .collect();
        assert_eq!(velocities.len(), 1);
        let expected = Vec2::new(0.0, 400.0);
        assert!(
            (velocities[0] - expected).length() < 1e-3,
            "negated 0 x should remain 0, got {:?}",
            velocities[0],
        );
    }

    #[test]
    fn source_with_no_velocity_produces_mirror_with_zero_velocity() {
        let mut world = World::new();
        // Source with Position2D but NO Velocity2D
        let source = world
            .spawn((Bolt, Position2D(Vec2::new(100.0, 200.0)), BaseSpeed(400.0)))
            .id();

        let config = MirrorConfig { inherit: false };
        config.fire(source, "mirror_protocol", &mut world);
        world.flush();

        // Per the correction: ALWAYS spawn (don't skip). Mirror gets Vec2::ZERO.
        let extra_count = world
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(&world)
            .count();
        assert_eq!(
            extra_count, 1,
            "should still spawn mirror bolt when source has no velocity"
        );

        let velocities: Vec<Vec2> = world
            .query_filtered::<&Velocity2D, With<ExtraBolt>>()
            .iter(&world)
            .map(|v| v.0)
            .collect();
        assert_eq!(velocities.len(), 1);
        assert!(
            velocities[0].length() < 1e-3,
            "mirror of no velocity should be zero, got {:?}",
            velocities[0],
        );
    }

    #[test]
    fn inherit_true_copies_source_bound_effects() {
        let mut world = World::new();

        let tree_a = Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));
        let source = world
            .spawn((
                Bolt,
                Position2D(Vec2::new(100.0, 200.0)),
                Velocity2D(Vec2::new(150.0, 350.0)),
                BaseSpeed(400.0),
                BoundEffects(vec![("amp".to_string(), tree_a)]),
            ))
            .id();

        let config = MirrorConfig { inherit: true };
        config.fire(source, "mirror_protocol", &mut world);
        world.flush();

        let inherited: Vec<&BoundEffects> = world
            .query_filtered::<&BoundEffects, With<ExtraBolt>>()
            .iter(&world)
            .collect();
        assert_eq!(inherited.len(), 1, "mirror bolt should have BoundEffects");
        assert!(
            inherited[0].0.iter().any(|(name, _)| name == "amp"),
            "inherited BoundEffects should contain amp",
        );
    }

    #[test]
    fn inherit_false_does_not_copy_bound_effects() {
        let mut world = World::new();

        let tree_a = Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(1.5),
        }));
        let source = world
            .spawn((
                Bolt,
                Position2D(Vec2::new(100.0, 200.0)),
                Velocity2D(Vec2::new(150.0, 350.0)),
                BaseSpeed(400.0),
                BoundEffects(vec![("amp".to_string(), tree_a)]),
            ))
            .id();

        let config = MirrorConfig { inherit: false };
        config.fire(source, "mirror_protocol", &mut world);
        world.flush();

        let inherited_count = world
            .query_filtered::<&BoundEffects, With<ExtraBolt>>()
            .iter(&world)
            .count();
        assert_eq!(
            inherited_count, 0,
            "inherit: false should not copy BoundEffects"
        );
    }

    #[test]
    fn spawned_bolt_has_bolt_and_extra_bolt_markers() {
        let mut world = World::new();
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(150.0, 350.0));

        let config = MirrorConfig { inherit: false };
        config.fire(source, "mirror_protocol", &mut world);
        world.flush();

        let both_count = world
            .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
            .iter(&world)
            .count();
        assert_eq!(
            both_count, 1,
            "spawned bolt should have both Bolt and ExtraBolt"
        );
    }

    #[test]
    fn spawned_bolt_has_birthing_component() {
        let mut world = World::new();
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(150.0, 350.0));

        let config = MirrorConfig { inherit: false };
        config.fire(source, "mirror_protocol", &mut world);
        world.flush();

        let birthing_count = world
            .query_filtered::<&Birthing, With<ExtraBolt>>()
            .iter(&world)
            .count();
        assert_eq!(
            birthing_count, 1,
            "spawned bolt should have Birthing component"
        );
    }
}
