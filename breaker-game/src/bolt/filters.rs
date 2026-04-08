//! Bolt domain query filters.

use bevy::prelude::*;

use crate::{
    bolt::components::{Bolt, BoltServing},
    shared::birthing::Birthing,
};

/// Query filter for active (non-serving) bolts.
///
/// Shared across bolt and physics systems that should skip serving bolts.
/// Also excludes bolts in the birthing animation.
pub(crate) type ActiveFilter = (With<Bolt>, Without<BoltServing>, Without<Birthing>);

/// Query filter for serving bolts (hovering, awaiting launch).
///
/// Used by `hover_bolt`.
pub(crate) type ServingFilter = (With<Bolt>, With<BoltServing>);

/// Query filter for serving bolts that are ready to launch (not birthing).
///
/// Used by `launch_bolt`. Extends [`ServingFilter`] with `Without<Birthing>`
/// because a bolt can be both serving and birthing at node entry.
pub(crate) type LaunchFilter = (With<Bolt>, With<BoltServing>, Without<Birthing>);

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use rantzsoft_physics2d::collision_layers::CollisionLayers;
    use rantzsoft_spatial2d::components::Scale2D;

    use super::*;
    use crate::shared::birthing::Birthing;

    /// Helper to create a `Birthing` component for tests.
    fn test_birthing() -> Birthing {
        Birthing {
            timer: Timer::from_seconds(0.3, TimerMode::Once),
            target_scale: Scale2D { x: 8.0, y: 8.0 },
            stashed_layers: CollisionLayers::default(),
        }
    }

    // Behavior 1: ActiveFilter excludes bolts with Birthing component
    #[test]
    fn active_filter_excludes_bolts_with_birthing() {
        let mut world = World::new();

        // Bolt entity WITH Birthing and without BoltServing
        let birthing_bolt = world.spawn((Bolt, test_birthing())).id();

        // Query with ActiveFilter
        let mut query = world.query_filtered::<Entity, ActiveFilter>();
        let matched: Vec<Entity> = query.iter(&world).collect();

        assert!(
            !matched.contains(&birthing_bolt),
            "ActiveFilter should NOT match a bolt with Birthing component"
        );
    }

    // Behavior 1 edge case: bolt with both Birthing AND BoltServing is excluded
    #[test]
    fn active_filter_excludes_bolt_with_birthing_and_serving() {
        let mut world = World::new();

        let bolt = world.spawn((Bolt, BoltServing, test_birthing())).id();

        let mut query = world.query_filtered::<Entity, ActiveFilter>();
        let matched: Vec<Entity> = query.iter(&world).collect();

        assert!(
            !matched.contains(&bolt),
            "ActiveFilter should exclude bolt with both Birthing and BoltServing"
        );
    }

    // Behavior 2: ActiveFilter still matches non-birthing active bolts
    #[test]
    fn active_filter_matches_non_birthing_active_bolts() {
        let mut world = World::new();

        // Bolt without BoltServing and without Birthing
        let active_bolt = world.spawn(Bolt).id();

        let mut query = world.query_filtered::<Entity, ActiveFilter>();
        let matched: Vec<Entity> = query.iter(&world).collect();

        assert!(
            matched.contains(&active_bolt),
            "ActiveFilter should match a bolt without Birthing and without BoltServing"
        );
    }

    // Behavior 2 edge case: bolt with many other components but no Birthing is still matched
    #[test]
    fn active_filter_matches_bolt_with_extra_components_but_no_birthing() {
        use crate::bolt::components::{BoltLifespan, ExtraBolt};

        let mut world = World::new();

        let bolt = world
            .spawn((
                Bolt,
                ExtraBolt,
                BoltLifespan(Timer::from_seconds(2.0, TimerMode::Once)),
            ))
            .id();

        let mut query = world.query_filtered::<Entity, ActiveFilter>();
        let matched: Vec<Entity> = query.iter(&world).collect();

        assert!(
            matched.contains(&bolt),
            "ActiveFilter should match bolt with ExtraBolt+BoltLifespan but no Birthing"
        );
    }
}
