use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::super::config_impl::*;
use crate::{
    cells::components::Cell,
    shared::{death_pipeline::DamageDealt, test_utils::TestAppBuilder},
};

pub(super) fn piercing_test_app() -> App {
    TestAppBuilder::new()
        .with_message_capture::<DamageDealt<Cell>>()
        .build()
}

pub(super) fn make_config() -> PiercingBeamConfig {
    PiercingBeamConfig {
        damage_mult: OrderedFloat(2.0),
        width:       OrderedFloat(20.0),
    }
}

pub(super) fn geometry_config() -> PiercingBeamConfig {
    PiercingBeamConfig {
        damage_mult: OrderedFloat(1.0),
        width:       OrderedFloat(20.0),
    }
}
