//! Locked behavior components.

use bevy::prelude::*;

/// Permanent marker identifying a cell as a lock-type cell.
///
/// Never removed. Identifies cells that participate in the lock/unlock mechanic.
#[derive(Component, Debug)]
pub struct LockCell;

/// State marker — cell is currently locked and immune to damage.
///
/// Removed by `check_lock_release` when all adjacent cells are destroyed.
#[derive(Component, Debug)]
pub struct Locked;

/// Adjacent entity IDs that must be destroyed to unlock this cell.
///
/// Newtype wrapper around a `Vec<Entity>`. Replaces the old `LockAdjacents`.
#[derive(Component, Debug)]
pub struct Locks(pub Vec<Entity>);

/// State marker — inserted when a lock cell is unlocked.
///
/// Used for visual/audio feedback on unlock.
#[derive(Component, Debug)]
pub struct Unlocked;
