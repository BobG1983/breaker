use bevy::prelude::*;

use super::{
    activate_from_registry,
    decay::DecayConfig,
    definition::{HazardDefinition, HazardKind, HazardTuning},
    resources::HazardRegistry,
};
use crate::prelude::TestAppBuilder;

fn decay_def() -> HazardDefinition {
    HazardDefinition {
        name:        "Decay".into(),
        description: String::new(),
        unlock_tier: 0,
        tuning:      HazardTuning::Decay {
            base_percent:      15.0,
            per_level_percent: 5.0,
        },
    }
}

#[test]
fn activate_from_registry_inserts_config_for_known_kind() {
    let mut app = TestAppBuilder::new().build();

    let mut registry = HazardRegistry::default();
    registry.insert(decay_def());
    app.world_mut().insert_resource(registry);

    app.add_systems(
        Update,
        |registry: Res<HazardRegistry>, mut commands: Commands| {
            let ok = activate_from_registry(&registry, HazardKind::Decay, &mut commands);
            assert!(ok, "known kind should return true");
        },
    );
    app.update();

    let cfg = app.world().resource::<DecayConfig>();
    assert!((cfg.base_percent - 15.0).abs() < f32::EPSILON);
    assert!((cfg.per_level_percent - 5.0).abs() < f32::EPSILON);
}

#[test]
fn activate_from_registry_returns_false_for_missing_kind() {
    let mut app = TestAppBuilder::new().build();
    app.world_mut().insert_resource(HazardRegistry::default());

    app.add_systems(
        Update,
        |registry: Res<HazardRegistry>, mut commands: Commands| {
            let ok = activate_from_registry(&registry, HazardKind::Decay, &mut commands);
            assert!(!ok, "missing kind should return false");
        },
    );
    app.update();

    assert!(
        app.world().get_resource::<DecayConfig>().is_none(),
        "no config should be inserted when definition is absent"
    );
}
