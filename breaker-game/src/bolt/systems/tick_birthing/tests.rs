use bevy::prelude::*;

use super::system::tick_birthing;
use crate::{
    prelude::*,
    shared::{BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, WALL_LAYER, birthing::BIRTHING_DURATION},
};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(FixedUpdate, tick_birthing);
    app
}

use crate::shared::test_utils::tick;

// Behavior 3: tick_birthing lerps scale from zero toward target_scale each tick
#[test]
fn scale_increases_from_zero_after_one_tick() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Scale2D { x: 0.0, y: 0.0 },
            CollisionLayers::default(),
            Birthing {
                timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
                target_scale: Scale2D { x: 8.0, y: 8.0 },
                stashed_layers: CollisionLayers::new(
                    BOLT_LAYER,
                    CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
                ),
            },
        ))
        .id();

    tick(&mut app);

    // After 1 tick at 1/64s: fraction ≈ 0.1042, ease-out t*(2-t) ≈ 0.1975
    // Scale should be approximately 8.0 * 0.1975 ≈ 1.58
    let scale = app
        .world()
        .get::<Scale2D>(entity)
        .expect("entity should exist");
    assert!(
        scale.x > 0.0,
        "Scale2D.x should increase from 0.0 after one tick, got {}",
        scale.x
    );
    assert!(
        scale.x > 1.0 && scale.x < 2.5,
        "Scale2D.x should be in ease-out range after one tick, got {}",
        scale.x
    );
    assert!(
        (scale.x - scale.y).abs() < f32::EPSILON,
        "Scale2D.x and y should match for square target, got ({}, {})",
        scale.x,
        scale.y
    );

    // Birthing should still be present
    assert!(
        app.world().get::<Birthing>(entity).is_some(),
        "Birthing should still be present after 1 tick"
    );

    // CollisionLayers should still be zeroed
    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("entity should have CollisionLayers");
    assert_eq!(
        layers.membership, 0,
        "CollisionLayers membership should still be 0 during birthing"
    );
    assert_eq!(
        layers.mask, 0,
        "CollisionLayers mask should still be 0 during birthing"
    );
}

// Behavior 4: tick_birthing completes after full duration -- restores exact target_scale
#[test]
fn scale_reaches_exact_target_after_full_duration() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Scale2D { x: 0.0, y: 0.0 },
            CollisionLayers::default(),
            Birthing {
                timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
                target_scale: Scale2D { x: 8.0, y: 8.0 },
                stashed_layers: CollisionLayers::new(
                    BOLT_LAYER,
                    CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
                ),
            },
        ))
        .id();

    // 10 ticks at 1/64s = 0.15625s > 0.15s -- timer should complete
    for _ in 0..10 {
        tick(&mut app);
    }

    let scale = app
        .world()
        .get::<Scale2D>(entity)
        .expect("entity should exist");
    assert!(
        (scale.x - 8.0).abs() < f32::EPSILON,
        "Scale2D.x should be exactly 8.0 after completion, got {}",
        scale.x
    );
    assert!(
        (scale.y - 8.0).abs() < f32::EPSILON,
        "Scale2D.y should be exactly 8.0 after completion, got {}",
        scale.y
    );

    // CollisionLayers should be restored
    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("entity should have CollisionLayers");
    assert_eq!(
        layers.membership, BOLT_LAYER,
        "CollisionLayers membership should be restored to BOLT_LAYER"
    );
    assert_eq!(
        layers.mask,
        CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
        "CollisionLayers mask should be restored"
    );

    // Birthing should be removed
    assert!(
        app.world().get::<Birthing>(entity).is_none(),
        "Birthing should be removed after completion"
    );
}

// Behavior 5: tick_birthing restores stashed_layers on completion
#[test]
fn restores_stashed_layers_on_completion() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Scale2D { x: 0.0, y: 0.0 },
            CollisionLayers::default(),
            Birthing {
                timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
                target_scale: Scale2D { x: 8.0, y: 8.0 },
                stashed_layers: CollisionLayers::new(
                    BOLT_LAYER,
                    CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
                ),
            },
        ))
        .id();

    for _ in 0..20 {
        tick(&mut app);
    }

    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("entity should have CollisionLayers");
    assert_eq!(
        layers.membership, BOLT_LAYER,
        "stashed membership should be restored exactly"
    );
    assert_eq!(
        layers.mask,
        CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
        "stashed mask should be restored exactly"
    );
}

