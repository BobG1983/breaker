use bevy::{math::curve::easing::EaseFunction, prelude::*};
use rantzsoft_spatial2d::components::MaxSpeed;

use super::{
    super::core::*,
    helpers::{custom_breaker_definition, test_breaker_definition},
};
use crate::{
    breaker::components::{
        BrakeDecel, BrakeTilt, BreakerAcceleration, BreakerBaseY, BreakerDeceleration,
        BreakerReflectionSpread, BumpEarlyWindow, BumpFeedback, BumpLateWindow,
        BumpPerfectCooldown, BumpPerfectWindow, BumpWeakCooldown, DashDuration,
        DashSpeedMultiplier, DashTilt, DashTiltEase, DecelEasing, SettleDuration, SettleTiltEase,
    },
    prelude::*,
    shared::{
        BaseHeight, BaseWidth,
        size::{MaxHeight, MaxWidth, MinHeight, MinWidth},
    },
};

// ── Behavior 11: .definition() transitions D+Mv+Da+Sp+Bm at once ──

#[test]
fn definition_transitions_five_dimensions_at_once() {
    let def = test_breaker_definition();
    let _builder: BreakerBuilder<
        HasDimensions,
        HasMovement,
        HasDashing,
        HasSpread,
        HasBump,
        Unvisual,
        NoRole,
    > = Breaker::builder().definition(&def);
}

#[test]
fn definition_does_not_transition_visual_or_role() {
    let def = test_breaker_definition();
    // V remains Unvisual, R remains NoRole
    let _builder: BreakerBuilder<
        HasDimensions,
        HasMovement,
        HasDashing,
        HasSpread,
        HasBump,
        Unvisual,
        NoRole,
    > = Breaker::builder().definition(&def);

    // If this compiles, V and R are unchanged.
}

// ── Behavior 12: .definition() stores correct dimension values ──

#[test]
fn definition_stores_correct_dimension_values() {
    let def = custom_breaker_definition(); // width: 150.0, height: 25.0, y_position: -300.0
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let bw = world.get::<BaseWidth>(entity);
    assert!(bw.is_some(), "entity should have BaseWidth");
    assert!(
        (bw.unwrap().0 - 150.0).abs() < f32::EPSILON,
        "BaseWidth should be 150.0"
    );

    let bh = world.get::<BaseHeight>(entity);
    assert!(bh.is_some(), "entity should have BaseHeight");
    assert!(
        (bh.unwrap().0 - 25.0).abs() < f32::EPSILON,
        "BaseHeight should be 25.0"
    );

    let by = world.get::<BreakerBaseY>(entity);
    assert!(by.is_some(), "entity should have BreakerBaseY");
    assert!(
        (by.unwrap().0 - (-300.0)).abs() < f32::EPSILON,
        "BreakerBaseY should be -300.0"
    );
}

// ── Behavior 13: .definition() stores correct movement values ──

#[test]
fn definition_stores_correct_movement_values() {
    let def = custom_breaker_definition();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let ms = world.get::<MaxSpeed>(entity);
    assert!(ms.is_some(), "entity should have MaxSpeed");
    assert!(
        (ms.unwrap().0 - 600.0).abs() < f32::EPSILON,
        "MaxSpeed should be 600.0"
    );

    let accel = world.get::<BreakerAcceleration>(entity);
    assert!(accel.is_some(), "entity should have BreakerAcceleration");
    assert!(
        (accel.unwrap().0 - 4000.0).abs() < f32::EPSILON,
        "BreakerAcceleration should be 4000.0"
    );

    let decel = world.get::<BreakerDeceleration>(entity);
    assert!(decel.is_some(), "entity should have BreakerDeceleration");
    assert!(
        (decel.unwrap().0 - 3000.0).abs() < f32::EPSILON,
        "BreakerDeceleration should be 3000.0"
    );

    let de = world.get::<DecelEasing>(entity);
    assert!(de.is_some(), "entity should have DecelEasing");
    let de = de.unwrap();
    assert!(
        matches!(de.ease, EaseFunction::CubicIn),
        "DecelEasing.ease should be CubicIn"
    );
    assert!(
        (de.strength - 2.0).abs() < f32::EPSILON,
        "DecelEasing.strength should be 2.0"
    );
}

// ── Behavior 14: .definition() stores correct dashing values ──

