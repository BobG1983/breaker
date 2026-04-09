use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::messages::BoltImpactBreaker,
    breaker::{
        components::{Breaker, BumpState},
        definition::BreakerDefinition,
        messages::BumpGrade,
        resources::ForceBumpGrade,
    },
};

#[test]
fn bolt_hit_with_active_forward_perfect() {
    let mut app = grade_bump_test_app();
    let config = BreakerDefinition::default();

    let entity = {
        let world = app.world_mut();
        let entity = Breaker::builder()
            .definition(&config)
            .headless()
            .primary()
            .spawn(&mut world.commands());
        world.flush();
        entity
    };
    app.world_mut().entity_mut(entity).insert(BumpState {
        active: true,
        timer: config.perfect_window * 0.5, // in the perfect zone
        ..Default::default()
    });

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(!bump.active, "should deactivate");
    assert!(
        (bump.cooldown - config.perfect_bump_cooldown).abs() < f32::EPSILON,
        "perfect forward should set perfect_bump_cooldown ({}), got {}",
        config.perfect_bump_cooldown,
        bump.cooldown
    );
    assert!(bump.post_hit_timer <= 0.0, "should clear post_hit_timer");

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1);
    assert_eq!(captured.0[0].grade, BumpGrade::Perfect);
}

#[test]
fn bolt_hit_with_active_forward_early() {
    let mut app = grade_bump_test_app();
    let config = BreakerDefinition::default();

    let entity = {
        let world = app.world_mut();
        let entity = Breaker::builder()
            .definition(&config)
            .headless()
            .primary()
            .spawn(&mut world.commands());
        world.flush();
        entity
    };
    app.world_mut().entity_mut(entity).insert(BumpState {
        active: true,
        timer: config.early_window + config.perfect_window, // just started
        ..Default::default()
    });

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(
        (bump.cooldown - config.weak_bump_cooldown).abs() < f32::EPSILON,
        "early forward should set weak_bump_cooldown ({}), got {}",
        config.weak_bump_cooldown,
        bump.cooldown
    );

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1);
    assert_eq!(captured.0[0].grade, BumpGrade::Early);
}

#[test]
fn bolt_hit_without_active_sets_post_hit_timer_no_message() {
    let mut app = grade_bump_test_app();
    let config = BreakerDefinition::default();

    let entity = {
        let world = app.world_mut();
        let entity = Breaker::builder()
            .definition(&config)
            .headless()
            .primary()
            .spawn(&mut world.commands());
        world.flush();
        entity
    };

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    let expected = config.perfect_window + config.late_window;
    assert!(
        (bump.post_hit_timer - expected).abs() < f32::EPSILON,
        "post_hit_timer should be set to perfect + late window, got {}",
        bump.post_hit_timer
    );

    let captured = app.world().resource::<CapturedBumps>();
    assert!(captured.0.is_empty(), "no message when bump not active");
}

#[test]
fn no_hit_no_change() {
    let mut app = grade_bump_test_app();
    let config = BreakerDefinition::default();

    let entity = {
        let world = app.world_mut();
        let entity = Breaker::builder()
            .definition(&config)
            .headless()
            .primary()
            .spawn(&mut world.commands());
        world.flush();
        entity
    };

    // No hit message
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(!bump.active);
    assert!(bump.post_hit_timer <= 0.0);

    let captured = app.world().resource::<CapturedBumps>();
    assert!(captured.0.is_empty());
}

// ── Bolt entity threading tests ──────────────────────────────────

#[test]
fn grade_bump_forward_sends_bolt_entity() {
    // Given: forward bump active, BoltImpactBreaker arrives with a specific bolt entity
    // When: grade_bump runs
    // Then: BumpPerformed.bolt matches the bolt from BoltImpactBreaker
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BoltImpactBreaker>()
        .add_message::<crate::breaker::messages::BumpPerformed>()
        .add_message::<crate::breaker::messages::BumpWhiffed>()
        .init_resource::<CapturedBumps>();

    let config = BreakerDefinition::default();

    // Spawn a bolt entity to reference
    let bolt_entity = app.world_mut().spawn_empty().id();

    // Use a dedicated resource with the specific bolt entity
    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: bolt_entity,
        breaker: Entity::PLACEHOLDER,
    })));
    app.add_systems(
        FixedUpdate,
        (
            enqueue_hit.before(crate::breaker::systems::bump::grade_bump),
            crate::breaker::systems::bump::grade_bump,
            capture_bumps.after(crate::breaker::systems::bump::grade_bump),
        ),
    );

    {
        let entity = {
            let world = app.world_mut();
            let entity = Breaker::builder()
                .definition(&config)
                .headless()
                .primary()
                .spawn(&mut world.commands());
            world.flush();
            entity
        };
        app.world_mut().entity_mut(entity).insert(BumpState {
            active: true,
            timer: config.perfect_window * 0.5, // in the perfect zone
            ..Default::default()
        });
    }

    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1, "should emit one BumpPerformed");
    assert_eq!(
        captured.0[0].bolt,
        Some(bolt_entity),
        "BumpPerformed.bolt should match the bolt entity from BoltImpactBreaker"
    );
}

