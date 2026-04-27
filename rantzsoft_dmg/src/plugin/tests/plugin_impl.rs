use bevy::prelude::*;

use crate::RantzDmgPlugin;

// ── Behavior 44: `RantzDmgPlugin` implements `bevy::prelude::Plugin` ──

fn requires_plugin<P: Plugin>(_: P) {}

#[test]
fn impl_plugin_and_app_builds_and_ticks() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzDmgPlugin);
    app.update();

    // Edge case: the trait bound is satisfied at the type level, not
    // only via dyn dispatch.
    requires_plugin(RantzDmgPlugin);
}
