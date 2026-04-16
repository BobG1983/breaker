//! Detects bolt-portal collisions and emits `PortalEntered` messages.

use bevy::prelude::*;

use crate::{
    cells::{behaviors::portal::components::PortalCell, messages::PortalEntered},
    prelude::*,
};

pub(crate) fn check_portal_entry(
    mut reader: MessageReader<BoltImpactCell>,
    portal_query: Query<(), With<PortalCell>>,
    mut writer: MessageWriter<PortalEntered>,
) {
    for msg in reader.read() {
        if portal_query.contains(msg.cell) {
            writer.write(PortalEntered {
                portal:            msg.cell,
                #[cfg(test)]
                bolt:              msg.bolt,
            });
        }
    }
}
