//! Shared test helpers for `check_lock_release` tests.

use bevy::prelude::*;

use super::super::system::check_lock_release;
pub(super) use crate::shared::test_utils::tick;
use crate::{
    cells::{
        behaviors::locked::systems::sync_lock_invulnerable::sync_lock_invulnerable,
        components::Cell,
    },
    shared::death_pipeline::destroyed::Destroyed,
};

// ---------------------------------------------------------------
// Test helpers -- message injection for Destroyed<Cell>
// ---------------------------------------------------------------

#[derive(Resource, Default)]
pub(super) struct TestDestroyedMessages(pub(super) Vec<Destroyed<Cell>>);

pub(super) fn enqueue_destroyed(
    msg_res: Res<TestDestroyedMessages>,
    mut writer: MessageWriter<Destroyed<Cell>>,
) {
    for msg in &msg_res.0 {
        writer.write(msg.clone());
    }
}

// ---------------------------------------------------------------
// Test app factory
// ---------------------------------------------------------------

/// App for testing `check_lock_release`.
pub(super) fn lock_release_app() -> App {
    use crate::shared::test_utils::TestAppBuilder;

    TestAppBuilder::new()
        .with_message::<Destroyed<Cell>>()
        .with_resource::<TestDestroyedMessages>()
        .with_system(
            FixedUpdate,
            (
                enqueue_destroyed.before(check_lock_release),
                check_lock_release,
                sync_lock_invulnerable.after(check_lock_release),
            ),
        )
        .build()
}
