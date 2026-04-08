//! System to tick bolt lifespan timers and request destruction on expiry.

use bevy::prelude::*;

use crate::{
    bolt::{
        components::{Bolt, BoltLifespan},
        messages::RequestBoltDestroyed,
    },
    shared::birthing::Birthing,
};

/// Query for active (non-birthing) bolts with lifespan timers.
type BoltLifespanQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static mut BoltLifespan), (With<Bolt>, Without<Birthing>)>;

/// Ticks [`BoltLifespan`] timers on bolt entities and writes
/// [`RequestBoltDestroyed`] when the timer finishes.
pub(crate) fn tick_bolt_lifespan(
    time: Res<Time>,
    mut query: BoltLifespanQuery,
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
    use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

    use super::*;
    use crate::bolt::components::ExtraBolt;

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

    // ── Behaviors 3-4: tick_bolt_lifespan skips bolts with Birthing ──

    /// Helper to create a `Birthing` component for tests.
    fn test_birthing() -> crate::shared::birthing::Birthing {
        use rantzsoft_physics2d::collision_layers::CollisionLayers;
        use rantzsoft_spatial2d::components::Scale2D;

        use crate::shared::birthing::BIRTHING_DURATION;

        crate::shared::birthing::Birthing {
            timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
            target_scale: Scale2D { x: 8.0, y: 8.0 },
            stashed_layers: CollisionLayers::default(),
        }
    }

    // Behavior 3: tick_bolt_lifespan skips bolts with Birthing
    #[test]
    fn tick_bolt_lifespan_skips_bolts_with_birthing() {
        let mut app = test_app();

        // Bolt WITH Birthing — should NOT produce RequestBoltDestroyed
        let _birthing_bolt = app
            .world_mut()
            .spawn((
                Bolt,
                ExtraBolt,
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::ZERO),
                BoltLifespan(Timer::from_seconds(0.5, TimerMode::Once)),
                test_birthing(),
            ))
            .id();

        // Tick 40 times (well past 0.5s lifespan)
        for _ in 0..40 {
            tick(&mut app);
        }

        let captured = app.world().resource::<CapturedRequestBoltDestroyed>();
        assert!(
            captured.0.is_empty(),
            "tick_bolt_lifespan should NOT destroy a bolt with Birthing component"
        );
    }

    // Behavior 3 edge case: non-birthing bolt with same lifespan IS destroyed
    #[test]
    fn tick_bolt_lifespan_skips_birthing_but_processes_non_birthing() {
        let mut app = test_app();

        // Bolt WITH Birthing — should be skipped
        let _birthing_bolt = app
            .world_mut()
            .spawn((
                Bolt,
                ExtraBolt,
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::ZERO),
                BoltLifespan(Timer::from_seconds(0.5, TimerMode::Once)),
                test_birthing(),
            ))
            .id();

        // Bolt WITHOUT Birthing — should be destroyed
        let normal_bolt = app
            .world_mut()
            .spawn((
                Bolt,
                ExtraBolt,
                Velocity2D(Vec2::new(0.0, 400.0)),
                Position2D(Vec2::ZERO),
                BoltLifespan(Timer::from_seconds(0.5, TimerMode::Once)),
            ))
            .id();

        for _ in 0..40 {
            tick(&mut app);
        }

        let captured = app.world().resource::<CapturedRequestBoltDestroyed>();
        assert_eq!(
            captured.0.len(),
            1,
            "only the non-birthing bolt should produce RequestBoltDestroyed"
        );
        assert_eq!(
            captured.0[0].bolt, normal_bolt,
            "RequestBoltDestroyed should reference the non-birthing bolt"
        );
    }

    // Behavior 4: tick_bolt_lifespan processes bolts without Birthing normally
    // (covered by existing `bolt_lifespan_destroys_bolt_on_expiry` test — this
    //  is the edge case that the filter change does not break existing behavior)
}