#[test]
fn definition_stores_correct_dashing_values() {
    let def = custom_breaker_definition();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let dsm = world.get::<DashSpeedMultiplier>(entity);
    assert!(dsm.is_some(), "entity should have DashSpeedMultiplier");
    assert!((dsm.unwrap().0 - 3.0).abs() < f32::EPSILON);

    let dd = world.get::<DashDuration>(entity);
    assert!(dd.is_some(), "entity should have DashDuration");
    assert!((dd.unwrap().0 - 0.2).abs() < f32::EPSILON);

    let dt = world.get::<DashTilt>(entity);
    assert!(dt.is_some(), "entity should have DashTilt");
    assert!(
        (dt.unwrap().0 - 20.0_f32.to_radians()).abs() < 1e-5,
        "DashTilt should be 20 degrees in radians"
    );

    let dte = world.get::<DashTiltEase>(entity);
    assert!(dte.is_some(), "entity should have DashTiltEase");
    assert!(matches!(dte.unwrap().0, EaseFunction::CubicInOut));

    let bt = world.get::<BrakeTilt>(entity);
    assert!(bt.is_some(), "entity should have BrakeTilt");
    let bt = bt.unwrap();
    assert!(
        (bt.angle - 30.0_f32.to_radians()).abs() < 1e-5,
        "BrakeTilt.angle should be 30 degrees in radians"
    );
    assert!((bt.duration - 0.3).abs() < f32::EPSILON);
    assert!(matches!(bt.ease, EaseFunction::QuadraticIn));

    let bd = world.get::<BrakeDecel>(entity);
    assert!(bd.is_some(), "entity should have BrakeDecel");
    assert!((bd.unwrap().0 - 3.0).abs() < f32::EPSILON);

    let sd = world.get::<SettleDuration>(entity);
    assert!(sd.is_some(), "entity should have SettleDuration");
    assert!((sd.unwrap().0 - 0.3).abs() < f32::EPSILON);

    let ste = world.get::<SettleTiltEase>(entity);
    assert!(ste.is_some(), "entity should have SettleTiltEase");
    assert!(matches!(ste.unwrap().0, EaseFunction::QuadraticOut));
}

// ── Behavior 15: .definition() stores correct spread value (degrees to radians) ──

#[test]
fn definition_stores_correct_spread_value() {
    let def = test_breaker_definition(); // reflection_spread: 75.0
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let spread = world.get::<BreakerReflectionSpread>(entity);
    assert!(
        spread.is_some(),
        "entity should have BreakerReflectionSpread"
    );
    assert!((spread.unwrap().0 - 75.0_f32.to_radians()).abs() < 1e-5);
}

#[test]
fn definition_spread_zero_produces_zero_radians() {
    let mut def = test_breaker_definition();
    def.reflection_spread = 0.0;
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let spread = world.get::<BreakerReflectionSpread>(entity);
    assert!(
        spread.is_some(),
        "entity should have BreakerReflectionSpread"
    );
    assert!((spread.unwrap().0 - 0.0).abs() < f32::EPSILON);
}

// ── Behavior 16: .definition() stores correct bump values ──

#[test]
fn definition_stores_correct_bump_values() {
    let def = custom_breaker_definition();
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let pw = world.get::<BumpPerfectWindow>(entity);
    assert!(pw.is_some(), "entity should have BumpPerfectWindow");
    assert!((pw.unwrap().0 - 0.2).abs() < f32::EPSILON);

    let ew = world.get::<BumpEarlyWindow>(entity);
    assert!(ew.is_some(), "entity should have BumpEarlyWindow");
    assert!((ew.unwrap().0 - 0.1).abs() < f32::EPSILON);

    let lw = world.get::<BumpLateWindow>(entity);
    assert!(lw.is_some(), "entity should have BumpLateWindow");
    assert!((lw.unwrap().0 - 0.1).abs() < f32::EPSILON);

    let pc = world.get::<BumpPerfectCooldown>(entity);
    assert!(pc.is_some(), "entity should have BumpPerfectCooldown");
    assert!((pc.unwrap().0 - 0.05).abs() < f32::EPSILON);

    let wc = world.get::<BumpWeakCooldown>(entity);
    assert!(wc.is_some(), "entity should have BumpWeakCooldown");
    assert!((wc.unwrap().0 - 0.2).abs() < f32::EPSILON);

    let bf = world.get::<BumpFeedback>(entity);
    assert!(bf.is_some(), "entity should have BumpFeedback");
    let bf = bf.unwrap();
    assert!((bf.duration - 0.2).abs() < f32::EPSILON);
    assert!((bf.peak - 30.0).abs() < f32::EPSILON);
    assert!((bf.peak_fraction - 0.4).abs() < f32::EPSILON);
    assert!(matches!(bf.rise_ease, EaseFunction::CubicOut));
    assert!(matches!(bf.fall_ease, EaseFunction::QuadraticIn));
}

// ── Behavior 17: .definition() computes min/max size defaults when None ──

