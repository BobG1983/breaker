//! Group J — `ProtocolPlugin` integration (Behavior 38).
//!
//! Pins that `ProtocolPlugin::build` does not crash with Fission additions,
//! and that `FissionCounter` IS initialised by `plugin.rs` (default
//! `{ kills: 0 }`) while `FissionConfig` is NOT (only `activate` inserts it).

use super::super::system::{FissionConfig, FissionCounter};
use crate::{mutators::protocols::plugin::ProtocolPlugin, prelude::*};

// ── Behavior 38 — ProtocolPlugin builds with Fission additions ─────────────-

#[test]
fn plugin_builds() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    app.add_plugins(ProtocolPlugin);

    for _ in 0..3 {
        app.update();
    }

    // FissionConfig is NOT inserted by plugin build — only activate inserts it.
    assert!(
        app.world().get_resource::<FissionConfig>().is_none(),
        "ProtocolPlugin::build must not insert FissionConfig"
    );

    // FissionCounter IS inserted by plugin build at default.
    let counter = app
        .world()
        .get_resource::<FissionCounter>()
        .expect("ProtocolPlugin::build must init_resource::<FissionCounter>()");
    assert_eq!(
        counter.kills, 0,
        "FissionCounter must start at default kills=0; got {}",
        counter.kills
    );
}

// ── Behavior 38 (edge case) — counter is exactly default at plugin-build ───-

#[test]
fn plugin_builds_with_fission_counter_at_default() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    app.add_plugins(ProtocolPlugin);
    app.update();

    let counter = *app.world().resource::<FissionCounter>();
    assert_eq!(
        counter,
        FissionCounter::default(),
        "plugin-build FissionCounter must equal default; got {counter:?}"
    );
}
