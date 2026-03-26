//! Ramping damage chip effect — accumulates bonus damage per cell hit, resets on breaker bounce.
//!
//! Observes [`RampingDamageApplied`] and inserts or stacks [`RampingDamageState`] on bolt entities.
//! [`increment_ramping_damage`] increases the bonus on cell hits.
//! [`reset_ramping_damage`] resets the bonus on non-bump breaker impacts.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::{
    bolt::{
        components::Bolt,
        messages::{BoltHitBreaker, BoltHitCell},
    },
    breaker::messages::BumpPerformed,
};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a ramping damage passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct RampingDamageApplied {
    /// Damage bonus added per cell hit.
    pub bonus_per_hit: f32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    /// Name of the chip that applied this effect.
    pub chip_name: String,
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Tracks ramping damage state on a bolt entity.
///
/// `current_bonus` starts at 0.0 and increases by `bonus_per_hit` on each cell hit.
/// Resets to 0.0 when the bolt hits the breaker without a bump.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct RampingDamageState {
    /// Current accumulated damage bonus.
    pub current_bonus: f32,
    /// Damage bonus added per cell hit.
    pub bonus_per_hit: f32,
}

// ---------------------------------------------------------------------------
// Observer — inserts / stacks RampingDamageState
// ---------------------------------------------------------------------------

/// Observer: handles ramping damage application — inserts or stacks
/// [`RampingDamageState`] on all bolt entities.
pub(crate) fn handle_ramping_damage(
    trigger: On<RampingDamageApplied>,
    mut query: Query<(Entity, Option<&mut RampingDamageState>), With<Bolt>>,
    mut commands: Commands,
) {
    let event = trigger.event();
    for (entity, existing) in &mut query {
        if let Some(mut state) = existing {
            state.bonus_per_hit += event.bonus_per_hit;
        } else {
            commands.entity(entity).insert(RampingDamageState {
                current_bonus: 0.0,
                bonus_per_hit: event.bonus_per_hit,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Increment system — increases current_bonus on cell hit
// ---------------------------------------------------------------------------

/// Increments `RampingDamageState::current_bonus` by `bonus_per_hit` for each
/// `BoltHitCell` message.
pub(crate) fn increment_ramping_damage(
    mut reader: MessageReader<BoltHitCell>,
    mut query: Query<&mut RampingDamageState>,
) {
    for msg in reader.read() {
        if let Ok(mut state) = query.get_mut(msg.bolt) {
            state.current_bonus += state.bonus_per_hit;
        }
    }
}

// ---------------------------------------------------------------------------
// Reset system — resets current_bonus on non-bump breaker impact
// ---------------------------------------------------------------------------

/// Registers all observers and systems for the ramping damage effect.
pub(crate) fn register(app: &mut App) {
    use crate::{breaker::BreakerSystems, effect::sets::EffectSystems, shared::PlayingState};

    app.add_observer(handle_ramping_damage);

    // Ramping damage increment + reset
    app.add_systems(
        FixedUpdate,
        (
            increment_ramping_damage.after(EffectSystems::Bridge),
            reset_ramping_damage.after(BreakerSystems::GradeBump),
        )
            .run_if(in_state(PlayingState::Active)),
    );
}

/// Resets `RampingDamageState::current_bonus` to 0.0 when a bolt hits the breaker
/// without a corresponding `BumpPerformed` message for the same bolt.
pub(crate) fn reset_ramping_damage(
    mut breaker_reader: MessageReader<BoltHitBreaker>,
    mut bump_reader: MessageReader<BumpPerformed>,
    mut query: Query<&mut RampingDamageState>,
) {
    let bumped: HashSet<Entity> = bump_reader.read().filter_map(|msg| msg.bolt).collect();
    for msg in breaker_reader.read() {
        if !bumped.contains(&msg.bolt)
            && let Ok(mut state) = query.get_mut(msg.bolt)
        {
            state.current_bonus = 0.0;
        }
    }
}
