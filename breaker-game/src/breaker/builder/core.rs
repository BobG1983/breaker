//! Typestate builder for breaker entity construction.
//!
//! Entry point: [`Breaker::builder()`]. The builder prevents invalid
//! combinations at compile time via seven typestate dimensions: Dimensions,
//! Movement, Dashing, Spread, Bump, Visual, and Role. `build()` and `spawn()`
//! are only available when all dimensions are satisfied.

use bevy::{math::curve::easing::EaseFunction, prelude::*};
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{
    MaxSpeed, Position2D, PreviousPosition, PreviousScale, Scale2D, Velocity2D,
};

use crate::{
    breaker::{
        components::{
            BrakeDecel, BrakeTilt, Breaker, BreakerAcceleration, BreakerBaseY, BreakerDeceleration,
            BreakerInitialized, BreakerReflectionSpread, BreakerTilt, BumpEarlyWindow,
            BumpFeedback, BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow, BumpState,
            BumpWeakCooldown, DashDuration, DashSpeedMultiplier, DashState, DashStateTimer,
            DashTilt, DashTiltEase, DecelEasing, ExtraBreaker, PrimaryBreaker, SettleDuration,
            SettleTiltEase,
        },
        definition::BreakerDefinition,
    },
    effect::{EffectCommandsExt, RootEffect, effects::life_lost::LivesCount},
    shared::{
        BOLT_LAYER, BREAKER_LAYER, BaseHeight, BaseWidth, CleanupOnNodeExit, CleanupOnRunEnd,
        GameDrawLayer,
        size::{MaxHeight, MaxWidth, MinHeight, MinWidth},
    },
};

// ── Typestate markers ───────────────────────────────────────────────────────

/// Dimensions not yet configured.
pub struct NoDimensions;
/// Dimensions configured: width, height, y position, and size constraints.
pub struct HasDimensions {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) y_position: f32,
    pub(crate) min_w: f32,
    pub(crate) max_w: f32,
    pub(crate) min_h: f32,
    pub(crate) max_h: f32,
}

/// Movement not yet configured.
pub struct NoMovement;
/// Movement configured: speed, acceleration, deceleration, and easing.
pub struct HasMovement {
    pub(crate) max_speed: f32,
    pub(crate) acceleration: f32,
    pub(crate) deceleration: f32,
    pub(crate) decel_ease: EaseFunction,
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
    pub(crate) mesh: Handle<Mesh>,
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
    pub max_speed: f32,
    /// Horizontal acceleration in world units per second squared.
    pub acceleration: f32,
    /// Horizontal deceleration in world units per second squared.
    pub deceleration: f32,
    /// Easing applied to deceleration based on speed ratio.
    pub decel_ease: EaseFunction,
    /// Strength of eased deceleration.
    pub decel_ease_strength: f32,
}

/// Dash/brake/settle configuration.
#[derive(Clone, Copy)]
pub struct DashSettings {
    /// Dash phase parameters.
    pub dash: DashParams,
    /// Brake phase parameters.
    pub brake: BrakeParams,
    /// Settle phase parameters.
    pub settle: SettleParams,
}

/// Dash phase parameters.
#[derive(Clone, Copy)]
pub struct DashParams {
    /// Speed multiplier relative to max speed.
    pub speed_multiplier: f32,
    /// Duration of the dash in seconds.
    pub duration: f32,
    /// Maximum tilt angle during dash in degrees.
    pub tilt_angle: f32,
    /// Easing for dash tilt ramp-up.
    pub tilt_ease: EaseFunction,
}

/// Brake phase parameters.
#[derive(Clone, Copy)]
pub struct BrakeParams {
    /// Maximum tilt angle during brake in degrees.
    pub tilt_angle: f32,
    /// Duration of the brake tilt ease in seconds.
    pub tilt_duration: f32,
    /// Easing for brake tilt.
    pub tilt_ease: EaseFunction,
    /// Brake deceleration multiplier relative to normal deceleration.
    pub decel_multiplier: f32,
}

