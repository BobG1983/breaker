//! Hazard domain — negative stackable hazards chosen during infinite play
//! at tier 9 and above.

pub mod definition;
pub(crate) mod hazards;
pub(crate) mod messages;
pub(crate) mod plugin;
pub mod resources;
pub(crate) mod systems;

use bevy::prelude::*;

use self::{definition::HazardKind, resources::HazardRegistry};

/// Activate a hazard using the registry-held tuning. Inserts the per-kind
/// config resource so the hazard's runtime systems see fresh values.
///
/// Returns `false` when no definition exists for `kind` — caller decides
/// how to surface that (scenario runner logs a warning and skips).
///
/// Exposed for the scenario runner's `InjectHazardStack` mutation so
/// hazard-runtime scenarios can install a hazard without going through the
/// `HazardSelect` UI. Production code should route selection through
/// `dispatch_hazard_selection` instead.
pub fn activate_from_registry(
    registry: &HazardRegistry,
    kind: HazardKind,
    commands: &mut Commands,
) -> bool {
    let Some(def) = registry.get(kind) else {
        return false;
    };
    hazards::activate(kind, &def.tuning, commands);
    true
}

#[cfg(test)]
mod tests {
    use super::{
        definition::{HazardDefinition, HazardKind, HazardTuning},
        hazards::decay::DecayConfig,
        resources::HazardRegistry,
        *,
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
}
