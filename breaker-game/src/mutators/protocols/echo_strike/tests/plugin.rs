//! Group J — `ProtocolPlugin` integration (Behavior 64).
//!
//! Pins that `ProtocolPlugin::build` does not crash with Echo Strike
//! additions, and that `EchoStrikeConfig` is NOT inserted at plugin-build
//! time (only `activate` inserts it).

use super::super::system::EchoStrikeConfig;
use crate::{mutators::MutatorsPlugin, prelude::*};

// ── Behavior 64 — ProtocolPlugin builds with Echo Strike additions ─────────-

#[test]
fn protocol_plugin_build_does_not_crash_with_echo_strike_additions() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    app.add_plugins(MutatorsPlugin);

    for _ in 0..3 {
        app.update();
    }

    // EchoStrikeConfig is NOT inserted by plugin build — only activate inserts it.
    assert!(
        app.world().get_resource::<EchoStrikeConfig>().is_none(),
        "ProtocolPlugin::build must not insert EchoStrikeConfig"
    );
}
