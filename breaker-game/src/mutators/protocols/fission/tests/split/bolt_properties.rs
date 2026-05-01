//! Group D — new bolt spawn properties (Behaviors 14-20).
//!
//! Pins that the new bolt's `Position2D` equals the parent's (exact); the
//! new bolt's `Velocity2D` is the parent's rotated by
//! `FISSION_DIVERGENCE_ANGLE_RAD`; the parent's velocity is unchanged;
//! `BoundEffects` / `StagedEffects` are cloned from parent; the new bolt has
//! `ExtraBolt` (not `PrimaryBolt`) and carries the full bolt bundle.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::super::{
    super::system::{FISSION_DIVERGENCE_ANGLE_RAD, FissionConfig},
    helpers::{
        bolt_pos_and_vel, bolts_other_than, build_fission_app, count_bolts, count_primary_bolts,
        install_fission_config, install_fission_counter, seed_active_protocols_with_fission,
        spawn_bolt_at_with_velocity, spawn_bolt_with_bound_effects, spawn_bolt_with_staged_effects,
        write_destroyed_cell,
    },
};
use crate::{
    bolt::{
        components::{ExtraBolt, PrimaryBolt},
        test_utils::default_bolt_definition,
    },
    effect_v3::{
        effects::{DamageBoostConfig, SpeedBoostConfig},
        types::{EffectType, Tree},
    },
    prelude::*,
    shared::size::BaseRadius,
};

// ── Behavior 14 — new bolt Position2D equals parent's Position2D ────────────

#[test]
fn new_bolt_position_equals_parent_position_at_origin() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let parent =
        spawn_bolt_at_with_velocity(&mut app, Vec2::new(200.0, 300.0), Vec2::new(0.0, 400.0));

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let others = bolts_other_than(&mut app, parent);
    assert_eq!(others.len(), 1, "exactly one new bolt expected");
    let (pos, _vel) = bolt_pos_and_vel(&app, others[0]);
    assert_eq!(
        pos,
        Vec2::new(200.0, 300.0),
        "new bolt position must equal parent's position exactly; got {pos:?}"
    );
}

// ── Behavior 14 (edge case) — negative position ────────────────────────────-

#[test]
fn new_bolt_position_at_negative_coords_matches_parent() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let parent =
        spawn_bolt_at_with_velocity(&mut app, Vec2::new(-150.0, -250.0), Vec2::new(0.0, 400.0));

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let others = bolts_other_than(&mut app, parent);
    assert_eq!(others.len(), 1);
    let (pos, _vel) = bolt_pos_and_vel(&app, others[0]);
    assert_eq!(
        pos,
        Vec2::new(-150.0, -250.0),
        "new bolt position must match parent at negative coords; got {pos:?}"
    );
}

// ── Behavior 14 (edge case) — origin position ──────────────────────────────-

#[test]
fn new_bolt_position_at_origin_matches_parent() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let others = bolts_other_than(&mut app, parent);
    assert_eq!(others.len(), 1);
    let (pos, _vel) = bolt_pos_and_vel(&app, others[0]);
    assert_eq!(
        pos,
        Vec2::ZERO,
        "new bolt position must equal parent at origin; got {pos:?}"
    );
}

// ── Behavior 15 — new bolt velocity is parent rotated by divergence angle ───

#[test]
fn new_bolt_velocity_is_parent_velocity_rotated_by_divergence_angle() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let parent_vel = Vec2::new(0.0, 400.0);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, parent_vel);

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let others = bolts_other_than(&mut app, parent);
    assert_eq!(others.len(), 1);
    let (_pos, vel) = bolt_pos_and_vel(&app, others[0]);

    let expected = Velocity2D(parent_vel)
        .rotate_by(FISSION_DIVERGENCE_ANGLE_RAD)
        .0;
    let delta = (vel - expected).length();
    assert!(
        delta < 1e-3,
        "new velocity must equal parent rotated by FISSION_DIVERGENCE_ANGLE_RAD; \
         expected {expected:?}, got {vel:?}, delta {delta}"
    );

    let speed = vel.length();
    assert!(
        (speed - 400.0).abs() < 1e-3,
        "new bolt speed magnitude must match parent (400.0); got {speed}"
    );
}

// ── Behavior 15 (edge case) — off-axis parent velocity (150.0, 400.0) ──────-

#[test]
fn new_bolt_velocity_off_axis_preserves_magnitude_and_rotation() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let parent_vel = Vec2::new(150.0, 400.0);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, parent_vel);

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let others = bolts_other_than(&mut app, parent);
    assert_eq!(others.len(), 1);
    let (_pos, vel) = bolt_pos_and_vel(&app, others[0]);

    let expected = Velocity2D(parent_vel)
        .rotate_by(FISSION_DIVERGENCE_ANGLE_RAD)
        .0;
    let delta = (vel - expected).length();
    assert!(
        delta < 1e-3,
        "new velocity must match rotated parent; expected {expected:?}, got {vel:?}"
    );

    let expected_speed = parent_vel.length();
    let actual_speed = vel.length();
    assert!(
        (actual_speed - expected_speed).abs() < 1e-3,
        "speed magnitude must be preserved; expected {expected_speed}, got {actual_speed}"
    );
}

