//! Group D — `ProtocolPlugin` integration (Behavior 24).
//!
//! Pins that `ProtocolPlugin::build` does not panic with `TierRegression`
//! additions and that it does NOT insert `TierRegressionConfig` or
//! `TierRegressionPending` at plugin build time (both are activation-driven,
//! not standing per-run defaults).

use super::super::system::{TierRegressionConfig, TierRegressionPending};
use crate::{prelude::*, protocol::plugin::ProtocolPlugin};

// ── 24 — ProtocolPlugin build does not insert tier-regression resources ────-

#[test]
fn protocol_plugin_build_does_not_insert_tier_regression_resources() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    app.add_plugins(ProtocolPlugin);

    for _ in 0..3 {
        app.update();
    }

    assert!(
        app.world().get_resource::<TierRegressionConfig>().is_none(),
        "ProtocolPlugin::build must NOT insert TierRegressionConfig — only activate inserts it"
    );
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
        "ProtocolPlugin::build must NOT init_resource::<TierRegressionPending>() — \
         pending is activation-driven, not a standing per-run default (mirror of \
         BurnoutConfig / ConductorConfig pattern)"
    );
}

// ── 24 edge — both resources must be absent after the very first tick too ──-

#[test]
fn protocol_plugin_build_does_not_insert_tier_regression_resources_on_first_tick() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    app.add_plugins(ProtocolPlugin);
    app.update();

    assert!(app.world().get_resource::<TierRegressionConfig>().is_none(),);
    assert!(
        app.world()
            .get_resource::<TierRegressionPending>()
            .is_none(),
    );
}
