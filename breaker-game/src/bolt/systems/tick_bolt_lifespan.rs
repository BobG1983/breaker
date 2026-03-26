//! System to tick bolt lifespan timers and request destruction on expiry.

use bevy::prelude::*;

use crate::bolt::{
    components::{Bolt, BoltLifespan},
    messages::RequestBoltDestroyed,
};

/// Ticks [`BoltLifespan`] timers on bolt entities and writes
/// [`RequestBoltDestroyed`] when the timer finishes.
pub(crate) fn tick_bolt_lifespan(
    time: Res<Time>,
    mut query: Query<(Entity, &mut BoltLifespan), With<Bolt>>,
    mut writer: MessageWriter<RequestBoltDestroyed>,
) {
    for (entity, mut lifespan) in &mut query {
        lifespan.0.tick(time.delta());
        if lifespan.0.just_finished() {
            writer.write(RequestBoltDestroyed { bolt: entity });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::components::ExtraBolt;
    use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

    #[derive(Resource, Default)]
    struct CapturedRequestBoltDestroyed(Vec<RequestBoltDestroyed>);

    fn capture_request_bolt_destroyed(
        mut reader: MessageReader<RequestBoltDestroyed>,
        mut captured: ResMut<CapturedRequestBoltDestroyed>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<RequestBoltDestroyed>()
            .init_resource::<CapturedRequestBoltDestroyed>()
            .add_systems(
                FixedUpdate,
                (tick_bolt_lifespan, capture_request_bolt_destroyed).chain(),
            );
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    /// Ticking many frames until a 0.5s timer expires should produce a
    /// `RequestBoltDestroyed` message for the bolt entity.
    #[test]
    fn bolt_lifespan_destroys_bolt_on_expiry() {
        let mut app = test_app();

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                ExtraBolt,
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::new(0.0, 0.0)),
                BoltLifespan(Timer::from_seconds(0.5, TimerMode::Once)),
            ))
            .id();

        // Default fixed timestep is 1/64s (~0.015625s).
        // 0.5s / 0.015625s = 32 ticks needed. Tick 40 to be safe.
        for _ in 0..40 {
            tick(&mut app);
        }

        let captured = app.world().resource::<CapturedRequestBoltDestroyed>();
        assert!(
            !captured.0.is_empty(),
            "BoltLifespan(0.5s) should produce RequestBoltDestroyed after expiry"
        );
        assert_eq!(
            captured.0[0].bolt, bolt,
            "RequestBoltDestroyed should reference the expired bolt entity"
        );
    }

    /// A bolt with a 2.0s lifespan should NOT be destroyed after only one tick.
    #[test]
    fn bolt_lifespan_does_not_destroy_before_expiry() {
        let mut app = test_app();

        app.world_mut().spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            BoltLifespan(Timer::from_seconds(2.0, TimerMode::Once)),
        ));

        tick(&mut app);

        let captured = app.world().resource::<CapturedRequestBoltDestroyed>();
        assert!(
            captured.0.is_empty(),
            "BoltLifespan(2.0s) should NOT produce RequestBoltDestroyed after 1 tick (~0.016s)"
        );
    }
}
