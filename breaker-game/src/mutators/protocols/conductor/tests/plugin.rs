//! Group P — `ProtocolPlugin` integration (Behavior 29).
//!
//! Pins that `ProtocolPlugin::build` does not crash with the Conductor module
//! wired in, and that `ConductorConfig` is NOT inserted at plugin-build time
//! (only `activate` inserts it).

use super::super::system::ConductorConfig;
use crate::{
    mutators::{MutatorsPlugin, protocols::resources::ActiveProtocols},
    prelude::*,
};

// ── Behavior 29 — ProtocolPlugin builds with Conductor additions ────────────

#[test]
fn protocol_plugin_build_does_not_crash_with_conductor_additions() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    app.add_plugins(MutatorsPlugin);

    for _ in 0..3 {
        app.update();
    }

    // ConductorConfig must NOT be inserted by plugin build — only `activate`
    // inserts it.
    assert!(
        app.world().get_resource::<ConductorConfig>().is_none(),
        "ProtocolPlugin::build must not insert ConductorConfig"
    );

    // ActiveProtocols is either absent, or present-but-empty.
    assert!(
        app.world()
            .get_resource::<ActiveProtocols>()
            .is_none_or(ActiveProtocols::is_empty),
        "ActiveProtocols must be absent or empty on plugin build"
    );
}
