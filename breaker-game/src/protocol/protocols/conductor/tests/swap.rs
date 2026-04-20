//! Group S — swap semantics on Perfect bump (Behaviors 1–5).
//!
//! Pins that a Perfect-graded `BumpPerformed` whose target bolt is an
//! `ExtraBolt` swaps the `PrimaryBolt` / `ExtraBolt` markers and the
//! `BoundEffects` / `StagedEffects` components between the bumped bolt and
//! the current primary bolt. Covers the happy-path marker swap, the
//! bound-component swap (including multi-entry vecs), the staged-component
//! swap (including empty-present vecs), and one-sided presence of
//! `BoundEffects` / `StagedEffects` on either side of the pair.

use super::helpers::{
    bound_fingerprints, build_conductor_app, has_extra, has_primary, make_distinct_bound,
    make_distinct_staged, seed_active_protocols_with_conductor, spawn_dummy_breaker,
    spawn_extra_bolt, spawn_extra_bolt_with_bound, spawn_extra_bolt_with_bound_and_staged,
    spawn_primary_bolt, spawn_primary_bolt_with_bound, spawn_primary_bolt_with_bound_and_staged,
    staged_fingerprints, write_bump_performed,
};
use crate::{breaker::messages::BumpGrade, effect_v3::types::Tree, prelude::*};

// ── Behavior 1 — Perfect bump on an ExtraBolt swaps the marker pair ─────────

#[test]
fn perfect_bump_on_extra_bolt_swaps_primary_and_extra_markers() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt(&mut app);
    let extra = spawn_extra_bolt(&mut app);

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        has_primary(&app, extra),
        "bumped bolt must gain PrimaryBolt"
    );
    assert!(
        !has_extra(&app, extra),
        "bumped bolt must lose ExtraBolt on promotion"
    );
    assert!(
        !has_primary(&app, primary),
        "old primary must lose PrimaryBolt"
    );
    assert!(
        has_extra(&app, primary),
        "old primary must gain ExtraBolt on demotion"
    );
}

// ── Behavior 2 — Perfect bump swaps BoundEffects ────────────────────────────

#[test]
fn perfect_bump_swaps_bound_effects_between_primary_and_bumped() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["PRIMARY_BOUND".to_string()],
        "bumped bolt must receive the old primary's BoundEffects"
    );
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["EXTRA_BOUND".to_string()],
        "old primary must receive the bumped bolt's BoundEffects"
    );
}

// ── Behavior 2 (edge case) — multi-entry vecs swap as a unit ────────────────

#[test]
fn perfect_bump_preserves_multi_entry_bound_vec_order_on_swap() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);

    // Primary has a two-entry vec.
    let primary_bound = BoundEffects(vec![
        ("A".to_string(), Tree::Sequence(vec![])),
        ("B".to_string(), Tree::Sequence(vec![])),
    ]);
    let primary = spawn_primary_bolt_with_bound(&mut app, primary_bound);

    // Extra has a single-entry vec.
    let extra_bound = BoundEffects(vec![("C".to_string(), Tree::Sequence(vec![]))]);
    let extra = spawn_extra_bolt_with_bound(&mut app, extra_bound);

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["A".to_string(), "B".to_string()],
        "two-entry vec moves to bumped bolt preserving order"
    );
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["C".to_string()],
        "single-entry vec moves to old primary"
    );
}

// ── Behavior 3 — Perfect bump swaps StagedEffects when both present ─────────

#[test]
fn perfect_bump_swaps_staged_effects_when_both_bolts_have_it() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound_and_staged(
        &mut app,
        make_distinct_bound("PB"),
        make_distinct_staged("PS"),
    );
    let extra = spawn_extra_bolt_with_bound_and_staged(
        &mut app,
        make_distinct_bound("EB"),
        make_distinct_staged("ES"),
    );

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        staged_fingerprints(&app, extra),
        vec!["PS".to_string()],
        "bumped bolt must receive old primary's StagedEffects"
    );
    assert_eq!(
        staged_fingerprints(&app, primary),
        vec!["ES".to_string()],
        "old primary must receive bumped bolt's StagedEffects"
    );
    // Sanity: bound also swapped per Behavior 2 semantics.
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["PB".to_string()],
        "sanity: bound swapped"
    );
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["EB".to_string()],
        "sanity: bound swapped"
    );
}

// ── Behavior 3 (edge case) — empty-present StagedEffects survives as present

