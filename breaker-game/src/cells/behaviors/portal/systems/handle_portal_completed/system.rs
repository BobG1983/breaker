//! Reads `PortalCompleted` and kills the portal cell via `KillYourself<Cell>`.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{cells::messages::PortalCompleted, prelude::*};

/// Converts each [`PortalCompleted`] event into a [`KillYourself<Cell>`]
/// targeting the completed portal cell. The actual despawn happens downstream
/// via the `rantzsoft_dmg` kill pipeline (`handle_kill::<Cell>` →
/// `Destroyed<Cell>` + `DespawnEntity`).
pub(crate) fn handle_portal_completed(
    mut reader: MessageReader<PortalCompleted>,
    mut writer: MessageWriter<KillYourself<Cell>>,
) {
    for msg in reader.read() {
        writer.write(KillYourself {
            victim:  msg.portal,
            killer:  None,
            _marker: PhantomData,
        });
    }
}
