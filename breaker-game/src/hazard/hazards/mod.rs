//! Per-hazard modules.
//!
//! Each module owns one hazard's config resource, activation logic, and
//! runtime system(s). Most are scaffold-level today — the headline runtime
//! behaviour lands as cross-domain plumbing (damage pipeline, cell HP
//! churn, breaker-shrink) stabilises.

use bevy::prelude::*;

use super::definition::{HazardKind, HazardTuning};

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
/// own `register(app)` function.
pub(crate) fn register(app: &mut App) {
    cascade::register(app);
    decay::register(app);
    diffusion::register(app);
    drift::register(app);
    echo_cells::register(app);
    erosion::register(app);
    fracture::register(app);
    gravity_surge::register(app);
    haste::register(app);
    momentum::register(app);
    overcharge::register(app);
    renewal::register(app);
    resonance::register(app);
    sympathy::register(app);
    tether::register(app);
    volatility::register(app);
}
