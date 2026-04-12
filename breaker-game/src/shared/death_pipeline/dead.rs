//! Dead marker component for entities confirmed dead by their domain kill handler.

use bevy::prelude::*;

/// Marker component inserted on an entity that has been confirmed dead by its
/// domain kill handler. Prevents double-processing — systems use `Without<Dead>`
/// to skip entities that are already dying.
///
/// The entity still exists in the world after `Dead` is inserted. It survives
/// through trigger evaluation and death bridges. It is finally despawned by
/// `process_despawn_requests` in `PostFixedUpdate`.
#[derive(Component)]
pub(crate) struct Dead;
