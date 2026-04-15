//! Placeholder system for the `Locked ↔ Invulnerable` coupling.
//!
//! The real coupling is implemented via component hooks on
//! [`crate::cells::behaviors::locked::components::Locked`] (`on_insert` and
//! `on_remove`). Those hooks run synchronously during command application,
//! which correctly handles both spawn-time insertions and same-tick removal
//! driven by [`check_lock_release`](super::check_lock_release).
//!
//! This function exists so the cells plugin (and test harnesses written by
//! writer-tests) can continue to reference `sync_lock_invulnerable` by name
//! without also having to know about the hook wiring. It takes [`Commands`]
//! so Bevy inserts it into the "has deferred state" scheduling category —
//! when the system's `apply_deferred` runs, its `CommandQueue::apply` first
//! calls `world.flush_commands()`, which picks up any commands the
//! `Locked` hook queued while applying a prior system's
//! `commands.entity(e).remove::<Locked>()` call.

use bevy::prelude::*;

/// No-op coupling system. The real mirroring happens inside `Locked`'s
/// `on_insert` and `on_remove` component hooks. The `Commands` parameter is
/// intentional — it makes the system participate in `ApplyDeferred` sync
/// points so hook-queued commands land in the same tick.
pub(crate) const fn sync_lock_invulnerable(_commands: Commands) {}
