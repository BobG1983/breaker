//! Hazard domain — negative stackable hazards chosen during infinite play
//! at tier 9 and above.
//!
//! Each per-hazard module owns one hazard's config resource, activation
//! logic, and runtime system(s). Most are scaffold-level today — the
//! headline runtime behaviour lands as cross-domain plumbing (damage
//! pipeline, cell HP churn, breaker-shrink) stabilises.

pub mod definition;
pub(crate) mod messages;
pub mod resources;
pub(crate) mod systems;

pub(crate) mod cascade;
pub(crate) mod decay;
pub(crate) mod diffusion;
pub(crate) mod drift;
pub(crate) mod echo_cells;
pub(crate) mod erosion;
pub(crate) mod fracture;
pub(crate) mod gravity_surge;
pub(crate) mod haste;
pub(crate) mod momentum;
pub(crate) mod overcharge;
pub(crate) mod renewal;
pub(crate) mod resonance;
pub(crate) mod sympathy;
pub(crate) mod tether;
pub(crate) mod volatility;

use bevy::prelude::*;

use self::{
    definition::{HazardKind, HazardTuning},
    resources::HazardRegistry,
};

/// Entry point called by `dispatch_hazard_selection` each time the player
/// picks a hazard. Extracts per-kind tuning into the appropriate config
/// resource (stacking behaviour is handled by `ActiveHazards`).
pub(crate) fn activate(kind: HazardKind, tuning: &HazardTuning, commands: &mut Commands) {
    match kind {
        HazardKind::Decay => decay::activate(tuning, commands),
        HazardKind::Drift => drift::activate(tuning, commands),
        HazardKind::Haste => haste::activate(tuning, commands),
        HazardKind::EchoCells => echo_cells::activate(tuning, commands),
        HazardKind::Erosion => erosion::activate(tuning, commands),
        HazardKind::Cascade => cascade::activate(tuning, commands),
        HazardKind::Fracture => fracture::activate(tuning, commands),
        HazardKind::Renewal => renewal::activate(tuning, commands),
        HazardKind::Volatility => volatility::activate(tuning, commands),
        HazardKind::GravitySurge => gravity_surge::activate(tuning, commands),
        HazardKind::Overcharge => overcharge::activate(tuning, commands),
        HazardKind::Resonance => resonance::activate(tuning, commands),
        HazardKind::Diffusion => diffusion::activate(tuning, commands),
        HazardKind::Tether => tether::activate(tuning, commands),
        HazardKind::Momentum => momentum::activate(tuning, commands),
        HazardKind::Sympathy => sympathy::activate(tuning, commands),
    }
}

/// Fan-out registration — each hazard registers its runtime systems via its
/// own `wire(app)` function.
pub(crate) fn wire(app: &mut App) {
    cascade::wire(app);
    decay::wire(app);
    diffusion::wire(app);
    drift::wire(app);
    echo_cells::wire(app);
    erosion::wire(app);
    fracture::wire(app);
    gravity_surge::wire(app);
    haste::wire(app);
    momentum::wire(app);
    overcharge::wire(app);
    renewal::wire(app);
    resonance::wire(app);
    sympathy::wire(app);
    tether::wire(app);
    volatility::wire(app);
}

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
    activate(kind, &def.tuning, commands);
    true
}

#[cfg(test)]
mod tests {
    use super::{
        decay::DecayConfig,
        definition::{HazardDefinition, HazardKind, HazardTuning},
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