/// Settle phase parameters.
#[derive(Clone, Copy)]
pub struct SettleParams {
    /// Duration of the settle phase in seconds.
    pub duration: f32,
    /// Easing for settle tilt return to zero.
    pub tilt_ease: EaseFunction,
}

/// Bump timing and feedback configuration.
#[derive(Clone, Copy)]
pub struct BumpSettings {
    /// Perfect bump timing window in seconds.
    pub perfect_window: f32,
    /// Early bump window in seconds.
    pub early_window: f32,
    /// Late bump window in seconds.
    pub late_window: f32,
    /// Cooldown after a perfect bump in seconds.
    pub perfect_cooldown: f32,
    /// Cooldown after an early/late bump or whiff in seconds.
    pub weak_cooldown: f32,
    /// Visual feedback parameters for the bump pop animation.
    pub feedback: BumpFeedbackSettings,
}

/// Bump pop animation feedback parameters.
#[derive(Clone, Copy)]
pub struct BumpFeedbackSettings {
    /// Duration of the bump pop animation in seconds.
    pub duration: f32,
    /// Maximum Y offset at peak in world units.
    pub peak: f32,
    /// Fraction of duration spent rising (0.0–1.0).
    pub peak_fraction: f32,
    /// Easing for the rise phase.
    pub rise_ease: EaseFunction,
    /// Easing for the fall phase.
    pub fall_ease: EaseFunction,
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
    pub(crate) lives: LivesSetting,
    pub(crate) effects: Option<Vec<RootEffect>>,
    pub(crate) color_rgb: Option<[f32; 3]>,
    pub(crate) override_width: Option<f32>,
    pub(crate) override_height: Option<f32>,
    pub(crate) override_y_position: Option<f32>,
    pub(crate) override_max_speed: Option<f32>,
    pub(crate) override_reflection_spread: Option<f32>,
}

// ── Builder ─────────────────────────────────────────────────────────────────

/// Breaker entity builder with seven typestate dimensions: `D` (Dimensions),
/// `Mv` (Movement), `Da` (Dashing), `Sp` (Spread), `Bm` (Bump), `V` (Visual),
/// and `R` (Role). `build()` and `spawn()` are only available when all
/// dimensions are satisfied.
pub struct BreakerBuilder<D, Mv, Da, Sp, Bm, V, R> {
    dimensions: D,
    movement: Mv,
    dashing: Da,
    spread: Sp,
    bump: Bm,
    visual: V,
    role: R,
    optional: OptionalBreakerData,
}

// ── Entry point ─────────────────────────────────────────────────────────────

impl Breaker {
    /// Creates a breaker builder in the unconfigured state.
    #[must_use]
    pub fn builder()
    -> BreakerBuilder<NoDimensions, NoMovement, NoDashing, NoSpread, NoBump, Unvisual, NoRole> {
        BreakerBuilder {
            dimensions: NoDimensions,
            movement: NoMovement,
            dashing: NoDashing,
            spread: NoSpread,
            bump: NoBump,
            visual: Unvisual,
            role: NoRole,
            optional: OptionalBreakerData::default(),
        }
    }
}

// ── Dimensions transition ───────────────────────────────────────────────────

impl<Mv, Da, Sp, Bm, V, R> BreakerBuilder<NoDimensions, Mv, Da, Sp, Bm, V, R> {
    /// Sets width, height, and `y_position`. Min/max default to 0.5x and 5x base.
    pub fn dimensions(
        self,
        width: f32,
        height: f32,
        y_position: f32,
    ) -> BreakerBuilder<HasDimensions, Mv, Da, Sp, Bm, V, R> {
        BreakerBuilder {
            dimensions: HasDimensions {
                width,
                height,
                y_position,
                min_w: width * 0.5,
                max_w: width * 5.0,
                min_h: height * 0.5,
                max_h: height * 5.0,
            },
            movement: self.movement,
            dashing: self.dashing,
            spread: self.spread,
            bump: self.bump,
            visual: self.visual,
            role: self.role,
            optional: self.optional,
        }
    }
}