#[test]
fn definition_computes_min_max_from_base_when_none() {
    let def = test_breaker_definition(); // width: 120.0, height: 20.0, min_w/max_w/min_h/max_h: None
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let min_w = world.get::<MinWidth>(entity);
    assert!(min_w.is_some(), "entity should have MinWidth");
    assert!(
        (min_w.unwrap().0 - 60.0).abs() < f32::EPSILON,
        "MinWidth should be 0.5 * 120.0 = 60.0"
    );

    let max_w = world.get::<MaxWidth>(entity);
    assert!(max_w.is_some(), "entity should have MaxWidth");
    assert!(
        (max_w.unwrap().0 - 600.0).abs() < f32::EPSILON,
        "MaxWidth should be 5.0 * 120.0 = 600.0"
    );

    let min_h = world.get::<MinHeight>(entity);
    assert!(min_h.is_some(), "entity should have MinHeight");
    assert!(
        (min_h.unwrap().0 - 10.0).abs() < f32::EPSILON,
        "MinHeight should be 0.5 * 20.0 = 10.0"
    );

    let max_h = world.get::<MaxHeight>(entity);
    assert!(max_h.is_some(), "entity should have MaxHeight");
    assert!(
        (max_h.unwrap().0 - 100.0).abs() < f32::EPSILON,
        "MaxHeight should be 5.0 * 20.0 = 100.0"
    );
}

#[test]
fn definition_zero_width_produces_zero_min_max() {
    let mut def = test_breaker_definition();
    def.width = 0.0;
    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let min_w = world.get::<MinWidth>(entity);
    assert!(min_w.is_some(), "entity should have MinWidth");
    assert!(
        (min_w.unwrap().0 - 0.0).abs() < f32::EPSILON,
        "MinWidth should be 0.0 for zero-width"
    );

    let max_w = world.get::<MaxWidth>(entity);
    assert!(max_w.is_some(), "entity should have MaxWidth");
    assert!(
        (max_w.unwrap().0 - 0.0).abs() < f32::EPSILON,
        "MaxWidth should be 0.0 for zero-width"
    );
}

// ── Behavior 18: .definition() uses explicit min/max when provided ──

#[test]
fn definition_uses_explicit_min_max_when_provided() {
    let mut def = test_breaker_definition();
    def.min_w = Some(80.0);
    def.max_w = Some(200.0);
    def.min_h = Some(15.0);
    def.max_h = Some(40.0);

    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let min_w = world.get::<MinWidth>(entity);
    assert!(min_w.is_some(), "entity should have MinWidth");
    assert!(
        (min_w.unwrap().0 - 80.0).abs() < f32::EPSILON,
        "MinWidth should be explicit 80.0"
    );

    let max_w = world.get::<MaxWidth>(entity);
    assert!(max_w.is_some(), "entity should have MaxWidth");
    assert!(
        (max_w.unwrap().0 - 200.0).abs() < f32::EPSILON,
        "MaxWidth should be explicit 200.0"
    );

    let min_h = world.get::<MinHeight>(entity);
    assert!(min_h.is_some(), "entity should have MinHeight");
    assert!(
        (min_h.unwrap().0 - 15.0).abs() < f32::EPSILON,
        "MinHeight should be explicit 15.0"
    );

    let max_h = world.get::<MaxHeight>(entity);
    assert!(max_h.is_some(), "entity should have MaxHeight");
    assert!(
        (max_h.unwrap().0 - 40.0).abs() < f32::EPSILON,
        "MaxHeight should be explicit 40.0"
    );
}

// ==========================================================================
// Wave 6C: definition() stamps bolt_lost and salvo_hit
// ==========================================================================

// ── Behavior 35: definition() stores bolt_lost from BreakerDefinition ──

#[test]
fn definition_stamps_bolt_lost_tree_in_bound_effects() {
    use crate::effect_v3::{
        storage::BoundEffects,
        types::{Tree, Trigger},
    };

    let ron_str = r#"(
        name: "TestBreaker",
        bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))),
        salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))),
        effects: [],
    )"#;
    let def: crate::breaker::definition::BreakerDefinition =
        ron::de::from_str(ron_str).expect("RON should parse");

    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects");

    // Check that BoltLostOccurred tree is present in BoundEffects
    let has_bolt_lost = bound
        .0
        .iter()
        .any(|(_, tree)| matches!(tree, Tree::When(Trigger::BoltLostOccurred, _)));
    assert!(
        has_bolt_lost,
        "BoundEffects should contain a When(BoltLostOccurred, ...) tree from bolt_lost"
    );
}

