//! Phantom bolt effect handler — spawns a temporary bolt with infinite piercing.
//!
//! Observes [`SpawnPhantomFired`] and spawns a phantom bolt entity with
//! [`ExtraBolt`], [`BoltLifespan`], and infinite [`Piercing`].

use bevy::prelude::*;

use crate::{
    bolt::components::{Bolt, BoltLifespan, ExtraBolt},
    chips::components::Piercing,
    effect::definition::EffectTarget,
};
use rantzsoft_spatial2d::components::Position2D;

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a spawn phantom effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct SpawnPhantomFired {
    /// How long the phantom persists in seconds.
    pub duration: f32,
    /// Maximum active phantoms at once.
    pub max_active: u32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The originating chip name, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

/// Marker component for phantom bolt entities spawned by this effect.
///
/// Used to count active phantoms and enforce `max_active` cap.
#[derive(Component, Debug)]
pub(crate) struct PhantomBolt;

/// Observer: handles phantom bolt spawning.
///
/// Spawns a new entity with [`ExtraBolt`], [`PhantomBolt`], [`BoltLifespan`],
/// and infinite [`Piercing`] at the source bolt's position. Respects the
/// `max_active` cap — does not spawn if the cap is already reached.
pub(crate) fn handle_spawn_phantom(
    trigger: On<SpawnPhantomFired>,
    mut commands: Commands,
    bolt_query: Query<&Position2D, With<Bolt>>,
    phantom_query: Query<Entity, With<PhantomBolt>>,
) {
    let event = trigger.event();

    // Respect max_active cap
    if phantom_query.iter().count() >= event.max_active as usize {
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

    commands.spawn((
        Bolt,
        ExtraBolt,
        PhantomBolt,
        Position2D(origin),
        Piercing(u32::MAX),
        BoltLifespan(Timer::from_seconds(event.duration, TimerMode::Once)),
    ));
}

/// Registers all observers and systems for the spawn phantom effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_spawn_phantom);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::components::{Bolt, BoltLifespan, ExtraBolt};
    use crate::chips::components::Piercing;
    use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_spawn_phantom);
        app
    }

    #[test]
    fn handle_spawn_phantom_does_not_panic() {
        use crate::effect::typed_events::SpawnPhantomFired;

        let mut app = test_app();

        app.world_mut().commands().trigger(SpawnPhantomFired {
            duration: 5.0,
            max_active: 2,
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();

        // Stub handler should not panic when receiving its typed event.
    }

    /// Triggering `SpawnPhantomFired` with a bolt target should spawn a new
    /// entity with `ExtraBolt`, `BoltLifespan`, and `Piercing(u32::MAX)`.
    #[test]
    fn spawn_phantom_spawns_bolt_with_lifespan() {
        let mut app = test_app();

        // Source bolt entity the phantom originates from
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(100.0, 200.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut().commands().trigger(SpawnPhantomFired {
            duration: 3.0,
            max_active: 5,
            targets: vec![EffectTarget::Entity(bolt)],
            source_chip: None,
        });
        app.world_mut().flush();

        // Check that a new entity was spawned with the expected components
        let phantom_count = app
            .world_mut()
            .query_filtered::<Entity, (With<ExtraBolt>, With<BoltLifespan>, With<Piercing>)>()
            .iter(app.world())
            .count();
        assert_eq!(
            phantom_count, 1,
            "SpawnPhantomFired should spawn 1 entity with ExtraBolt + BoltLifespan + Piercing, found {phantom_count}"
        );

        // Verify the piercing is set to infinite (u32::MAX)
        let piercing = app
            .world_mut()
            .query_filtered::<&Piercing, With<ExtraBolt>>()
            .iter(app.world())
            .next()
            .expect("phantom bolt should have Piercing");
        assert_eq!(
            piercing.0,
            u32::MAX,
            "phantom bolt Piercing should be u32::MAX (infinite), got {}",
            piercing.0
        );
    }

    /// When `max_active` phantoms already exist, triggering another
    /// `SpawnPhantomFired` should NOT spawn additional entities.
    #[test]
    fn spawn_phantom_respects_max_active() {
        let mut app = test_app();

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(50.0, 100.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        // Simulate 2 existing phantom bolts already active
        app.world_mut().spawn((ExtraBolt, PhantomBolt));
        app.world_mut().spawn((ExtraBolt, PhantomBolt));

        // Trigger with max_active: 2 -- should NOT spawn because 2 already exist
        app.world_mut().commands().trigger(SpawnPhantomFired {
            duration: 3.0,
            max_active: 2,
            targets: vec![EffectTarget::Entity(bolt)],
            source_chip: None,
        });
        app.world_mut().flush();

        // Count phantom-marked entities (should still be 2, not 3)
        let phantom_count = app
            .world_mut()
            .query_filtered::<Entity, With<PhantomBolt>>()
            .iter(app.world())
            .count();
        assert_eq!(
            phantom_count, 2,
            "max_active:2 with 2 existing phantoms should not spawn more, found {phantom_count}"
        );
    }
}
