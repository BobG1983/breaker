//! Part E: Builder `.survival()` and `.survival_permanent()` tests (behaviors 27-38).
//!
//! These tests exercise `.survival(pattern, timer_secs)`,
//! `.survival_permanent(pattern)`, `.with_behavior(CellBehavior::Survival)`,
//! `.with_behavior(CellBehavior::SurvivalPermanent)`, and `.definition(&def)`
//! against the `spawn_inner()` match arms. They assert the `SurvivalTurret`
//! marker, `SurvivalPattern`, `SurvivalTimer`, `BoltImmune`, and
//! `BumpVulnerable` components are inserted correctly.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    cells::{
        behaviors::survival::components::{
            BoltImmune, BumpVulnerable, SurvivalPattern, SurvivalTimer, SurvivalTurret,
        },
        components::RegenRate,
        definition::{AttackPattern, CellBehavior},
    },
    prelude::*,
};

// ── Behavior 27: .survival(StraightDown, 10.0) inserts SurvivalTurret marker ──

#[test]
fn spawn_with_survival_sugar_inserts_turret_marker() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .survival(AttackPattern::StraightDown, 10.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<SurvivalTurret>(entity).is_some(),
        "entity should have SurvivalTurret marker"
    );
    // Edge case guard: entity also has Cell and Hp
    let hp = world
        .get::<Hp>(entity)
        .expect("entity should have Hp from builder");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Hp should be 20.0, got {}",
        hp.current
    );
}

// ── Behavior 28: .survival(Spread(3), 5.0) inserts SurvivalPattern(Spread(3)) ──

#[test]
fn spawn_with_survival_sugar_inserts_correct_pattern() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .survival(AttackPattern::Spread(3), 5.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let pattern = world
        .get::<SurvivalPattern>(entity)
        .expect("entity should have SurvivalPattern");
    assert_eq!(
        pattern.0,
        AttackPattern::Spread(3),
        "SurvivalPattern should be Spread(3)"
    );
}

// ── Behavior 29: .survival(StraightDown, 10.0) inserts SurvivalTimer with correct values ──

#[test]
fn spawn_with_survival_sugar_inserts_timer_with_correct_values() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .survival(AttackPattern::StraightDown, 10.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let timer = world
        .get::<SurvivalTimer>(entity)
        .expect("entity should have SurvivalTimer");
    assert!(
        (timer.remaining - 10.0).abs() < f32::EPSILON,
        "SurvivalTimer.remaining should be 10.0, got {}",
        timer.remaining
    );
    assert!(!timer.started, "SurvivalTimer.started should be false");
}

// Behavior 29 edge: timer_secs 0.5
#[test]
fn spawn_with_survival_sugar_small_timer() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .survival(AttackPattern::StraightDown, 0.5)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let timer = world
        .get::<SurvivalTimer>(entity)
        .expect("entity should have SurvivalTimer");
    assert!(
        (timer.remaining - 0.5).abs() < f32::EPSILON,
        "SurvivalTimer.remaining should be 0.5, got {}",
        timer.remaining
    );
    assert!(!timer.started);
}

// ── Behavior 30: .survival() inserts BoltImmune marker ──

#[test]
fn spawn_with_survival_sugar_inserts_bolt_immune() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .survival(AttackPattern::StraightDown, 10.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<BoltImmune>(entity).is_some(),
        "entity should have BoltImmune marker"
    );
}

// ── Behavior 31: .survival() inserts BumpVulnerable marker ──

#[test]
fn spawn_with_survival_sugar_inserts_bump_vulnerable() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .survival(AttackPattern::StraightDown, 10.0)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<BumpVulnerable>(entity).is_some(),
        "entity should have BumpVulnerable marker"
    );
}

// ── Behavior 32: .survival_permanent() inserts markers but NO SurvivalTimer ──

#[test]
fn spawn_with_survival_permanent_inserts_markers_without_timer() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .survival_permanent(AttackPattern::StraightDown)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(
        world.get::<SurvivalTurret>(entity).is_some(),
        "entity should have SurvivalTurret"
    );
    let pattern = world
        .get::<SurvivalPattern>(entity)
        .expect("entity should have SurvivalPattern");
    assert_eq!(pattern.0, AttackPattern::StraightDown);
    assert!(
        world.get::<BoltImmune>(entity).is_some(),
        "entity should have BoltImmune"
    );
    assert!(
        world.get::<BumpVulnerable>(entity).is_some(),
        "entity should have BumpVulnerable"
    );
    assert!(
        world.get::<SurvivalTimer>(entity).is_none(),
        "SurvivalPermanent should NOT insert SurvivalTimer"
    );
}

// Behavior 32 edge: Spread(4)
#[test]
fn spawn_with_survival_permanent_spread_4_has_correct_pattern() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .survival_permanent(AttackPattern::Spread(4))
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    let pattern = world
        .get::<SurvivalPattern>(entity)
        .expect("entity should have SurvivalPattern");
    assert_eq!(
        pattern.0,
        AttackPattern::Spread(4),
        "SurvivalPattern should be Spread(4)"
    );
}

// ── Behavior 33: Cell without survival has no survival components ──