// ── Behavior 15 (edge case) — negative parent velocity ─────────────────────-

#[test]
fn new_bolt_velocity_negative_components_rotate_correctly() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let parent_vel = Vec2::new(-200.0, -300.0);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, parent_vel);

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let others = bolts_other_than(&mut app, parent);
    assert_eq!(others.len(), 1);
    let (_pos, vel) = bolt_pos_and_vel(&app, others[0]);

    let expected = Velocity2D(parent_vel)
        .rotate_by(FISSION_DIVERGENCE_ANGLE_RAD)
        .0;
    let delta = (vel - expected).length();
    assert!(
        delta < 1e-3,
        "negative parent velocity must rotate correctly; expected {expected:?}, got {vel:?}"
    );
}

// ── Behavior 16 — parent bolt's Velocity2D is unchanged after the split ─────

#[test]
fn parent_velocity_unchanged_after_split() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let parent_vel = Vec2::new(0.0, 400.0);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, parent_vel);

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let (_pos, vel) = bolt_pos_and_vel(&app, parent);
    assert_eq!(
        vel, parent_vel,
        "parent velocity must remain unchanged after split; got {vel:?}"
    );
}

// ── Behavior 16 (edge case) — two successive splits leave parent unchanged ─-

#[test]
fn two_successive_splits_leave_parent_velocity_unchanged() {
    let mut app = build_fission_app();
    install_fission_config(
        &mut app,
        FissionConfig {
            kills_per_split:      1,
            divergence_angle_rad: 0.0,
        },
    );
    seed_active_protocols_with_fission(&mut app, 1);
    let parent_vel = Vec2::new(0.0, 400.0);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, parent_vel);

    // First split.
    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);
    // Second split.
    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let (_pos, vel) = bolt_pos_and_vel(&app, parent);
    assert_eq!(
        vel, parent_vel,
        "parent velocity must remain unchanged after two splits; got {vel:?}"
    );
}

// ── Behavior 17 — new bolt inherits parent's BoundEffects ───────────────────

#[test]
fn new_bolt_inherits_parent_bound_effects() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);

    let bound = BoundEffects(vec![(
        "chip_damage".to_string(),
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(1.5),
        })),
    )]);
    let parent = spawn_bolt_with_bound_effects(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), bound);

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let others = bolts_other_than(&mut app, parent);
    assert_eq!(others.len(), 1);
    let new_bound = app
        .world()
        .get::<BoundEffects>(others[0])
        .expect("new bolt must have BoundEffects");
    assert_eq!(
        new_bound.0.len(),
        1,
        "new bolt must have exactly one bound effect; got {}",
        new_bound.0.len()
    );
    assert_eq!(
        new_bound.0[0].0, "chip_damage",
        "new bolt's bound-effect name must match parent"
    );
    match &new_bound.0[0].1 {
        Tree::Fire(EffectType::DamageBoost(cfg)) => {
            assert_eq!(
                cfg.multiplier,
                OrderedFloat(1.5),
                "multiplier must match parent; got {:?}",
                cfg.multiplier
            );
        }
        other => panic!("expected Fire(DamageBoost), got {other:?}"),
    }
}

// ── Behavior 17 (edge case) — parent with empty BoundEffects ───────────────-

#[test]
fn new_bolt_inherits_empty_bound_effects_when_parent_is_empty() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);

    let bound = BoundEffects(vec![]);
    let parent = spawn_bolt_with_bound_effects(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), bound);

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let others = bolts_other_than(&mut app, parent);
    assert_eq!(others.len(), 1);
    let new_bound = app
        .world()
        .get::<BoundEffects>(others[0])
        .expect("new bolt must have BoundEffects (empty)");
    assert!(
        new_bound.0.is_empty(),
        "new bolt BoundEffects must be empty; got {} entries",
        new_bound.0.len()
    );
}

// ── Behavior 17 (edge case) — parent with TWO entries ──────────────────────-

#[test]
fn new_bolt_inherits_two_bound_effects_in_order() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);

    let bound = BoundEffects(vec![
        (
            "chip_a".to_string(),
            Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                multiplier: OrderedFloat(1.5),
            })),
        ),
        (
            "chip_b".to_string(),
            Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(2.0),
            })),
        ),
    ]);
    let parent = spawn_bolt_with_bound_effects(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), bound);

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let others = bolts_other_than(&mut app, parent);
    assert_eq!(others.len(), 1);
    let new_bound = app
        .world()
        .get::<BoundEffects>(others[0])
        .expect("new bolt must have BoundEffects");
    assert_eq!(new_bound.0.len(), 2, "new bolt must inherit two entries");
    assert_eq!(new_bound.0[0].0, "chip_a", "first entry must be chip_a");
    assert_eq!(new_bound.0[1].0, "chip_b", "second entry must be chip_b");
}

// ── Behavior 18 — new bolt inherits parent's StagedEffects ──────────────────