// ── Movement transition ─────────────────────────────────────────────────────

impl<D, Da, Sp, Bm, V, R> BreakerBuilder<D, NoMovement, Da, Sp, Bm, V, R> {
    /// Configures movement parameters: speed, acceleration, deceleration.
    pub fn movement(
        self,
        settings: MovementSettings,
    ) -> BreakerBuilder<D, HasMovement, Da, Sp, Bm, V, R> {
        BreakerBuilder {
            dimensions: self.dimensions,
            movement: HasMovement {
                max_speed: settings.max_speed,
                acceleration: settings.acceleration,
                deceleration: settings.deceleration,
                decel_ease: settings.decel_ease,
                decel_ease_strength: settings.decel_ease_strength,
            },
            dashing: self.dashing,
            spread: self.spread,
            bump: self.bump,
            visual: self.visual,
            role: self.role,
            optional: self.optional,
        }
    }
}

// ── Dashing transition ──────────────────────────────────────────────────────

impl<D, Mv, Sp, Bm, V, R> BreakerBuilder<D, Mv, NoDashing, Sp, Bm, V, R> {
    /// Configures dash, brake, and settle parameters.
    pub fn dashing(
        self,
        settings: DashSettings,
    ) -> BreakerBuilder<D, Mv, HasDashing, Sp, Bm, V, R> {
        BreakerBuilder {
            dimensions: self.dimensions,
            movement: self.movement,
            dashing: HasDashing { settings },
            spread: self.spread,
            bump: self.bump,
            visual: self.visual,
            role: self.role,
            optional: self.optional,
        }
    }
}

// ── Spread transition ───────────────────────────────────────────────────────

impl<D, Mv, Da, Bm, V, R> BreakerBuilder<D, Mv, Da, NoSpread, Bm, V, R> {
    /// Sets the reflection spread angle in degrees.
    pub fn spread(self, degrees: f32) -> BreakerBuilder<D, Mv, Da, HasSpread, Bm, V, R> {
        BreakerBuilder {
            dimensions: self.dimensions,
            movement: self.movement,
            dashing: self.dashing,
            spread: HasSpread {
                spread_degrees: degrees,
            },
            bump: self.bump,
            visual: self.visual,
            role: self.role,
            optional: self.optional,
        }
    }
}

// ── Bump transition ─────────────────────────────────────────────────────────

impl<D, Mv, Da, Sp, V, R> BreakerBuilder<D, Mv, Da, Sp, NoBump, V, R> {
    /// Configures bump timing windows, cooldowns, and feedback animation.
    pub fn bump(self, settings: BumpSettings) -> BreakerBuilder<D, Mv, Da, Sp, HasBump, V, R> {
        BreakerBuilder {
            dimensions: self.dimensions,
            movement: self.movement,
            dashing: self.dashing,
            spread: self.spread,
            bump: HasBump { settings },
            visual: self.visual,
            role: self.role,
            optional: self.optional,
        }
    }
}

// ── Visual transitions ──────────────────────────────────────────────────────

impl<D, Mv, Da, Sp, Bm, R> BreakerBuilder<D, Mv, Da, Sp, Bm, Unvisual, R> {
    /// Configures the breaker for rendered mode with mesh and material.
    pub fn rendered(
        self,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) -> BreakerBuilder<D, Mv, Da, Sp, Bm, Rendered, R> {
        let color_rgb = self
            .optional
            .color_rgb
            .unwrap_or(crate::breaker::definition::DEFAULT_COLOR_RGB);
        let color = crate::shared::color_from_rgb(color_rgb);
        BreakerBuilder {
            dimensions: self.dimensions,
            movement: self.movement,
            dashing: self.dashing,
            spread: self.spread,
            bump: self.bump,
            visual: Rendered {
                mesh: meshes.add(Rectangle::new(1.0, 1.0)),
                material: materials.add(ColorMaterial::from_color(color)),
            },
            role: self.role,
            optional: self.optional,
        }
    }

