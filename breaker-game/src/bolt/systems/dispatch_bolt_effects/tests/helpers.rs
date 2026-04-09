use bevy::prelude::*;

use crate::bolt::{
    definition::BoltDefinition, systems::dispatch_bolt_effects::system::dispatch_bolt_effects,
};

pub(super) const TEST_BOLT_NAME: &str = "TestBolt";

pub(super) fn test_app_with_dispatch(def: BoltDefinition) -> App {
    use crate::shared::test_utils::TestAppBuilder;

    TestAppBuilder::new()
        .with_bolt_registry_entry(&def.name.clone(), def)
        .with_system(Update, dispatch_bolt_effects)
        .build()
}
