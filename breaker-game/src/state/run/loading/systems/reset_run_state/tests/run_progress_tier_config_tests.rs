use super::helpers::test_app;
use crate::{
    mutators::hazards::definition::types::HazardKind,
    shared::RunSeed,
    state::run::{
        generation::types::TierModifierPool,
        resources::{RunProgress, TierConfig},
    },
};

// ── Wave 1B Group D — reset_run_state zeroes RunProgress and resets TierConfig ──

// Behavior 10: reset_run_state zeroes a non-default RunProgress back to default
#[test]
fn reset_run_state_zeroes_run_progress_from_nonzero() {
    let mut app = test_app();
    app.world_mut().insert_resource(RunProgress {
        tier_index:          5,
        node_in_tier:        3,
        total_nodes_cleared: 42,
    });
    app.world_mut().insert_resource(RunSeed(Some(42)));
    app.update();

    let res = app.world().resource::<RunProgress>();
    assert_eq!(res.tier_index, 0, "tier_index must be zeroed");
    assert_eq!(res.node_in_tier, 0, "node_in_tier must be zeroed");
    assert_eq!(
        res.total_nodes_cleared, 0,
        "total_nodes_cleared must be zeroed"
    );
}

// Behavior 10 edge case: u32::MAX → 0 (proves assignment, not subtraction)
#[test]
fn reset_run_state_zeroes_run_progress_from_max() {
    let mut app = test_app();
    app.world_mut().insert_resource(RunProgress {
        tier_index:          u32::MAX,
        node_in_tier:        u32::MAX,
        total_nodes_cleared: u32::MAX,
    });
    app.world_mut().insert_resource(RunSeed(Some(42)));
    app.update();

    let res = app.world().resource::<RunProgress>();
    assert_eq!(res.tier_index, 0);
    assert_eq!(res.node_in_tier, 0);
    assert_eq!(res.total_nodes_cleared, 0);
}

// Behavior 11: reset_run_state zeroes RunProgress even when RunSeed is None
#[test]
fn reset_run_state_zeroes_run_progress_when_run_seed_is_none() {
    let mut app = test_app();
    app.world_mut().insert_resource(RunProgress {
        tier_index:          2,
        node_in_tier:        1,
        total_nodes_cleared: 7,
    });
    app.world_mut().insert_resource(RunSeed(None));
    app.update();

    let res = app.world().resource::<RunProgress>();
    assert_eq!(res.tier_index, 0);
    assert_eq!(res.node_in_tier, 0);
    assert_eq!(res.total_nodes_cleared, 0);
}

// Behavior 11 edge case: idempotent — second update leaves RunProgress at all-zero
#[test]
fn reset_run_state_zeroes_run_progress_stays_zero_on_double_tick() {
    let mut app = test_app();
    app.world_mut().insert_resource(RunProgress {
        tier_index:          2,
        node_in_tier:        1,
        total_nodes_cleared: 7,
    });
    app.world_mut().insert_resource(RunSeed(None));
    app.update();
    app.update();

    let res = app.world().resource::<RunProgress>();
    assert_eq!(res.tier_index, 0);
    assert_eq!(res.node_in_tier, 0);
    assert_eq!(res.total_nodes_cleared, 0);
}

// Behavior 12: reset_run_state resets TierConfig to its default
#[test]
fn reset_run_state_resets_tier_config_from_nonzero() {
    let mut app = test_app();
    app.world_mut().insert_resource(TierConfig {
        tier_index:           4,
        modifier_pool_handle: None,
        hazard_stack:         vec![HazardKind::Decay, HazardKind::Drift, HazardKind::Volatility],
    });
    app.world_mut().insert_resource(RunSeed(Some(42)));
    app.update();

    let res = app.world().resource::<TierConfig>();
    assert_eq!(res.tier_index, 0);
    assert!(res.modifier_pool_handle.is_none());
    assert!(res.hazard_stack.is_empty());
}

// Behavior 12 edge case: when modifier_pool_handle is Some, post-reset value is None
#[test]
fn reset_run_state_resets_tier_config_clears_modifier_pool_handle() {
    use bevy::prelude::Handle;

    let mut app = test_app();
    app.world_mut().insert_resource(TierConfig {
        tier_index:           2,
        modifier_pool_handle: Some(Handle::<TierModifierPool>::default()),
        hazard_stack:         vec![],
    });
    app.world_mut().insert_resource(RunSeed(Some(42)));
    app.update();

    let res = app.world().resource::<TierConfig>();
    assert_eq!(res.tier_index, 0);
    assert!(
        res.modifier_pool_handle.is_none(),
        "modifier_pool_handle must be None after reset, even when Some before"
    );
    assert!(res.hazard_stack.is_empty());
}