#[test]
fn definition_stamps_bolt_lost_even_with_empty_effects() {
    use crate::effect_v3::{
        storage::BoundEffects,
        types::{Tree, Trigger},
    };

    let ron_str = r#"(
        name: "TestBreaker",
        bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))),
        salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))),
        effects: [],
    )"#;
    let def: crate::breaker::definition::BreakerDefinition =
        ron::de::from_str(ron_str).expect("RON should parse");

    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects");

    let bolt_lost_count = bound
        .0
        .iter()
        .filter(|(_, tree)| matches!(tree, Tree::When(Trigger::BoltLostOccurred, _)))
        .count();
    assert!(
        bolt_lost_count >= 1,
        "bolt_lost should be stamped even when effects is empty"
    );
}

// ── Behavior 36: definition() stores salvo_hit from BreakerDefinition ──

#[test]
fn definition_stamps_salvo_hit_tree_in_bound_effects() {
    use crate::effect_v3::{
        storage::BoundEffects,
        types::{EntityKind, Tree, Trigger},
    };

    let ron_str = r#"(
        name: "TestBreaker",
        bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))),
        salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(TimePenalty((seconds: 3.0))))),
        effects: [],
    )"#;
    let def: crate::breaker::definition::BreakerDefinition =
        ron::de::from_str(ron_str).expect("RON should parse");

    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects");

    let has_salvo_hit = bound
        .0
        .iter()
        .any(|(_, tree)| matches!(tree, Tree::When(Trigger::Impacted(EntityKind::Salvo), _)));
    assert!(
        has_salvo_hit,
        "BoundEffects should contain a When(Impacted(Salvo), ...) tree from salvo_hit"
    );
}

// ── Behavior 37: definition() stamps bolt_lost, salvo_hit, AND effects all together ──

#[test]
fn definition_stamps_bolt_lost_salvo_hit_and_effects_together() {
    use crate::effect_v3::{
        storage::BoundEffects,
        types::{EntityKind, Tree, Trigger},
    };

    let ron_str = r#"(
        name: "TestBreaker",
        bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))),
        salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))),
        effects: [
            Stamp(Bolt, When(PerfectBumped, Fire(SpeedBoost((multiplier: 1.5))))),
        ],
    )"#;
    let def: crate::breaker::definition::BreakerDefinition =
        ron::de::from_str(ron_str).expect("RON should parse");

    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects");

    // Should have at least 3 trees: bolt_lost + salvo_hit + the SpeedBoost effect
    assert!(
        bound.0.len() >= 3,
        "BoundEffects should have at least 3 trees (bolt_lost + salvo_hit + effect), got {}",
        bound.0.len()
    );

    let has_bolt_lost = bound
        .0
        .iter()
        .any(|(_, tree)| matches!(tree, Tree::When(Trigger::BoltLostOccurred, _)));
    let has_salvo_hit = bound
        .0
        .iter()
        .any(|(_, tree)| matches!(tree, Tree::When(Trigger::Impacted(EntityKind::Salvo), _)));
    let has_speed_boost = bound
        .0
        .iter()
        .any(|(_, tree)| matches!(tree, Tree::When(Trigger::PerfectBumped, _)));

    assert!(has_bolt_lost, "bolt_lost tree should be stamped");
    assert!(has_salvo_hit, "salvo_hit tree should be stamped");
    assert!(
        has_speed_boost,
        "effects PerfectBumped tree should be stamped"
    );
}

#[test]
fn definition_stamps_only_bolt_lost_and_salvo_hit_when_effects_empty() {
    use crate::effect_v3::storage::BoundEffects;

    let ron_str = r#"(
        name: "TestBreaker",
        bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))),
        salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))),
        effects: [],
    )"#;
    let def: crate::breaker::definition::BreakerDefinition =
        ron::de::from_str(ron_str).expect("RON should parse");

    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects");

    // With empty effects, should have exactly 2 trees: bolt_lost + salvo_hit
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should have exactly 2 trees when effects is empty, got {}",
        bound.0.len()
    );
}

// ── Behavior 38: bolt_lost tree is NOT duplicated in effects array ──

#[test]
fn definition_does_not_duplicate_bolt_lost_tree() {
    use crate::effect_v3::{
        storage::BoundEffects,
        types::{Tree, Trigger},
    };

    let ron_str = r#"(
        name: "TestBreaker",
        bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))),
        salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))),
        effects: [],
    )"#;
    let def: crate::breaker::definition::BreakerDefinition =
        ron::de::from_str(ron_str).expect("RON should parse");

    let mut world = World::new();
    let entity = Breaker::builder()
        .definition(&def)
        .headless()
        .primary()
        .spawn(&mut world.commands());
    world.flush();

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("entity should have BoundEffects");

    let bolt_lost_count = bound
        .0
        .iter()
        .filter(|(_, tree)| matches!(tree, Tree::When(Trigger::BoltLostOccurred, _)))
        .count();
    assert_eq!(
        bolt_lost_count, 1,
        "BoltLostOccurred tree should appear exactly once, not duplicated, got {bolt_lost_count}",
    );
}
