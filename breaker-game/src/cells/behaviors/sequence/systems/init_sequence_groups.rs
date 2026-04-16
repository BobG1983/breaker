//! Initializes sequence groups at `OnEnter(NodeState::Playing)`.
//!
//! Scans every sequence cell and inserts `SequenceActive` on each cell whose
//! `SequencePosition` is 0. In well-formed data there is exactly one such cell
//! per `SequenceGroup`, so this amounts to "one active member per group".
//!
//! Design note: the system deliberately does NOT deduplicate. In malformed
//! data where two cells in the same group both sit at `position = 0`, BOTH
//! receive `SequenceActive` — the malformed state surfaces at play time
//! instead of being silently hidden by a first-wins `HashMap`. Cross-cell
//! validation belongs at RON load time and is out of scope here.
//!
//! Assumes all sequence cells are spawned before `OnEnter(NodeState::Playing)`
//! fires (cells spawn via `spawn_cells_from_layout` on
//! `OnEnter(NodeState::Loading)` and the state machine traverses
//! `Loading → AnimateIn → Playing`). Mid-Playing cell spawns will not receive
//! `SequenceActive` unless the spawn path inserts it directly — revisit if a
//! future feature spawns sequence cells after Playing has started.

use bevy::prelude::*;

use crate::cells::behaviors::sequence::components::{
    SequenceActive, SequenceCell, SequenceGroup, SequencePosition,
};

type SequenceInitQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static SequencePosition), (With<SequenceCell>, With<SequenceGroup>)>;

/// Inserts `SequenceActive` on every sequence cell whose
/// `SequencePosition.0 == 0`.
pub(crate) fn init_sequence_groups(mut commands: Commands, query: SequenceInitQuery) {
    for (entity, position) in &query {
        if position.0 == 0 {
            commands.entity(entity).insert(SequenceActive);
        }
    }
}
