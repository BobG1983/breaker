//! Hazard definition types — `HazardKind`, `HazardTuning`, `HazardDefinition`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Identifies a hazard by its variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HazardKind {
    /// Decay — cell-HP loss over time.
    Decay,
    /// Drift — periodic lateral force on bolts.
    Drift,
    /// Haste — cell-timer acceleration.
    Haste,
    /// `EchoCells` — delayed-respawn cells.
    EchoCells,
    /// Erosion — breaker shrinks on whiff.
    Erosion,
    /// Cascade — per-clear cell-heal.
    Cascade,
    /// Fracture — per-level cell splits.
    Fracture,
    /// Renewal — periodic node refresh.
    Renewal,
    /// Volatility — cell HP grows over time.
    Volatility,
    /// `GravitySurge` — periodic gravity on bolts.
    GravitySurge,
    /// Overcharge — bolt-damage boost and penalty.
    Overcharge,
    /// Resonance — kill chains spawn slow-moving waves toward the Breaker.
    Resonance,
    /// Diffusion — shared cell damage.
    Diffusion,
    /// Tether — paired cell damage sharing.
    Tether,
    /// Momentum — required hits per cell.
    Momentum,
    /// Sympathy — paired cell heal sharing.
    Sympathy,
}

impl HazardKind {
    /// Canonical slice of every variant.
    pub const ALL: &[Self] = &[
        Self::Decay,
        Self::Drift,
        Self::Haste,
        Self::EchoCells,
        Self::Erosion,
        Self::Cascade,
        Self::Fracture,
        Self::Renewal,
        Self::Volatility,
        Self::GravitySurge,
        Self::Overcharge,
        Self::Resonance,
        Self::Diffusion,
        Self::Tether,
        Self::Momentum,
        Self::Sympathy,
    ];

    /// Returns the lowercase, underscore-separated slug used in `SourceId`
    /// construction (`"hazard:<slug>"`).
    #[must_use]
    pub const fn kind_slug(&self) -> &'static str {
        match self {
            Self::Decay => "decay",
            Self::Drift => "drift",
            Self::Haste => "haste",
            Self::EchoCells => "echo_cells",
            Self::Erosion => "erosion",
            Self::Cascade => "cascade",
            Self::Fracture => "fracture",
            Self::Renewal => "renewal",
            Self::Volatility => "volatility",
            Self::GravitySurge => "gravity_surge",
            Self::Overcharge => "overcharge",
            Self::Resonance => "resonance",
            Self::Diffusion => "diffusion",
            Self::Tether => "tether",
            Self::Momentum => "momentum",
            Self::Sympathy => "sympathy",
        }
    }
}

/// Per-hazard tuning data carried by [`HazardDefinition`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum HazardTuning {
    Decay {
        base_percent:      f32,
        per_level_percent: f32,
    },
    Drift {
        force:           f32,
        period_secs:     f32,
        per_level_force: f32,
    },
    Haste {
        base_percent:      f32,
        per_level_percent: f32,
    },
    EchoCells {
        delay_secs:           f32,
        base_hp:              f32,
        per_level_multiplier: f32,
    },
    Erosion {
        shrink_rate:      f32,
        min_width_frac:   f32,
        restore_nonwhiff: f32,
        restore_perfect:  f32,
    },
    Cascade {
        base_heal:      f32,
        per_level_heal: f32,
    },
    Fracture {
        base_splits:      u32,
        per_level_splits: u32,
    },
    Renewal {
        base_period_secs:         f32,
        per_level_reduction_frac: f32,
    },
    Volatility {
        hp_per_interval: f32,
        interval_secs:   f32,
        max_multiplier:  f32,
    },
    GravitySurge {
        base_duration_secs:      f32,
        per_level_duration_secs: f32,
        base_strength:           f32,
        per_level_strength_frac: f32,
    },
    Overcharge {
        base_frac:      f32,
        per_level_frac: f32,
    },
    Resonance {
        /// Kills within the window required before additional kills spawn waves.
        kills_to_trigger:      u32,
        /// Base window (seconds) at stack 1.
        base_window:           f32,
        /// Additional window seconds per stack beyond the first.
        window_per_level:      f32,
        /// Wave travel speed in world units/second.
        wave_speed:            f32,
        /// Base slow duration (seconds) applied to the breaker at stack 1.
        base_slow_duration:    f32,
        /// Base slow strength at stack 1 (0.0..1.0).
        base_slow_strength:    f32,
        /// Logarithmic scaling coefficient on slow duration.
        slow_duration_scaling: f32,
        /// Logarithmic scaling coefficient on slow strength.
        slow_strength_scaling: f32,
        /// World-unit distance at which a wave counts as contacting the breaker.
        contact_threshold:     f32,
        /// Maximum wave lifetime (seconds) before despawn without applying slow.
        wave_max_lifetime:     f32,
    },
    Diffusion {
        base_share_frac:      f32,
        per_level_share_frac: f32,
        depth_every_levels:   u32,
    },
    Tether {
        base_share_frac:         f32,
        per_level_share_frac:    f32,
        base_coverage_frac:      f32,
        per_level_coverage_frac: f32,
    },
    Momentum {
        base_hp_per_hit:      f32,
        per_level_hp_per_hit: f32,
    },
    Sympathy {
        base_heal_frac:      f32,
        per_level_heal_frac: f32,
        depth_every_levels:  u32,
    },
}

impl HazardTuning {
    /// Returns the [`HazardKind`] associated with this tuning.
    #[must_use]
    pub(crate) const fn kind(&self) -> HazardKind {
        match self {
            Self::Decay { .. } => HazardKind::Decay,
            Self::Drift { .. } => HazardKind::Drift,
            Self::Haste { .. } => HazardKind::Haste,
            Self::EchoCells { .. } => HazardKind::EchoCells,
            Self::Erosion { .. } => HazardKind::Erosion,
            Self::Cascade { .. } => HazardKind::Cascade,
            Self::Fracture { .. } => HazardKind::Fracture,
            Self::Renewal { .. } => HazardKind::Renewal,
            Self::Volatility { .. } => HazardKind::Volatility,
            Self::GravitySurge { .. } => HazardKind::GravitySurge,
            Self::Overcharge { .. } => HazardKind::Overcharge,
            Self::Resonance { .. } => HazardKind::Resonance,
            Self::Diffusion { .. } => HazardKind::Diffusion,
            Self::Tether { .. } => HazardKind::Tether,
            Self::Momentum { .. } => HazardKind::Momentum,
            Self::Sympathy { .. } => HazardKind::Sympathy,
        }
    }
}

/// A complete hazard definition — the payload loaded from `.hazard.ron`
/// (future commit) and stored in [`crate::mutators::hazards::resources::HazardRegistry`].
#[derive(Asset, TypePath, Clone, Debug, Serialize, Deserialize)]
pub struct HazardDefinition {
    /// Display name shown on the hazard card.
    pub(crate) name:        String,
    /// Flavor text shown below the name.
    pub(crate) description: String,
    /// Meta-progression unlock tier. Defaults to 0 (always unlocked).
    #[serde(default)]
    pub(crate) unlock_tier: u32,
    /// Per-hazard tuning data.
    pub(crate) tuning:      HazardTuning,
}

impl HazardDefinition {
    /// Returns the [`HazardKind`] of the inner tuning.
    #[must_use]
    pub(crate) const fn kind(&self) -> HazardKind {
        self.tuning.kind()
    }
}