// Behavior 13: reset_run_state resets TierConfig when RunSeed is None
#[test]
fn reset_run_state_resets_tier_config_when_run_seed_is_none() {
    let mut app = test_app();
    app.world_mut().insert_resource(TierConfig {
        tier_index:           9,
        modifier_pool_handle: None,
        hazard_stack:         vec![HazardKind::Sympathy],
    });
    app.world_mut().insert_resource(RunSeed(None));
    app.update();

    let res = app.world().resource::<TierConfig>();
    assert_eq!(res.tier_index, 0);
    assert!(res.modifier_pool_handle.is_none());
    assert!(res.hazard_stack.is_empty());
}

// Behavior 13 edge case: idempotent — second update leaves TierConfig at default
#[test]
fn reset_run_state_resets_tier_config_stays_default_on_double_tick() {
    let mut app = test_app();
    app.world_mut().insert_resource(TierConfig {
        tier_index:           9,
        modifier_pool_handle: None,
        hazard_stack:         vec![HazardKind::Sympathy],
    });
    app.world_mut().insert_resource(RunSeed(None));
    app.update();
    app.update();

    let res = app.world().resource::<TierConfig>();
    assert_eq!(res.tier_index, 0);
    assert!(res.modifier_pool_handle.is_none());
    assert!(res.hazard_stack.is_empty());
}

// Behavior 14: idempotent — after first reset, a second reset still produces defaults.
// Insert non-default sentinels so the no-op stub leaves them non-zero (RED fails).
// After the first update, assert zeroed (proves system ran and reset). After the second
// update, assert still zeroed (proves reset of already-reset values is safe).
#[test]
fn reset_run_state_run_progress_and_tier_config_already_default_stays_default() {
    let mut app = test_app();
    app.world_mut().insert_resource(RunProgress {
        tier_index:          1,
        node_in_tier:        1,
        total_nodes_cleared: 1,
    });
    app.world_mut().insert_resource(TierConfig {
        tier_index:           1,
        modifier_pool_handle: None,
        hazard_stack:         vec![HazardKind::Decay],
    });
    app.world_mut().insert_resource(RunSeed(Some(42)));
    app.update();

    let progress = app.world().resource::<RunProgress>();
    assert_eq!(progress.tier_index, 0);
    assert_eq!(progress.node_in_tier, 0);
    assert_eq!(progress.total_nodes_cleared, 0);

    let cfg = app.world().resource::<TierConfig>();
    assert_eq!(cfg.tier_index, 0);
    assert!(cfg.modifier_pool_handle.is_none());
    assert!(cfg.hazard_stack.is_empty());
}

// Behavior 14 edge case: two consecutive updates leave both resources at defaults.
#[test]
fn reset_run_state_run_progress_and_tier_config_stay_default_after_two_ticks() {
    let mut app = test_app();
    app.world_mut().insert_resource(RunProgress {
        tier_index:          1,
        node_in_tier:        1,
        total_nodes_cleared: 1,
    });
    app.world_mut().insert_resource(TierConfig {
        tier_index:           1,
        modifier_pool_handle: None,
        hazard_stack:         vec![HazardKind::Decay],
    });
    app.world_mut().insert_resource(RunSeed(Some(42)));
    app.update();
    app.update();

    let progress = app.world().resource::<RunProgress>();
    assert_eq!(progress.tier_index, 0);
    assert_eq!(progress.node_in_tier, 0);
    assert_eq!(progress.total_nodes_cleared, 0);

    let cfg = app.world().resource::<TierConfig>();
    assert_eq!(cfg.tier_index, 0);
    assert!(cfg.modifier_pool_handle.is_none());
    assert!(cfg.hazard_stack.is_empty());
}

// ── Wave 1B Group E — structural guard for the integrated reset signature ──

// Behavior 15: source text of system.rs contains ResMut<RunProgress> and ResMut<TierConfig>
// exactly once each, and contains at least one reset assignment for each.
#[test]
fn reset_run_state_body_resets_run_progress_and_tier_config_to_default() {
    let source = include_str!("../system.rs");

    // Use concat! to avoid this file matching itself when grep tools scan the source.
    let run_progress_param = concat!("ResMut<", "RunProgress>");
    let tier_config_param = concat!("ResMut<", "TierConfig>");
    let run_progress_reset = concat!("RunProgress", "::default()");
    let tier_config_reset = concat!("TierConfig", "::default()");

    assert_eq!(
        source.matches(run_progress_param).count(),
        1,
        "system.rs must contain exactly one ResMut<RunProgress> parameter"
    );
    assert_eq!(
        source.matches(tier_config_param).count(),
        1,
        "system.rs must contain exactly one ResMut<TierConfig> parameter"
    );
    assert!(
        source.matches(run_progress_reset).count() >= 1,
        "system.rs must contain at least one RunProgress::default() reset assignment"
    );
    assert!(
        source.matches(tier_config_reset).count() >= 1,
        "system.rs must contain at least one TierConfig::default() reset assignment"
    );
}
