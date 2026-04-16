//! Typestate markers, settings structs, builder struct, and optional data.

use bevy::{math::curve::easing::EaseFunction, prelude::*};

use crate::effect_v3::types::RootNode;

// ── Typestate markers ───────────────────────────────────────────────────────

/// Dimensions not yet configured.
pub struct NoDimensions;
/// Dimensions configured: width, height, y position, and size constraints.
pub struct HasDimensions {
    pub(crate) width:      f32,
    pub(crate) height:     f32,
    pub(crate) y_position: f32,
    pub(crate) min_w:      f32,
    pub(crate) max_w:      f32,
    pub(crate) min_h:      f32,
    pub(crate) max_h:      f32,
}

/// Movement not yet configured.
pub struct NoMovement;
/// Movement configured: speed, acceleration, deceleration, and easing.
pub struct HasMovement {
    pub(crate) max_speed:           f32,
    pub(crate) acceleration:        f32,
    pub(crate) deceleration:        f32,
    pub(crate) decel_ease:          EaseFunction,
    pub(crate) decel_ease_strength: f32,
}

/// Dashing not yet configured.
pub struct NoDashing;
/// Dashing configured: dash, brake, and settle parameters.
pub struct HasDashing {
    pub(crate) settings: DashSettings,
}

/// Spread not yet configured.
pub struct NoSpread;
/// Spread configured: reflection spread in degrees.
pub struct HasSpread {
    pub(crate) spread_degrees: f32,
}

/// Bump not yet configured.
pub struct NoBump;
/// Bump configured: timing windows, cooldowns, and feedback.
pub struct HasBump {
    pub(crate) settings: BumpSettings,
}

/// Visual dimension not yet chosen.
pub struct Unvisual;
/// Rendered breaker with mesh and material.
pub struct Rendered {
    pub(crate) mesh:     Handle<Mesh>,
    pub(crate) material: Handle<ColorMaterial>,
}
/// Headless breaker without visual components.
pub struct Headless;

/// Role not yet chosen.
pub struct NoRole;
/// Primary breaker role (persists across nodes).
pub struct Primary;
/// Extra breaker role (cleaned up on node exit).
pub struct Extra;

// ── Settings structs ───────────────────────────────────────────────────────

/// Movement configuration: speed, acceleration, deceleration.
#[derive(Clone, Copy)]
pub struct MovementSettings {
    /// Maximum horizontal speed in world units per second.
    pub max_speed:           f32,
    /// Horizontal acceleration in world units per second squared.
    pub acceleration:        f32,
    /// Horizontal deceleration in world units per second squared.
    pub deceleration:        f32,
    /// Easing applied to deceleration based on speed ratio.
    pub decel_ease:          EaseFunction,
    /// Strength of eased deceleration.
    pub decel_ease_strength: f32,
}

/// Dash/brake/settle configuration.
#[derive(Clone, Copy)]
pub struct DashSettings {
    /// Dash phase parameters.
    pub dash:   DashParams,
    /// Brake phase parameters.
    pub brake:  BrakeParams,
    /// Settle phase parameters.
    pub settle: SettleParams,
}

/// Dash phase parameters.
#[derive(Clone, Copy)]
pub struct DashParams {
    /// Speed multiplier relative to max speed.
    pub speed_multiplier: f32,
    /// Duration of the dash in seconds.
    pub duration:         f32,
    /// Maximum tilt angle during dash in degrees.
    pub tilt_angle:       f32,
    /// Easing for dash tilt ramp-up.
    pub tilt_ease:        EaseFunction,
}

/// Brake phase parameters.
#[derive(Clone, Copy)]
pub struct BrakeParams {
    /// Maximum tilt angle during brake in degrees.
    pub tilt_angle:       f32,
    /// Duration of the brake tilt ease in seconds.
    pub tilt_duration:    f32,
    /// Easing for brake tilt.
    pub tilt_ease:        EaseFunction,
    /// Brake deceleration multiplier relative to normal deceleration.
    pub decel_multiplier: f32,
}

/// Settle phase parameters.
#[derive(Clone, Copy)]
pub struct SettleParams {
    /// Duration of the settle phase in seconds.
    pub duration:  f32,
    /// Easing for settle tilt return to zero.
    pub tilt_ease: EaseFunction,
}

/// Bump timing and feedback configuration.
#[derive(Clone, Copy)]
pub struct BumpSettings {
    /// Perfect bump timing window in seconds.
    pub perfect_window:   f32,
    /// Early bump window in seconds.
    pub early_window:     f32,
    /// Late bump window in seconds.
    pub late_window:      f32,
    /// Cooldown after a perfect bump in seconds.
    pub perfect_cooldown: f32,
    /// Cooldown after an early/late bump or whiff in seconds.
    pub weak_cooldown:    f32,
    /// Visual feedback parameters for the bump pop animation.
    pub feedback:         BumpFeedbackSettings,
}

/// Bump pop animation feedback parameters.
#[derive(Clone, Copy)]
pub struct BumpFeedbackSettings {
    /// Duration of the bump pop animation in seconds.
    pub duration:      f32,
    /// Maximum Y offset at peak in world units.
    pub peak:          f32,
    /// Fraction of duration spent rising (0.0–1.0).
    pub peak_fraction: f32,
    /// Easing for the rise phase.
    pub rise_ease:     EaseFunction,
    /// Easing for the fall phase.
    pub fall_ease:     EaseFunction,
}

// ── Optional data ───────────────────────────────────────────────────────

/// Tri-state for the lives field: not set, explicitly infinite, or a count.
#[derive(Default)]
pub(crate) enum LivesSetting {
    /// Caller has not set lives — use definition value or default to infinite.
    #[default]
    Unset,
    /// Explicitly infinite lives.
    Infinite,
    /// Explicit life count.
    Count(u32),
}

#[derive(Default)]
pub(crate) struct OptionalBreakerData {
    pub(crate) lives:                      LivesSetting,
    pub(crate) effects:                    Option<Vec<RootNode>>,
    pub(crate) color_rgb:                  Option<[f32; 3]>,
    pub(crate) override_width:             Option<f32>,
    pub(crate) override_height:            Option<f32>,
    pub(crate) override_x_position:        Option<f32>,
    pub(crate) override_y_position:        Option<f32>,
    pub(crate) override_max_speed:         Option<f32>,
    pub(crate) override_reflection_spread: Option<f32>,
    pub(crate) bolt_lost:                  Option<RootNode>,
    pub(crate) salvo_hit:                  Option<RootNode>,
}

// ── Builder ─────────────────────────────────────────────────────────────────

/// Breaker entity builder with seven typestate dimensions: `D` (Dimensions),
/// `Mv` (Movement), `Da` (Dashing), `Sp` (Spread), `Bm` (Bump), `V` (Visual),
/// and `R` (Role). `build()` and `spawn()` are only available when all
/// dimensions are satisfied.
pub struct BreakerBuilder<D, Mv, Da, Sp, Bm, V, R> {
    pub(crate) dimensions: D,
    pub(crate) movement:   Mv,
    pub(crate) dashing:    Da,
    pub(crate) spread:     Sp,
    pub(crate) bump:       Bm,
    pub(crate) visual:     V,
    pub(crate) role:       R,
    pub(crate) optional:   OptionalBreakerData,
}
