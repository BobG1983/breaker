//! Group B — `overcharge_count_kills` system.
//!
//! Every test wires ONLY `overcharge_count_kills` via `wire_count_only`,
//! bypassing the `register`-installed run conditions. Install a canonical
//! config + 1 Overcharge stack for parity with register-wired tests (the
//! system itself reads neither).

use super::{
    super::system::OverchargeKillCount,
    helpers::{
        add_overcharge_stacks, canonical_config, install_overcharge_config, run_fixed_update,
        spawn_bolt, spawn_bolt_with_count, spawn_cell, test_app_playing, wire_count_only,
        write_cell_destroyed,
    },
};

// ── Behavior 8 — first kill inserts OverchargeKillCount(1) ───────────────

#[test]
fn first_kill_inserts_count_of_one() {
    let mut app = test_app_playing();
    wire_count_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, Some(bolt));

    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 1);
}

#[test]
fn first_kill_does_not_attach_component_to_victim_cell() {
    // Edge: the component attaches only to the killer Bolt, not the victim.
    let mut app = test_app_playing();
    wire_count_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, Some(bolt));

    run_fixed_update(&mut app);

    assert!(
        app.world().get::<OverchargeKillCount>(cell).is_none(),
        "the victim cell must not receive an OverchargeKillCount"
    );
}

// ── Behavior 9 — subsequent kills accumulate ─────────────────────────────

#[test]
fn subsequent_kills_accumulate() {
    let mut app = test_app_playing();
    wire_count_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);
    let cell_a = spawn_cell(&mut app);
    let cell_b = spawn_cell(&mut app);
    let cell_c = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell_a, Some(bolt));
    write_cell_destroyed(&mut app, cell_b, Some(bolt));
    write_cell_destroyed(&mut app, cell_c, Some(bolt));
    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 3);
}

#[test]
fn subsequent_ticks_with_no_messages_do_not_change_count() {
    // Edge: after accumulating, an empty tick leaves count unchanged.
    let mut app = test_app_playing();
    wire_count_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);
    let cell_a = spawn_cell(&mut app);
    let cell_b = spawn_cell(&mut app);
    let cell_c = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell_a, Some(bolt));
    write_cell_destroyed(&mut app, cell_b, Some(bolt));
    write_cell_destroyed(&mut app, cell_c, Some(bolt));
    run_fixed_update(&mut app);
    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 3);
}

// ── Behavior 10 — killer = None is ignored ───────────────────────────────

#[test]
fn ignores_kills_with_no_killer() {
    let mut app = test_app_playing();
    wire_count_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, None);

    run_fixed_update(&mut app);

    assert!(
        app.world().get::<OverchargeKillCount>(bolt).is_none(),
        "no count should be inserted for environmental kills"
    );
}

#[test]
fn environmental_kill_does_not_attach_to_any_bolt() {
    // Edge: with two bolts present, an environmental kill still attaches
    // the component to neither.
    let mut app = test_app_playing();
    wire_count_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, None);

    run_fixed_update(&mut app);

    assert!(app.world().get::<OverchargeKillCount>(bolt_a).is_none());
    assert!(app.world().get::<OverchargeKillCount>(bolt_b).is_none());
}

// ── Behavior 11 — killer is non-Bolt is ignored ──────────────────────────

#[test]
fn ignores_kills_by_non_bolt_killers() {
    let mut app = test_app_playing();
    wire_count_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let not_bolt = app.world_mut().spawn_empty().id();
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, Some(not_bolt));

    run_fixed_update(&mut app);

    assert!(
        app.world().get::<OverchargeKillCount>(not_bolt).is_none(),
        "killer must be a Bolt to accumulate"
    );
}

#[test]
fn non_bolt_killer_does_not_leak_to_real_bolt() {
    // Edge: with both a non-Bolt and a real Bolt killer, only the real
    // Bolt accumulates.
    let mut app = test_app_playing();
    wire_count_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let not_bolt = app.world_mut().spawn_empty().id();
    let bolt = spawn_bolt(&mut app);
    let cell_a = spawn_cell(&mut app);
    let cell_b = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell_a, Some(not_bolt));
    write_cell_destroyed(&mut app, cell_b, Some(bolt));

    run_fixed_update(&mut app);

    assert!(app.world().get::<OverchargeKillCount>(not_bolt).is_none());
    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 1);
}

// ── Behavior 12 — multi-kill-in-one-tick batches into a single write ────

