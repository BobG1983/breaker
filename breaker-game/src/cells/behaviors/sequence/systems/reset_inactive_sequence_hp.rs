//! Reverts damage on non-active sequence cells before death detection.
//!
//! Runs in `FixedUpdate`, ordered `.after(DeathPipelineSystems::ApplyDamage)`
//! and `.before(DeathPipelineSystems::DetectDeaths)`. For every sequence cell
//! lacking `SequenceActive`, if `hp.current < hp.starting` (i.e. the cell
//! took damage this tick), restores `hp.current` to `hp.max.unwrap_or(starting)`
//! AND clears `KilledBy.dealer` to `None`.
//!
//! **Guard anchored on `hp.starting`, not on the healing ceiling.** An
//! undamaged cell spawns with `current == starting`. `apply_damage::<Cell>`
//! is the only system that lowers `current`, so `current < starting` is the
//! exact predicate for "was hit this tick." Using `current < ceiling` would
//! fire every frame for any cell whose `hp.max` exceeds `hp.starting`,
//! giving non-active sequence cells a free unconditional heal up to `max`
//! and clearing `KilledBy.dealer` even in the absence of any damage.
//!
//! The `KilledBy.dealer` clear is essential: `apply_damage::<Cell>` stamps
//! `killed_by.dealer = msg.dealer` gated on `is_none()` — if the reset system
//! only restores HP without clearing the dealer, a future legitimate killing
//! blow against the cell (once it becomes active) is silently rejected by
//! the `is_none()` gate, latching stale dealer state across the reset. The
//! clear restores the cell to its spawn-equivalent: full HP, no recorded
//! killer, ready for a fresh kill record when it actually enters its active
//! window.

use bevy::prelude::*;

use crate::{
    cells::behaviors::sequence::components::{SequenceActive, SequenceCell},
    prelude::*,
};

type InactiveSequenceHpQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Hp, &'static mut KilledBy),
    (With<SequenceCell>, Without<SequenceActive>),
>;

/// Restores `hp.current` to the healing ceiling and clears `killed_by.dealer`
/// on any non-active sequence cell that took damage this tick.
///
/// Detection is `hp.current < hp.starting` — `apply_damage::<Cell>` is the
/// only system that lowers `current`, so an untouched cell sits at exactly
/// `starting`. Clearing `KilledBy.dealer` is required because `apply_damage`
/// stamps it on the killing blow via an `is_none()` gate; a stale dealer
/// would latch across the reset and silently reject a future legitimate kill
/// once the cell becomes active.
pub(crate) fn reset_inactive_sequence_hp(mut query: InactiveSequenceHpQuery) {
    for (mut hp, mut killed_by) in &mut query {
        if hp.current < hp.starting {
            hp.current = hp.max.unwrap_or(hp.starting);
            killed_by.dealer = None;
        }
    }
}
