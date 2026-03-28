use bevy::prelude::*;

/// Subtracts seconds from the node timer.
///
/// Placeholder — needs run domain types (`NodeTimer` resource or
/// `ApplyTimePenalty` message).
pub(crate) fn fire(_entity: Entity, seconds: f32, _world: &mut World) {
    info!("time penalty: subtract {:.1}s from node timer", seconds);
}

/// Adds seconds back to the node timer — reverses the penalty.
pub(crate) fn reverse(_entity: Entity, seconds: f32, _world: &mut World) {
    info!(
        "time penalty reverse: add {:.1}s back to node timer",
        seconds
    );
    // Placeholder — needs run domain types (NodeTimer resource or message)
}

/// Registers systems for `TimePenalty` effect.
pub(crate) fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_completes_without_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        // Placeholder effect — should not panic.
        fire(entity, 5.0, &mut world);
    }

    #[test]
    fn reverse_completes_without_panic() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        // Placeholder — should add seconds back once NodeTimer exists.
        reverse(entity, 5.0, &mut world);
    }
}