    /// Configures the breaker for headless mode (no rendering components).
    pub fn headless(self) -> BreakerBuilder<D, Mv, Da, Sp, Bm, Headless, R> {
        BreakerBuilder {
            dimensions: self.dimensions,
            movement: self.movement,
            dashing: self.dashing,
            spread: self.spread,
            bump: self.bump,
            visual: Headless,
            role: self.role,
            optional: self.optional,
        }
    }
}

// ── Role transitions ────────────────────────────────────────────────────────

impl<D, Mv, Da, Sp, Bm, V> BreakerBuilder<D, Mv, Da, Sp, Bm, V, NoRole> {
    /// Sets the breaker role to primary (persists across nodes).
    pub fn primary(self) -> BreakerBuilder<D, Mv, Da, Sp, Bm, V, Primary> {
        BreakerBuilder {
            dimensions: self.dimensions,
            movement: self.movement,
            dashing: self.dashing,
            spread: self.spread,
            bump: self.bump,
            visual: self.visual,
            role: Primary,
            optional: self.optional,
        }
    }

    /// Sets the breaker role to extra (cleaned up on node exit).
    pub fn extra(self) -> BreakerBuilder<D, Mv, Da, Sp, Bm, V, Extra> {
        BreakerBuilder {
            dimensions: self.dimensions,
            movement: self.movement,
            dashing: self.dashing,
            spread: self.spread,
            bump: self.bump,
            visual: self.visual,
            role: Extra,
            optional: self.optional,
        }
    }
}

// ── definition() convenience ────────────────────────────────────────────────

impl<V, R> BreakerBuilder<NoDimensions, NoMovement, NoDashing, NoSpread, NoBump, V, R> {
    /// Configure the breaker from a `BreakerDefinition`.
    ///
    /// Transitions Dimensions, Movement, Dashing, Spread, and Bump dimensions
    /// in one call. Also stores lives, effects, and `color_rgb` in optional data.
    ///
    /// Call `.definition()` **before** `.with_*()` overrides and `.rendered()` —
    /// it overwrites lives, effects, and color from the definition.
    pub fn definition(
        mut self,
        def: &BreakerDefinition,
    ) -> BreakerBuilder<HasDimensions, HasMovement, HasDashing, HasSpread, HasBump, V, R> {
        // Store optional data from definition
        self.optional.lives = def
            .life_pool
            .map_or(LivesSetting::Infinite, LivesSetting::Count);
        if !def.effects.is_empty() {
            self.optional.effects = Some(def.effects.clone());
        }
        self.optional.color_rgb = Some(def.color_rgb);

        BreakerBuilder {
            dimensions: HasDimensions {
                width: def.width,
                height: def.height,
                y_position: def.y_position,
                min_w: def.min_w.unwrap_or(def.width * 0.5),
                max_w: def.max_w.unwrap_or(def.width * 5.0),
                min_h: def.min_h.unwrap_or(def.height * 0.5),
                max_h: def.max_h.unwrap_or(def.height * 5.0),
            },
            movement: HasMovement {
                max_speed: def.max_speed,
                acceleration: def.acceleration,
                deceleration: def.deceleration,
                decel_ease: def.decel_ease,
                decel_ease_strength: def.decel_ease_strength,
            },
            dashing: HasDashing {
                settings: DashSettings {
                    dash: DashParams {
                        speed_multiplier: def.dash_speed_multiplier,
                        duration: def.dash_duration,
                        tilt_angle: def.dash_tilt_angle,
                        tilt_ease: def.dash_tilt_ease,
                    },
                    brake: BrakeParams {
                        tilt_angle: def.brake_tilt_angle,
                        tilt_duration: def.brake_tilt_duration,
                        tilt_ease: def.brake_tilt_ease,
                        decel_multiplier: def.brake_decel_multiplier,
                    },
                    settle: SettleParams {
                        duration: def.settle_duration,
                        tilt_ease: def.settle_tilt_ease,
                    },
                },
            },
            spread: HasSpread {
                spread_degrees: def.reflection_spread,
            },
            bump: HasBump {
                settings: BumpSettings {
                    perfect_window: def.perfect_window,
                    early_window: def.early_window,
                    late_window: def.late_window,
                    perfect_cooldown: def.perfect_bump_cooldown,
                    weak_cooldown: def.weak_bump_cooldown,
                    feedback: BumpFeedbackSettings {
                        duration: def.bump_visual_duration,
                        peak: def.bump_visual_peak,
                        peak_fraction: def.bump_visual_peak_fraction,
                        rise_ease: def.bump_visual_rise_ease,
                        fall_ease: def.bump_visual_fall_ease,
                    },
                },
            },
            visual: self.visual,
            role: self.role,
            optional: self.optional,
        }
    }
}

