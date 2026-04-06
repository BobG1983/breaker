//! Terminal build/spawn impls and shared core building helpers.

use bevy::{math::curve::easing::EaseFunction, prelude::*};
use rantzsoft_lifecycle::CleanupOnExit;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{
    MaxSpeed, Position2D, PreviousPosition, PreviousScale, Scale2D, Velocity2D,
};

use super::types::*;
use crate::{
    breaker::components::{
        BrakeDecel, BrakeTilt, Breaker, BreakerAcceleration, BreakerBaseY, BreakerDeceleration,
        BreakerInitialized, BreakerReflectionSpread, BreakerTilt, BumpEarlyWindow, BumpFeedback,
        BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow, BumpState, BumpWeakCooldown,
        DashDuration, DashSpeedMultiplier, DashState, DashStateTimer, DashTilt, DashTiltEase,
        DecelEasing, ExtraBreaker, PrimaryBreaker, SettleDuration, SettleTiltEase,
    },
    effect::{EffectCommandsExt, effects::life_lost::LivesCount},
    shared::{
        BOLT_LAYER, BREAKER_LAYER, BaseHeight, BaseWidth, GameDrawLayer,
        size::{MaxHeight, MaxWidth, MinHeight, MinWidth},
    },
    state::types::{NodeState, RunState},
};

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

    // Core identity + state (does NOT include GameDrawLayer — added by Rendered builds only)
    let core = (
        Breaker,
        BreakerInitialized,
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
            CleanupOnExit::<RunState>::default(),
            GameDrawLayer::Breaker,
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
            CleanupOnExit::<NodeState>::default(),
            GameDrawLayer::Breaker,
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
        (core, PrimaryBreaker, CleanupOnExit::<RunState>::default())
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
        (core, ExtraBreaker, CleanupOnExit::<NodeState>::default())
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
