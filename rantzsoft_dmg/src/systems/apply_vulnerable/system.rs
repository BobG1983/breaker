//! `apply_vulnerable::<T>` — target-side vulnerability multiplier stage.
//!
//! Reads `DamageDealt<T>` messages, looks up the target's `VulnerableStack`,
//! and multiplies `msg.amount` by both the persistent-lane aggregate and the
//! one-shot-lane drain. Both aggregates honour any per-entry source filter
//! against `msg.source`. Runs in `DmgSystems::ApplyVulnerable` inside
//! `FixedUpdate`.

use bevy::prelude::*;

use crate::{components::VulnerableStack, messages::DamageDealt, traits::Dmgable};

/// For each pending `DamageDealt<T>`, multiply `msg.amount` by the target's
/// persistent-lane aggregate times the one-shot-lane drain. Both aggregates
/// are scoped to `msg.source`: filtered entries contribute only when their
/// filter equals `msg.source`, and matching one-shots are consumed
/// (non-matching one-shots are preserved on the lane). Messages whose target
/// has no stack component are passed through unchanged.
pub(crate) fn apply_vulnerable<T: Dmgable>(
    mut reader: MessageMutator<DamageDealt<T>>,
    mut stacks: Query<&mut VulnerableStack>,
) {
    for msg in reader.read() {
        if let Ok(mut stack) = stacks.get_mut(msg.target) {
            let emission_source = msg.source.as_ref();
            msg.amount *= stack.aggregate_persistent(emission_source)
                * stack.aggregate_and_consume_one_shots(emission_source);
        }
    }
}