#[test]
fn multi_kills_in_one_tick_batch_into_single_component_write() {
    let mut app = test_app_playing();
    wire_count_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);
    for _ in 0..5 {
        let cell = spawn_cell(&mut app);
        write_cell_destroyed(&mut app, cell, Some(bolt));
    }

    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 5, "five kills in one tick must sum to 5");
}

#[test]
fn multi_kills_in_one_tick_accumulate_on_preexisting_component() {
    // Edge: the get_mut branch also accumulates the full batch.
    let mut app = test_app_playing();
    wire_count_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt_with_count(&mut app, 2);
    for _ in 0..5 {
        let cell = spawn_cell(&mut app);
        write_cell_destroyed(&mut app, cell, Some(bolt));
    }

    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(bolt).unwrap();
    assert_eq!(count.0, 7, "2 existing + 5 new = 7");
}

// ── Behavior 13 — mixed kills across multiple bolts batch per-bolt ──────

#[test]
fn multi_bolt_kills_batch_per_bolt() {
    let mut app = test_app_playing();
    wire_count_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);
    let cell_0 = spawn_cell(&mut app);
    let cell_1 = spawn_cell(&mut app);
    let cell_2 = spawn_cell(&mut app);
    let cell_3 = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell_0, Some(bolt_a));
    write_cell_destroyed(&mut app, cell_1, Some(bolt_a));
    write_cell_destroyed(&mut app, cell_2, Some(bolt_b));
    write_cell_destroyed(&mut app, cell_3, Some(bolt_a));

    run_fixed_update(&mut app);

    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_a).unwrap().0, 3);
    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_b).unwrap().0, 1);
}

#[test]
fn multi_bolt_kills_unaffected_by_interleaved_environmental_kill() {
    // Edge: an interleaved environmental kill is filtered; per-bolt totals
    // match the previous test.
    let mut app = test_app_playing();
    wire_count_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app);
    let bolt_b = spawn_bolt(&mut app);
    let cell_0 = spawn_cell(&mut app);
    let cell_1 = spawn_cell(&mut app);
    let cell_2 = spawn_cell(&mut app);
    let cell_env = spawn_cell(&mut app);
    let cell_3 = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell_0, Some(bolt_a));
    write_cell_destroyed(&mut app, cell_1, Some(bolt_a));
    write_cell_destroyed(&mut app, cell_2, Some(bolt_b));
    write_cell_destroyed(&mut app, cell_env, None);
    write_cell_destroyed(&mut app, cell_3, Some(bolt_a));

    run_fixed_update(&mut app);

    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_a).unwrap().0, 3);
    assert_eq!(app.world().get::<OverchargeKillCount>(bolt_b).unwrap().0, 1);
}

// ── Behavior 14 — no messages pending is a no-op ─────────────────────────

#[test]
fn no_messages_is_noop() {
    let mut app = test_app_playing();
    wire_count_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);

    run_fixed_update(&mut app);

    assert!(
        app.world().get::<OverchargeKillCount>(bolt).is_none(),
        "no message, no component"
    );
}

#[test]
fn five_empty_ticks_never_attach_component() {
    // Edge: five consecutive empty ticks still leave the bolt without
    // the component.
    let mut app = test_app_playing();
    wire_count_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app);

    for _ in 0..5 {
        run_fixed_update(&mut app);
    }

    assert!(app.world().get::<OverchargeKillCount>(bolt).is_none());
}

// ── Behavior 15 — despawned killer id does not panic ─────────────────────

#[test]
fn despawned_killer_id_is_ignored_without_panic() {
    let mut app = test_app_playing();
    wire_count_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let stale_bolt = spawn_bolt(&mut app);
    app.world_mut().despawn(stale_bolt);
    let cell = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell, Some(stale_bolt));

    run_fixed_update(&mut app);

    // No panic; no component anywhere.
    assert!(
        app.world().get::<OverchargeKillCount>(stale_bolt).is_none(),
        "stale id must not accumulate"
    );
}

#[test]
fn despawned_killer_does_not_contaminate_live_bolt() {
    // Edge: a live bolt in the same tick still gets its own count.
    let mut app = test_app_playing();
    wire_count_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let stale_bolt = spawn_bolt(&mut app);
    app.world_mut().despawn(stale_bolt);
    let live_bolt = spawn_bolt(&mut app);
    let cell_stale = spawn_cell(&mut app);
    let cell_live = spawn_cell(&mut app);
    write_cell_destroyed(&mut app, cell_stale, Some(stale_bolt));
    write_cell_destroyed(&mut app, cell_live, Some(live_bolt));

    run_fixed_update(&mut app);

    let count = app.world().get::<OverchargeKillCount>(live_bolt).unwrap();
    assert_eq!(count.0, 1);
}
