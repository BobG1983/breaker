//! Sequence behavior components.
//!
//! Plain marker and newtype components. Behavior lives in the sibling
//! `systems/` module: `init_sequence_groups` seeds the first active member at
//! `OnEnter(NodeState::Playing)`, `reset_inactive_sequence_hp` reverts stray
//! damage on non-active members, and `advance_sequence` promotes the next
//! position when an active member dies.

use bevy::prelude::*;

/// Permanent marker identifying a cell as a sequence-type cell.
///
/// Never removed. Inserted alongside `SequenceGroup` + `SequencePosition` when
/// `CellBehavior::Sequence` is resolved at spawn time.
#[derive(Component, Debug)]
pub struct SequenceCell;

/// Group id for a sequence-participating cell. Cells with the same
/// `SequenceGroup` value form one sequence; advancement is scoped to group.
#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SequenceGroup(pub u32);

/// Zero-based position index within a `SequenceGroup`. Position 0 is the
/// initial active member; higher positions activate in order as lower ones
/// die.
#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SequencePosition(pub u32);

/// State marker — this sequence cell is currently the active member of its
/// group. Inserted by `init_sequence_groups` (`OnEnter(NodeState::Playing)`)
/// and `advance_sequence` (on death). Never auto-removed — when a cell dies
/// the marker is removed implicitly via despawn.
#[derive(Component, Debug)]
pub struct SequenceActive;
