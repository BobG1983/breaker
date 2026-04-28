//! `apply_damage_boosts::<T>` — dealer-side damage-boost multiplier stage.
//!
//! Reads `DamageDealt<T>` messages, looks up the dealer's
//! `DamageBoostStack`, and multiplies `msg.amount` by BOTH the persistent-
//! lane aggregate AND the one-shot-lane aggregate (which drains the
//! matching one-shots as a side effect). Both aggregates honour any
//! per-entry source filter against `msg.source`. Runs in
//! `DmgSystems::ApplyDamageBoosts` inside `FixedUpdate`.

use bevy::prelude::*;

use crate::{components::DamageBoostStack, messages::DamageDealt, traits::Dmgable};

/// For each pending `DamageDealt<T>` with a `dealer`, multiply `msg.amount`
/// by the dealer's persistent-lane aggregate times the one-shot-lane drain.
/// Both aggregates are scoped to `msg.source`: filtered entries contribute
/// only when their filter equals `msg.source`, and matching one-shots are
/// consumed (non-matching one-shots are preserved on the lane). Messages
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
            let emission_source = msg.source.as_ref();
            msg.amount *= stack.aggregate_persistent(emission_source)
                * stack.aggregate_and_consume_one_shots(emission_source);
        }
    }
}
