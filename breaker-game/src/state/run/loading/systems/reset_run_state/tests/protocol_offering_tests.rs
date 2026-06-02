use super::helpers::test_app;

// ── Wave 2F Group A — ProtocolOfferingCount lifecycle ───────────────────

/// Behavior 1: `reset_run_state` zeroes `ProtocolOfferingCount` from a
/// non-zero prior value.
#[test]
fn reset_run_state_zeroes_protocol_offering_count_from_nonzero() {
    use crate::mutators::protocols::resources::ProtocolOfferingCount;

    let mut app = test_app();
    app.world_mut().insert_resource(ProtocolOfferingCount(7));
    app.world_mut()
        .insert_resource(crate::shared::RunSeed(Some(42)));
    app.update();

    assert_eq!(
        app.world().resource::<ProtocolOfferingCount>().0,
        0,
        "reset_run_state must zero ProtocolOfferingCount(7) to 0"
    );
}

#[test]
fn reset_run_state_zeroes_protocol_offering_count_from_max() {
    // Edge case for Behavior 1: u32::MAX → 0 (proves assignment, not subtraction).
    use crate::mutators::protocols::resources::ProtocolOfferingCount;

    let mut app = test_app();
    app.world_mut()
        .insert_resource(ProtocolOfferingCount(u32::MAX));
    app.update();

    assert_eq!(
        app.world().resource::<ProtocolOfferingCount>().0,
        0,
        "reset_run_state must reset ProtocolOfferingCount(u32::MAX) to 0"
    );
}

/// Behavior 2: `reset_run_state` zeroes `ProtocolOfferingCount` even when
/// `RunSeed` is `None`.
#[test]
fn reset_run_state_zeroes_protocol_offering_count_when_run_seed_is_none() {
    use crate::mutators::protocols::resources::ProtocolOfferingCount;

    let mut app = test_app();
    app.world_mut().insert_resource(ProtocolOfferingCount(5));
    app.world_mut()
        .insert_resource(crate::shared::RunSeed(None));
    app.update();

    assert_eq!(
        app.world().resource::<ProtocolOfferingCount>().0,
        0,
        "reset_run_state must zero ProtocolOfferingCount(5) to 0 even with RunSeed(None)"
    );
}

#[test]
fn reset_run_state_zeroes_protocol_offering_count_edge_case_one() {
    // Edge case for Behavior 2: prior value 1 → 0 (proves reset-to-default,
    // not decrement).
    use crate::mutators::protocols::resources::ProtocolOfferingCount;

    let mut app = test_app();
    app.world_mut().insert_resource(ProtocolOfferingCount(1));
    app.update();

    assert_eq!(
        app.world().resource::<ProtocolOfferingCount>().0,
        0,
        "ProtocolOfferingCount(1) must reset to 0"
    );
}

/// Behavior 3: `reset_run_state` zeroes `ProtocolOfferingCount` when it is
/// already at `0` (idempotent).
#[test]
fn reset_run_state_zeroes_protocol_offering_count_when_already_zero() {
    use crate::mutators::protocols::resources::ProtocolOfferingCount;

    let mut app = test_app();
    app.world_mut()
        .insert_resource(ProtocolOfferingCount::default());
    app.world_mut()
        .insert_resource(crate::shared::RunSeed(Some(42)));
    app.update();

    assert_eq!(
        app.world().resource::<ProtocolOfferingCount>().0,
        0,
        "ProtocolOfferingCount(0) must stay 0 after reset"
    );
}

#[test]
fn reset_run_state_protocol_offering_count_stays_zero_on_double_tick() {
    // Edge case for Behavior 3: two consecutive ticks, count stays 0.
    use crate::mutators::protocols::resources::ProtocolOfferingCount;

    let mut app = test_app();
    app.world_mut()
        .insert_resource(ProtocolOfferingCount::default());
    app.update();
    app.update();

    assert_eq!(
        app.world().resource::<ProtocolOfferingCount>().0,
        0,
        "ProtocolOfferingCount must remain 0 after two ticks of reset_run_state"
    );
}
