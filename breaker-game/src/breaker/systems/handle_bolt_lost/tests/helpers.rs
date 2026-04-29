//! Shared test helpers for `handle_bolt_lost` system tests.

use bevy::prelude::*;

use super::super::handle_bolt_lost;
use crate::{prelude::*, state::run::node::messages::ReduceNodeTimer};

/// Builds a minimal `App` that registers the messages `handle_bolt_lost`
/// reads/writes (`BoltLost`, `ReduceNodeTimer`) and adds the system to
/// `Update`. We use `Update` (not `FixedUpdate`) so each `app.update()`
/// runs the system exactly once — fixed timestep accumulation is not
/// load-bearing for unit tests of this system.
pub(super) fn handle_bolt_lost_test_app() -> App {
    TestAppBuilder::new()
        .with_message::<BoltLost>()
        .with_message::<ReduceNodeTimer>()
        .with_system(Update, handle_bolt_lost)
        .build()
}

/// Spawns a breaker entity with the given `BoltLossBehavior` and returns its
/// entity id. Optionally inserts an `Hp` component when `hp` is `Some`.
pub(super) fn spawn_test_breaker(
    app: &mut App,
    behavior: crate::breaker::components::BoltLossBehavior,
    hp: Option<Hp>,
) -> Entity {
    let mut entity_commands = app.world_mut().spawn((Breaker, behavior));
    if let Some(hp) = hp {
        entity_commands.insert(hp);
    }
    entity_commands.id()
}

/// Writes one `BoltLost` message targeting `breaker`. Bolt entity is a fresh
/// placeholder (the system never dereferences it for the behaviors tested
/// here — only `breaker` matters).
pub(super) fn send_bolt_lost(app: &mut App, breaker: Entity) {
    app.world_mut()
        .resource_mut::<Messages<BoltLost>>()
        .write(BoltLost {
            bolt: Entity::PLACEHOLDER,
            breaker,
        });
}

/// Collects the currently-buffered `ReduceNodeTimer` messages.
pub(super) fn collect_reduce_messages(app: &App) -> Vec<ReduceNodeTimer> {
    app.world()
        .resource::<Messages<ReduceNodeTimer>>()
        .iter_current_update_messages()
        .cloned()
        .collect()
}
