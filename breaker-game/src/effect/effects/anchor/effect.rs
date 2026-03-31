//! Anchor effect -- breaker plants after a stationary delay, modifying bump behavior.

use bevy::prelude::*;

use super::super::bump_force::ActiveBumpForces;
use crate::{
    breaker::components::{BreakerState, BreakerVelocity},
    shared::playing_state::PlayingState,
};

/// Configuration component for the Anchor effect on a breaker entity.
///
/// Stores bump multipliers and the delay before planting occurs.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct AnchorActive {
    /// Bump force multiplier when planted.
    pub bump_force_multiplier: f32,
    /// Perfect window multiplier when planted.
    pub perfect_window_multiplier: f32,
    /// Seconds the breaker must remain stationary before planting.
    pub plant_delay: f32,
}

/// Countdown timer for anchor planting.
///
/// Inserted when the breaker becomes stationary, ticks down each frame.
/// When it reaches zero, `AnchorPlanted` is inserted and this timer is removed.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct AnchorTimer(pub f32);

/// Marker indicating the breaker is anchored (planted).
///
/// Read by the bump system for multiplier application (out of scope here).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnchorPlanted;

/// Fires the Anchor effect on an entity.
///
/// Inserts `AnchorActive` with the given config values. If the entity already
/// has `AnchorActive`, it is overwritten (Bevy's insert replaces existing).
pub(crate) fn fire(
    entity: Entity,
    bump_force_multiplier: f32,
    perfect_window_multiplier: f32,
    plant_delay: f32,
    _source_chip: &str,
    world: &mut World,
) {
    if world.get_entity(entity).is_err() {
        return;
    }
    world.entity_mut(entity).insert(AnchorActive {
        bump_force_multiplier,
        perfect_window_multiplier,
        plant_delay,
    });
}

/// Reverses the Anchor effect on an entity.
///
/// Removes `AnchorActive`, `AnchorTimer`, and `AnchorPlanted` from the entity.
/// Bevy's `remove` is graceful if the component is absent.
pub(crate) fn reverse(
    entity: Entity,
    _bump_force_multiplier: f32,
    _perfect_window_multiplier: f32,
    _plant_delay: f32,
    _source_chip: &str,
    world: &mut World,
) {
    if world.get_entity(entity).is_err() {
        return;
    }
    world
        .entity_mut(entity)
        .remove::<AnchorActive>()
        .remove::<AnchorTimer>()
        .remove::<AnchorPlanted>();
}

/// Query data for the anchor tick system.
type AnchorTickQuery = (
    Entity,
    &'static AnchorActive,
    &'static BreakerVelocity,
    &'static BreakerState,
    Option<&'static mut AnchorTimer>,
    Option<&'static AnchorPlanted>,
    Option<&'static mut ActiveBumpForces>,
);

/// Tick system for anchor planting state machine.
///
/// Watches `BreakerVelocity` and `BreakerState` to manage `AnchorTimer` countdown
/// and `AnchorPlanted` insertion/removal.
///
/// - Stationary: zero velocity AND (Idle or Settling state)
/// - Moving: nonzero velocity OR Dashing/Braking state
pub(crate) fn tick_anchor(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<AnchorTickQuery>,
) {
    let dt = time.delta_secs();
    for (entity, active, velocity, state, timer, planted, active_forces) in &mut query {
        let is_stationary = velocity.x.abs() < f32::EPSILON
            && matches!(state, BreakerState::Idle | BreakerState::Settling);

        if !is_stationary {
            // Movement detected -- cancel timer and planted.
            if timer.is_some() {
                commands.entity(entity).remove::<AnchorTimer>();
            }
            if planted.is_some() {
                // Pop the anchor's bump force multiplier on un-plant
                if let Some(mut forces) = active_forces
                    && let Some(pos) = forces
                        .0
                        .iter()
                        .position(|&v| (v - active.bump_force_multiplier).abs() < f32::EPSILON)
                {
                    forces.0.swap_remove(pos);
                }
                commands.entity(entity).remove::<AnchorPlanted>();
            }
        } else if let Some(mut t) = timer {
            // Stationary with active timer -- tick it down.
            t.0 -= dt;
            if t.0 <= 0.0 {
                commands.entity(entity).remove::<AnchorTimer>();
                commands.entity(entity).insert(AnchorPlanted);
                // Push the anchor's bump force multiplier on plant
                if let Some(mut forces) = active_forces {
                    forces.0.push(active.bump_force_multiplier);
                } else {
                    commands
                        .entity(entity)
                        .insert(ActiveBumpForces(vec![active.bump_force_multiplier]));
                }
            }
        } else if planted.is_none() {
            // Stationary, no timer, not planted -- start the timer.
            commands
                .entity(entity)
                .insert(AnchorTimer(active.plant_delay));
        }
        // else: stationary and already planted -- steady state, do nothing.
    }
}

/// Registers runtime systems for the Anchor effect.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        tick_anchor.run_if(in_state(PlayingState::Active)),
    );
}
