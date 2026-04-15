use std::collections::HashSet;

use bevy::{ecs::world::CommandQueue, prelude::*};
use rantzsoft_spatial2d::components::Spatial2D;

use super::{super::system::bolt_lost, helpers::*};
use crate::{
    bolt::{components::ExtraBolt, messages::BoltLost},
    prelude::*,
    shared::{GameDrawLayer, death_pipeline::kill_yourself::KillYourself},
};

fn spawn_bolt_in_app(app: &mut App, build_fn: impl FnOnce(&mut Commands) -> Entity) -> Entity {
    let world = app.world_mut();
    let mut queue = CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        build_fn(&mut commands)
    };
    queue.apply(world);
    entity
}

// ── Behavior 2: End-to-end despawn via the unified pipeline ─────────────

#[test]
fn extra_bolt_below_floor_is_despawned() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let def = make_default_bolt_definition();
    let entity = spawn_bolt_in_app(&mut app, |commands| {
        Bolt::builder()
            .at_position(Vec2::new(0.0, playfield.bottom() - 100.0))
            .definition(&def)
            .with_velocity(Velocity2D(Vec2::new(0.0, -400.0)))
            .extra()
            .headless()
            .spawn(commands)
    });
    tick(&mut app);

    assert!(
        app.world().get_entity(entity).is_err(),
        "extra bolt should be despawned when lost — end-to-end via unified pipeline \
         (bolt_lost writes KillYourself<Bolt>, handle_kill::<Bolt> writes DespawnEntity, \
         process_despawn_requests despawns the entity within the same tick)"
    );
}

// Behavior 2 edge case: a second tick after the first leaves the entity gone and panics nothing.
#[test]
fn extra_bolt_below_floor_is_despawned_idempotent_across_ticks() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let def = make_default_bolt_definition();
    let entity = spawn_bolt_in_app(&mut app, |commands| {
        Bolt::builder()
            .at_position(Vec2::new(0.0, playfield.bottom() - 100.0))
            .definition(&def)
            .with_velocity(Velocity2D(Vec2::new(0.0, -400.0)))
            .extra()
            .headless()
            .spawn(commands)
    });
    tick(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_entity(entity).is_err(),
        "extra bolt should remain despawned after a second tick — no double-despawn panic"
    );
}

#[test]
fn extra_bolt_sends_bolt_lost_on_despawn() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    app.init_resource::<BoltLostCount>();
    app.add_systems(FixedUpdate, count_bolt_lost.after(bolt_lost));

    let def = make_default_bolt_definition();
    spawn_bolt_in_app(&mut app, |commands| {
        Bolt::builder()
            .at_position(Vec2::new(0.0, playfield.bottom() - 100.0))
            .definition(&def)
            .with_velocity(Velocity2D(Vec2::new(0.0, -400.0)))
            .extra()
            .headless()
            .spawn(commands)
    });
    tick(&mut app);

    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(count.0, 1, "BoltLost message should be sent for extra bolt");
}

#[test]
fn baseline_bolt_still_respawns_with_extra_present() {
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    // Baseline bolt (no ExtraBolt)
    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    // Extra bolt
    let def = make_default_bolt_definition();
    spawn_bolt_in_app(&mut app, |commands| {
        Bolt::builder()
            .at_position(Vec2::new(50.0, playfield.bottom() - 100.0))
            .definition(&def)
            .with_velocity(Velocity2D(Vec2::new(0.0, -400.0)))
            .extra()
            .headless()
            .spawn(commands)
    });
    tick(&mut app);

    // Baseline bolt should still exist (respawned)
    let bolt_count = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .count();
    assert_eq!(bolt_count, 1, "only baseline bolt should remain");

    // Verify it's the baseline (no ExtraBolt)
    let extra_count = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
        .iter(app.world())
        .count();
    assert_eq!(extra_count, 0, "extra bolt should be gone");
}

// =========================================================================
// Extra bolt death via KillYourself<Bolt> unified death pipeline
// =========================================================================

// ── Behavior 1: Extra bolt writes KillYourself<Bolt> ──

