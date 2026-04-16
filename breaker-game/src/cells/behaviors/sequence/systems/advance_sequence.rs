//! Promotes the next position in a sequence group when the active member
//! dies.
//!
//! Runs in `FixedUpdate`, ordered `.after(EffectV3Systems::Death)` so that
//! `Destroyed<Cell>` messages written by `HandleKill` are visible and the
//! victim entity is still queryable (despawn happens later in
//! `FixedPostUpdate`). For each destroyed cell that was an active sequence
//! member, inserts `SequenceActive` on every other sequence cell in the same
//! group whose `SequencePosition.0 == victim_position + 1`.
//!
//! Duplicate next-position cells in the same group all receive the marker
//! (same graceful-malformed-data stance as `init_sequence_groups`).

use bevy::prelude::*;

use crate::{
    cells::behaviors::sequence::components::{
        SequenceActive, SequenceCell, SequenceGroup, SequencePosition,
    },
    prelude::*,
};

type DyingActiveSequenceQuery<'w, 's> = Query<
    'w,
    's,
    (&'static SequenceGroup, &'static SequencePosition),
    (With<Cell>, With<SequenceCell>, With<SequenceActive>),
>;

type NextSequenceMemberQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static SequenceGroup, &'static SequencePosition),
    (With<Cell>, With<SequenceCell>, Without<SequenceActive>),
>;

/// Reads `Destroyed<Cell>` and promotes the next sequence position.
pub(crate) fn advance_sequence(
    mut commands: Commands,
    mut destroyed: MessageReader<Destroyed<Cell>>,
    dying: DyingActiveSequenceQuery,
    candidates: NextSequenceMemberQuery,
) {
    for msg in destroyed.read() {
        // Only act when the victim was an active sequence cell.
        let Ok((group, position)) = dying.get(msg.victim) else {
            continue;
        };
        let target_group = group.0;
        let target_position = position.0 + 1;

        for (candidate, candidate_group, candidate_position) in &candidates {
            if candidate_group.0 == target_group && candidate_position.0 == target_position {
                commands.entity(candidate).insert(SequenceActive);
            }
        }
    }
}