#[test]
fn spawn_without_survival_has_no_survival_components() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<SurvivalTurret>(entity).is_none());
    assert!(world.get::<SurvivalPattern>(entity).is_none());
    assert!(world.get::<SurvivalTimer>(entity).is_none());
    assert!(world.get::<BoltImmune>(entity).is_none());
    assert!(world.get::<BumpVulnerable>(entity).is_none());

    // Guard: prove the builder ran
    let hp = world
        .get::<Hp>(entity)
        .expect("entity should have Hp from builder");
    assert!(
        (hp.current - 20.0).abs() < f32::EPSILON,
        "Hp should be 20.0, got {}",
        hp.current
    );
}

// ── Behavior 34: .with_behavior(CellBehavior::Survival) matches .survival() sugar ──

#[test]
fn spawn_with_behavior_survival_matches_survival_sugar() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::Survival {
                pattern:    AttackPattern::Spread(3),
                timer_secs: 5.0,
            })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<SurvivalTurret>(entity).is_some());
    let pattern = world
        .get::<SurvivalPattern>(entity)
        .expect("should have SurvivalPattern");
    assert_eq!(pattern.0, AttackPattern::Spread(3));
    let timer = world
        .get::<SurvivalTimer>(entity)
        .expect("should have SurvivalTimer");
    assert!((timer.remaining - 5.0).abs() < f32::EPSILON);
    assert!(!timer.started);
    assert!(world.get::<BoltImmune>(entity).is_some());
    assert!(world.get::<BumpVulnerable>(entity).is_some());
}

// ── Behavior 35: .with_behavior(CellBehavior::SurvivalPermanent) matches .survival_permanent() sugar ──

#[test]
fn spawn_with_behavior_survival_permanent_matches_sugar() {
    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .with_behavior(CellBehavior::SurvivalPermanent {
                pattern: AttackPattern::StraightDown,
            })
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .hp(20.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<SurvivalTurret>(entity).is_some());
    let pattern = world
        .get::<SurvivalPattern>(entity)
        .expect("should have SurvivalPattern");
    assert_eq!(pattern.0, AttackPattern::StraightDown);
    assert!(world.get::<BoltImmune>(entity).is_some());
    assert!(world.get::<BumpVulnerable>(entity).is_some());
    assert!(
        world.get::<SurvivalTimer>(entity).is_none(),
        "SurvivalPermanent should NOT insert SurvivalTimer"
    );
}

// ── Behavior 36: .definition(&def) with Survival behavior inserts all survival components ──

#[test]
fn spawn_survival_through_definition_inserts_all_components() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![CellBehavior::Survival {
        pattern:    AttackPattern::Spread(2),
        timer_secs: 8.0,
    }]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<SurvivalTurret>(entity).is_some());
    let pattern = world
        .get::<SurvivalPattern>(entity)
        .expect("should have SurvivalPattern");
    assert_eq!(pattern.0, AttackPattern::Spread(2));
    let timer = world
        .get::<SurvivalTimer>(entity)
        .expect("should have SurvivalTimer");
    assert!((timer.remaining - 8.0).abs() < f32::EPSILON);
    assert!(!timer.started);
    assert!(world.get::<BoltImmune>(entity).is_some());
    assert!(world.get::<BumpVulnerable>(entity).is_some());
}

// ── Behavior 37: .definition(&def) with SurvivalPermanent behavior inserts components without timer ──

#[test]
fn spawn_survival_permanent_through_definition_inserts_without_timer() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![CellBehavior::SurvivalPermanent {
        pattern: AttackPattern::StraightDown,
    }]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    assert!(world.get::<SurvivalTurret>(entity).is_some());
    let pattern = world
        .get::<SurvivalPattern>(entity)
        .expect("should have SurvivalPattern");
    assert_eq!(pattern.0, AttackPattern::StraightDown);
    assert!(world.get::<BoltImmune>(entity).is_some());
    assert!(world.get::<BumpVulnerable>(entity).is_some());
    assert!(
        world.get::<SurvivalTimer>(entity).is_none(),
        "SurvivalPermanent through definition should NOT insert SurvivalTimer"
    );
}

// ── Behavior 38: Survival + Regen through definition inserts both ──

#[test]
fn spawn_survival_with_regen_through_definition_inserts_both() {
    let mut def = test_cell_definition();
    def.behaviors = Some(vec![
        CellBehavior::Regen { rate: 2.0 },
        CellBehavior::Survival {
            pattern:    AttackPattern::StraightDown,
            timer_secs: 10.0,
        },
    ]);

    let mut world = World::new();
    let entity = spawn_cell_in_world(&mut world, |commands| {
        Cell::builder()
            .definition(&def)
            .position(Vec2::ZERO)
            .dimensions(70.0, 24.0)
            .headless()
            .spawn(commands)
    });

    let regen_rate = world
        .get::<RegenRate>(entity)
        .expect("entity should have RegenRate");
    assert!((regen_rate.0 - 2.0).abs() < f32::EPSILON);
    assert!(world.get::<SurvivalTurret>(entity).is_some());
    let pattern = world
        .get::<SurvivalPattern>(entity)
        .expect("should have SurvivalPattern");
    assert_eq!(pattern.0, AttackPattern::StraightDown);
    let timer = world
        .get::<SurvivalTimer>(entity)
        .expect("should have SurvivalTimer");
    assert!((timer.remaining - 10.0).abs() < f32::EPSILON);
    assert!(!timer.started);
    assert!(world.get::<BoltImmune>(entity).is_some());
    assert!(world.get::<BumpVulnerable>(entity).is_some());
}