#[test]
fn perfect_bump_preserves_empty_staged_effects_as_present_component() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);

    // Primary has empty-but-present StagedEffects.
    let primary = spawn_primary_bolt_with_bound_and_staged(
        &mut app,
        make_distinct_bound("PB"),
        StagedEffects(vec![]),
    );
    // Extra has a single-entry StagedEffects.
    let extra = spawn_extra_bolt_with_bound_and_staged(
        &mut app,
        make_distinct_bound("EB"),
        make_distinct_staged("ES"),
    );

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        staged_fingerprints(&app, extra),
        Vec::<String>::new(),
        "bumped bolt's StagedEffects is now the old primary's empty vec"
    );
    assert!(
        app.world().get::<StagedEffects>(extra).is_some(),
        "StagedEffects component must be PRESENT on the bumped bolt — empty vec \
         must not be conflated with component-absent"
    );
    assert_eq!(
        staged_fingerprints(&app, primary),
        vec!["ES".to_string()],
        "old primary must receive the bumped bolt's single-entry StagedEffects"
    );
}

// ── Behavior 4 — one-sided StagedEffects: old primary has it, bumped does not

#[test]
fn perfect_bump_moves_staged_effects_from_primary_to_bumped_when_only_primary_has_it() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound_and_staged(
        &mut app,
        make_distinct_bound("PB"),
        make_distinct_staged("PS"),
    );
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EB"));

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        app.world().get::<StagedEffects>(extra).is_some(),
        "bumped bolt must gain StagedEffects"
    );
    assert_eq!(
        staged_fingerprints(&app, extra),
        vec!["PS".to_string()],
        "bumped bolt must hold the old primary's StagedEffects fingerprint"
    );
    assert!(
        app.world().get::<StagedEffects>(primary).is_none(),
        "old primary must lose StagedEffects entirely — receiver's absent side \
         moves in"
    );
    // Sanity: bound swap still fires.
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["PB".to_string()],
        "sanity: BoundEffects swapped regardless of staged presence"
    );
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["EB".to_string()],
        "sanity: BoundEffects swapped regardless of staged presence"
    );
}

// ── Behavior 4 (edge case) — symmetric: bumped has StagedEffects, primary does not

#[test]
fn perfect_bump_moves_staged_effects_from_bumped_to_primary_when_only_bumped_has_it() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PB"));
    let extra = spawn_extra_bolt_with_bound_and_staged(
        &mut app,
        make_distinct_bound("EB"),
        make_distinct_staged("ES"),
    );

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        app.world().get::<StagedEffects>(primary).is_some(),
        "old primary must gain StagedEffects"
    );
    assert_eq!(
        staged_fingerprints(&app, primary),
        vec!["ES".to_string()],
        "old primary must hold the bumped bolt's StagedEffects"
    );
    assert!(
        app.world().get::<StagedEffects>(extra).is_none(),
        "bumped bolt must lose StagedEffects — receiver's absent side moves in"
    );
}

// ── Behavior 5 — one-sided BoundEffects: old primary has it, bumped does not

#[test]
fn perfect_bump_moves_bound_effects_from_primary_to_bumped_when_only_primary_has_it() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PB"));
    let extra = spawn_extra_bolt(&mut app);

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        app.world().get::<BoundEffects>(extra).is_some(),
        "bumped bolt must gain BoundEffects"
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["PB".to_string()],
        "bumped bolt must hold the old primary's fingerprint"
    );
    assert!(
        app.world().get::<BoundEffects>(primary).is_none(),
        "old primary must lose BoundEffects — receiver's absent side moves in"
    );
    assert!(
        has_primary(&app, extra) && has_extra(&app, primary),
        "markers still swap per Behavior 1 semantics"
    );
}

// ── Behavior 5 (edge case) — both bolts have neither bound nor staged ───────

#[test]
fn perfect_bump_with_neither_bound_nor_staged_on_either_bolt_swaps_markers_only() {
    let mut app = build_conductor_app();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt(&mut app);
    let extra = spawn_extra_bolt(&mut app);

    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    // Markers swap.
    assert!(has_primary(&app, extra), "markers swap — extra promoted");
    assert!(has_extra(&app, primary), "markers swap — primary demoted");

    // No BoundEffects / StagedEffects on either side (they had none, still none).
    assert!(
        app.world().get::<BoundEffects>(extra).is_none(),
        "bumped bolt still has no BoundEffects"
    );
    assert!(
        app.world().get::<BoundEffects>(primary).is_none(),
        "old primary still has no BoundEffects"
    );
    assert!(
        app.world().get::<StagedEffects>(extra).is_none(),
        "bumped bolt still has no StagedEffects"
    );
    assert!(
        app.world().get::<StagedEffects>(primary).is_none(),
        "old primary still has no StagedEffects"
    );
}