// ── Optional chainable methods (any typestate) ──────────────────────────────

impl<D, Mv, Da, Sp, Bm, V, R> BreakerBuilder<D, Mv, Da, Sp, Bm, V, R> {
    /// Sets the life pool. None = infinite lives, Some(n) = n lives.
    #[must_use]
    pub const fn with_lives(mut self, lives: Option<u32>) -> Self {
        self.optional.lives = match lives {
            Some(n) => LivesSetting::Count(n),
            None => LivesSetting::Infinite,
        };
        self
    }

    /// Sets the effect chains.
    #[must_use]
    pub fn with_effects(mut self, effects: Vec<RootEffect>) -> Self {
        self.optional.effects = Some(effects);
        self
    }

    /// Sets the color RGB (HDR values, may exceed 1.0 for bloom).
    #[must_use]
    pub const fn with_color(mut self, rgb: [f32; 3]) -> Self {
        self.optional.color_rgb = Some(rgb);
        self
    }
}

// ── Override methods (require relevant dimension to be satisfied) ────────

impl<Mv, Da, Sp, Bm, V, R> BreakerBuilder<HasDimensions, Mv, Da, Sp, Bm, V, R> {
    /// Overrides the width set by `.dimensions()` or `.definition()`.
    #[must_use]
    pub const fn with_width(mut self, w: f32) -> Self {
        self.optional.override_width = Some(w);
        self
    }

    /// Overrides the height set by `.dimensions()` or `.definition()`.
    #[must_use]
    pub const fn with_height(mut self, h: f32) -> Self {
        self.optional.override_height = Some(h);
        self
    }

    /// Overrides the `y_position` set by `.dimensions()` or `.definition()`.
    #[must_use]
    pub const fn with_y_position(mut self, y: f32) -> Self {
        self.optional.override_y_position = Some(y);
        self
    }
}

impl<D, Da, Sp, Bm, V, R> BreakerBuilder<D, HasMovement, Da, Sp, Bm, V, R> {
    /// Overrides the `max_speed` set by `.movement()` or `.definition()`.
    #[must_use]
    pub const fn with_max_speed(mut self, speed: f32) -> Self {
        self.optional.override_max_speed = Some(speed);
        self
    }
}

impl<D, Mv, Da, Bm, V, R> BreakerBuilder<D, Mv, Da, HasSpread, Bm, V, R> {
    /// Overrides the reflection spread set by `.spread()` or `.definition()`.
    /// Value is in degrees (will be converted to radians in `build()`).
    #[must_use]
    pub const fn with_reflection_spread(mut self, degrees: f32) -> Self {
        self.optional.override_reflection_spread = Some(degrees);
        self
    }
}

// ── Private helpers ─────────────────────────────────────────────────────────

/// Extracted values from typestate markers, ready for `build_core`.
struct CoreParams {
    // Dimensions
    width: f32,
    height: f32,
    y_position: f32,
    min_w: f32,
    max_w: f32,
    min_h: f32,
    max_h: f32,
    // Movement
    max_speed: f32,
    acceleration: f32,
    deceleration: f32,
    decel_ease: EaseFunction,
    decel_ease_strength: f32,
    // Dashing
    dash: DashSettings,
    // Spread
    spread_degrees: f32,
    // Bump
    bump: BumpSettings,
}

