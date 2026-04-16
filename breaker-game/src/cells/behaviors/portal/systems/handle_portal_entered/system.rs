//! Mock handler: converts `PortalEntered` → `PortalCompleted` immediately.
//! Will be replaced with real sub-node logic in the node refactor.

use bevy::prelude::*;

use crate::cells::messages::{PortalCompleted, PortalEntered};

/// Mock handler: converts `PortalEntered` into `PortalCompleted` immediately.
///
/// Will be replaced with real sub-node logic in the node refactor.
pub(crate) fn handle_portal_entered(
    mut reader: MessageReader<PortalEntered>,
    mut writer: MessageWriter<PortalCompleted>,
) {
    for msg in reader.read() {
        writer.write(PortalCompleted { portal: msg.portal });
    }
}