#[test]
fn new_bolt_inherits_parent_staged_effects() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);

    let staged = StagedEffects(vec![(
        "staged_zap".to_string(),
        Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        })),
    )]);
    let parent =
        spawn_bolt_with_staged_effects(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), staged);

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let others = bolts_other_than(&mut app, parent);
    assert_eq!(others.len(), 1);
    let new_staged = app
        .world()
        .get::<StagedEffects>(others[0])
        .expect("new bolt must have StagedEffects");
    assert_eq!(new_staged.0.len(), 1);
    assert_eq!(new_staged.0[0].0, "staged_zap");
}

// ── Behavior 18 (edge case) — parent with empty StagedEffects ──────────────-

#[test]
fn new_bolt_inherits_empty_staged_effects_when_parent_is_empty() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);

    let staged = StagedEffects(vec![]);
    let parent =
        spawn_bolt_with_staged_effects(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), staged);

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let others = bolts_other_than(&mut app, parent);
    assert_eq!(others.len(), 1);
    let new_staged = app
        .world()
        .get::<StagedEffects>(others[0])
        .expect("new bolt must have StagedEffects (empty)");
    assert!(
        new_staged.0.is_empty(),
        "empty StagedEffects must propagate empty; got {} entries",
        new_staged.0.len()
    );
}

// ── Behavior 18 (edge case) — parent has NO StagedEffects component ────────-

#[test]
fn new_bolt_has_no_staged_effects_when_parent_has_none() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));
    // No StagedEffects installed on parent.

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let others = bolts_other_than(&mut app, parent);
    assert_eq!(others.len(), 1);
    assert!(
        app.world().get::<StagedEffects>(others[0]).is_none(),
        "new bolt must NOT have StagedEffects when parent has none"
    );
}

// ── Behavior 19 — new bolt is NOT PrimaryBolt; it has ExtraBolt ─────────────

#[test]
fn new_bolt_is_not_primary_and_has_extra_bolt() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    // Parent still has PrimaryBolt.
    assert!(
        app.world().get::<PrimaryBolt>(parent).is_some(),
        "parent must still have PrimaryBolt"
    );
    assert!(
        app.world().get::<ExtraBolt>(parent).is_none(),
        "parent must NOT have ExtraBolt"
    );

    // New bolt has ExtraBolt, not PrimaryBolt.
    let others = bolts_other_than(&mut app, parent);
    assert_eq!(others.len(), 1);
    let new_bolt = others[0];
    assert!(
        app.world().get::<PrimaryBolt>(new_bolt).is_none(),
        "new bolt must NOT have PrimaryBolt"
    );
    assert!(
        app.world().get::<ExtraBolt>(new_bolt).is_some(),
        "new bolt must have ExtraBolt"
    );
}

// ── Behavior 19 (edge case) — only one PrimaryBolt after two successive splits

#[test]
fn only_one_primary_bolt_after_two_successive_splits() {
    let mut app = build_fission_app();
    install_fission_config(
        &mut app,
        FissionConfig {
            kills_per_split:      1,
            divergence_angle_rad: 0.0,
        },
    );
    seed_active_protocols_with_fission(&mut app, 1);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    // First split.
    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);
    // Second split.
    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    assert_eq!(
        count_primary_bolts(&mut app),
        1,
        "exactly one PrimaryBolt must remain after two successive splits"
    );
    assert_eq!(
        count_bolts(&mut app),
        3,
        "three bolts total after two splits"
    );
}

// ── Behavior 20 — new bolt carries full bolt bundle ─────────────────────────

#[test]
fn new_bolt_has_full_bolt_bundle_components() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let others = bolts_other_than(&mut app, parent);
    assert_eq!(others.len(), 1);
    let new_bolt = others[0];
    assert!(
        app.world().get::<Bolt>(new_bolt).is_some(),
        "new bolt must have Bolt marker"
    );
    assert!(
        app.world().get::<Position2D>(new_bolt).is_some(),
        "new bolt must have Position2D"
    );
    assert!(
        app.world().get::<Velocity2D>(new_bolt).is_some(),
        "new bolt must have Velocity2D"
    );
    assert!(
        app.world().get::<BaseRadius>(new_bolt).is_some(),
        "new bolt must have BaseRadius"
    );
}

// ── Behavior 20 (edge case) — BaseRadius matches default_bolt_definition ───-

#[test]
fn new_bolt_base_radius_matches_default_definition() {
    let mut app = build_fission_app();
    seed_active_protocols_with_fission(&mut app, 8);
    install_fission_counter(&mut app, 7);
    let parent = spawn_bolt_at_with_velocity(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0));

    write_destroyed_cell(&mut app, Some(parent));
    tick(&mut app);

    let others = bolts_other_than(&mut app, parent);
    assert_eq!(others.len(), 1);
    let expected_radius = default_bolt_definition().radius;
    let new_radius = app
        .world()
        .get::<BaseRadius>(others[0])
        .expect("new bolt must have BaseRadius")
        .0;
    assert!(
        (new_radius - expected_radius).abs() < f32::EPSILON,
        "new bolt BaseRadius ({new_radius}) must match default definition ({expected_radius})"
    );
}
