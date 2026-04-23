//! Despawn-request processor.
//!
//! P4 ships a no-op stub so `RantzDmgPlugin::build` can schedule the system
//! in `FixedPostUpdate` without the crate depending on P6's implementation.
//! The real body — draining `MessageReader<DespawnEntity>` and issuing
//! `commands.entity(e).despawn()` — lands in P6.

use bevy::prelude::MessageReader;

use crate::messages::DespawnEntity;

/// Stub. Drains pending requests and does nothing with them. Replaced in P6.
///
/// The `MessageReader<DespawnEntity>` parameter is kept so P6 is a drop-in
/// body swap (no signature change, no plugin rewiring). The stub iterates
/// over the queue so the `MessageReader` is genuinely consumed — this
/// mirrors how P6 will consume it and keeps the function out of the
/// `missing_const_for_fn` / `unused_mut` / dead-code lint corner.
pub(crate) fn process_despawn_requests(mut messages: MessageReader<DespawnEntity>) {
    for _ in messages.read() {
        // P6 will populate the body with `commands.entity(e).despawn()`.
    }
}
