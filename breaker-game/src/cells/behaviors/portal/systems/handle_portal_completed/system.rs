//! Reads `PortalCompleted` and kills the portal cell via `KillYourself<Cell>`.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    cells::messages::PortalCompleted, prelude::*,
    shared::death_pipeline::kill_yourself::KillYourself,
};

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
