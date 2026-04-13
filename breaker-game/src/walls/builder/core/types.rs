//! Typestate markers, builder struct, side data trait, and optional data.

use bevy::prelude::*;

pub(in crate::walls::builder) use crate::effect_v3::types::RootNode;

// ── Default half_thickness constant ────────────────────────────────────────

/// Default wall `half_thickness` when no definition or override is provided.
pub(crate) const DEFAULT_HALF_THICKNESS: f32 = 90.0;

// ── Side typestate markers ────────────────────────────────────────────────

/// Side not yet configured.
pub(crate) struct NoSide;

/// Left wall marker — stores playfield data for left edge.
pub(crate) struct Left {
    pub(crate) playfield_left: f32,
    pub(crate) half_height:    f32,
}

/// Right wall marker — stores playfield data for right edge.
pub(crate) struct Right {
    pub(crate) playfield_right: f32,
    pub(crate) half_height:     f32,
}

/// Ceiling wall marker — stores playfield data for top edge.
pub(crate) struct Ceiling {
    pub(crate) playfield_top: f32,
    pub(crate) half_width:    f32,
}

/// Floor wall marker — stores playfield data for bottom edge.
pub(crate) struct Floor {
    pub(crate) playfield_bottom: f32,
    pub(crate) half_width:       f32,
}

// ── Visual typestate markers ──────────────────────────────────────────────

/// No visual components — wall is invisible (default).
pub(crate) struct Invisible;

/// Wall has mesh and material handles for rendering.
#[cfg_attr(
    not(test),
    allow(
        dead_code,
        reason = "constructed by .visible() — test-only until system-param callers exist"
    )
)]
pub(crate) struct Visible {
    pub(crate) mesh:     Handle<Mesh>,
    pub(crate) material: Handle<ColorMaterial>,
}

// ── SideData trait ─────────────────────────────────────────────────────────

/// Trait for computing wall position and half-extents from a resolved
/// `half_thickness` value.
pub(crate) trait SideData {
    /// Compute the world position of this wall given the resolved `half_thickness`.
    fn compute_position(&self, ht: f32) -> Vec2;
    /// Compute the half-extents of this wall given the resolved `half_thickness`.
    fn compute_half_extents(&self, ht: f32) -> Vec2;
}

impl SideData for Left {
    fn compute_position(&self, ht: f32) -> Vec2 {
        Vec2::new(self.playfield_left - ht, 0.0)
    }

    fn compute_half_extents(&self, ht: f32) -> Vec2 {
        Vec2::new(ht, self.half_height)
    }
}

impl SideData for Right {
    fn compute_position(&self, ht: f32) -> Vec2 {
        Vec2::new(self.playfield_right + ht, 0.0)
    }

    fn compute_half_extents(&self, ht: f32) -> Vec2 {
        Vec2::new(ht, self.half_height)
    }
}

impl SideData for Ceiling {
    fn compute_position(&self, ht: f32) -> Vec2 {
        Vec2::new(0.0, self.playfield_top + ht)
    }

    fn compute_half_extents(&self, ht: f32) -> Vec2 {
        Vec2::new(self.half_width, ht)
    }
}

impl SideData for Floor {
    fn compute_position(&self, _ht: f32) -> Vec2 {
        Vec2::new(0.0, self.playfield_bottom)
    }

    fn compute_half_extents(&self, ht: f32) -> Vec2 {
        Vec2::new(self.half_width, ht)
    }
}

// ── Lifetime ───────────────────────────────────────────────────────────────

/// Wall lifetime — builder-only data, NOT a component.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) enum Lifetime {
    /// Wall persists indefinitely.
    #[default]
    Permanent,
    /// Wall despawns after the given duration in seconds.
    #[cfg(test)]
    Timed(f32),
    /// Wall despawns after one rebound.
    #[cfg(test)]
    OneShot,
}

// ── OptionalWallData ───────────────────────────────────────────────────────

/// Optional data stored by chainable builder methods.
#[derive(Default)]
pub(crate) struct OptionalWallData {
    pub(crate) definition_half_thickness: Option<f32>,
    pub(crate) definition_color_rgb:      Option<[f32; 3]>,
    pub(crate) definition_effects:        Option<Vec<RootNode>>,
    pub(crate) override_half_thickness:   Option<f32>,
    pub(crate) override_color_rgb:        Option<[f32; 3]>,
    pub(crate) override_effects:          Option<Vec<RootNode>>,
}

// ── Builder ────────────────────────────────────────────────────────────────

/// Wall entity builder with typestate generics for Side and Visual.
///
/// `S` determines wall placement (Left/Right/Ceiling/Floor).
/// `V` determines visual rendering (Invisible/Visible). Defaults to Invisible.
pub(crate) struct WallBuilder<S, V = Invisible> {
    pub(crate) side:     S,
    pub(crate) optional: OptionalWallData,
    pub(crate) lifetime: Lifetime,
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "read by Visible build — test-only until system-param callers exist"
        )
    )]
    pub(crate) visual:   V,
}
