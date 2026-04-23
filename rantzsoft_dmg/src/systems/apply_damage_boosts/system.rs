//! `apply_damage_boosts::<T>` — dealer-side damage-boost multiplier stage.
//!
//! Reads `DamageDealt<T>` messages, looks up the dealer's
//! `DamageBoostStack`, and multiplies `msg.amount` by BOTH the persistent-
//! lane aggregate AND the one-shot-lane aggregate (which drains the
//! one-shots as a side effect). Runs in `DmgSystems::ApplyDamageBoosts`
//! inside `FixedUpdate`.

use bevy::prelude::*;

use crate::{components::DamageBoostStack, messages::DamageDealt, traits::Dmgable};

/// For each pending `DamageDealt<T>` with a `dealer`, multiply
/// `msg.amount` by the dealer's
/// `aggregate_persistent() * aggregate_and_consume_one_shots()`. Messages
/// without a dealer (environmental damage) or whose dealer has no stack
/// component are passed through unchanged.
pub(crate) fn apply_damage_boosts<T: Dmgable>(
    mut reader: MessageMutator<DamageDealt<T>>,
    mut stacks: Query<&mut DamageBoostStack>,
) {
    for msg in reader.read() {
        if let Some(dealer) = msg.dealer
            && let Ok(mut stack) = stacks.get_mut(dealer)
        {
            msg.amount *= stack.aggregate_persistent() * stack.aggregate_and_consume_one_shots();
        }
    }
}
