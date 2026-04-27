//! Coverage-fill: chip-dispatch ↔ Until contracts not exercised elsewhere.
//!
//! Audit gap (Contracts 1+2 from the `Until(TimeExpires(_), _)` wiring audit at
//! commit 945254fd):
//!
//! Contract 1 (Test 1 below): `dispatch_chip_effects` must stamp a
//! `Tree::When(_, Tree::Until(...))` tree under exactly the canonical chip
//! source key `chip:<template>:<rarity>` — same key flow that
//! `dispatch_chip_effects_uses_template_name_and_rarity_in_source` (in
//! `source_tests.rs`) verifies for `Tree::Fire`. Without this test, a refactor
//! could route When/Until shapes through a different source-key path and
//! diverge silently.
//!
//! Contract 2 (Test 2 below): Selecting the same chip 3 times must produce 3
//! `BoundEffects` stamps under the same source key, but a single subsequent
//! When-trigger walk must arm the inner Until exactly ONCE (one timer entry,
//! one stack entry, one Until self-bind). If a future change ever rarity-tags
//! or stack-tags the source key per-stack, this idempotency would silently
//! collapse to N timers — a regression with no other coverage.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    breaker::messages::{BumpGrade, BumpPerformed},
    chips::{
        definition::{ChipDefinition, Rarity},
        systems::dispatch_chip_effects::tests::helpers::*,
    },
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        triggers::{bump::bridges::system::on_perfect_bumped, time::components::EffectTimers},
        types::{ReversibleEffectType, ScopedTree, StampTarget, Tree, Trigger},
        walking::UntilApplied,
    },
    prelude::*,
};

/// Build the Surge-shape tree: `When(PerfectBumped, Until(TimeExpires(d), Fire(SpeedBoost(m))))`.
///
/// Mirrors the asset in `assets/chips/standard/surge.chip.ron` for the common
/// rarity (multiplier 1.5, duration 1.5).
fn surge_when_until_speed_tree(duration: f32, multiplier: f32) -> Tree {
    Tree::When(
        Trigger::PerfectBumped,
        Box::new(Tree::Until(
            Trigger::TimeExpires(OrderedFloat(duration)),
            Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                SpeedBoostConfig {
                    multiplier: OrderedFloat(multiplier),
                },
            ))),
        )),
    )
}

/// Build a `ChipDefinition` matching Surge's common-rarity shape: target Bolt
/// (deferred-dispatch path), `When(PerfectBumped, Until(TimeExpires(1.5),
/// Fire(SpeedBoost(1.5))))`, `template_name` "Surge", rarity Common.
fn surge_common_definition() -> ChipDefinition {
    let mut def = ChipDefinition::test_on(
        "Basic Surge",
        StampTarget::Bolt,
        surge_when_until_speed_tree(1.5, 1.5),
        3,
    );
    def.template_name = Some("Surge".to_owned());
    def.rarity = Rarity::Common;
    def
}

// ── Test 1 ─────────────────────────────────────────────────────────────────
//
// Contract 1: dispatch_chip_effects stamps a Stamp(Bolt, When(_, Until(...)))
// tree to the Breaker's BoundEffects under exactly `chip:<template>:<rarity>`
// — same canonical SourceId form W5 mandates for Tree::Fire.

#[test]
fn dispatch_chip_effects_stamps_until_bearing_tree_under_chip_template_rarity_source() {
    let mut app = test_app();

    insert_chip(&mut app, surge_common_definition());
    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Basic Surge");

    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects must exist on the breaker after dispatch");
    assert_eq!(
        bound.0.len(),
        1,
        "exactly one stamp from one ChipSelected → one BoundEffects entry"
    );

    let (source_name, tree) = &bound.0[0];

    // Build the expected source via the canonical SourceIdExt API.
    let expected_source = SourceId::chip("Surge").rarity(Rarity::Common).build();
    assert_eq!(
        source_name.as_str(),
        expected_source.0.as_ref(),
        "B45/W5: Until-bearing trees must dispatch under the canonical \
         chip:<template>:<rarity> source-key form (same as Tree::Fire)"
    );

    // Tree shape preserved: Stamp(Bolt, When(_, Until(...))) → When(_, Until(...))
    assert_eq!(
        tree,
        &surge_when_until_speed_tree(1.5, 1.5),
        "the stamped tree must structurally equal the chip's effect tree"
    );
}

/// Asserts the post-arm `BoundEffects` shape after three same-source dispatches
/// followed by one `PerfectBumped` walk. Three outer `When` entries must
/// survive as re-arm anchors. Exactly one self-bound `Until` must appear under
/// the chip source.
///
/// Precondition: must be called after exactly one `Trigger::PerfectBumped` walk
/// has completed against `breaker`. Calling this in any other phase produces a
/// confusing assertion failure.
fn assert_post_arm_bound_state(app: &App, breaker: Entity, expected_source_str: &str) {
    let bound_post = app.world().get::<BoundEffects>(breaker).unwrap();

    let when_count_post = bound_post
        .0
        .iter()
        .filter(|(name, tree)| {
            name == expected_source_str && matches!(tree, Tree::When(Trigger::PerfectBumped, _))
        })
        .count();
    assert_eq!(
        when_count_post, 3,
        "all three outer When entries must survive the arm — they're the re-arm anchors"
    );

    let until_count_post = bound_post
        .0
        .iter()
        .filter(|(name, tree)| {
            name == expected_source_str
                && matches!(
                    tree,
                    Tree::Until(Trigger::TimeExpires(d), _) if *d == OrderedFloat(1.5)
                )
        })
        .count();
    assert_eq!(
        until_count_post, 1,
        "exactly ONE self-bound Until — three same-source stamps must collapse to one Until self-bind"
    );
}

