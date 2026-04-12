//! System to propagate `BreakerDefinition` registry changes to live game state.

use bevy::{ecs::system::SystemParam, prelude::*};
use rantzsoft_spatial2d::components::MaxSpeed;

use crate::{
    breaker::{BreakerRegistry, SelectedBreaker, components::*},
    prelude::*,
    shared::{
        death_pipeline::Hp,
        size::{MaxHeight, MaxWidth, MinHeight, MinWidth},
    },
};

/// Bundled system parameters for the breaker change propagation system.
#[derive(SystemParam)]
pub(crate) struct BreakerChangeContext<'w, 's> {
    /// Currently selected breaker name.
    selected: Res<'w, SelectedBreaker>,
    /// Breaker registry (rebuilt by `propagate_registry`).
    registry: Res<'w, BreakerRegistry>,
    /// Breaker entities for re-stamping components.
    breaker_query: Query<'w, 's, Entity, With<Breaker>>,
    /// Breaker `BoundEffects` for populating from definition.
    /// Deferred until `effect_v3` `BoundEffects` migration is complete.
    #[allow(
        dead_code,
        reason = "deferred until effect_v3 BoundEffects migration complete"
    )]
    breaker_chains_query: Query<'w, 's, &'static mut BoundEffects, With<Breaker>>,
    /// Command buffer for entity modifications.
    commands: Commands<'w, 's>,
}

/// Detects when `propagate_registry` has rebuilt the `BreakerRegistry`
/// and if the selected breaker was modified:
/// 1. Re-stamps all definition-derived components on breaker entities
/// 2. Inserts `Hp` if breaker has `life_pool`
/// 3. Rebuilds breaker entity `BoundEffects`
pub(crate) fn propagate_breaker_changes(mut ctx: BreakerChangeContext) {
    if !ctx.registry.is_changed() || ctx.registry.is_added() {
        return;
    }

    // Check if the selected breaker exists in the registry
    let Some(def) = ctx.registry.get(&ctx.selected.0) else {
        return;
    };
    let def = def.clone();

    // Re-stamp all definition-derived components on breaker entities.
    // Split into multiple insert calls to stay within Bevy's Bundle tuple arity limit.
    for entity in &ctx.breaker_query {
        ctx.commands
            .entity(entity)
            .insert((
                MaxSpeed(def.max_speed),
                BreakerAcceleration(def.acceleration),
                BreakerDeceleration(def.deceleration),
                DecelEasing {
                    ease: def.decel_ease,
                    strength: def.decel_ease_strength,
                },
                BaseWidth(def.width),
                BaseHeight(def.height),
                MinWidth(def.min_w.unwrap_or(def.width * 0.5)),
                MaxWidth(def.max_w.unwrap_or(def.width * 5.0)),
                MinHeight(def.min_h.unwrap_or(def.height * 0.5)),
                MaxHeight(def.max_h.unwrap_or(def.height * 5.0)),
                BreakerBaseY(def.y_position),
                BreakerReflectionSpread(def.reflection_spread.to_radians()),
            ))
            .insert((
                DashSpeedMultiplier(def.dash_speed_multiplier),
                DashDuration(def.dash_duration),
                DashTilt(def.dash_tilt_angle.to_radians()),
                DashTiltEase(def.dash_tilt_ease),
                BrakeTilt {
                    angle: def.brake_tilt_angle.to_radians(),
                    duration: def.brake_tilt_duration,
                    ease: def.brake_tilt_ease,
                },
                BrakeDecel(def.brake_decel_multiplier),
                SettleDuration(def.settle_duration),
                SettleTiltEase(def.settle_tilt_ease),
            ))
            .insert((
                BumpPerfectWindow(def.perfect_window),
                BumpEarlyWindow(def.early_window),
                BumpLateWindow(def.late_window),
                BumpPerfectCooldown(def.perfect_bump_cooldown),
                BumpWeakCooldown(def.weak_bump_cooldown),
                BumpFeedback {
                    duration: def.bump_visual_duration,
                    peak: def.bump_visual_peak,
                    peak_fraction: def.bump_visual_peak_fraction,
                    rise_ease: def.bump_visual_rise_ease,
                    fall_ease: def.bump_visual_fall_ease,
                },
            ));

        // Insert Hp if breaker has a finite life pool
        if let Some(pool) = def.life_pool {
            #[allow(clippy::cast_precision_loss, reason = "life pool values are small u32")]
            ctx.commands.entity(entity).insert(Hp::new(pool as f32));
        }
    }

    // TODO(effect_v3 migration): Re-stamp breaker definition effects.
    // Hot-reload propagation of effect trees is deferred until all domains
    // use effect_v3 BoundEffects. Currently the entity still has old-domain
    // BoundEffects which is incompatible with new RootNode/Tree types.
    let _ = &def.effects;
}
