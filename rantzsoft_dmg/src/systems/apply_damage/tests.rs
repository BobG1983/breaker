use std::marker::PhantomData;

use bevy::prelude::*;

use super::system::*;
use crate::{
    components::{Dead, Hp, Invulnerable, KilledBy},
    messages::DamageDealt,
    traits::Dmgable,
};

#[derive(Component)]
struct TestT;
impl Dmgable for TestT {}

#[derive(Component)]
struct OtherT;
impl Dmgable for OtherT {}

#[track_caller]
fn assert_f32_eq(actual: f32, expected: f32) {
    if expected.is_infinite() {
        assert!(
            actual.is_infinite() && actual.is_sign_positive() == expected.is_sign_positive(),
            "expected {expected}, got {actual}"
        );
    } else {
        assert!(
            (actual - expected).abs() < f32::EPSILON,
            "expected {expected}, got {actual}"
        );
    }
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<DamageDealt<TestT>>();
    app.add_systems(FixedUpdate, apply_damage::<TestT>);
    app
}

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

fn enqueue(app: &mut App, msg: DamageDealt<TestT>) {
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<TestT>>>()
        .write(msg);
}

fn mk_msg(
    dealer: Option<Entity>,
    attributed_to: Option<Entity>,
    target: Entity,
    amount: f32,
) -> DamageDealt<TestT> {
    DamageDealt::<TestT> {
        dealer,
        attributed_to,
        target,
        amount,
        source: None,
        _marker: PhantomData,
    }
}

// ── Behavior 96: Hp decrements by msg.amount ──

#[test]
fn hp_decrements_by_msg_amount() {
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(30.0))).id();

    enqueue(&mut app, mk_msg(None, None, e, 10.0));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 20.0);
    assert!(app.world().get::<Dead>(e).is_none());
    assert!(app.world().get::<KilledBy>(e).is_none());
}

#[test]
fn zero_amount_leaves_hp_unchanged_and_no_killed_by() {
    // Edge case 96a.
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(30.0))).id();

    enqueue(&mut app, mk_msg(None, None, e, 0.0));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 30.0);
    assert!(app.world().get::<KilledBy>(e).is_none());
}

// ── Behavior 97: killing blow inserts KilledBy { killer: Some(e) } ──

#[test]
fn killing_blow_inserts_killed_by_with_dealer() {
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, e, 10.0));
    tick(&mut app);

    let killed_by = app
        .world()
        .get::<KilledBy>(e)
        .expect("KilledBy should be inserted on killing blow");
    assert_eq!(killed_by.killer, Some(dealer));
    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 0.0);
}

#[test]
fn overkill_inserts_killed_by_and_leaves_negative_hp() {
    // Edge case 97a.
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, e, 25.0));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(e).unwrap();
    assert_eq!(killed_by.killer, Some(dealer));
    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, -15.0);
}

#[test]
fn environmental_kill_inserts_killed_by_with_none_dealer() {
    // Edge case 97b.
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();

    enqueue(&mut app, mk_msg(None, None, e, 10.0));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(e).unwrap();
    assert!(killed_by.killer.is_none());
}

// ── Behavior 98: non-killing hit does NOT insert KilledBy ──

#[test]
fn non_killing_hit_does_not_insert_killed_by() {
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(30.0))).id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, e, 10.0));
    tick(&mut app);

    assert!(app.world().get::<KilledBy>(e).is_none());
}

#[test]
fn second_non_killing_hit_still_no_killed_by() {
    // Edge case 98a.
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(30.0))).id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, e, 10.0));
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 20.0);

    enqueue(&mut app, mk_msg(Some(dealer), None, e, 5.0));
    tick(&mut app);
    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 15.0);
    assert!(app.world().get::<KilledBy>(e).is_none());
}

// ── Behavior 99: entity with Dead marker is skipped ──

#[test]
fn dead_entity_is_skipped() {
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(10.0), Dead)).id();

    enqueue(&mut app, mk_msg(None, None, e, 5.0));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
    assert!(app.world().get::<KilledBy>(e).is_none());
}

#[test]
fn dead_entity_with_low_hp_is_still_skipped() {
    // Edge case 99a.
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(5.0), Dead)).id();

    enqueue(&mut app, mk_msg(None, None, e, 10.0));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 5.0);
}

// ── Behavior 100: entity without Hp is silently skipped ──

#[test]
fn entity_without_hp_is_skipped() {
    let mut app = test_app();
    let e = app.world_mut().spawn(TestT).id();

    enqueue(&mut app, mk_msg(None, None, e, 5.0));
    tick(&mut app);

    assert!(app.world().get::<Hp>(e).is_none());
    assert!(app.world().get::<KilledBy>(e).is_none());
}