// Edge case: Stashed layers with only membership set (mask = 0)
#[test]
fn restores_stashed_layers_with_zero_mask() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Scale2D { x: 0.0, y: 0.0 },
            CollisionLayers::default(),
            Birthing {
                timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
                target_scale: Scale2D { x: 8.0, y: 8.0 },
                stashed_layers: CollisionLayers::new(0x01, 0x00),
            },
        ))
        .id();

    for _ in 0..20 {
        tick(&mut app);
    }

    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("entity should have CollisionLayers");
    assert_eq!(
        layers.membership, 0x01,
        "stashed membership 0x01 should be restored as-is"
    );
    assert_eq!(
        layers.mask, 0x00,
        "stashed mask 0x00 should be restored as-is"
    );
}

// Behavior 6: tick_birthing handles non-square target_scale
#[test]
fn handles_non_square_target_scale() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Scale2D { x: 0.0, y: 0.0 },
            CollisionLayers::default(),
            Birthing {
                timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
                target_scale: Scale2D { x: 12.0, y: 6.0 },
                stashed_layers: CollisionLayers::new(0x01, 0x0E),
            },
        ))
        .id();

    for _ in 0..20 {
        tick(&mut app);
    }

    let scale = app
        .world()
        .get::<Scale2D>(entity)
        .expect("entity should exist");
    assert!(
        (scale.x - 12.0).abs() < f32::EPSILON,
        "Scale2D.x should be exactly 12.0 for non-square target, got {}",
        scale.x
    );
    assert!(
        (scale.y - 6.0).abs() < f32::EPSILON,
        "Scale2D.y should be exactly 6.0 for non-square target, got {}",
        scale.y
    );
}

// Edge case: One axis zero target
#[test]
fn handles_zero_y_target_scale() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Scale2D { x: 0.0, y: 0.0 },
            CollisionLayers::default(),
            Birthing {
                timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
                target_scale: Scale2D { x: 8.0, y: 0.0 },
                stashed_layers: CollisionLayers::new(0x01, 0x0E),
            },
        ))
        .id();

    for _ in 0..20 {
        tick(&mut app);
    }

    let scale = app
        .world()
        .get::<Scale2D>(entity)
        .expect("entity should exist");
    assert!(
        (scale.x - 8.0).abs() < f32::EPSILON,
        "Scale2D.x should be 8.0, got {}",
        scale.x
    );
    assert!(
        scale.y.abs() < f32::EPSILON,
        "Scale2D.y should remain 0.0 throughout, got {}",
        scale.y
    );
}

// Edge case: Non-default CollisionLayers stay zeroed during birthing
#[test]
fn non_default_layers_stay_zeroed_during_birthing() {
    let mut app = test_app();

    // Entity starts with non-zero CollisionLayers that were stashed by begin_node_birthing.
    // During birthing, the entity's live layers should be zeroed regardless.
    let entity = app
        .world_mut()
        .spawn((
            Scale2D { x: 0.0, y: 0.0 },
            CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
            Birthing {
                timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
                target_scale: Scale2D { x: 8.0, y: 8.0 },
                stashed_layers: CollisionLayers::new(
                    BOLT_LAYER,
                    CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
                ),
            },
        ))
        .id();

    // After 1 tick, layers should still be whatever they were — tick_birthing
    // only restores layers on completion, it doesn't zero them mid-animation.
    // (begin_node_birthing is responsible for zeroing at insertion time.)
    tick(&mut app);

    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("entity should have CollisionLayers");
    // The entity started with non-zero layers. tick_birthing doesn't touch layers
    // until completion — it's begin_node_birthing / .birthed() that zeros them.
    // This test documents that tick_birthing does NOT zero layers mid-animation.
    assert_eq!(
        layers.membership, BOLT_LAYER,
        "tick_birthing should not zero layers mid-animation (that's the builder's job)"
    );

    // After completion, stashed layers are restored
    for _ in 0..10 {
        tick(&mut app);
    }

    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("entity should have CollisionLayers");
    assert_eq!(
        layers.membership, BOLT_LAYER,
        "stashed layers should be restored on completion"
    );
    assert_eq!(
        layers.mask,
        CELL_LAYER | WALL_LAYER | BREAKER_LAYER,
        "stashed mask should be restored on completion"
    );
}

