//! Terminal build/spawn impls and shared core building helpers.

use bevy::{math::curve::easing::EaseFunction, prelude::*};
use rantzsoft_spatial2d::components::{MaxSpeed, PreviousPosition};

use super::types::*;
use crate::{
    breaker::components::{
        BrakeDecel, BrakeTilt, BreakerAcceleration, BreakerBaseY, BreakerDeceleration,
        BreakerInitialized, BreakerReflectionSpread, BreakerTilt, BumpEarlyWindow, BumpFeedback,
        BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow, BumpState, BumpWeakCooldown,
        DashDuration, DashSpeedMultiplier, DashState, DashStateTimer, DashTilt, DashTiltEase,
        DecelEasing, ExtraBreaker, PrimaryBreaker, SettleDuration, SettleTiltEase,
    },
    effect_v3::{commands::EffectCommandsExt, types::RootNode},
    prelude::*,
    shared::{
        BaseHeight, BaseWidth, GameDrawLayer,
        size::{MaxHeight, MaxWidth, MinHeight, MinWidth},
    },
};

// ── Private helpers ─────────────────────────────────────────────────────────

/// Extracted values from typestate markers, ready for `build_core`.
struct CoreParams {
    // Dimensions
    width:               f32,
    height:              f32,
    x_position:          f32,
    y_position:          f32,
    min_w:               f32,
    max_w:               f32,
    min_h:               f32,
    max_h:               f32,
    // Movement
    max_speed:           f32,
    acceleration:        f32,
    deceleration:        f32,
    decel_ease:          EaseFunction,
    decel_ease_strength: f32,
    // Dashing
    dash:                DashSettings,
    // Spread
    spread_degrees:      f32,
    // Bump
    bump:                BumpSettings,
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
    let x_position = optional.override_x_position.unwrap_or(0.0);
    let y_position = optional.override_y_position.unwrap_or(dims.y_position);
    let max_speed = optional.override_max_speed.unwrap_or(mv.max_speed);
    let spread_degrees = optional
        .override_reflection_spread
        .unwrap_or(sp.spread_degrees);

    CoreParams {
        width,
        height,
        x_position,
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
fn build_core(params: &CoreParams, _optional: &OptionalBreakerData) -> impl Bundle + use<> {
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
        Position2D(Vec2::new(params.x_position, params.y_position)),
        PreviousPosition(Vec2::new(params.x_position, params.y_position)),
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
            ease:     params.decel_ease,
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
            angle:    params.dash.brake.tilt_angle.to_radians(),
            duration: params.dash.brake.tilt_duration,
            ease:     params.dash.brake.tilt_ease,
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
            duration:      params.bump.feedback.duration,
            peak:          params.bump.feedback.peak,
            peak_fraction: params.bump.feedback.peak_fraction,
            rise_ease:     params.bump.feedback.rise_ease,
            fall_ease:     params.bump.feedback.fall_ease,
        },
    );

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
    )
}

/// Insert `Hp` on the entity if the breaker has a finite life pool.
fn apply_hp(commands: &mut Commands, entity: Entity, optional: &OptionalBreakerData) {
    if let LivesSetting::Count(n) = &optional.lives {
        commands.entity(entity).insert(Hp::new(*n as f32));
    }
}

// ── spawn() terminal impls ────────────────────────────────────────────────

impl BreakerBuilder<HasDimensions, HasMovement, HasDashing, HasSpread, HasBump, Rendered, Primary> {
    /// Spawns a rendered primary breaker entity, including effect dispatch.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let effects = self.optional.effects.clone();
        let params = core_params_from(
            &self.dimensions,
            &self.movement,
            &self.dashing,
            &self.spread,
            &self.bump,
            &self.optional,
        );
        let core = build_core(&params, &self.optional);
        let entity = commands
            .spawn((
                core,
                PrimaryBreaker,
                CleanupOnExit::<RunState>::default(),
                GameDrawLayer::Breaker,
                Mesh2d(self.visual.mesh),
                MeshMaterial2d(self.visual.material),
            ))
            .id();
        apply_hp(commands, entity, &self.optional);
        if let Some(effects) = effects.filter(|e| !e.is_empty()) {
            stamp_root_nodes(commands, entity, &effects);
        }
        stamp_required_effects(commands, entity, &self.optional);
        entity
    }
}

