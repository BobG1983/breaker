// Wave 2B Group E — BoltPlugin no longer initializes GameRng

use bevy::prelude::*;

use crate::{bolt::BoltPlugin, shared::rng::GameRng};

// Behavior 14: BoltPlugin::build does NOT call init_resource::<GameRng>()
#[test]
fn bolt_plugin_does_not_init_game_rng() {
    // Given: a fresh App with MinimalPlugins + BoltPlugin, and no other code inserts GameRng.
    // When: the plugin's build runs (synchronously during add_plugins).
    // Then: GameRng is absent — BoltPlugin no longer owns GameRng initialization.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(BoltPlugin);

    assert!(
        app.world().get_resource::<GameRng>().is_none(),
        "BoltPlugin must NOT call init_resource::<GameRng>() — GameRng should be absent after \
         add_plugins(BoltPlugin)"
    );
}

// Behavior 14 edge case A: user-provided GameRng is not removed by BoltPlugin
#[test]
fn bolt_plugin_does_not_remove_user_provided_game_rng() {
    // Given: GameRng explicitly inserted BEFORE BoltPlugin.
    // Then: GameRng is still present after add_plugins (BoltPlugin didn't strip it).
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<GameRng>();
    app.add_plugins(BoltPlugin);

    assert!(
        app.world().get_resource::<GameRng>().is_some(),
        "A user-inserted GameRng must survive add_plugins(BoltPlugin)"
    );
}
