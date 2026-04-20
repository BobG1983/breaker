//! Group F — `ProtocolPlugin` integration (Behavior 29).
//!
//! Pins that `ProtocolPlugin::build` does not crash with Iron Curtain
//! additions, and that `IronCurtainConfig` is NOT inserted at plugin-build
//! time (only `activate` inserts it).

use super::super::system::IronCurtainConfig;
use crate::{prelude::*, protocol::plugin::ProtocolPlugin};

// ── Behavior 29 — ProtocolPlugin builds with Iron Curtain additions ────────-

#[test]
fn protocol_plugin_build_does_not_crash_with_iron_curtain_additions() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    app.add_plugins(ProtocolPlugin);

    for _ in 0..3 {
        app.update();
    }

    // IronCurtainConfig is NOT inserted by plugin build — only activate inserts it.
    assert!(
        app.world().get_resource::<IronCurtainConfig>().is_none(),
        "ProtocolPlugin::build must not insert IronCurtainConfig"
    );
}