#[test]
fn extra_bolt_writes_kill_yourself_bolt_instead_of_despawning() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayfieldConfig>()
        .init_resource::<GameRng>()
        .add_message::<BoltLost>()
        .add_message::<KillYourself<Bolt>>()
        .init_resource::<CapturedKillYourselfBolt>()
        .add_systems(FixedUpdate, (bolt_lost, capture_kill_yourself_bolt).chain());

    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let def = make_default_bolt_definition();
    let entity = spawn_bolt_in_app(&mut app, |commands| {
        Bolt::builder()
            .at_position(Vec2::new(50.0, playfield.bottom() - 100.0))
            .definition(&def)
            .with_velocity(Velocity2D(Vec2::new(0.0, -400.0)))
            .extra()
            .headless()
            .spawn(commands)
    });
    tick(&mut app);

    let captured = app.world().resource::<CapturedKillYourselfBolt>();
    assert_eq!(
        captured.0.len(),
        1,
        "extra bolt should write exactly one KillYourself<Bolt>"
    );
    assert_eq!(
        captured.0[0].victim, entity,
        "KillYourself<Bolt>.victim should equal the extra bolt entity"
    );
    assert_eq!(
        captured.0[0].killer, None,
        "out-of-bounds extra bolt should not attribute a killer"
    );

    // Entity should STILL BE ALIVE — this test app does NOT wire handle_kill<Bolt>,
    // so the unified pipeline never runs. This test isolates the emission contract.
    assert!(
        app.world().get_entity(entity).is_ok(),
        "extra bolt entity should still be alive — this test does not wire the downstream pipeline"
    );
}

// Behavior 1 edge case: two extra bolts in the same tick.
#[test]
fn extra_bolt_two_in_one_tick_write_two_kill_yourself_bolt_messages() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayfieldConfig>()
        .init_resource::<GameRng>()
        .add_message::<BoltLost>()
        .add_message::<KillYourself<Bolt>>()
        .init_resource::<CapturedKillYourselfBolt>()
        .add_systems(FixedUpdate, (bolt_lost, capture_kill_yourself_bolt).chain());

    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let def = make_default_bolt_definition();
    let entity_a = spawn_bolt_in_app(&mut app, |commands| {
        Bolt::builder()
            .at_position(Vec2::new(50.0, playfield.bottom() - 100.0))
            .definition(&def)
            .with_velocity(Velocity2D(Vec2::new(0.0, -400.0)))
            .extra()
            .headless()
            .spawn(commands)
    });
    let entity_b = spawn_bolt_in_app(&mut app, |commands| {
        Bolt::builder()
            .at_position(Vec2::new(-50.0, playfield.bottom() - 100.0))
            .definition(&def)
            .with_velocity(Velocity2D(Vec2::new(0.0, -400.0)))
            .extra()
            .headless()
            .spawn(commands)
    });
    tick(&mut app);

    let captured = app.world().resource::<CapturedKillYourselfBolt>();
    assert_eq!(
        captured.0.len(),
        2,
        "two extra bolts should produce two KillYourself<Bolt> messages"
    );

    // Order is not asserted — use a HashSet<Entity> comparison.
    let victims: HashSet<Entity> = captured.0.iter().map(|m| m.victim).collect();
    assert!(
        victims.contains(&entity_a),
        "captured victims should contain entity_a"
    );
    assert!(
        victims.contains(&entity_b),
        "captured victims should contain entity_b"
    );
    for msg in &captured.0 {
        assert_eq!(
            msg.killer, None,
            "out-of-bounds extra bolts should never attribute a killer"
        );
    }
}

// ── Behavior 1b: BoltLost AND KillYourself<Bolt> in same tick, same entity ──

#[test]
fn extra_bolt_writes_bolt_lost_and_kill_yourself_same_tick_same_entity() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayfieldConfig>()
        .init_resource::<GameRng>()
        .add_message::<BoltLost>()
        .add_message::<KillYourself<Bolt>>()
        .init_resource::<CapturedBoltLost>()
        .init_resource::<CapturedKillYourselfBolt>()
        .add_systems(
            FixedUpdate,
            (bolt_lost, capture_bolt_lost, capture_kill_yourself_bolt).chain(),
        );

    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    let def = make_default_bolt_definition();
    let extra_entity = spawn_bolt_in_app(&mut app, |commands| {
        Bolt::builder()
            .at_position(Vec2::new(0.0, playfield.bottom() - 100.0))
            .definition(&def)
            .with_velocity(Velocity2D(Vec2::new(0.0, -400.0)))
            .extra()
            .headless()
            .spawn(commands)
    });
    tick(&mut app);

    // Joint assertion: BOTH messages fire in the SAME tick for the SAME entity.
    // Closes the regression hole where a buggy migration replaces BoltLost with
    // KillYourself<Bolt> instead of writing both.
    let captured_bolt_lost = app.world().resource::<CapturedBoltLost>();
    assert_eq!(
        captured_bolt_lost.0.len(),
        1,
        "exactly one BoltLost should be written"
    );
    assert_eq!(
        captured_bolt_lost.0[0].bolt, extra_entity,
        "BoltLost.bolt should reference the extra bolt entity"
    );

    let captured_kill_yourself = app.world().resource::<CapturedKillYourselfBolt>();
    assert_eq!(
        captured_kill_yourself.0.len(),
        1,
        "exactly one KillYourself<Bolt> should be written"
    );
    assert_eq!(
        captured_kill_yourself.0[0].victim, extra_entity,
        "KillYourself<Bolt>.victim should equal the SAME extra bolt entity"
    );
    assert_eq!(
        captured_kill_yourself.0[0].killer, None,
        "out-of-bounds extra bolt should not attribute a killer"
    );
}

