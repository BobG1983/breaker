//! Hazard sub-domain fan-out: `activate`, `wire`, `activate_from_registry`.
//!
//! Extracted from `mod.rs` to keep that file wiring-only per the
//! file-splitting rule.

use bevy::prelude::*;

use super::{
    cascade, decay,
    definition::{HazardKind, HazardTuning},
    diffusion, drift, echo_cells, erosion, fracture, gravity_surge, haste, momentum, overcharge,
    renewal, resonance,
    resources::HazardRegistry,
    sympathy, tether, volatility,
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
