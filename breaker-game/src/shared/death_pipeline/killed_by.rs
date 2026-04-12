//! Kill attribution component.

use bevy::prelude::*;

/// Set by `apply_damage<T>` on the killing blow only — the damage message that
/// crosses `Hp` from positive to zero.
///
/// Read by death detection systems to determine who killed the entity.
/// First kill wins — if `KilledBy` is already set, do not overwrite.
#[derive(Component, Default, Debug)]
pub(crate) struct KilledBy {
    /// The entity that originated the damage. `Some(entity)` for attributed
    /// kills, `None` for environmental deaths.
    pub dealer: Option<Entity>,
}