// ── Behavior 3: Baseline bolt does NOT write KillYourself<Bolt> ──

#[test]
fn baseline_bolt_does_not_write_kill_yourself_bolt() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayfieldConfig>()
        .init_resource::<GameRng>()
        .add_message::<BoltLost>()
        .add_message::<KillYourself<Bolt>>()
        .init_resource::<CapturedKillYourselfBolt>()
        .add_systems(FixedUpdate, (bolt_lost, capture_kill_yourself_bolt).chain());

    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    // Baseline bolt (no ExtraBolt marker)
    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );

    tick(&mut app);

    let captured = app.world().resource::<CapturedKillYourselfBolt>();
    assert!(
        captured.0.is_empty(),
        "baseline bolt should NOT write KillYourself<Bolt> — it gets respawned"
    );

    // Edge case: after tick, exactly ONE bolt entity still exists and it
    // does NOT have ExtraBolt.
    let bolt_count = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .count();
    assert_eq!(bolt_count, 1, "exactly one bolt should remain after tick");
    let extra_count = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
        .iter(app.world())
        .count();
    assert_eq!(extra_count, 0, "the remaining bolt must not have ExtraBolt");
}

// ── Behavior 4: Baseline bolt still sends BoltLost for game-logic purposes ──

#[test]
fn baseline_bolt_still_sends_bolt_lost_message() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayfieldConfig>()
        .init_resource::<GameRng>()
        .add_message::<BoltLost>()
        // `bolt_lost` takes `MessageWriter<KillYourself<Bolt>>` as a plain
        // SystemParam, so the message type MUST be registered even though
        // this test does not assert on it.
        .add_message::<KillYourself<Bolt>>()
        .init_resource::<BoltLostCount>()
        .add_systems(FixedUpdate, (bolt_lost, count_bolt_lost).chain());

    let playfield = PlayfieldConfig::default();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -250.0)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));

    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );

    tick(&mut app);

    let count = app.world().resource::<BoltLostCount>();
    assert_eq!(
        count.0, 1,
        "baseline bolt should still send BoltLost for game-logic purposes"
    );
}

// ── Behavior 6: BoltLost entity fields for extra bolt ──

#[test]
fn extra_bolt_lost_sends_correct_bolt_and_breaker_entities() {
    let mut app = test_app();

    app.init_resource::<CapturedBoltLost>();
    app.add_systems(FixedUpdate, capture_bolt_lost.after(bolt_lost));

    let playfield = PlayfieldConfig::default();
    let breaker_entity = app
        .world_mut()
        .spawn((
            Breaker,
            Position2D(Vec2::new(0.0, -250.0)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ))
        .id();

    let def = make_default_bolt_definition();
    let extra_bolt_entity = spawn_bolt_in_app(&mut app, |commands| {
        Bolt::builder()
            .at_position(Vec2::new(0.0, playfield.bottom() - 100.0))
            .definition(&def)
            .with_velocity(Velocity2D(Vec2::new(0.0, -400.0)))
            .extra()
            .headless()
            .spawn(commands)
    });
    tick(&mut app);

    let captured = app.world().resource::<CapturedBoltLost>();
    assert_eq!(
        captured.0.len(),
        1,
        "exactly one BoltLost message should be captured for extra bolt"
    );
    assert_eq!(
        captured.0[0].bolt, extra_bolt_entity,
        "BoltLost.bolt should equal the extra bolt entity"
    );
    assert_eq!(
        captured.0[0].breaker, breaker_entity,
        "BoltLost.breaker should equal the breaker entity"
    );
}
