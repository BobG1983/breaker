//! Group I — `ProtocolPlugin` integration (Behavior 59).
//!
//! Pins that `ProtocolPlugin::build` does not crash with the Reckless Dash
//! additions, and that `RecklessDashConfig` is NOT inserted at plugin-build
//! time (only `activate` inserts it).

use super::super::system::RecklessDashConfig;
use crate::{mutators::MutatorsPlugin, prelude::*};

// ── Behavior 59 — ProtocolPlugin builds with Reckless Dash additions ───────-

#[test]
fn protocol_plugin_build_does_not_crash_with_reckless_dash_additions() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    app.add_plugins(MutatorsPlugin);

    for _ in 0..3 {
        app.update();
    }

    // RecklessDashConfig is NOT inserted by plugin build — only activate inserts it.
    assert!(
        app.world().get_resource::<RecklessDashConfig>().is_none(),
        "ProtocolPlugin::build must not insert RecklessDashConfig"
    );
}
