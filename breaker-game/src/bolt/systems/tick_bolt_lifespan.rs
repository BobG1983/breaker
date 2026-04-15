//! System to tick bolt lifespan timers and request destruction on expiry.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    bolt::components::{Bolt, BoltLifespan},
    shared::{birthing::Birthing, death_pipeline::kill_yourself::KillYourself},
};

/// Query for active (non-birthing) bolts with lifespan timers.
type BoltLifespanQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static mut BoltLifespan), (With<Bolt>, Without<Birthing>)>;

/// Ticks [`BoltLifespan`] timers on bolt entities and writes
/// [`KillYourself<Bolt>`] when the timer finishes.
pub(crate) fn tick_bolt_lifespan(
    time: Res<Time>,
    mut query: BoltLifespanQuery,
    mut writer: MessageWriter<KillYourself<Bolt>>,
) {
    for (entity, mut lifespan) in &mut query {
        lifespan.0.tick(time.delta());
        if lifespan.0.just_finished() {
            writer.write(KillYourself::<Bolt> {
                victim:  entity,
                killer:  None,
                _marker: PhantomData,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

    use super::tick_bolt_lifespan;
    use crate::{
        bolt::components::{Bolt, BoltLifespan, ExtraBolt},
        effect_v3::EffectV3Plugin,
        shared::{
            death_pipeline::{
                DeathPipelinePlugin, despawn_entity::DespawnEntity, destroyed::Destroyed,
                kill_yourself::KillYourself, sets::DeathPipelineSystems,
                systems::tests::helpers::register_effect_v3_test_infrastructure,
            },
            test_utils::{MessageCollector, TestAppBuilder, attach_message_capture, tick},
        },
    };

    #[derive(Resource, Default)]
    struct CapturedKillYourselfBolt(Vec<KillYourself<Bolt>>);

    fn capture_kill_yourself_bolt(
        mut reader: MessageReader<KillYourself<Bolt>>,
        mut captured: ResMut<CapturedKillYourselfBolt>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
        }
    }

    fn test_app() -> App {
        TestAppBuilder::new()
            .with_message::<KillYourself<Bolt>>()
            .with_resource::<CapturedKillYourselfBolt>()
            .with_system(
                FixedUpdate,
                (tick_bolt_lifespan, capture_kill_yourself_bolt).chain(),
            )
            .build()
    }

    /// Builds a plugin-integration app wiring `tick_bolt_lifespan` before
    /// `DeathPipelineSystems::HandleKill` so the same-tick despawn assertion
    /// holds.
    fn build_lifespan_integration_app() -> App {
        let mut app = TestAppBuilder::new().build();
        app.add_plugins(DeathPipelinePlugin);
        register_effect_v3_test_infrastructure(&mut app);
        app.add_plugins(EffectV3Plugin);

        app.add_systems(
            FixedUpdate,
            tick_bolt_lifespan.before(DeathPipelineSystems::HandleKill),
        );

        attach_message_capture::<Destroyed<Bolt>>(&mut app);
        attach_message_capture::<DespawnEntity>(&mut app);
        app
    }

    // ── Behavior 7: tick_bolt_lifespan writes KillYourself<Bolt> on timer expiry ──

    /// Ticking many frames until a 0.5s timer expires should produce exactly
    /// one `KillYourself<Bolt>` message for the bolt entity.
    #[test]
    fn bolt_lifespan_writes_kill_yourself_bolt_on_expiry() {
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

        let captured = app.world().resource::<CapturedKillYourselfBolt>();
        assert!(
            !captured.0.is_empty(),
            "BoltLifespan(0.5s) should produce KillYourself<Bolt> after expiry"
        );
        assert_eq!(
            captured.0[0].victim, bolt,
            "KillYourself<Bolt> should reference the expired bolt entity"
        );
        assert_eq!(
            captured.0[0].killer, None,
            "lifespan expiry should not attribute a killer"
        );
        // `just_finished()` fires only once — multiple frames after expiry must NOT re-emit.
        assert_eq!(
            captured.0.len(),
            1,
            "exactly one KillYourself<Bolt> should be produced — just_finished() fires once"
        );
    }

    // ── Behavior 8: bolt with future expiry does NOT write KillYourself<Bolt> early ──

    /// A bolt with a 2.0s lifespan should NOT produce `KillYourself`<Bolt> after only one tick.
    #[test]
    fn bolt_lifespan_does_not_write_kill_yourself_bolt_before_expiry() {
        let mut app = test_app();

        app.world_mut().spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            BoltLifespan(Timer::from_seconds(2.0, TimerMode::Once)),
        ));

        tick(&mut app);

        let captured = app.world().resource::<CapturedKillYourselfBolt>();
        assert!(
            captured.0.is_empty(),
            "BoltLifespan(2.0s) should NOT produce KillYourself<Bolt> after 1 tick (~0.016s)"
        );
    }

    /// Edge case: tick twice — still empty (still under 2.0s by a wide margin).
    #[test]
    fn bolt_lifespan_does_not_write_kill_yourself_bolt_before_expiry_two_ticks() {
        let mut app = test_app();

        app.world_mut().spawn((
            Bolt,
            ExtraBolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            BoltLifespan(Timer::from_seconds(2.0, TimerMode::Once)),
        ));

        tick(&mut app);
        tick(&mut app);

        let captured = app.world().resource::<CapturedKillYourselfBolt>();
        assert!(
            captured.0.is_empty(),
            "BoltLifespan(2.0s) should NOT produce KillYourself<Bolt> after 2 ticks"
        );
    }

    // ── Behavior 9-10: tick_bolt_lifespan skips bolts with Birthing ──

    /// Helper to create a `Birthing` component for tests.
    fn test_birthing() -> crate::shared::birthing::Birthing {
        use rantzsoft_physics2d::collision_layers::CollisionLayers;
        use rantzsoft_spatial2d::components::Scale2D;

        use crate::shared::birthing::BIRTHING_DURATION;

        crate::shared::birthing::Birthing {
            timer:          Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
            target_scale:   Scale2D { x: 8.0, y: 8.0 },
            stashed_layers: CollisionLayers::default(),
        }
    }

    // Behavior 9: tick_bolt_lifespan skips bolts with Birthing
    #[test]
    fn tick_bolt_lifespan_skips_bolts_with_birthing() {
        let mut app = test_app();

        // Bolt WITH Birthing — should NOT produce KillYourself<Bolt>
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

        let captured = app.world().resource::<CapturedKillYourselfBolt>();
        assert!(
            captured.0.is_empty(),
            "tick_bolt_lifespan should NOT write KillYourself<Bolt> for a bolt with Birthing"
        );
    }

    // Behavior 10: non-birthing bolt with same lifespan IS destroyed alongside birthing
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

        // Bolt WITHOUT Birthing — should produce KillYourself<Bolt>
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

        let captured = app.world().resource::<CapturedKillYourselfBolt>();
        assert_eq!(
            captured.0.len(),
            1,
            "only the non-birthing bolt should produce KillYourself<Bolt>"
        );
        assert_eq!(
            captured.0[0].victim, normal_bolt,
            "KillYourself<Bolt> should reference the non-birthing bolt"
        );
    }

    // ── Behavior 11: End-to-end lifespan death via DeathPipelinePlugin ──

    /// End-to-end integration test: a bolt with an expiring `BoltLifespan` timer
    /// produces `Destroyed<Bolt>` and `DespawnEntity` messages, and the bolt
    /// entity is actually despawned within a single tick via the unified
    /// pipeline. Injects `KillYourself<Bolt>` indirectly through
    /// `tick_bolt_lifespan` instead of a direct enqueue stub.
    #[test]
    fn bolt_lifespan_end_to_end_despawns_via_unified_pipeline() {
        let mut app = build_lifespan_integration_app();

        // Bolt does NOT have #[require(Spatial2D)], so Position2D MUST be in the bundle.
        // Bolt does NOT need Hp, KilledBy, ExtraBolt, or Velocity2D for this path.
        //
        // Timer duration must be < one fixed tick (1/64s ≈ 0.0156s) so it
        // finishes on the first `tick()` call. Looping multiple ticks would
        // cause `clear_messages::<M>` in First to wipe the collector.
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                BoltLifespan(Timer::from_seconds(0.001, TimerMode::Once)),
            ))
            .id();

        tick(&mut app);

        // Strict equality is REQUIRED — catches dedupe regressions if the
        // timer accidentally re-fires across the tick window.
        let destroyed = app.world().resource::<MessageCollector<Destroyed<Bolt>>>();
        assert_eq!(
            destroyed.0.len(),
            1,
            "exactly one Destroyed<Bolt> should be emitted"
        );
        assert_eq!(destroyed.0[0].victim, bolt);
        assert_eq!(destroyed.0[0].victim_pos, Vec2::new(0.0, 0.0));
        assert_eq!(destroyed.0[0].killer, None);
        assert_eq!(destroyed.0[0].killer_pos, None);

        let despawns = app.world().resource::<MessageCollector<DespawnEntity>>();
        assert_eq!(
            despawns.0.len(),
            1,
            "exactly one DespawnEntity should be emitted"
        );
        assert_eq!(despawns.0[0].entity, bolt);

        assert!(
            app.world().get_entity(bolt).is_err(),
            "Bolt entity should be despawned by process_despawn_requests within the same tick"
        );
    }
}
