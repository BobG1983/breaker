//! Cell type definition — RON-deserialized data for a single cell type.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    cells::behaviors::{armored::components::ArmorDirection, phantom::components::PhantomPhase},
    effect_v3::types::RootNode,
};

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

/// Describes how a survival turret fires its salvos.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum AttackPattern {
    /// Single projectile straight down.
    StraightDown,
    /// N projectiles in a downward cone (count >= 2).
    Spread(u32),
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
    /// Cell detonates on death, dealing `damage` to all live cells within `radius`.
    Volatile { damage: f32, radius: f32 },
    /// Cell is part of a numbered sequence group. Only the cell at the
    /// currently-active position accepts damage. When it dies, `position + 1`
    /// becomes active. Group membership and position are pure `u32`;
    /// cross-cell validation (one `position == 0` per group, no duplicate
    /// positions) is a RON-load-time concern and is out of scope here.
    Sequence { group: u32, position: u32 },
    /// Cell absorbs hits from a specific facing direction unless the bolt
    /// carries enough piercing to break through. `value` must be in the
    /// closed range `1..=3` (validated at RON load time); `facing` defaults
    /// to `ArmorDirection::Bottom` at the builder layer.
    Armored { value: u8, facing: ArmorDirection },
    /// Cell cycles through Solid, Telegraph, and Ghost phases, becoming
    /// intangible (collision layers zeroed) during the Ghost phase.
    Phantom {
        cycle_secs:     f32,
        telegraph_secs: f32,
        starting_phase: PhantomPhase,
    },
    /// Cell emits an inverse-square attraction field that pulls bolts
    /// toward its center within `radius`. `strength` is the force
    /// coefficient for the inverse-square formula.
    Magnetic { radius: f32, strength: f32 },
    /// Cell is a turret that periodically fires salvos. Self-destruct
    /// timer starts on first shot. Bolt-immune: bolt collisions do not
    /// deal damage. Bump-vulnerable: breaker collision deals lethal damage.
    Survival {
        /// Attack pattern (`StraightDown` or `Spread`(count)).
        pattern:    AttackPattern,
        /// Self-destruct timer in seconds.
        timer_secs: f32,
    },
    /// Like Survival, but the turret never self-destructs — permanent
    /// turret (boss variant). Still bolt-immune and bump-vulnerable.
    SurvivalPermanent {
        /// Attack pattern (`StraightDown` or `Spread`(count)).
        pattern: AttackPattern,
    },
    /// Cell spawns a portal entity on death. The portal allows the bolt
    /// to enter a sub-node with a tier offset relative to the current node.
    /// `sub_node_tier_offset` may be negative (easier sub-node) or positive
    /// (harder sub-node). No validation constraint on the offset value.
    Portal {
        /// Tier offset for the sub-node. Positive = harder, negative = easier.
        sub_node_tier_offset: i32,
    },
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
    /// Validates that all fields of this definition are well-formed at runtime.
    ///
    /// Checks:
    /// - `alias` must not be empty or the reserved `"."`.
    /// - Each `CellBehavior::Regen { rate }` must have a finite positive rate.
    /// - Each `CellBehavior::Guarded` must pass `GuardedBehavior::validate()`.
    /// - Each `CellBehavior::Volatile { damage, radius }` must have finite positive fields.
    /// - Each `CellBehavior::Armored { value }` must have `value` in `1..=3`.
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
                        crate::shared::validation::positive_finite_f32("Regen rate", *rate)?;
                    }
                    CellBehavior::Guarded(guarded) => {
                        guarded.validate()?;
                    }
                    CellBehavior::Volatile { damage, radius } => {
                        crate::shared::validation::positive_finite_f32("Volatile damage", *damage)?;
                        crate::shared::validation::positive_finite_f32("Volatile radius", *radius)?;
                    }
                    CellBehavior::Sequence { .. } | CellBehavior::Portal { .. } => {
                        // No per-variant validation — all fields are
                        // structurally valid for any value.
                    }
                    CellBehavior::Armored { value, facing: _ } => {
                        if *value == 0 || *value > 3 {
                            return Err(format!("Armored value must be in 1..=3, got {value}"));
                        }
                    }
                    CellBehavior::Phantom {
                        cycle_secs,
                        telegraph_secs,
                        ..
                    } => {
                        use crate::cells::behaviors::phantom::components::PhantomConfig;
                        PhantomConfig {
                            cycle_secs:     *cycle_secs,
                            telegraph_secs: *telegraph_secs,
                        }
                        .validate()?;
                    }
                    CellBehavior::Magnetic { radius, strength } => {
                        crate::shared::validation::positive_finite_f32("Magnetic radius", *radius)?;
                        crate::shared::validation::positive_finite_f32(
                            "Magnetic strength",
                            *strength,
                        )?;
                    }
                    CellBehavior::Survival {
                        timer_secs,
                        pattern,
                    } => {
                        crate::shared::validation::positive_finite_f32(
                            "Survival timer_secs",
                            *timer_secs,
                        )?;
                        if let AttackPattern::Spread(n) = pattern
                            && *n < 2
                        {
                            return Err(format!("Survival Spread count must be >= 2, got {n}"));
                        }
                    }
                    CellBehavior::SurvivalPermanent { pattern } => {
                        if let AttackPattern::Spread(n) = pattern
                            && *n < 2
                        {
                            return Err(format!(
                                "SurvivalPermanent Spread count must be >= 2, got {n}"
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
