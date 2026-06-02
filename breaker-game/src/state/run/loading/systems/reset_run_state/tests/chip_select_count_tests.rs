use super::helpers::test_app;

// ── Wave 2C Group A — ChipSelectCount lifecycle ──────────────────────────

/// Behavior 1: `reset_run_state` zeroes `ChipSelectCount` from a non-zero
/// prior value.
#[test]
fn reset_run_state_zeroes_chip_select_count_from_nonzero() {
    use crate::shared::rng::ChipSelectCount;

    let mut app = test_app();
    app.world_mut().insert_resource(ChipSelectCount(7));
    app.world_mut()
        .insert_resource(crate::shared::RunSeed(Some(42)));
    app.update();

    assert_eq!(
        app.world().resource::<ChipSelectCount>().0,
        0,
        "reset_run_state must zero ChipSelectCount(7) to 0"
    );
}

#[test]
fn reset_run_state_zeroes_chip_select_count_from_max() {
    // Edge case for Behavior 1: u32::MAX → 0 (proves assignment, not subtraction).
    use crate::shared::rng::ChipSelectCount;

    let mut app = test_app();
    app.world_mut().insert_resource(ChipSelectCount(u32::MAX));
    app.update();

    assert_eq!(
        app.world().resource::<ChipSelectCount>().0,
        0,
        "reset_run_state must reset ChipSelectCount(u32::MAX) to 0"
    );
}

/// Behavior 2: `reset_run_state` zeroes `ChipSelectCount` even when
/// `RunSeed` is `None`.
#[test]
fn reset_run_state_zeroes_chip_select_count_when_run_seed_is_none() {
    use crate::shared::rng::ChipSelectCount;

    let mut app = test_app();
    app.world_mut().insert_resource(ChipSelectCount(5));
    app.world_mut()
        .insert_resource(crate::shared::RunSeed(None));
    app.update();

    assert_eq!(
        app.world().resource::<ChipSelectCount>().0,
        0,
        "reset_run_state must zero ChipSelectCount(5) to 0 even with RunSeed(None)"
    );
}

#[test]
fn reset_run_state_zeroes_chip_select_count_edge_case_one() {
    // Edge case for Behavior 2: prior value 1 → 0 (proves reset-to-default,
    // not decrement).
    use crate::shared::rng::ChipSelectCount;

    let mut app = test_app();
    app.world_mut().insert_resource(ChipSelectCount(1));
    app.update();

    assert_eq!(
        app.world().resource::<ChipSelectCount>().0,
        0,
        "ChipSelectCount(1) must reset to 0 (not decrement)"
    );
}

/// Behavior 3: `reset_run_state` zeroes `ChipSelectCount` when already 0
/// (idempotent).
#[test]
fn reset_run_state_zeroes_chip_select_count_when_already_zero() {
    use crate::shared::rng::ChipSelectCount;

    let mut app = test_app();
    app.world_mut().insert_resource(ChipSelectCount::default());
    app.world_mut()
        .insert_resource(crate::shared::RunSeed(Some(42)));
    app.update();

    assert_eq!(
        app.world().resource::<ChipSelectCount>().0,
        0,
        "ChipSelectCount(0) must stay 0 after reset"
    );
}

#[test]
fn reset_run_state_chip_select_count_stays_zero_on_double_tick() {
    // Edge case for Behavior 3: two consecutive ticks, count stays 0.
    use crate::shared::rng::ChipSelectCount;

    let mut app = test_app();
    app.world_mut().insert_resource(ChipSelectCount::default());
    app.update();
    app.update();

    assert_eq!(
        app.world().resource::<ChipSelectCount>().0,
        0,
        "ChipSelectCount must remain 0 after two ticks of reset_run_state"
    );
}

/// Behavior 21 — integration sanity: reset re-zeroes count even after
/// several chip-select entries have advanced it.
#[test]
fn reset_run_state_rezeroes_chip_select_count_after_several_entries() {
    use crate::shared::rng::ChipSelectCount;

    let mut app = test_app();
    app.world_mut().insert_resource(ChipSelectCount(4));
    app.update();

    assert_eq!(
        app.world().resource::<ChipSelectCount>().0,
        0,
        "reset_run_state must re-zero ChipSelectCount(4) representing 4 prior chip selects"
    );
}

#[test]
fn reset_run_state_rezeroes_chip_select_count_edge_already_zero() {
    use crate::shared::rng::ChipSelectCount;

    let mut app = test_app();
    app.world_mut().insert_resource(ChipSelectCount(0));
    app.update();

    assert_eq!(
        app.world().resource::<ChipSelectCount>().0,
        0,
        "reset_run_state must leave ChipSelectCount(0) at 0"
    );
}

#[test]
fn reset_run_state_rezeroes_chip_select_count_from_u32_max() {
    use crate::shared::rng::ChipSelectCount;

    let mut app = test_app();
    app.world_mut().insert_resource(ChipSelectCount(u32::MAX));
    app.update();

    assert_eq!(
        app.world().resource::<ChipSelectCount>().0,
        0,
        "reset_run_state must reset ChipSelectCount(u32::MAX) to 0 without panic"
    );
}
