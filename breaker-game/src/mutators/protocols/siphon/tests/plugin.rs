//! Group H — `ProtocolPlugin` integration (Behaviors 43–44).
//!
//! Pins that `ProtocolPlugin::build` initialises `SiphonStreak` (as
//! `init_resource::<SiphonStreak>()`) and that the full plugin builds and
//! updates without crashing.

use super::super::system::{SiphonConfig, SiphonStreak};
use crate::{mutators::protocols::plugin::ProtocolPlugin, prelude::*};

// ── Behavior 43 — ProtocolPlugin::build initialises SiphonStreak ────────────

#[test]
fn protocol_plugin_initialises_siphon_streak() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    app.add_plugins(ProtocolPlugin);
    app.update();

    let streak = app
        .world()
        .get_resource::<SiphonStreak>()
        .expect("ProtocolPlugin::build should init_resource::<SiphonStreak>()");
    assert_eq!(
        *streak,
        SiphonStreak::default(),
        "initialised SiphonStreak should equal default; got {streak:?}"
    );
}

// ── Behavior 44 — ProtocolPlugin::build does not crash with Siphon additions ─

#[test]
fn protocol_plugin_build_does_not_crash_with_siphon_additions() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    app.add_plugins(ProtocolPlugin);

    for _ in 0..3 {
        app.update();
    }

    // SiphonStreak remains at default.
    let streak = app
        .world()
        .get_resource::<SiphonStreak>()
        .expect("SiphonStreak must be initialised by ProtocolPlugin");
    assert_eq!(
        *streak,
        SiphonStreak::default(),
        "SiphonStreak should remain default across three updates; got {streak:?}"
    );

    // SiphonConfig stays absent — plugin does not insert it; only activate does.
    assert!(
        app.world().get_resource::<SiphonConfig>().is_none(),
        "ProtocolPlugin::build must not insert SiphonConfig"
    );
}
