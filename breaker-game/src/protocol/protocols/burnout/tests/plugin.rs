//! Group K — `ProtocolPlugin` integration (Behavior K1).
//!
//! Pins that `ProtocolPlugin::build` does not crash with the Burnout
//! additions, and that `BurnoutConfig` is NOT inserted at plugin-build time
//! (only `activate` inserts it).

use super::super::system::config::BurnoutConfig;
use crate::{prelude::*, protocol::plugin::ProtocolPlugin};

// ── K1 — ProtocolPlugin builds with Burnout additions ──────────────────────-

#[test]
fn protocol_plugin_build_does_not_crash_with_burnout_additions() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    app.add_plugins(ProtocolPlugin);

    for _ in 0..3 {
        app.update();
    }

    assert!(
        app.world().get_resource::<BurnoutConfig>().is_none(),
        "ProtocolPlugin::build must not insert BurnoutConfig"
    );
}
