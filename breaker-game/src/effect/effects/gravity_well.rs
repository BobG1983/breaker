//! Gravity well effect handler — creates a gravity well that pulls cells.
//!
//! Observes [`GravityWellFired`] and spawns a gravity well entity with a
//! [`GravityWell`] marker, [`Position2D`], and a timer.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::effect::definition::EffectTarget;

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a gravity well effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct GravityWellFired {
    /// Attraction strength.
    pub strength: f32,
    /// Duration in seconds.
    pub duration: f32,
    /// Effect radius in world units.
    pub radius: f32,
    /// Maximum active wells at once.
    pub max: u32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker component for gravity well entities.
///
/// Used to identify and count active gravity wells for the `max` cap.
#[derive(Component, Debug)]
pub(crate) struct GravityWell;

/// Observer: handles gravity well creation.
///
/// Spawns a new entity with [`GravityWell`] marker and [`Position2D`] at the
/// source bolt's position. Respects the `max` cap — does not spawn if the
/// cap is already reached.
pub(crate) fn handle_gravity_well(
    trigger: On<GravityWellFired>,
    mut commands: Commands,
    bolt_query: Query<&Position2D>,
    well_query: Query<Entity, With<GravityWell>>,
) {
    let event = trigger.event();

    // Respect max cap
    if well_query.iter().count() >= event.max as usize {
        return;
    }

    // Get origin from targets
    let origin = event
        .targets
        .iter()
        .find_map(|t| match t {
            EffectTarget::Entity(e) => bolt_query.get(*e).ok().map(|p| p.0),
            EffectTarget::Location(pos) => Some(*pos),
        })
        .unwrap_or(Vec2::ZERO);

    commands.spawn((GravityWell, Position2D(origin)));
}

/// Registers all observers and systems for the gravity well effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_gravity_well);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::components::Bolt;
    use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_gravity_well);
        app
    }

    #[test]
    fn handle_gravity_well_does_not_panic() {
        use crate::effect::typed_events::GravityWellFired;

        let mut app = test_app();

        app.world_mut().commands().trigger(GravityWellFired {
            strength: 50.0,
            duration: 5.0,
            radius: 100.0,
            max: 2,
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();

        // Stub handler should not panic when receiving its typed event.
    }

    /// Triggering `GravityWellFired` with a bolt target at (50,100) should
    /// spawn a new entity with `GravityWell` marker and `Position2D(50,100)`.
    #[test]
    fn gravity_well_spawns_entity_at_position() {
        let mut app = test_app();

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(50.0, 100.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut().commands().trigger(GravityWellFired {
            strength: 10.0,
            duration: 3.0,
            radius: 100.0,
            max: 3,
            targets: vec![EffectTarget::Entity(bolt)],
            source_chip: None,
        });
        app.world_mut().flush();

        // A new GravityWell entity should exist
        let wells: Vec<(Entity, &Position2D)> = app
            .world_mut()
            .query_filtered::<(Entity, &Position2D), With<GravityWell>>()
            .iter(app.world())
            .collect();
        assert_eq!(
            wells.len(),
            1,
            "GravityWellFired should spawn 1 entity with GravityWell marker, found {}",
            wells.len()
        );

        let (_, pos) = wells[0];
        assert!(
            (pos.0.x - 50.0).abs() < f32::EPSILON && (pos.0.y - 100.0).abs() < f32::EPSILON,
            "GravityWell should be at bolt position (50.0, 100.0), got ({}, {})",
            pos.0.x,
            pos.0.y,
        );
    }

    /// When `max` gravity wells already exist, triggering another
    /// `GravityWellFired` should NOT spawn additional entities.
    #[test]
    fn gravity_well_respects_max_active() {
        let mut app = test_app();

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        // Simulate 3 existing gravity wells
        app.world_mut()
            .spawn((GravityWell, Position2D(Vec2::new(10.0, 10.0))));
        app.world_mut()
            .spawn((GravityWell, Position2D(Vec2::new(20.0, 20.0))));
        app.world_mut()
            .spawn((GravityWell, Position2D(Vec2::new(30.0, 30.0))));

        // Trigger with max:3 -- should NOT spawn because 3 already exist
        app.world_mut().commands().trigger(GravityWellFired {
            strength: 10.0,
            duration: 3.0,
            radius: 100.0,
            max: 3,
            targets: vec![EffectTarget::Entity(bolt)],
            source_chip: None,
        });
        app.world_mut().flush();

        let well_count = app
            .world_mut()
            .query_filtered::<Entity, With<GravityWell>>()
            .iter(app.world())
            .count();
        assert_eq!(
            well_count, 3,
            "max:3 with 3 existing wells should not spawn more, found {well_count}"
        );
    }
}
