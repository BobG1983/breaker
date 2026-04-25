//! Group K — `ProtocolPlugin` integration (Behavior K1).
//!
//! Pins that `ProtocolPlugin::build` does not crash with the Afterimage
//! additions, and that `AfterimageConfig` is NOT inserted at plugin-build
//! time (only `activate` inserts it).

use super::super::system::AfterimageConfig;
use crate::{mutators::protocols::plugin::ProtocolPlugin, prelude::*};

// ── K1 — ProtocolPlugin builds with Afterimage additions ──────────────────

#[test]
fn protocol_plugin_build_does_not_crash_with_afterimage_additions() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    app.add_plugins(ProtocolPlugin);

    for _ in 0..3 {
        app.update();
    }

    assert!(
        app.world().get_resource::<AfterimageConfig>().is_none(),
        "ProtocolPlugin::build must NOT insert AfterimageConfig — only activate inserts it"
    );
}
