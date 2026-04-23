//! `apply_vulnerable::<T>` — target-side vulnerability multiplier stage.
//!
//! Reads `DamageDealt<T>` messages, looks up the target's
//! `VulnerableStack`, and multiplies `msg.amount` by BOTH the persistent-
//! lane aggregate AND the one-shot-lane aggregate (which drains the
//! one-shots as a side effect). Runs in `DmgSystems::ApplyVulnerable`
//! inside `FixedUpdate`.

use bevy::prelude::*;

use crate::{components::VulnerableStack, messages::DamageDealt, traits::Dmgable};

/// For each pending `DamageDealt<T>`, multiply `msg.amount` by the
/// target's `aggregate_persistent() * aggregate_and_consume_one_shots()`.
/// Messages whose target has no stack component are passed through
/// unchanged.
pub(crate) fn apply_vulnerable<T: Dmgable>(
    mut reader: MessageMutator<DamageDealt<T>>,
    mut stacks: Query<&mut VulnerableStack>,
) {
    for msg in reader.read() {
        if let Ok(mut stack) = stacks.get_mut(msg.target) {
            msg.amount *= stack.aggregate_persistent() * stack.aggregate_and_consume_one_shots();
        }
    }
}
