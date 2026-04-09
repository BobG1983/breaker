use bevy::prelude::*;

use crate::{
    bolt::{definition::BoltDefinition, registry::BoltRegistry},
    debug::hot_reload::systems::propagate_bolt_definition::system::propagate_bolt_definition,
};

pub(super) const TEST_BOLT_NAME: &str = "TestBolt";

/// Creates a minimal `BoltDefinition` with standard values.
pub(super) fn make_bolt_def(name: &str) -> BoltDefinition {
    BoltDefinition {
        name: name.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    }
}

/// Creates a test app with the `propagate_bolt_definition` system.
pub(super) fn test_app() -> App {
    use crate::shared::test_utils::TestAppBuilder;

    TestAppBuilder::new()
        .with_resource::<BoltRegistry>()
        .with_system(Update, propagate_bolt_definition)
        .build()
}

/// Seeds the registry with a definition and flushes the Added state.
///
/// Returns the app ready for mutation-triggered propagation.
pub(super) fn seed_and_flush(app: &mut App, def: BoltDefinition) {
    {
        let mut registry = app.world_mut().resource_mut::<BoltRegistry>();
        registry.insert(def.name.clone(), def);
    }
    // Flush Added change detection (two updates to clear is_added)
    app.update();
    app.update();
}

/// Mutates the registry by clearing and re-inserting a definition.
pub(super) fn mutate_registry(app: &mut App, def: BoltDefinition) {
    let mut registry = app.world_mut().resource_mut::<BoltRegistry>();
    registry.clear();
    registry.insert(def.name.clone(), def);
}