// ── Test 2 ─────────────────────────────────────────────────────────────────
//
// Contract 2: Three ChipSelected("Surge") in the same frame produce three
// stamps under the same chip:Surge:Common source. A subsequent
// `Trigger::PerfectBumped` walk arms the inner Until exactly ONCE — proving
// that `arm_time_expires_timer` + `ensure_until_bound`'s shared idempotency
// collapses the duplicates to a single timer / single stack / single Until
// self-bind under the common source.

#[test]
fn three_dispatches_of_same_chip_yield_one_bound_until_entry_and_one_timer_after_arm() {
    // Step A: drive 3 dispatches via the standard chip-dispatch test app.
    // Pre-register the BumpPerformed message + bridge before the first
    // `app.update()` so message infrastructure exists when the bridge runs.
    let mut app = test_app();
    app.add_message::<BumpPerformed>();
    app.add_systems(Update, on_perfect_bumped);

    insert_chip(&mut app, surge_common_definition());
    let breaker = spawn_breaker(&mut app);

    select_chip(&mut app, "Basic Surge");
    select_chip(&mut app, "Basic Surge");
    select_chip(&mut app, "Basic Surge");

    app.update();

    // Pre-condition: dispatch produced 3 BoundEffects entries, all under the
    // same chip:Surge:Common source.
    let expected_source = SourceId::chip("Surge").rarity(Rarity::Common).build();
    let expected_source_str: &str = expected_source.0.as_ref();

    let bound_pre = app
        .world()
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects must exist after dispatch");
    let surge_count_pre = bound_pre
        .0
        .iter()
        .filter(|(name, _)| name == expected_source_str)
        .count();
    assert_eq!(
        surge_count_pre, 3,
        "three ChipSelected must produce three BoundEffects stamps under the same chip source"
    );

    // Step B: drive the inner-arming trigger via the production
    // `on_perfect_bumped` bridge (registered above). The bridge calls
    // `walk_bound_effects` with `Trigger::PerfectBumped` for the breaker
    // entity — same arming pathway production uses.
    //
    // Clear pending selections so the second `app.update()` doesn't re-send
    // the same chip messages (they'd be inventory-rejected anyway, but
    // clearing keeps the test step semantically clean).
    app.world_mut()
        .resource_mut::<PendingChipSelections>()
        .0
        .clear();
    app.world_mut().write_message(BumpPerformed {
        grade: BumpGrade::Perfect,
        bolt: None,
        breaker,
    });
    app.update();

    // EffectStack: exactly ONE SpeedBoost entry under chip:Surge:Common —
    // the three duplicate When entries must NOT each fire the inner Until.
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .expect("EffectStack must exist after the When trigger walks the inner Until");
    let stack_entries: Vec<&(SourceId, SpeedBoostConfig)> = stack.iter().collect();
    assert_eq!(
        stack_entries.len(),
        1,
        "three same-source When/Until stamps must collapse to ONE stack entry on the first arm"
    );
    assert_eq!(
        stack_entries[0].0, expected_source,
        "the single stack entry must carry the canonical chip:Surge:Common source"
    );
    assert_eq!(
        stack_entries[0].1.multiplier,
        OrderedFloat(1.5),
        "the inner Fire(SpeedBoost(1.5)) must produce a 1.5 multiplier — not 4.5 (3x stacking)"
    );

    // EffectTimers: exactly ONE timer entry — `arm_time_expires_timer`'s
    // (duration, source) idempotency must collapse 3 stamps to 1 timer.
    let timers = app
        .world()
        .get::<EffectTimers>(breaker)
        .expect("EffectTimers must be armed after the inner Until walks");
    assert_eq!(
        timers.timers.len(),
        1,
        "three same-source dispatches followed by one walk must yield ONE timer entry"
    );
    assert_eq!(
        timers.timers[0],
        (
            OrderedFloat(1.5),
            OrderedFloat(1.5),
            expected_source.clone(),
        ),
        "(remaining, original, source) must be (1.5, 1.5, chip:Surge:Common)"
    );

    // UntilApplied: contains exactly ONE entry under the chip source.
    let until_applied = app
        .world()
        .get::<UntilApplied>(breaker)
        .expect("UntilApplied must be installed after the inner Until evaluates");
    assert!(
        until_applied.0.contains(expected_source_str),
        "UntilApplied must contain the chip:Surge:Common source"
    );

    // BoundEffects: the three When outers must survive (they're the re-arm
    // entry-points), and exactly ONE self-bound Until must appear under the
    // chip source. Asserted via helper to keep test body under clippy's
    // too_many_lines threshold.
    assert_post_arm_bound_state(&app, breaker, expected_source_str);
}
