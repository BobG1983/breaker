//! Locked behavior components.

use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::shared::death_pipeline::Invulnerable;

/// Permanent marker identifying a cell as a lock-type cell.
///
/// Never removed. Identifies cells that participate in the lock/unlock mechanic.
#[derive(Component, Debug)]
pub struct LockCell;

/// State marker — cell is currently locked and immune to damage.
///
/// Removed by `check_lock_release` when all adjacent cells are destroyed.
///
/// Adds and removes [`Invulnerable`] automatically via component hooks so the
/// invariant "`has::<Locked>() ⇔ has::<Invulnerable>()` while `Locked` is
/// present" holds for every caller of `insert(Locked)` / `remove::<Locked>()`.
/// Removing [`Invulnerable`] directly without removing `Locked` is allowed —
/// the hook does not re-assert the coupling every frame.
#[derive(Component, Debug)]
#[component(on_insert = insert_invulnerable, on_remove = remove_invulnerable)]
pub struct Locked;

fn insert_invulnerable(mut world: DeferredWorld, context: HookContext) {
    // Skip if `Invulnerable` is already on the entity (idempotent and
    // preserves the "coupling is one-shot on insert, not every-frame
    // enforcement" semantic — see `locked::tests::L2` edge cases).
    let already_invulnerable = world.get::<Invulnerable>(context.entity).is_some();
    if already_invulnerable {
        return;
    }
    world.commands().entity(context.entity).insert(Invulnerable);
}

fn remove_invulnerable(mut world: DeferredWorld, context: HookContext) {
    if let Ok(mut entity) = world.commands().get_entity(context.entity) {
        entity.remove::<Invulnerable>();
    }
}

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
