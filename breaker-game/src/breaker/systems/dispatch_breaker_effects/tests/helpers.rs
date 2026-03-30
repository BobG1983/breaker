use bevy::prelude::*;

use super::super::system::dispatch_breaker_effects;
use crate::breaker::{SelectedBreaker, definition::BreakerDefinition, registry::BreakerRegistry};

pub(super) const TEST_BREAKER_NAME: &str = "TestBreaker";

pub(super) fn test_app_with_dispatch(def: BreakerDefinition) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let mut registry = BreakerRegistry::default();
    registry.insert(def.name.clone(), def);
    app.insert_resource(registry)
        .insert_resource(SelectedBreaker(TEST_BREAKER_NAME.to_owned()))
        .add_systems(Update, dispatch_breaker_effects);
    app
}