fn core_params_from(
    dims: &HasDimensions,
    mv: &HasMovement,
    da: &HasDashing,
    sp: &HasSpread,
    bm: &HasBump,
    optional: &OptionalBreakerData,
) -> CoreParams {
    let width = optional.override_width.unwrap_or(dims.width);
    let height = optional.override_height.unwrap_or(dims.height);
    let y_position = optional.override_y_position.unwrap_or(dims.y_position);
    let max_speed = optional.override_max_speed.unwrap_or(mv.max_speed);
    let spread_degrees = optional
        .override_reflection_spread
        .unwrap_or(sp.spread_degrees);

    CoreParams {
        width,
        height,
        y_position,
        min_w: dims.min_w,
        max_w: dims.max_w,
        min_h: dims.min_h,
        max_h: dims.max_h,
        max_speed,
        acceleration: mv.acceleration,
        deceleration: mv.deceleration,
        decel_ease: mv.decel_ease,
        decel_ease_strength: mv.decel_ease_strength,
        dash: da.settings,
        spread_degrees,
        bump: bm.settings,
    }
}

/// Builds the core component tuple shared by all terminal states.
fn build_core(params: &CoreParams, optional: &OptionalBreakerData) -> impl Bundle + use<> {
    let lives = match &optional.lives {
        LivesSetting::Unset | LivesSetting::Infinite => None,
        LivesSetting::Count(n) => Some(*n),
    };

    // Core identity + state
    let core = (
        Breaker,
        BreakerInitialized,
        GameDrawLayer::Breaker,
        Velocity2D::default(),
        DashState::default(),
        BreakerTilt::default(),
        BumpState::default(),
        DashStateTimer::default(),
    );

    // Spatial
    let spatial = (
        Position2D(Vec2::new(0.0, params.y_position)),
        PreviousPosition(Vec2::new(0.0, params.y_position)),
    );

    // Scale
    let scale = (
        Scale2D {
            x: params.width,
            y: params.height,
        },
        PreviousScale {
            x: params.width,
            y: params.height,
        },
    );

    // Physics
    let physics = (
        Aabb2D::new(
            Vec2::ZERO,
            Vec2::new(params.width / 2.0, params.height / 2.0),
        ),
        CollisionLayers::new(BREAKER_LAYER, BOLT_LAYER),
    );

    // Dimension stats
    let dimension_stats = (
        BaseWidth(params.width),
        BaseHeight(params.height),
        MinWidth(params.min_w),
        MaxWidth(params.max_w),
        MinHeight(params.min_h),
        MaxHeight(params.max_h),
        BreakerBaseY(params.y_position),
    );

    // Movement stats
    let movement_stats = (
        MaxSpeed(params.max_speed),
        BreakerAcceleration(params.acceleration),
        BreakerDeceleration(params.deceleration),
        DecelEasing {
            ease: params.decel_ease,
            strength: params.decel_ease_strength,
        },
    );

    // Dash stats — note degree-to-radian conversions
    let dash_stats = (
        DashSpeedMultiplier(params.dash.dash.speed_multiplier),
        DashDuration(params.dash.dash.duration),
        DashTilt(params.dash.dash.tilt_angle.to_radians()),
        DashTiltEase(params.dash.dash.tilt_ease),
        BrakeTilt {
            angle: params.dash.brake.tilt_angle.to_radians(),
            duration: params.dash.brake.tilt_duration,
            ease: params.dash.brake.tilt_ease,
        },
        BrakeDecel(params.dash.brake.decel_multiplier),
        SettleDuration(params.dash.settle.duration),
        SettleTiltEase(params.dash.settle.tilt_ease),
    );

    // Spread — degree-to-radian conversion
    let spread_component = BreakerReflectionSpread(params.spread_degrees.to_radians());

    // Bump stats
    let bump_stats = (
        BumpPerfectWindow(params.bump.perfect_window),
        BumpEarlyWindow(params.bump.early_window),
        BumpLateWindow(params.bump.late_window),
        BumpPerfectCooldown(params.bump.perfect_cooldown),
        BumpWeakCooldown(params.bump.weak_cooldown),
        BumpFeedback {
            duration: params.bump.feedback.duration,
            peak: params.bump.feedback.peak,
            peak_fraction: params.bump.feedback.peak_fraction,
            rise_ease: params.bump.feedback.rise_ease,
            fall_ease: params.bump.feedback.fall_ease,
        },
    );

    // Lives — always present
    let lives_component = LivesCount(lives);

    (
        core,
        spatial,
        scale,
        physics,
        dimension_stats,
        movement_stats,
        dash_stats,
        spread_component,
        bump_stats,
        lives_component,
    )
}

