//! Deferred despawn request message.

use bevy::prelude::*;

/// Deferred despawn request. Sent after death animations and trigger evaluation
/// complete. Processed by `process_despawn_requests` in `PostFixedUpdate`.
///
/// Use this instead of calling `commands.entity(e).despawn()` directly in death
/// handling. The entity must survive through the full death chain.
#[derive(Message, Clone, Debug)]
pub(crate) struct DespawnEntity {
    /// The entity to despawn.
    pub entity: Entity,
}
