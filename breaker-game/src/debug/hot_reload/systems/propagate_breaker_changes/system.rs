//! System to propagate `BreakerDefinition` registry changes to live game state.

use bevy::{ecs::system::SystemParam, prelude::*};
use rantzsoft_spatial2d::components::MaxSpeed;

use crate::{
    breaker::{BreakerRegistry, SelectedBreaker, components::*},
    effect::{Target, effects::life_lost::LivesCount},
    prelude::*,
    shared::size::{MaxHeight, MaxWidth, MinHeight, MinWidth},
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
    breaker_chains_query: Query<'w, 's, &'static mut BoundEffects, With<Breaker>>,
    /// Command buffer for entity modifications.
    commands: Commands<'w, 's>,
}

/// Detects when `propagate_registry` has rebuilt the `BreakerRegistry`
/// and if the selected breaker was modified:
/// 1. Re-stamps all definition-derived components on breaker entities
/// 2. Resets `LivesCount` if breaker has `life_pool`
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
                LivesCount(def.life_pool),
            ));
    }

    // Resolve On targets to entity BoundEffects
    // Preserve chip-sourced entries (non-empty chip name), remove definition-sourced
    for mut chains in &mut ctx.breaker_chains_query {
        chains.0.retain(|(chip_name, _)| !chip_name.is_empty());
    }
    for root in &def.effects {
        let RootEffect::On { target, then } = root;
        match target {
            Target::Breaker => {
                for mut chains in &mut ctx.breaker_chains_query {
                    for child in then {
                        chains.0.push((String::new(), child.clone()));
                    }
                }
            }
            // At hot-reload time, bolt/cell/wall targets are not resolved here
            Target::Bolt
            | Target::AllBolts
            | Target::Cell
            | Target::AllCells
            | Target::Wall
            | Target::AllWalls => {}
        }
    }
}