// ── build() and spawn() terminal impls ─────────────────────────────────────

impl BreakerBuilder<HasDimensions, HasMovement, HasDashing, HasSpread, HasBump, Rendered, Primary> {
    /// Builds the component bundle for a rendered primary breaker.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let params = core_params_from(
            &self.dimensions,
            &self.movement,
            &self.dashing,
            &self.spread,
            &self.bump,
            &self.optional,
        );
        let core = build_core(&params, &self.optional);
        (
            core,
            PrimaryBreaker,
            CleanupOnRunEnd,
            Mesh2d(self.visual.mesh),
            MeshMaterial2d(self.visual.material),
        )
    }

    /// Spawns a rendered primary breaker entity, including effect dispatch.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let effects = self.optional.effects.clone();
        let entity = commands.spawn(self.build()).id();
        if let Some(effects) = effects.filter(|e| !e.is_empty()) {
            commands.dispatch_initial_effects(effects, None);
        }
        entity
    }
}

impl BreakerBuilder<HasDimensions, HasMovement, HasDashing, HasSpread, HasBump, Rendered, Extra> {
    /// Builds the component bundle for a rendered extra breaker.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let params = core_params_from(
            &self.dimensions,
            &self.movement,
            &self.dashing,
            &self.spread,
            &self.bump,
            &self.optional,
        );
        let core = build_core(&params, &self.optional);
        (
            core,
            ExtraBreaker,
            CleanupOnNodeExit,
            Mesh2d(self.visual.mesh),
            MeshMaterial2d(self.visual.material),
        )
    }

    /// Spawns a rendered extra breaker entity, including effect dispatch.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let effects = self.optional.effects.clone();
        let entity = commands.spawn(self.build()).id();
        if let Some(effects) = effects.filter(|e| !e.is_empty()) {
            commands.dispatch_initial_effects(effects, None);
        }
        entity
    }
}

impl BreakerBuilder<HasDimensions, HasMovement, HasDashing, HasSpread, HasBump, Headless, Primary> {
    /// Builds the component bundle for a headless primary breaker.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let params = core_params_from(
            &self.dimensions,
            &self.movement,
            &self.dashing,
            &self.spread,
            &self.bump,
            &self.optional,
        );
        let core = build_core(&params, &self.optional);
        (core, PrimaryBreaker, CleanupOnRunEnd)
    }

    /// Spawns a headless primary breaker entity, including effect dispatch.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let effects = self.optional.effects.clone();
        let entity = commands.spawn(self.build()).id();
        if let Some(effects) = effects.filter(|e| !e.is_empty()) {
            commands.dispatch_initial_effects(effects, None);
        }
        entity
    }
}

impl BreakerBuilder<HasDimensions, HasMovement, HasDashing, HasSpread, HasBump, Headless, Extra> {
    /// Builds the component bundle for a headless extra breaker.
    #[must_use]
    pub fn build(self) -> impl Bundle {
        let params = core_params_from(
            &self.dimensions,
            &self.movement,
            &self.dashing,
            &self.spread,
            &self.bump,
            &self.optional,
        );
        let core = build_core(&params, &self.optional);
        (core, ExtraBreaker, CleanupOnNodeExit)
    }

    /// Spawns a headless extra breaker entity, including effect dispatch.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let effects = self.optional.effects.clone();
        let entity = commands.spawn(self.build()).id();
        if let Some(effects) = effects.filter(|e| !e.is_empty()) {
            commands.dispatch_initial_effects(effects, None);
        }
        entity
    }
}
