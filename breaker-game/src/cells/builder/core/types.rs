//! Typestate markers, optional data, and builder struct for cell construction.

use bevy::prelude::*;

#[cfg(test)]
use crate::cells::definition::Toughness;
use crate::{
    cells::{components::CellDamageVisuals, definition::CellBehavior},
    effect_v3::types::RootNode,
};

/// Default cell color (linear RGB) -- white.
pub(in crate::cells::builder) const DEFAULT_CELL_COLOR_RGB: [f32; 3] = [1.0, 1.0, 1.0];

// ── Typestate markers ───────────────────────────────────────────────────────

/// Position not yet set.
pub(crate) struct NoPosition;
/// Position configured with a spawn location.
pub(crate) struct HasPosition {
    pub(in crate::cells::builder) pos: Vec2,
}
/// Dimensions not yet set.
pub(crate) struct NoDimensions;
/// Dimensions configured with width and height.
pub(crate) struct HasDimensions {
    pub(in crate::cells::builder) width: f32,
    pub(in crate::cells::builder) height: f32,
}
/// Health not yet set.
pub(crate) struct NoHealth;
/// Health configured with hit points.
pub(crate) struct HasHealth {
    pub(in crate::cells::builder) hp: f32,
}

// ── Visual dimension markers ────────────────────────────────────────────────

/// Visual dimension not yet chosen.
pub(crate) struct Unvisual;
/// Rendered cell with mesh and material.
pub(crate) struct Rendered {
    pub(crate) mesh: Handle<Mesh>,
    pub(crate) material: Handle<ColorMaterial>,
}
/// Headless cell without visual components (test-only — production uses rendered).
#[cfg(test)]
pub(crate) struct Headless;

// ── Optional data ───────────────────────────────────────────────────────────

/// Stores values extracted from a `CellTypeDefinition` via `.definition()`.
pub(in crate::cells::builder) struct CellDefinitionParams {
    pub(in crate::cells::builder) alias: String,
    pub(in crate::cells::builder) required_to_clear: bool,
    pub(in crate::cells::builder) damage_visuals: CellDamageVisuals,
    pub(in crate::cells::builder) behaviors: Vec<CellBehavior>,
    pub(in crate::cells::builder) effects: Option<Vec<RootNode>>,
    pub(in crate::cells::builder) color_rgb: [f32; 3],
}

/// Data for spawning guardian children around a guarded cell.
pub(in crate::cells::builder) struct GuardedSpawnData {
    /// Ring slot indices (0-7) where guardians should be placed.
    pub slots: Vec<u8>,
    /// Guardian cell configuration.
    pub guardian_config: GuardianSpawnConfig,
    /// Pre-computed visual handles for rendered guardians (None for headless).
    pub guardian_visuals: Option<(Handle<Mesh>, Handle<ColorMaterial>)>,
}

/// Configuration for each guardian child entity.
pub(crate) struct GuardianSpawnConfig {
    /// Hit points for the guardian.
    pub(crate) hp: f32,
    /// HDR RGB color for the guardian.
    pub(crate) color_rgb: [f32; 3],
    /// Slide speed in world units per second.
    pub(crate) slide_speed: f32,
    /// Guardian dimension (square: `cell_height` x `cell_height`).
    pub(crate) cell_height: f32,
    /// Horizontal grid step.
    pub(crate) step_x: f32,
    /// Vertical grid step.
    pub(crate) step_y: f32,
}

#[derive(Default)]
pub(in crate::cells::builder) struct OptionalCellData {
    pub(in crate::cells::builder) definition_params: Option<CellDefinitionParams>,
    pub(in crate::cells::builder) override_hp: Option<f32>,
    pub(in crate::cells::builder) alias: Option<String>,
    pub(in crate::cells::builder) required_to_clear: Option<bool>,
    pub(in crate::cells::builder) damage_visuals: Option<CellDamageVisuals>,
    pub(in crate::cells::builder) effects: Option<Vec<RootNode>>,
    pub(in crate::cells::builder) color_rgb: Option<[f32; 3]>,
    pub(in crate::cells::builder) behaviors: Vec<CellBehavior>,
    pub(in crate::cells::builder) locked_entities: Option<Vec<Entity>>,
    pub(in crate::cells::builder) guarded_data: Option<GuardedSpawnData>,
    #[cfg(test)]
    pub(in crate::cells::builder) toughness: Option<Toughness>,
}

// ── Builder ─────────────────────────────────────────────────────────────────

/// Typestate builder for cell entity construction.
pub(crate) struct CellBuilder<P, D, H, V> {
    pub(in crate::cells::builder) position: P,
    pub(in crate::cells::builder) dimensions: D,
    pub(in crate::cells::builder) health: H,
    pub(in crate::cells::builder) visual: V,
    pub(in crate::cells::builder) optional: OptionalCellData,
}

// ── Private helpers ─────────────────────────────────────────────────────────

/// Extracted values from typestate markers, ready for `build_core`.
pub(in crate::cells::builder) struct CoreParams {
    pub(in crate::cells::builder) pos: Vec2,
    pub(in crate::cells::builder) width: f32,
    pub(in crate::cells::builder) height: f32,
    pub(in crate::cells::builder) hp: f32,
}
