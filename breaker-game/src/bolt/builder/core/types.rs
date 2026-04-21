use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::effect_v3::{storage::BoundEffects, types::Tree};

/// Default bolt radius when neither `definition()` nor `with_radius()` is called.
pub(in crate::bolt::builder) const DEFAULT_RADIUS: f32 = 8.0;

/// Default bolt color (linear RGB) — bright teal, used when no definition or override provides one.
pub(crate) const DEFAULT_BOLT_COLOR_RGB: [f32; 3] = [6.0, 5.0, 0.5];

// ── Typestate markers ───────────────────────────────────────────────────────

/// Position not yet set.
pub struct NoPosition;
/// Position configured with a spawn location.
pub struct HasPosition {
    pub(in crate::bolt::builder) pos: Vec2,
}
/// Speed not yet set.
pub struct NoSpeed;
/// Speed configured with base, min, and max values.
pub struct HasSpeed {
    pub(in crate::bolt::builder) base: f32,
    pub(in crate::bolt::builder) min:  f32,
    pub(in crate::bolt::builder) max:  f32,
}
/// Angle constraints not yet set.
pub struct NoAngle;
/// Angle constraints configured with horizontal and vertical minimums.
pub struct HasAngle {
    pub(in crate::bolt::builder) min_angle_h: f32,
    pub(in crate::bolt::builder) min_angle_v: f32,
}
/// Motion mode not yet chosen.
pub struct NoMotion;
/// Bolt is held above the breaker, waiting for the player to bump.
pub struct Serving;
/// Bolt has an explicit velocity and launches immediately.
pub struct HasVelocity {
    pub(in crate::bolt::builder) vel: Velocity2D,
}
/// Role not yet chosen.
pub struct NoRole;
/// Primary bolt — cleaned up on run end, triggers bolt-lost on despawn.
pub struct Primary;
/// Extra bolt — cleaned up on node exit, no bolt-lost trigger.
pub struct Extra;

// ── Visual dimension markers ───────────────────────────────────────────────

/// Visual dimension not yet chosen.
pub struct Unvisual;
/// Rendered bolt with mesh and material.
pub struct Rendered {
    pub(crate) mesh:     Handle<Mesh>,
    pub(crate) material: Handle<ColorMaterial>,
}
/// Headless bolt without visual components.
pub struct Headless;

// ── Optional data ───────────────────────────────────────────────────────────

#[derive(Default)]
pub(in crate::bolt::builder) struct OptionalBoltData {
    /// Origin tag for telemetry and debug logs. Set via `.spawned_by(...)`.
    pub(in crate::bolt::builder) spawned_by:               Option<String>,
    /// Seconds-to-live. Spawns a `Lifespan` component when present.
    pub(in crate::bolt::builder) lifespan:                 Option<f32>,
    /// Explicit radius override. Wins over the definition's `(min, max)` radius
    /// range and over `DEFAULT_RADIUS`.
    pub(in crate::bolt::builder) radius:                   Option<f32>,
    /// `BoundEffects` copied verbatim from a parent bolt (e.g. split children
    /// inherit the parent's `effect_v3` bindings). Applied during terminal build.
    pub(in crate::bolt::builder) inherited_effects:        Option<BoundEffects>,
    /// Additional effect trees layered on top of `inherited_effects` and
    /// definition-default effects. Set via `.with_effects(...)`.
    pub(in crate::bolt::builder) with_effects:             Option<Vec<(String, Tree)>>,
    /// Snapshot of a `BoltDefinition`'s tuning values (name, base damage, angle
    /// spread, spawn offset, radius range). Set via `.definition(...)`.
    pub(in crate::bolt::builder) definition_params:        Option<BoltDefinitionParams>,
    /// Caller-provided base damage that overrides `definition_params.base_damage`.
    pub(in crate::bolt::builder) override_base_damage:     Option<f32>,
    /// Caller-provided definition name that overrides `definition_params.name`.
    pub(in crate::bolt::builder) override_definition_name: Option<String>,
    /// Caller-provided angle spread that overrides `definition_params.angle_spread`.
    pub(in crate::bolt::builder) override_angle_spread:    Option<f32>,
    /// Caller-provided spawn-Y offset that overrides `definition_params.spawn_offset_y`.
    pub(in crate::bolt::builder) override_spawn_offset_y:  Option<f32>,
    /// Linear-RGB color override. Falls back to `DEFAULT_BOLT_COLOR_RGB`.
    pub(in crate::bolt::builder) color_rgb:                Option<[f32; 3]>,
    /// `true` once the terminal build has emitted a `BoltBirthed` message for
    /// this entity. Used to guard against double-birth in split-and-respawn
    /// paths.
    pub(in crate::bolt::builder) birthed:                  bool,
    /// Extra bits `ORed` into the default `CollisionLayers` mask.
    ///
    /// The default mask is `CELL_LAYER | WALL_LAYER | BREAKER_LAYER`. Callers
    /// that need additional mask bits (e.g. the afterimage protocol phantom
    /// bolt, which also matches `BOLT_LAYER`) set them via
    /// [`BoltBuilder::with_extra_mask_bits`].
    pub(in crate::bolt::builder) extra_mask_bits:          u32,
}

pub(in crate::bolt::builder) struct BoltDefinitionParams {
    pub(in crate::bolt::builder) name:           String,
    pub(in crate::bolt::builder) base_damage:    f32,
    pub(in crate::bolt::builder) angle_spread:   f32,
    pub(in crate::bolt::builder) spawn_offset_y: f32,
    pub(in crate::bolt::builder) min_radius:     Option<f32>,
    pub(in crate::bolt::builder) max_radius:     Option<f32>,
}

// ── Builder ─────────────────────────────────────────────────────────────────

/// Typestate builder for bolt entity construction.
pub struct BoltBuilder<P, S, A, M, R, V> {
    pub(in crate::bolt::builder) position: P,
    pub(in crate::bolt::builder) speed:    S,
    pub(in crate::bolt::builder) angle:    A,
    pub(in crate::bolt::builder) motion:   M,
    pub(in crate::bolt::builder) role:     R,
    pub(in crate::bolt::builder) visual:   V,
    pub(in crate::bolt::builder) optional: OptionalBoltData,
}

// ── Private helpers ─────────────────────────────────────────────────────────

/// Extracted values from typestate markers, ready for `build_core`.
pub(in crate::bolt::builder) struct CoreParams {
    pub(in crate::bolt::builder) pos:         Vec2,
    pub(in crate::bolt::builder) base_speed:  f32,
    pub(in crate::bolt::builder) min_speed:   f32,
    pub(in crate::bolt::builder) max_speed:   f32,
    pub(in crate::bolt::builder) min_angle_h: f32,
    pub(in crate::bolt::builder) min_angle_v: f32,
    pub(in crate::bolt::builder) vel:         Velocity2D,
}
