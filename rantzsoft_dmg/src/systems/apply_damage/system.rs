//! `apply_damage::<T>` — applies final damage to HP and attributes kills.
//!
//! Reads `DamageDealt<T>` messages (amounts already mutated by upstream
//! stages), decrements `Hp::current`, and — on the killing blow — inserts
//! `KilledBy { killer }` via `Commands`. Runs in `DmgSystems::ApplyDamage`
//! inside `FixedUpdate`, chained strictly after `invulnerable_filter::<T>`.

use bevy::prelude::*;

use crate::{
    components::{Dead, Hp, KilledBy},
    messages::DamageDealt,
    traits::Dmgable,
};

/// Decrement HP by the final message amount. Issue `KilledBy` insertion
/// on the killing-blow message (the one that crosses `hp.current` from
/// positive to `<= 0.0`). First-kill-wins is intrinsic: subsequent
/// same-tick messages against the same target observe
/// `was_positive = false` because `hp.current` is already non-positive.
///
/// Targets that are already `Dead`, or do not carry the `T` marker, are
/// silently skipped via the query's `With<T>` + `Without<Dead>` filter.
/// Targets missing `Hp` are likewise skipped (query miss).
///
/// **Invulnerability is NOT filtered here.** The query shape deliberately
/// omits `Without<Invulnerable>` — upstream `invulnerable_filter::<T>`
/// (chained earlier in the same `DmgSystems::ApplyDamage` set) zeroes the
/// message amount for invulnerable targets before `apply_damage` sees it.
/// Do not add an `Invulnerable` filter to this query: the B153 contract
/// test pins the current shape.
pub(crate) fn apply_damage<T: Dmgable>(
    mut reader: MessageReader<DamageDealt<T>>,
    mut targets: Query<&mut Hp, (With<T>, Without<Dead>)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let Ok(mut hp) = targets.get_mut(msg.target) else {
            continue;
        };
        let was_positive = hp.current > 0.0;
        hp.current -= msg.amount;
        if was_positive && hp.current <= 0.0 {
            commands.entity(msg.target).insert(KilledBy {
                killer: msg.attributed_to.or(msg.dealer),
            });
        }
    }
}