// Behavior 7: tick_birthing does not affect entities without Birthing
#[test]
fn does_not_affect_entities_without_birthing() {
    let mut app = test_app();

    // Entity A: has Birthing
    app.world_mut().spawn((
        Scale2D { x: 0.0, y: 0.0 },
        CollisionLayers::default(),
        Birthing {
            timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
            target_scale: Scale2D { x: 8.0, y: 8.0 },
            stashed_layers: CollisionLayers::new(0x01, 0x0E),
        },
    ));

    // Entity B: no Birthing -- should be unaffected
    let entity_b = app
        .world_mut()
        .spawn((Scale2D { x: 8.0, y: 8.0 }, CollisionLayers::new(0x01, 0x0E)))
        .id();

    tick(&mut app);

    let scale_b = app
        .world()
        .get::<Scale2D>(entity_b)
        .expect("entity B should exist");
    assert!(
        (scale_b.x - 8.0).abs() < f32::EPSILON,
        "Entity B Scale2D.x should remain 8.0, got {}",
        scale_b.x
    );
    assert!(
        (scale_b.y - 8.0).abs() < f32::EPSILON,
        "Entity B Scale2D.y should remain 8.0, got {}",
        scale_b.y
    );

    let layers_b = app
        .world()
        .get::<CollisionLayers>(entity_b)
        .expect("entity B should have CollisionLayers");
    assert_eq!(
        layers_b.membership, 0x01,
        "Entity B CollisionLayers membership should remain 0x01"
    );
    assert_eq!(
        layers_b.mask, 0x0E,
        "Entity B CollisionLayers mask should remain 0x0E"
    );
}

// Behavior 8: tick_birthing handles multiple birthing entities simultaneously
#[test]
fn handles_multiple_birthing_entities() {
    let mut app = test_app();

    let entity_a = app
        .world_mut()
        .spawn((
            Scale2D { x: 0.0, y: 0.0 },
            CollisionLayers::default(),
            Birthing {
                timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
                target_scale: Scale2D { x: 8.0, y: 8.0 },
                stashed_layers: CollisionLayers::new(0x01, 0x0E),
            },
        ))
        .id();

    let entity_b = app
        .world_mut()
        .spawn((
            Scale2D { x: 0.0, y: 0.0 },
            CollisionLayers::default(),
            Birthing {
                timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
                target_scale: Scale2D { x: 16.0, y: 16.0 },
                stashed_layers: CollisionLayers::new(0x02, 0x0D),
            },
        ))
        .id();

    for _ in 0..20 {
        tick(&mut app);
    }

    // Entity A
    let scale_a = app
        .world()
        .get::<Scale2D>(entity_a)
        .expect("entity A exists");
    assert!(
        (scale_a.x - 8.0).abs() < f32::EPSILON,
        "Entity A Scale2D.x should be 8.0, got {}",
        scale_a.x
    );
    assert!(
        (scale_a.y - 8.0).abs() < f32::EPSILON,
        "Entity A Scale2D.y should be 8.0, got {}",
        scale_a.y
    );
    let layers_a = app
        .world()
        .get::<CollisionLayers>(entity_a)
        .expect("entity A layers");
    assert_eq!(layers_a.membership, 0x01);
    assert_eq!(layers_a.mask, 0x0E);
    assert!(
        app.world().get::<Birthing>(entity_a).is_none(),
        "Entity A Birthing should be removed"
    );

    // Entity B
    let scale_b = app
        .world()
        .get::<Scale2D>(entity_b)
        .expect("entity B exists");
    assert!(
        (scale_b.x - 16.0).abs() < f32::EPSILON,
        "Entity B Scale2D.x should be 16.0, got {}",
        scale_b.x
    );
    assert!(
        (scale_b.y - 16.0).abs() < f32::EPSILON,
        "Entity B Scale2D.y should be 16.0, got {}",
        scale_b.y
    );
    let layers_b = app
        .world()
        .get::<CollisionLayers>(entity_b)
        .expect("entity B layers");
    assert_eq!(layers_b.membership, 0x02);
    assert_eq!(layers_b.mask, 0x0D);
    assert!(
        app.world().get::<Birthing>(entity_b).is_none(),
        "Entity B Birthing should be removed"
    );
}

