use bevy::prelude::*;

use super::helpers::*;
use crate::{
    breaker::{
        components::BumpState,
        definition::BreakerDefinition,
        messages::{BumpGrade, BumpWhiffed},
        resources::ForceBumpGrade,
        systems::bump::grade_bump,
    },
    prelude::*,
};

#[test]
fn bolt_hit_with_active_forward_perfect() {
    let mut app = grade_bump_test_app();
    let config = BreakerDefinition::default();

    let entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    app.world_mut().entity_mut(entity).insert(BumpState {
        active: true,
        timer: config.perfect_window * 0.5, // in the perfect zone
        ..Default::default()
    });

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt:    Entity::PLACEHOLDER,
        breaker: entity,
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

    let entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    app.world_mut().entity_mut(entity).insert(BumpState {
        active: true,
        timer: config.early_window + config.perfect_window, // just started
        ..Default::default()
    });

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt:    Entity::PLACEHOLDER,
        breaker: entity,
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

    let entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt:    Entity::PLACEHOLDER,
        breaker: entity,
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

    let entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);

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

    let entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    app.world_mut().entity_mut(entity).insert(BumpState {
        active: true,
        timer: config.perfect_window * 0.5, // in the perfect zone
        ..Default::default()
    });

    // Use a dedicated resource with the specific bolt entity
    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt:    bolt_entity,
        breaker: entity,
    })));
    app.add_systems(
        FixedUpdate,
        (
            enqueue_hit.before(crate::breaker::systems::bump::grade_bump),
            crate::breaker::systems::bump::grade_bump,
            capture_bumps.after(crate::breaker::systems::bump::grade_bump),
        ),
    );

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

    let bolt_entity = app.world_mut().spawn_empty().id();

    let breaker_entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt:    bolt_entity,
        breaker: breaker_entity,
    })));
    app.add_systems(
        FixedUpdate,
        (
            enqueue_hit.before(crate::breaker::systems::bump::grade_bump),
            crate::breaker::systems::bump::grade_bump,
            capture_bumps.after(crate::breaker::systems::bump::grade_bump),
        ),
    );

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

    let entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    app.world_mut().entity_mut(entity).insert(BumpState {
        active: true,
        timer: config.perfect_window * 0.5, // in the perfect zone — would normally grade Perfect
        ..Default::default()
    });

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt:    Entity::PLACEHOLDER,
        breaker: entity,
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

    let entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    app.world_mut().entity_mut(entity).insert(BumpState {
        active: true,
        timer: config.perfect_window * 0.5, // in the perfect zone
        ..Default::default()
    });

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt:    Entity::PLACEHOLDER,
        breaker: entity,
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

    let entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    app.world_mut().entity_mut(entity).insert(BumpState {
        active: true,
        timer: config.perfect_window * 0.5, // in the perfect zone
        ..Default::default()
    });

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt:    Entity::PLACEHOLDER,
        breaker: entity,
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

// ── Wave 1B: multi-breaker / iter_mut migration tests ─────────────────────

// Inline resource + system for multi-hit injection (Behavior 2 only).
// Do NOT modify helpers.rs — TestHitMessage/enqueue_hit is kept as-is for
// all existing single-hit tests.
#[derive(Resource, Default)]
struct TestHitMessages(Vec<BoltImpactBreaker>);

fn enqueue_hits(
    mut msg_res: ResMut<TestHitMessages>,
    mut writer: MessageWriter<BoltImpactBreaker>,
) {
    for msg in msg_res.0.drain(..) {
        writer.write(msg);
    }
}

fn grade_bump_multi_hit_test_app() -> App {
    TestAppBuilder::new()
        .with_message::<BoltImpactBreaker>()
        .with_message::<BumpPerformed>()
        .with_message::<BumpWhiffed>()
        .with_resource::<CapturedBumps>()
        .with_resource::<TestHitMessages>()
        .with_system(
            FixedUpdate,
            (
                enqueue_hits.before(grade_bump),
                grade_bump,
                capture_bumps.after(grade_bump),
            ),
        )
        .build()
}