#[test]
fn grade_bump_sets_last_hit_bolt_when_no_active_bump() {
    // Given: no active forward bump, BoltImpactBreaker arrives with a specific bolt entity
    // When: grade_bump runs
    // Then: BumpState.last_hit_bolt == Some(bolt_entity)
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BoltImpactBreaker>()
        .add_message::<crate::breaker::messages::BumpPerformed>()
        .add_message::<crate::breaker::messages::BumpWhiffed>()
        .init_resource::<CapturedBumps>();

    let config = BreakerDefinition::default();

    let bolt_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: bolt_entity,
        breaker: Entity::PLACEHOLDER,
    })));
    app.add_systems(
        FixedUpdate,
        (
            enqueue_hit.before(crate::breaker::systems::bump::grade_bump),
            crate::breaker::systems::bump::grade_bump,
            capture_bumps.after(crate::breaker::systems::bump::grade_bump),
        ),
    );

    let breaker_entity = {
        let world = app.world_mut();
        let entity = Breaker::builder()
            .definition(&config)
            .headless()
            .primary()
            .spawn(&mut world.commands());
        world.flush();
        entity
    };

    tick(&mut app);

    let bump = app.world().get::<BumpState>(breaker_entity).unwrap();
    assert_eq!(
        bump.last_hit_bolt,
        Some(bolt_entity),
        "BumpState.last_hit_bolt should be set to the bolt entity when no active bump"
    );
}

// ── ForceBumpGrade override tests ─────────────────────────────

#[test]
fn grade_bump_uses_force_grade_when_some() {
    // Given: ForceBumpGrade(Some(Late)), forward bump active in perfect zone
    // When: grade_bump runs with a BoltImpactBreaker
    // Then: BumpPerformed.grade should be Late (overridden), not Perfect (calculated)
    let mut app = grade_bump_test_app();
    let config = BreakerDefinition::default();

    app.insert_resource(ForceBumpGrade(Some(BumpGrade::Late)));

    {
        let entity = {
            let world = app.world_mut();
            let entity = Breaker::builder()
                .definition(&config)
                .headless()
                .primary()
                .spawn(&mut world.commands());
            world.flush();
            entity
        };
        app.world_mut().entity_mut(entity).insert(BumpState {
            active: true,
            timer: config.perfect_window * 0.5, // in the perfect zone — would normally grade Perfect
            ..Default::default()
        });
    }

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1, "should emit one BumpPerformed");
    assert_eq!(
        captured.0[0].grade,
        BumpGrade::Late,
        "grade_bump should use ForceBumpGrade override (Late), not calculated grade (Perfect)"
    );
}

#[test]
fn grade_bump_ignores_force_grade_when_none() {
    // Given: ForceBumpGrade(None), forward bump active in perfect zone
    // When: grade_bump runs with a BoltImpactBreaker
    // Then: BumpPerformed.grade should be Perfect (normal calculation)
    let mut app = grade_bump_test_app();
    let config = BreakerDefinition::default();

    app.insert_resource(ForceBumpGrade(None));

    {
        let entity = {
            let world = app.world_mut();
            let entity = Breaker::builder()
                .definition(&config)
                .headless()
                .primary()
                .spawn(&mut world.commands());
            world.flush();
            entity
        };
        app.world_mut().entity_mut(entity).insert(BumpState {
            active: true,
            timer: config.perfect_window * 0.5, // in the perfect zone
            ..Default::default()
        });
    }

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1, "should emit one BumpPerformed");
    assert_eq!(
        captured.0[0].grade,
        BumpGrade::Perfect,
        "grade_bump should use normal grading when ForceBumpGrade is None"
    );
}

#[test]
fn grade_bump_works_without_force_grade_resource() {
    // Given: no ForceBumpGrade resource inserted, forward bump active in perfect zone
    // When: grade_bump runs with a BoltImpactBreaker
    // Then: BumpPerformed.grade should be Perfect (backward compatible)
    let mut app = grade_bump_test_app();
    let config = BreakerDefinition::default();

    // Intentionally do NOT insert ForceBumpGrade resource

    {
        let entity = {
            let world = app.world_mut();
            let entity = Breaker::builder()
                .definition(&config)
                .headless()
                .primary()
                .spawn(&mut world.commands());
            world.flush();
            entity
        };
        app.world_mut().entity_mut(entity).insert(BumpState {
            active: true,
            timer: config.perfect_window * 0.5, // in the perfect zone
            ..Default::default()
        });
    }

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt: Entity::PLACEHOLDER,
        breaker: Entity::PLACEHOLDER,
    })));

    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1, "should emit one BumpPerformed");
    assert_eq!(
        captured.0[0].grade,
        BumpGrade::Perfect,
        "grade_bump should work normally when ForceBumpGrade resource is absent"
    );
}