// Behavior 9: tick_birthing scale lerp is linear (fraction-based)
#[test]
fn scale_lerp_is_linear_at_midpoint() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Scale2D { x: 0.0, y: 0.0 },
            CollisionLayers::default(),
            Birthing {
                timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
                target_scale: Scale2D { x: 10.0, y: 10.0 },
                stashed_layers: CollisionLayers::new(0x01, 0x0E),
            },
        ))
        .id();

    // Approximately 5 ticks at 1/64s to reach ~50% of 0.15s
    for _ in 0..5 {
        tick(&mut app);
    }

    let scale = app.world().get::<Scale2D>(entity).expect("entity exists");
    // At ~50% fraction, ease-out t*(2-t) = 0.5*1.5 = 0.75, scale ≈ 10.0*0.75 = 7.5
    assert!(
        scale.x > 6.0 && scale.x < 9.0,
        "Scale2D.x should be in ease-out midpoint range (~7.5), got {}",
        scale.x
    );
    assert!(
        (scale.x - scale.y).abs() < f32::EPSILON,
        "Scale2D.x and y should match for square target, got ({}, {})",
        scale.x,
        scale.y
    );
}

// Behavior 27: Full birthing lifecycle -- spawn with .birthed() -> tick_birthing -> completion
#[test]
fn full_birthing_lifecycle_from_builder() {
    use bevy::ecs::world::CommandQueue;
    use rantzsoft_spatial2d::components::Velocity2D;

    use crate::bolt::{components::Bolt as BoltMarker, definition::BoltDefinition};

    let mut app = test_app();

    let def = BoltDefinition {
        name: "Bolt".to_string(),
        base_speed: 400.0,
        min_speed: 200.0,
        max_speed: 800.0,
        radius: 8.0,
        base_damage: 10.0,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    };

    // Spawn a bolt with .birthed() through the builder
    let entity = {
        let mut queue = CommandQueue::default();
        let entity = {
            let mut commands = Commands::new(&mut queue, app.world_mut());
            BoltMarker::builder()
                .definition(&def)
                .at_position(Vec2::ZERO)
                .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
                .extra()
                .birthed()
                .headless()
                .spawn(&mut commands)
        };
        queue.apply(app.world_mut());
        entity
    };

    // Before ticking: should have Birthing, zeroed scale, zeroed layers
    assert!(
        app.world().get::<Birthing>(entity).is_some(),
        "Bolt should have Birthing after .birthed().spawn()"
    );
    let scale = app
        .world()
        .get::<Scale2D>(entity)
        .expect("entity should have Scale2D");
    assert!(
        scale.x.abs() < f32::EPSILON && scale.y.abs() < f32::EPSILON,
        "Scale should be zeroed before ticking"
    );
    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("entity should have CollisionLayers");
    assert_eq!(
        *layers,
        CollisionLayers::default(),
        "CollisionLayers should be zeroed before ticking"
    );

    // Tick enough to complete birthing (10+ ticks)
    for _ in 0..12 {
        tick(&mut app);
    }

    // After completion: scale restored, layers restored, Birthing removed
    let scale = app
        .world()
        .get::<Scale2D>(entity)
        .expect("entity should have Scale2D");
    assert!(
        (scale.x - 8.0).abs() < f32::EPSILON,
        "Scale2D.x should be 8.0 after completion, got {}",
        scale.x
    );
    assert!(
        (scale.y - 8.0).abs() < f32::EPSILON,
        "Scale2D.y should be 8.0 after completion, got {}",
        scale.y
    );

    let layers = app
        .world()
        .get::<CollisionLayers>(entity)
        .expect("entity should have CollisionLayers");
    assert_eq!(
        *layers,
        CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
        "CollisionLayers should be restored after completion"
    );

    assert!(
        app.world().get::<Birthing>(entity).is_none(),
        "Birthing should be removed after completion"
    );
}
