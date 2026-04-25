//! Group I — `ProtocolPlugin` integration (Behavior 53).
//!
//! Pins that `ProtocolPlugin::build` does not crash with Debt Collector
//! additions, and that `DebtCollectorConfig` is NOT inserted at plugin-build
//! time (only `activate` inserts it).

use super::super::system::DebtCollectorConfig;
use crate::{mutators::MutatorsPlugin, prelude::*};

// ── Behavior 53 — ProtocolPlugin builds with Debt Collector additions ──────-

#[test]
fn protocol_plugin_build_does_not_crash_with_debt_collector_additions() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    app.add_plugins(MutatorsPlugin);

    for _ in 0..3 {
        app.update();
    }

    // DebtCollectorConfig is NOT inserted by plugin build — only activate inserts it.
    assert!(
        app.world().get_resource::<DebtCollectorConfig>().is_none(),
        "ProtocolPlugin::build must not insert DebtCollectorConfig"
    );
}
