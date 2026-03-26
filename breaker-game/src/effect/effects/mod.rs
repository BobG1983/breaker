//! Effect handlers for unified behavior leaf effects.

use bevy::prelude::*;

pub(crate) mod attraction;
pub(crate) mod bolt_size_boost;
pub(crate) mod bolt_speed_boost;
pub(crate) mod breaker_speed_boost;
pub(crate) mod bump_force_boost;
pub(crate) mod chain_bolt;
pub(crate) mod chain_hit;
pub(crate) mod chain_lightning;
pub(crate) mod damage_boost;
pub(crate) mod entropy_engine;
pub(crate) mod gravity_well;
pub(crate) mod life_lost;
pub(crate) mod multi_bolt;
pub(crate) mod piercing;
pub(crate) mod piercing_beam;
pub(crate) mod pulse;
pub(crate) mod ramping_damage;
pub(crate) mod random_effect;
pub(crate) mod second_wind;
pub(crate) mod shield;
pub(crate) mod shockwave;
pub(crate) mod spawn_bolt;
pub(crate) mod spawn_phantom;
pub(crate) mod speed_boost;
pub(crate) mod tilt_control_boost;
pub(crate) mod time_penalty;
pub(crate) mod width_boost;

pub(crate) use attraction::handle_attraction;
pub(crate) use bolt_size_boost::handle_bolt_size_boost;
pub(crate) use bolt_speed_boost::handle_bolt_speed_boost;
pub(crate) use breaker_speed_boost::handle_breaker_speed_boost;
pub(crate) use bump_force_boost::handle_bump_force_boost;
pub(crate) use chain_hit::handle_chain_hit;
pub(crate) use damage_boost::handle_damage_boost;
pub(crate) use piercing::handle_piercing;
pub(crate) use tilt_control_boost::handle_tilt_control_boost;
pub(crate) use width_boost::handle_width_boost;

/// Stacks a `u32` component field on an entity.
///
/// - If `per_stack` is 0, this is a no-op regardless of `field`.
/// - If `field` is `Some`, adds `per_stack` when below the cap.
/// - If `field` is `None`, inserts the component with `per_stack` as the initial value.
pub(super) fn stack_u32<C, F>(
    entity: Entity,
    field: Option<&mut u32>,
    per_stack: u32,
    max_stacks: u32,
    commands: &mut Commands,
    constructor: F,
) where
    C: Component,
    F: FnOnce(u32) -> C,
{
    if per_stack == 0 {
        return;
    }
    if let Some(current) = field {
        if *current / per_stack < max_stacks {
            *current += per_stack;
        }
    } else {
        commands.entity(entity).insert(constructor(per_stack));
    }
}

/// Stacks an `f32` component field on an entity.
///
/// - If `per_stack` is 0.0, this is a no-op regardless of `field`.
/// - If `field` is `Some`, adds `per_stack` when below the cap.
/// - If `field` is `None`, inserts the component with `per_stack` as the initial value.
pub(super) fn stack_f32<C, F>(
    entity: Entity,
    field: Option<&mut f32>,
    per_stack: f32,
    max_stacks: u32,
    commands: &mut Commands,
    constructor: F,
) where
    C: Component,
    F: FnOnce(f32) -> C,
{
    if per_stack == 0.0 {
        return;
    }
    if let Some(current) = field {
        // Compare via f64 to avoid u32→f32 precision loss lint.
        if f64::from(*current / per_stack) < f64::from(max_stacks) {
            *current += per_stack;
        }
    } else {
        commands.entity(entity).insert(constructor(per_stack));
    }
}
