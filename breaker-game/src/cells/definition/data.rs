//! Cell type definition — RON-deserialized data for a single cell type.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::effect_v3::types::RootNode;

/// Categorizes cell durability with a fallback base HP method.
///
/// Production code should always use [`ToughnessConfig`] for HP computation;
/// [`default_base_hp()`](Toughness::default_base_hp) is a hardcoded fallback
/// for tests and scenarios that lack the config resource.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum Toughness {
    Weak,
    #[default]
    Standard,
    Tough,
}

impl Toughness {
    /// Hardcoded fallback base HP for each tier, used only when `ToughnessConfig`
    /// is unavailable (e.g. tests without the resource).
    #[must_use]
    pub(crate) const fn default_base_hp(self) -> f32 {
        match self {
            Self::Weak => 10.0,
            Self::Standard => 20.0,
            Self::Tough => 30.0,
        }
    }
}

/// Configuration for a guarded cell's guardian children.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct GuardedBehavior {
    /// Fraction of parent cell HP assigned to each guardian (0.0, 1.0].
    pub guardian_hp_fraction: f32,
    /// HDR RGB color for guardian cells.
    pub guardian_color_rgb:   [f32; 3],
    /// Slide speed in world units per second.
    pub slide_speed:          f32,
}

impl GuardedBehavior {
    /// Validates that all fields are well-formed.
    ///
    /// Checks:
    /// - `guardian_hp_fraction` must be in (0.0, 1.0].
    /// - `slide_speed` must be non-negative and finite (>= 0.0).
    ///
    /// # Errors
    ///
    /// Returns an error string describing the first invalid field found.
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.guardian_hp_fraction <= 0.0
            || self.guardian_hp_fraction > 1.0
            || !self.guardian_hp_fraction.is_finite()
        {
            return Err(format!(
                "guardian_hp_fraction must be in (0.0, 1.0], got {}",
                self.guardian_hp_fraction
            ));
        }
        if self.slide_speed < 0.0 || !self.slide_speed.is_finite() {
            return Err(format!(
                "slide_speed must be non-negative and finite, got {}",
                self.slide_speed
            ));
        }
        if !self.guardian_color_rgb.iter().all(|v| v.is_finite()) {
            return Err(format!(
                "guardian_color_rgb must be finite, got {:?}",
                self.guardian_color_rgb
            ));
        }
        Ok(())
    }
}

/// Behavioral variants that can be attached to a cell type.
///
/// Each variant represents a distinct runtime behavior. A cell type may have
/// zero or more behaviors via `Option<Vec<CellBehavior>>`.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) enum CellBehavior {
    /// Cell regenerates HP at the given rate per second.
    Regen { rate: f32 },
    /// Cell has guardian children that slide in a ring.
    Guarded(GuardedBehavior),
}

/// A cell type definition loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub(crate) struct CellTypeDefinition {
    /// Unique identifier.
    pub id:                String,
    /// Alias used in node layout grids — may be multi-character.
    pub alias:             String,
    /// Toughness category — determines base HP via `ToughnessConfig`.
    #[serde(default)]
    pub toughness:         Toughness,
    /// HDR RGB color.
    pub color_rgb:         [f32; 3],
    /// Whether this cell counts toward node completion.
    pub required_to_clear: bool,
    /// HDR intensity multiplier for damaged cells at full health.
    pub damage_hdr_base:   f32,
    /// Minimum green channel value for damage color feedback.
    pub damage_green_min:  f32,
    /// Blue channel range added based on health fraction.
    pub damage_blue_range: f32,
    /// Base blue channel value for damage color feedback.
    pub damage_blue_base:  f32,
    /// Optional behavior list (regen, guarded, etc.). Defaults to `None`.
    #[serde(default)]
    pub behaviors:         Option<Vec<CellBehavior>>,
    /// Optional effect chains for this cell type. Defaults to `None`.
    #[serde(default)]
    pub effects:           Option<Vec<RootNode>>,
}

impl CellTypeDefinition {
    /// Cell color as a Bevy [`Color`].
    #[must_use]
    pub(crate) const fn color(&self) -> Color {
        crate::shared::color_from_rgb(self.color_rgb)
    }

    /// Validates that all fields of this definition are well-formed at runtime.
    ///
    /// Checks:
    /// - `alias` must not be empty or the reserved `"."`.
    /// - Each `CellBehavior::Regen { rate }` must have a finite positive rate.
    /// - Each `CellBehavior::Guarded` must pass `GuardedBehavior::validate()`.
    ///
    /// # Errors
    ///
    /// Returns an error string describing the first invalid field found.
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.alias.is_empty() {
            return Err("alias must not be empty".to_owned());
        }
        if self.alias == "." {
            return Err("alias must not be the reserved '.'".to_owned());
        }
        if let Some(ref behaviors) = self.behaviors {
            for behavior in behaviors {
                match behavior {
                    CellBehavior::Regen { rate } => {
                        if *rate <= 0.0 || !rate.is_finite() {
                            return Err(format!(
                                "Regen rate must be positive and finite, got {rate}"
                            ));
                        }
                    }
                    CellBehavior::Guarded(guarded) => {
                        guarded.validate()?;
                    }
                }
            }
        }
        Ok(())
    }
}