fn grade_bump_with_whiff_capture_test_app() -> App {
    TestAppBuilder::new()
        .with_message::<BoltImpactBreaker>()
        .with_message::<BumpPerformed>()
        .with_message::<BumpWhiffed>()
        .with_resource::<CapturedBumps>()
        .with_resource::<CapturedWhiffs>()
        .insert_resource(TestHitMessage(None))
        .with_system(
            FixedUpdate,
            (
                enqueue_hit.before(grade_bump),
                grade_bump,
                capture_bumps.after(grade_bump),
                capture_whiffs.after(grade_bump),
            ),
        )
        .build()
}

// ── Behavior 1: two breakers present, only targeted one receives BumpPerformed ──

#[test]
fn grade_bump_does_not_panic_with_two_breakers_when_one_targeted() {
    let mut app = grade_bump_test_app();
    let config = BreakerDefinition::default();

    let real = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    let other_breaker = crate::breaker::test_utils::spawn_breaker(&mut app, 50.0, 0.0);

    app.world_mut().entity_mut(real).insert(BumpState {
        active: true,
        timer: config.perfect_window * 0.5, // in the perfect zone
        ..Default::default()
    });
    // other_breaker keeps default BumpState (active: false, timer: 0.0)

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt:    Entity::PLACEHOLDER,
        breaker: real,
    })));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1, "should emit exactly one BumpPerformed");
    assert_eq!(
        captured.0[0].grade,
        BumpGrade::Perfect,
        "targeted breaker should grade Perfect"
    );
    assert_eq!(
        captured.0[0].breaker, real,
        "BumpPerformed should attribute to the real breaker entity"
    );

    let real_state = app.world().get::<BumpState>(real).unwrap();
    assert!(
        !real_state.active,
        "real breaker BumpState should be deactivated after grading"
    );

    let other_state = app.world().get::<BumpState>(other_breaker).unwrap();
    assert!(
        !other_state.active,
        "other breaker BumpState should remain inactive"
    );
}

// ── Behavior 2: both breakers receive hits → two independent BumpPerformed ───

#[test]
fn grade_bump_emits_one_bump_performed_per_targeted_breaker() {
    let mut app = grade_bump_multi_hit_test_app();
    let config = BreakerDefinition::default();

    let a = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    let b = crate::breaker::test_utils::spawn_breaker(&mut app, 50.0, 0.0);

    for entity in [a, b] {
        app.world_mut().entity_mut(entity).insert(BumpState {
            active: true,
            timer: config.perfect_window * 0.5,
            ..Default::default()
        });
    }

    app.world_mut().resource_mut::<TestHitMessages>().0.extend([
        BoltImpactBreaker {
            bolt:    Entity::PLACEHOLDER,
            breaker: a,
        },
        BoltImpactBreaker {
            bolt:    Entity::PLACEHOLDER,
            breaker: b,
        },
    ]);
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 2, "should emit one BumpPerformed per hit");

    assert!(
        captured
            .0
            .iter()
            .any(|m| m.breaker == a && m.grade == BumpGrade::Perfect),
        "BumpPerformed for breaker A (Perfect) missing; captured: {:?}",
        captured.0
    );
    assert!(
        captured
            .0
            .iter()
            .any(|m| m.breaker == b && m.grade == BumpGrade::Perfect),
        "BumpPerformed for breaker B (Perfect) missing; captured: {:?}",
        captured.0
    );

    assert!(
        !app.world().get::<BumpState>(a).unwrap().active,
        "breaker A should be deactivated"
    );
    assert!(
        !app.world().get::<BumpState>(b).unwrap().active,
        "breaker B should be deactivated"
    );
}

// ── Behavior 3: hit targeting A leaves B's BumpState untouched ───────────────