// ── Behavior 101: entity without T marker is skipped ──

#[test]
fn entity_without_t_marker_is_skipped() {
    let mut app = test_app();
    let e = app.world_mut().spawn((OtherT, Hp::new(10.0))).id();

    enqueue(&mut app, mk_msg(None, None, e, 10.0));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
    assert!(app.world().get::<KilledBy>(e).is_none());
}

// ── Behavior 102: first-kill-wins ──

#[test]
fn first_kill_wins_with_two_messages_same_tick() {
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();
    let dealer_a = app.world_mut().spawn_empty().id();
    let dealer_b = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer_a), None, e, 10.0));
    enqueue(&mut app, mk_msg(Some(dealer_b), None, e, 5.0));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(e).unwrap();
    assert_eq!(killed_by.killer, Some(dealer_a));
}

#[test]
fn first_kill_wins_with_three_messages_same_tick() {
    // Edge case 102a.
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();
    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();
    let c = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(a), None, e, 10.0));
    enqueue(&mut app, mk_msg(Some(b), None, e, 5.0));
    enqueue(&mut app, mk_msg(Some(c), None, e, 2.0));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(e).unwrap();
    assert_eq!(killed_by.killer, Some(a));
    assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, -7.0);
}

#[test]
fn first_kill_wins_when_first_is_non_killing() {
    // Edge case 102b.
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();
    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();
    let c = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(a), None, e, 3.0));
    enqueue(&mut app, mk_msg(Some(b), None, e, 7.0));
    enqueue(&mut app, mk_msg(Some(c), None, e, 5.0));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(e).unwrap();
    assert_eq!(killed_by.killer, Some(b));
}

// ── Behavior 103: dealer copied verbatim ──

#[test]
fn killing_blow_copies_dealer_verbatim() {
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(1.0))).id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, e, 1.0));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(e).unwrap();
    assert_eq!(killed_by.killer, Some(dealer));
}

#[test]
fn killing_blow_preserves_none_dealer() {
    // Edge case 103a.
    let mut app = test_app();
    let e = app.world_mut().spawn((TestT, Hp::new(1.0))).id();

    enqueue(&mut app, mk_msg(None, None, e, 1.0));
    tick(&mut app);

    let killed_by = app.world().get::<KilledBy>(e).unwrap();
    assert!(killed_by.killer.is_none());
}

// ── Behavior 153: apply_damage's query does NOT filter Without<Invulnerable>.
//     Tested in isolation (no invulnerable_filter upstream) ──

#[test]
fn apply_damage_applies_to_invulnerable_when_filter_bypassed() {
    // Bare app, no plugin, no register_dmgable — only apply_damage::<TestT>.
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(10.0), Invulnerable))
        .id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, victim, 3.0));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 7.0);
    assert!(app.world().get::<KilledBy>(victim).is_none());
}

#[test]
fn apply_damage_inserts_killed_by_on_invulnerable_when_filter_bypassed() {
    // Edge case 153a.
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Hp::new(5.0), Invulnerable))
        .id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, victim, 5.0));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 0.0);
    let killed_by = app.world().get::<KilledBy>(victim).unwrap();
    assert_eq!(killed_by.killer, Some(dealer));
}

// ── W2 Behavior 9: KilledBy.killer = msg.attributed_to.or(msg.dealer) ──
//
// The one-line change at apply_damage.rs:46 folds attribution logic into
// the insertion: .insert(KilledBy { killer: msg.attributed_to.or(msg.dealer) })
//
// Tests observe the final KilledBy component post-tick. The four sub-tests
// cover the full truth table of (attributed_to, dealer) combinations.

#[test]
fn killed_by_killer_is_attributed_to_when_both_set() {
    // attributed_to: Some(D2), dealer: Some(D1), D1 != D2 → killer = D2.
    let mut app = test_app();
    let victim = app.world_mut().spawn((TestT, Hp::new(1.0))).id();
    let dealer = app.world_mut().spawn_empty().id();
    let attributed = app.world_mut().spawn_empty().id();
    assert_ne!(dealer, attributed);

    enqueue(
        &mut app,
        mk_msg(Some(dealer), Some(attributed), victim, 5.0),
    );
    tick(&mut app);

    let killed_by = app
        .world()
        .get::<KilledBy>(victim)
        .expect("KilledBy should be inserted");
    assert_eq!(
        killed_by.killer,
        Some(attributed),
        "attributed_to wins over dealer in kill attribution"
    );
}

