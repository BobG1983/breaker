use bevy::prelude::*;

use super::super::{
    super::system::momentum_split_check,
    helpers::{
        add_momentum_stacks, canonical_momentum_config, install_momentum_config, test_app_playing,
    },
};

pub(super) fn split_test_app() -> App {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, momentum_split_check);
    install_momentum_config(&mut app, canonical_momentum_config());
    add_momentum_stacks(&mut app, 1);
    app
}