#[test]
fn grade_bump_leaves_non_targeted_breaker_state_untouched() {
    let mut app = grade_bump_test_app();
    let config = BreakerDefinition::default();

    let a = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    let b = crate::breaker::test_utils::spawn_breaker(&mut app, 50.0, 0.0);

    let b_timer = config.perfect_window + config.early_window;

    app.world_mut().entity_mut(a).insert(BumpState {
        active: true,
        timer: config.perfect_window * 0.5,
        ..Default::default()
    });
    app.world_mut().entity_mut(b).insert(BumpState {
        active: true,
        timer: b_timer,
        ..Default::default()
    });

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker {
        bolt:    Entity::PLACEHOLDER,
        breaker: a,
    })));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert_eq!(captured.0.len(), 1, "only one BumpPerformed (for A)");
    assert_eq!(captured.0[0].breaker, a, "BumpPerformed should target A");
    assert_eq!(captured.0[0].grade, BumpGrade::Perfect);

    let a_state = app.world().get::<BumpState>(a).unwrap();
    assert!(!a_state.active, "A should be deactivated after grading");

    let b_state = app.world().get::<BumpState>(b).unwrap();
    assert!(
        b_state.active,
        "B should remain active (not hit this frame)"
    );
    assert!(
        (b_state.timer - b_timer).abs() < f32::EPSILON,
        "B's timer should be unchanged; expected {b_timer}, got {}",
        b_state.timer
    );
}

// ── Behavior 4: retroactive path is per-breaker ───────────────────────────────

#[test]
fn grade_bump_retroactive_path_is_per_breaker() {
    let mut app = grade_bump_test_app();
    let config = BreakerDefinition::default();

    let a = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    let b = crate::breaker::test_utils::spawn_breaker(&mut app, 50.0, 0.0);
    // Both start with BumpState::default() — active: false

    let bolt = app.world_mut().spawn_empty().id();

    app.insert_resource(TestHitMessage(Some(BoltImpactBreaker { bolt, breaker: b })));
    tick(&mut app);

    let captured = app.world().resource::<CapturedBumps>();
    assert!(
        captured.0.is_empty(),
        "no BumpPerformed when no bump was active"
    );

    let a_state = app.world().get::<BumpState>(a).unwrap();
    assert_eq!(
        a_state.last_hit_bolt, None,
        "A's last_hit_bolt should remain None (was not targeted)"
    );
    assert!(
        a_state.post_hit_timer <= 0.0,
        "A's post_hit_timer should remain 0 (was not targeted)"
    );

    let b_state = app.world().get::<BumpState>(b).unwrap();
    assert_eq!(
        b_state.last_hit_bolt,
        Some(bolt),
        "B's last_hit_bolt should be set to the bolt entity"
    );
    let expected_timer = config.perfect_window + config.late_window;
    assert!(
        (b_state.post_hit_timer - expected_timer).abs() < f32::EPSILON,
        "B's post_hit_timer should be perfect_window + late_window = {expected_timer}, got {}",
        b_state.post_hit_timer
    );
}

// ── Behavior 5: whiff is per-breaker under iter_mut ───────────────────────────

#[test]
fn grade_bump_whiff_is_per_breaker_under_iter_mut() {
    let mut app = grade_bump_with_whiff_capture_test_app();
    let config = BreakerDefinition::default();

    let a = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    let b = crate::breaker::test_utils::spawn_breaker(&mut app, 50.0, 0.0);

    // A: forward window expired (timer at 0, no hit incoming)
    app.world_mut().entity_mut(a).insert(BumpState {
        active: true,
        timer: 0.0,
        ..Default::default()
    });
    // B: still in window (timer > 0)
    let b_timer = config.perfect_window * 0.5;
    app.world_mut().entity_mut(b).insert(BumpState {
        active: true,
        timer: b_timer,
        ..Default::default()
    });

    // No hit message this frame
    tick(&mut app);

    let whiff_count = app.world().resource::<CapturedWhiffs>().0;
    assert_eq!(whiff_count, 1, "exactly one whiff should fire (for A only)");

    let a_state = app.world().get::<BumpState>(a).unwrap();
    assert!(!a_state.active, "A should be deactivated by whiff expiry");
    assert!(
        (a_state.timer - 0.0).abs() < f32::EPSILON,
        "A's timer should be 0.0 after whiff"
    );

    let b_state = app.world().get::<BumpState>(b).unwrap();
    assert!(
        b_state.active,
        "B should remain active (window not expired)"
    );
    assert!(
        (b_state.timer - b_timer).abs() < f32::EPSILON,
        "B's timer should be unchanged; expected {b_timer}, got {}",
        b_state.timer
    );

    let captured = app.world().resource::<CapturedBumps>();
    assert!(captured.0.is_empty(), "no BumpPerformed — no hit arrived");
}
