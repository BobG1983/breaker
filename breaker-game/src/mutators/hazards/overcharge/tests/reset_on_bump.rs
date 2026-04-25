//! Group C — `overcharge_reset_on_bump` system.
//!
//! Every test wires ONLY `overcharge_reset_on_bump` via `wire_reset_only`.

use super::{
    super::system::OverchargeKillCount,
    helpers::{
        run_fixed_update, spawn_bolt, spawn_bolt_with_count, test_app_playing, wire_reset_only,
        write_bump,
    },
};

// ── Behavior 16 — bump resets the bumping bolt's count to 0 ──────────────

#[test]
fn bump_resets_kill_count_for_bumping_bolt() {
    let mut app = test_app_playing();
    wire_reset_only(&mut app);
    let bolt = spawn_bolt_with_count(&mut app, 5);

    write_bump(&mut app, Some(bolt));
    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 0);
}

#[test]
fn bump_is_a_hard_overwrite_not_a_decrement() {
    // Edge: even u32::MAX resets to 0 in one tick (hard overwrite).
    let mut app = test_app_playing();
    wire_reset_only(&mut app);
    let bolt = spawn_bolt_with_count(&mut app, u32::MAX);

    write_bump(&mut app, Some(bolt));
    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 0, "reset is a hard overwrite, not a decrement");
}

// ── Behavior 17 — bump with bolt:None is a no-op ─────────────────────────

#[test]
fn bump_without_bolt_is_noop() {
    let mut app = test_app_playing();
    wire_reset_only(&mut app);
    let bolt = spawn_bolt_with_count(&mut app, 3);

    write_bump(&mut app, None);
    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 3, "no bolt identified on the bump → no reset");
}

#[test]
fn bump_none_does_not_reset_any_bolt() {
    // Edge: with two bolts each at 7, a bolt:None bump affects neither.
    let mut app = test_app_playing();
    wire_reset_only(&mut app);
    let bolt_a = spawn_bolt_with_count(&mut app, 7);
    let bolt_b = spawn_bolt_with_count(&mut app, 7);

    write_bump(&mut app, None);
    run_fixed_update(&mut app);

    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_a).unwrap().0, 7);
    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_b).unwrap().0, 7);
}

// ── Behavior 18 — bump on bolt lacking the component is no-op (no insert) ─

#[test]
fn bump_on_bolt_without_kill_count_does_not_insert_component() {
    let mut app = test_app_playing();
    wire_reset_only(&mut app);
    let bolt = spawn_bolt(&mut app);

    write_bump(&mut app, Some(bolt));
    run_fixed_update(&mut app);

    assert!(
        app.world().get::<OverchargeKillCount>(bolt).is_none(),
        "reset must not create the component"
    );
}

#[test]
fn repeated_bump_on_bolt_without_component_still_no_insert() {
    // Edge: a second bump for the same bolt in a later tick is still a
    // no-op — no panic, no insert.
    let mut app = test_app_playing();
    wire_reset_only(&mut app);
    let bolt = spawn_bolt(&mut app);

    write_bump(&mut app, Some(bolt));
    run_fixed_update(&mut app);
    write_bump(&mut app, Some(bolt));
    run_fixed_update(&mut app);

    assert!(app.world().get::<OverchargeKillCount>(bolt).is_none());
}

// ── Behavior 19 — multiple bumps in one tick still end at 0 ──────────────

#[test]
fn multiple_bumps_one_tick_same_bolt_end_at_zero() {
    let mut app = test_app_playing();
    wire_reset_only(&mut app);
    let bolt = spawn_bolt_with_count(&mut app, 9);

    write_bump(&mut app, Some(bolt));
    write_bump(&mut app, Some(bolt));
    write_bump(&mut app, Some(bolt));
    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 0);
}

#[test]
fn interleaved_none_bumps_do_not_disturb_reset() {
    // Edge: Some(bolt), None, Some(bolt), None, Some(bolt), None — count
    // still ends at 0 because the Some branch overwrites to 0 each time
    // and None skips.
    let mut app = test_app_playing();
    wire_reset_only(&mut app);
    let bolt = spawn_bolt_with_count(&mut app, 9);

    write_bump(&mut app, Some(bolt));
    write_bump(&mut app, None);
    write_bump(&mut app, Some(bolt));
    write_bump(&mut app, None);
    write_bump(&mut app, Some(bolt));
    write_bump(&mut app, None);
    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 0);
}

// ── Behavior 20 — two bolts' counts reset independently ─────────────────

#[test]
fn two_bolts_each_reset_independently() {
    let mut app = test_app_playing();
    wire_reset_only(&mut app);
    let bolt_a = spawn_bolt_with_count(&mut app, 4);
    let bolt_b = spawn_bolt_with_count(&mut app, 8);

    write_bump(&mut app, Some(bolt_a));
    run_fixed_update(&mut app);

    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_a).unwrap().0, 0);
    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_b).unwrap().0, 8);
}

#[test]
fn second_tick_bump_on_other_bolt_resets_it_too() {
    // Edge: a second tick with the other bolt's bump brings both to 0.
    let mut app = test_app_playing();
    wire_reset_only(&mut app);
    let bolt_a = spawn_bolt_with_count(&mut app, 4);
    let bolt_b = spawn_bolt_with_count(&mut app, 8);

    write_bump(&mut app, Some(bolt_a));
    run_fixed_update(&mut app);
    write_bump(&mut app, Some(bolt_b));
    run_fixed_update(&mut app);

    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_a).unwrap().0, 0);
    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_b).unwrap().0, 0);
}

// ── Behavior 21 — bump for a despawned bolt is a no-op ───────────────────

#[test]
fn bump_for_despawned_bolt_does_not_panic() {
    let mut app = test_app_playing();
    wire_reset_only(&mut app);
    let bolt = spawn_bolt_with_count(&mut app, 5);
    let stale_id = bolt;
    app.world_mut().despawn(bolt);

    write_bump(&mut app, Some(stale_id));
    run_fixed_update(&mut app);

    // Entity is gone; get_entity returns Err.
    assert!(
        app.world().get_entity(stale_id).is_err(),
        "despawned bolt must be gone"
    );
    // No orphaned component exists for the stale id.
    assert!(app.world().get::<OverchargeKillCount>(stale_id).is_none());
}