#[test]
fn killed_by_killer_falls_back_to_dealer_when_attributed_to_none() {
    // attributed_to: None, dealer: Some(D1) → killer = D1 (preserves existing shape).
    //
    // NOTE (RED-phase false-pass caveat): this test alone cannot fail RED,
    // because the pre-W2 stub at `apply_damage.rs:46` writes
    // `KilledBy { killer: msg.dealer }` and when `attributed_to` is None,
    // `msg.attributed_to.or(msg.dealer) == msg.dealer`. The contract
    // distinguishing stub from GREEN is pinned by BOTH
    // `w2_killed_by_killer_uses_attributed_to_when_dealer_none` (case 3 —
    // behavioral pin) AND
    // `w2_apply_damage_insertion_uses_attributed_to_or_dealer` (case 2 —
    // structural pin). This test documents the fallback case so that a
    // future change removing the fallback (e.g. `killer: msg.attributed_to`
    // alone) would be caught.
    let mut app = test_app();
    let victim = app.world_mut().spawn((TestT, Hp::new(1.0))).id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, victim, 5.0));
    tick(&mut app);

    let killed_by = app
        .world()
        .get::<KilledBy>(victim)
        .expect("KilledBy should be inserted");
    assert_eq!(killed_by.killer, Some(dealer));
}

#[test]
fn killed_by_killer_is_none_when_both_none() {
    // attributed_to: None, dealer: None → killer = None (environmental).
    let mut app = test_app();
    let victim = app.world_mut().spawn((TestT, Hp::new(1.0))).id();

    enqueue(&mut app, mk_msg(None, None, victim, 5.0));
    tick(&mut app);

    let killed_by = app
        .world()
        .get::<KilledBy>(victim)
        .expect("KilledBy should be inserted");
    assert!(killed_by.killer.is_none());
}

#[test]
fn killed_by_killer_uses_attributed_to_when_dealer_none() {
    // attributed_to: Some(D2), dealer: None → killer = D2 (ripple-emit shape).
    let mut app = test_app();
    let victim = app.world_mut().spawn((TestT, Hp::new(1.0))).id();
    let attributed = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(None, Some(attributed), victim, 5.0));
    tick(&mut app);

    let killed_by = app
        .world()
        .get::<KilledBy>(victim)
        .expect("KilledBy should be inserted");
    assert_eq!(killed_by.killer, Some(attributed));
}

// ── W2 Behavior 10: KilledBy only inserted on the killing blow ──

#[test]
fn non_killing_blow_with_positive_hp_skips_killed_by() {
    // 2.0 damage on 5.0 HP: was_positive && hp.current (3.0) > 0 → no insert.
    let mut app = test_app();
    let victim = app.world_mut().spawn((TestT, Hp::new(5.0))).id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, victim, 2.0));
    tick(&mut app);

    assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 3.0);
    assert!(app.world().get::<KilledBy>(victim).is_none());
}

#[test]
fn zero_damage_on_positive_hp_skips_killed_by() {
    // Edge: amount 0.0 → was_positive true but post-HP still positive → no insert.
    let mut app = test_app();
    let victim = app.world_mut().spawn((TestT, Hp::new(5.0))).id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, victim, 0.0));
    tick(&mut app);

    assert!(app.world().get::<KilledBy>(victim).is_none());
}

#[test]
fn pre_dead_hp_does_not_insert_killed_by() {
    // Edge: victim already at Hp 0.0 before damage → was_positive false → no insert.
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((
            TestT,
            Hp {
                current:  0.0,
                starting: 5.0,
                max:      None,
            },
        ))
        .id();
    let dealer = app.world_mut().spawn_empty().id();

    enqueue(&mut app, mk_msg(Some(dealer), None, victim, 1.0));
    tick(&mut app);

    assert!(app.world().get::<KilledBy>(victim).is_none());
}

#[test]
fn exact_kill_inserts_killed_by() {
    // Edge: exact kill (amount == starting) inserts KilledBy (positive regression pin).
    let mut app = test_app();
    let victim = app.world_mut().spawn((TestT, Hp::new(5.0))).id();
    let dealer = app.world_mut().spawn_empty().id();
    let attributed = app.world_mut().spawn_empty().id();

    enqueue(
        &mut app,
        mk_msg(Some(dealer), Some(attributed), victim, 5.0),
    );
    tick(&mut app);

    let killed_by = app
        .world()
        .get::<KilledBy>(victim)
        .expect("KilledBy should be inserted on exact kill");
    assert_eq!(killed_by.killer, Some(attributed));
}
