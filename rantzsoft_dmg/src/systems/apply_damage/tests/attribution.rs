use super::helpers::{TestT, enqueue, mk_msg, test_app, tick};
use crate::components::{Hp, KilledBy};

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