impl BreakerBuilder<HasDimensions, HasMovement, HasDashing, HasSpread, HasBump, Rendered, Extra> {
    /// Spawns a rendered extra breaker entity, including effect dispatch.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let effects = self.optional.effects.clone();
        let params = core_params_from(
            &self.dimensions,
            &self.movement,
            &self.dashing,
            &self.spread,
            &self.bump,
            &self.optional,
        );
        let core = build_core(&params, &self.optional);
        let entity = commands
            .spawn((
                core,
                ExtraBreaker,
                CleanupOnExit::<NodeState>::default(),
                GameDrawLayer::Breaker,
                Mesh2d(self.visual.mesh),
                MeshMaterial2d(self.visual.material),
            ))
            .id();
        apply_hp(commands, entity, &self.optional);
        if let Some(effects) = effects.filter(|e| !e.is_empty()) {
            stamp_root_nodes(commands, entity, &effects);
        }
        stamp_required_effects(commands, entity, &self.optional);
        entity
    }
}

impl BreakerBuilder<HasDimensions, HasMovement, HasDashing, HasSpread, HasBump, Headless, Primary> {
    /// Spawns a headless primary breaker entity, including effect dispatch.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let effects = self.optional.effects.clone();
        let params = core_params_from(
            &self.dimensions,
            &self.movement,
            &self.dashing,
            &self.spread,
            &self.bump,
            &self.optional,
        );
        let core = build_core(&params, &self.optional);
        let entity = commands
            .spawn((core, PrimaryBreaker, CleanupOnExit::<RunState>::default()))
            .id();
        apply_hp(commands, entity, &self.optional);
        if let Some(effects) = effects.filter(|e| !e.is_empty()) {
            stamp_root_nodes(commands, entity, &effects);
        }
        stamp_required_effects(commands, entity, &self.optional);
        entity
    }
}

impl BreakerBuilder<HasDimensions, HasMovement, HasDashing, HasSpread, HasBump, Headless, Extra> {
    /// Spawns a headless extra breaker entity, including effect dispatch.
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let effects = self.optional.effects.clone();
        let params = core_params_from(
            &self.dimensions,
            &self.movement,
            &self.dashing,
            &self.spread,
            &self.bump,
            &self.optional,
        );
        let core = build_core(&params, &self.optional);
        let entity = commands
            .spawn((core, ExtraBreaker, CleanupOnExit::<NodeState>::default()))
            .id();
        apply_hp(commands, entity, &self.optional);
        if let Some(effects) = effects.filter(|e| !e.is_empty()) {
            stamp_root_nodes(commands, entity, &effects);
        }
        stamp_required_effects(commands, entity, &self.optional);
        entity
    }
}

/// Stamps `bolt_lost` and `salvo_hit` required-effect trees onto the entity.
fn stamp_required_effects(commands: &mut Commands, entity: Entity, optional: &OptionalBreakerData) {
    for root in optional.bolt_lost.iter().chain(optional.salvo_hit.iter()) {
        if let RootNode::Stamp(_target, tree) = root {
            commands.stamp_effect(entity, String::new(), tree.clone());
        }
        // Spawn root nodes register observers — not handled at builder spawn time.
    }
}

/// For each `Stamp(target, tree)` in `effects`, calls `stamp_effect` on the entity.
/// `Spawn` root nodes are ignored at builder spawn time (they register observers separately).
fn stamp_root_nodes(commands: &mut Commands, entity: Entity, effects: &[RootNode]) {
    for root in effects {
        match root {
            RootNode::Stamp(_target, tree) => {
                commands.stamp_effect(entity, String::new(), tree.clone());
            }
            RootNode::Spawn(..) => {
                // Spawn root nodes register observers — not handled at builder spawn time.
            }
        }
    }
}
