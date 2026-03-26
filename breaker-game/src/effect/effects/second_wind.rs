//! Second wind effect handler — spawns invisible bottom wall that bounces bolt once.
//!
//! Observes [`SecondWindFired`] and spawns a [`SecondWindWall`] entity.

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, Scale2D};

use crate::effect::definition::EffectTarget;
use crate::shared::{PlayfieldConfig, BOLT_LAYER, WALL_LAYER};
use crate::wall::components::{Wall, WallSize};

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker for the invisible bottom wall spawned by the `SecondWind` effect.
/// Filtered out of attraction queries via `Without<SecondWindWall>`.
#[derive(Component, Debug, Default)]
pub(crate) struct SecondWindWall;

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a second wind effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct SecondWindFired {
    /// Duration of invulnerability in seconds.
    pub invuln_secs: f32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Observer: handles second wind — spawns invisible bottom wall.
///
/// Spawns a [`SecondWindWall`] entity at the bottom of the playfield. The wall
/// is invisible (no [`GameDrawLayer`]) and will be despawned after a bolt hits
/// it, giving the player one free bounce before losing a bolt.
pub(crate) fn handle_second_wind(
    _trigger: On<SecondWindFired>,
    mut commands: Commands,
    playfield: Res<PlayfieldConfig>,
    existing: Query<Entity, With<SecondWindWall>>,
) {
    // Don't spawn a duplicate wall.
    if !existing.is_empty() {
        return;
    }

    let half_width = playfield.width / 2.0;
    let half_height = playfield.wall_half_thickness();
    let y = playfield.bottom() - half_height;

    commands.spawn((
        Wall,
        SecondWindWall,
        WallSize {
            half_width,
            half_height,
        },
        Position2D(Vec2::new(0.0, y)),
        Scale2D {
            x: half_width,
            y: half_height,
        },
        Aabb2D::new(Vec2::ZERO, Vec2::new(half_width, half_height)),
        CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
        // NO GameDrawLayer — invisible wall
    ));
}

/// Registers all observers and systems for the second wind effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_second_wind);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{GameDrawLayer, BOLT_LAYER, WALL_LAYER};
    use crate::wall::components::{Wall, WallSize};
    use rantzsoft_physics2d::collision_layers::CollisionLayers;
    use rantzsoft_spatial2d::components::Position2D;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<PlayfieldConfig>()
            .add_observer(handle_second_wind);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn trigger_second_wind(app: &mut App) {
        app.world_mut().commands().trigger(SecondWindFired {
            invuln_secs: 3.0,
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();
    }

    #[test]
    fn handle_second_wind_does_not_panic() {
        let mut app = test_app();

        trigger_second_wind(&mut app);

        // Stub handler should not panic when receiving its typed event.
    }

    // =========================================================================
    // Part A: SecondWind wall spawning
    // =========================================================================

    #[test]
    fn handle_second_wind_spawns_wall_at_bottom_of_playfield() {
        // Given: PlayfieldConfig default (width=800, height=600). No SecondWindWall entity.
        // When: SecondWindFired fires
        // Then: A new entity exists with Wall, SecondWindWall, WallSize, Position2D,
        //       CollisionLayers. Position2D.y is below the playfield bottom (-300.0).
        let mut app = test_app();

        trigger_second_wind(&mut app);
        tick(&mut app);

        let playfield = PlayfieldConfig::default();

        // Verify SecondWindWall entity exists
        let sw_count = app
            .world_mut()
            .query_filtered::<Entity, With<SecondWindWall>>()
            .iter(app.world())
            .count();
        assert_eq!(
            sw_count, 1,
            "handle_second_wind should spawn exactly one SecondWindWall entity"
        );

        // Verify it has Wall component
        let has_wall = app
            .world_mut()
            .query_filtered::<Entity, (With<SecondWindWall>, With<Wall>)>()
            .iter(app.world())
            .count();
        assert_eq!(
            has_wall, 1,
            "SecondWindWall entity should also have Wall component"
        );

        // Verify it has WallSize component
        let has_wall_size = app
            .world_mut()
            .query_filtered::<Entity, (With<SecondWindWall>, With<WallSize>)>()
            .iter(app.world())
            .count();
        assert_eq!(
            has_wall_size, 1,
            "SecondWindWall entity should have WallSize component"
        );

        // Verify it has CollisionLayers
        let layers: Vec<&CollisionLayers> = app
            .world_mut()
            .query_filtered::<&CollisionLayers, With<SecondWindWall>>()
            .iter(app.world())
            .collect();
        assert_eq!(
            layers.len(),
            1,
            "SecondWindWall entity should have CollisionLayers"
        );

        // Verify Position2D is below playfield bottom
        let positions: Vec<&Position2D> = app
            .world_mut()
            .query_filtered::<&Position2D, With<SecondWindWall>>()
            .iter(app.world())
            .collect();
        assert_eq!(
            positions.len(),
            1,
            "SecondWindWall entity should have Position2D"
        );
        assert!(
            positions[0].0.y <= playfield.bottom(),
            "SecondWindWall Position2D.y ({}) should be at or below playfield bottom ({})",
            positions[0].0.y,
            playfield.bottom(),
        );
    }

    #[test]
    fn second_wind_wall_is_invisible_no_game_draw_layer() {
        // Given: No SecondWindWall entity
        // When: SecondWindFired fires
        // Then: Spawned entity exists but does NOT have GameDrawLayer component (invisible wall)
        let mut app = test_app();

        trigger_second_wind(&mut app);
        tick(&mut app);

        // First: entity must exist (guards against trivially passing when nothing spawns)
        let sw_entities: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<SecondWindWall>>()
            .iter(app.world())
            .collect();
        assert_eq!(
            sw_entities.len(),
            1,
            "SecondWindWall entity must exist before checking draw layer"
        );

        // Then: it must NOT have GameDrawLayer
        assert!(
            app.world().get::<GameDrawLayer>(sw_entities[0]).is_none(),
            "SecondWindWall should NOT have GameDrawLayer (invisible wall)"
        );
    }

    #[test]
    fn handle_second_wind_does_not_spawn_duplicate_wall() {
        // Given: A SecondWindWall entity already exists
        // When: SecondWindFired fires again
        // Then: Total SecondWindWall entity count is still 1
        let mut app = test_app();

        // Pre-spawn a SecondWindWall entity
        app.world_mut().spawn((
            SecondWindWall,
            Wall,
            WallSize {
                half_width: 400.0,
                half_height: 5.0,
            },
            Position2D(Vec2::new(0.0, -310.0)),
            CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
        ));

        trigger_second_wind(&mut app);
        tick(&mut app);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<SecondWindWall>>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 1,
            "handle_second_wind should not spawn a duplicate when SecondWindWall already exists"
        );
    }

    #[test]
    fn handle_second_wind_spawns_wall_after_previous_despawned() {
        // Given: Previous SecondWindWall entity was despawned
        // When: SecondWindFired fires
        // Then: New SecondWindWall entity exists (count = 1)
        let mut app = test_app();

        // Spawn and then despawn a SecondWindWall
        let prev = app.world_mut().spawn(SecondWindWall).id();
        app.world_mut().despawn(prev);
        app.update();

        // Verify no SecondWindWall exists
        let count_before = app
            .world_mut()
            .query_filtered::<Entity, With<SecondWindWall>>()
            .iter(app.world())
            .count();
        assert_eq!(
            count_before, 0,
            "precondition: no SecondWindWall should exist after despawn"
        );

        trigger_second_wind(&mut app);
        tick(&mut app);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<SecondWindWall>>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 1,
            "handle_second_wind should spawn a new wall after previous was despawned"
        );
    }
}
