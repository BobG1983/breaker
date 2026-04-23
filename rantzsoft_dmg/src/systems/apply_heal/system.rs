//! `apply_heal::<T>` — applies heal messages to HP with per-message ceiling.
//!
//! Reads `HealDealt<T>` messages and increases `Hp::current`, clamped per
//! message by `HealCap::Starting` or `HealCap::Max`. Skips entities that
//! are `Dead` or `Invulnerable`. Runs in `DmgSystems::ApplyHeal` AFTER
//! `handle_kill::<T>`.
//!
//! ## Numeric guards
//!
//! - `amount <= 0.0` (including `0.0`, negatives, `NEG_INFINITY`): skipped.
//! - `amount.is_nan()`: skipped.
//! - `hp.current >= ceiling`: skipped (over-cap short-circuit — a heal
//!   never lowers HP).

use bevy::prelude::*;

use crate::{
    components::{Dead, HealCap, Hp, Invulnerable},
    messages::HealDealt,
    traits::Dmgable,
};

type HealTargetQuery<'w, 's, T> =
    Query<'w, 's, &'static mut Hp, (With<T>, Without<Dead>, Without<Invulnerable>)>;

/// Apply each `HealDealt<T>` message to its target's `Hp`.
///
/// Skip conditions (silent — the message is simply discarded):
/// - Target carries `Dead` or `Invulnerable` (query filter).
/// - Target is missing `Hp` or the `T` marker (query miss).
/// - `msg.amount <= 0.0` (zero, negative, or `NEG_INFINITY`).
/// - `msg.amount.is_nan()`.
/// - `hp.current >= ceiling` (over-cap short-circuit).
pub(crate) fn apply_heal<T: Dmgable>(
    mut reader: MessageReader<HealDealt<T>>,
    mut targets: HealTargetQuery<T>,
) {
    for msg in reader.read() {
        if msg.amount <= 0.0 {
            continue;
        }
        if msg.amount.is_nan() {
            continue;
        }
        let Ok(mut hp) = targets.get_mut(msg.target) else {
            continue;
        };
        let ceiling = match msg.cap {
            HealCap::Starting => hp.starting,
            HealCap::Max => hp.max.unwrap_or(hp.starting),
        };
        if hp.current >= ceiling {
            continue;
        }
        hp.current = (hp.current + msg.amount).min(ceiling);
    }
}
