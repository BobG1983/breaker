use bevy::prelude::*;

use super::super::system::dispatch_bolt_effects;
use crate::bolt::{definition::BoltDefinition, registry::BoltRegistry};

pub(super) const TEST_BOLT_NAME: &str = "TestBolt";

pub(super) fn test_app_with_dispatch(def: BoltDefinition) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let mut registry = BoltRegistry::default();
    registry.insert(def.name.clone(), def);
    app.insert_resource(registry)
        .add_systems(Update, dispatch_bolt_effects);
    app
}
