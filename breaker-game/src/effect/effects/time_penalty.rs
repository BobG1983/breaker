use bevy::prelude::*;

use crate::run::node::messages::{ApplyTimePenalty, ReverseTimePenalty};

/// Sends an [`ApplyTimePenalty`] message to subtract seconds from the node timer.
///
/// The `apply_time_penalty` system in the node subdomain reads the message
/// and applies the subtraction with clamping and expiry detection.
pub(crate) fn fire(_entity: Entity, seconds: f32, _source_chip: &str, world: &mut World) {
    world
        .resource_mut::<Messages<ApplyTimePenalty>>()
        .write(ApplyTimePenalty { seconds });
}

/// Sends a [`ReverseTimePenalty`] message to add seconds back to the node timer.
///
/// The `reverse_time_penalty` system in the node subdomain reads the message
/// and adds time back, clamping to `NodeTimer::total`.
pub(crate) fn reverse(_entity: Entity, seconds: f32, _source_chip: &str, world: &mut World) {
    world
        .resource_mut::<Messages<ReverseTimePenalty>>()
        .write(ReverseTimePenalty { seconds });
}

/// Registers systems for `TimePenalty` effect.
pub(crate) fn register(_app: &mut App) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::node::messages::{ApplyTimePenalty, ReverseTimePenalty};

    // ── fire() message-writing tests ──────────────────────────────

    #[test]
    fn fire_sends_apply_time_penalty_message() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<ApplyTimePenalty>();
        let entity = app.world_mut().spawn_empty().id();

        fire(entity, 5.0, "", app.world_mut());

        let messages = app.world().resource::<Messages<ApplyTimePenalty>>();
        let written: Vec<&ApplyTimePenalty> = messages.iter_current_update_messages().collect();
        assert_eq!(
            written.len(),
            1,
            "fire() should write exactly 1 ApplyTimePenalty message, got {}",
            written.len()
        );
        assert!(
            (written[0].seconds - 5.0).abs() < f32::EPSILON,
            "message seconds should be 5.0, got {}",
            written[0].seconds
        );
    }

    #[test]
    fn fire_with_zero_seconds_sends_message() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<ApplyTimePenalty>();
        let entity = app.world_mut().spawn_empty().id();

        fire(entity, 0.0, "", app.world_mut());

        let messages = app.world().resource::<Messages<ApplyTimePenalty>>();
        let written: Vec<&ApplyTimePenalty> = messages.iter_current_update_messages().collect();
        assert_eq!(
            written.len(),
            1,
            "fire() with 0.0 should still write exactly 1 message, got {}",
            written.len()
        );
        assert!(
            (written[0].seconds - 0.0).abs() < f32::EPSILON,
            "message seconds should be 0.0, got {}",
            written[0].seconds
        );
    }

    // ── reverse() message-writing tests ───────────────────────────

    #[test]
    fn reverse_sends_reverse_time_penalty_message() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<ReverseTimePenalty>();
        let entity = app.world_mut().spawn_empty().id();

        reverse(entity, 5.0, "", app.world_mut());

        let messages = app.world().resource::<Messages<ReverseTimePenalty>>();
        let written: Vec<&ReverseTimePenalty> = messages.iter_current_update_messages().collect();
        assert_eq!(
            written.len(),
            1,
            "reverse() should write exactly 1 ReverseTimePenalty message, got {}",
            written.len()
        );
        assert!(
            (written[0].seconds - 5.0).abs() < f32::EPSILON,
            "message seconds should be 5.0, got {}",
            written[0].seconds
        );
    }

    #[test]
    fn reverse_with_zero_seconds_sends_message() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<ReverseTimePenalty>();
        let entity = app.world_mut().spawn_empty().id();

        reverse(entity, 0.0, "", app.world_mut());

        let messages = app.world().resource::<Messages<ReverseTimePenalty>>();
        let written: Vec<&ReverseTimePenalty> = messages.iter_current_update_messages().collect();
        assert_eq!(
            written.len(),
            1,
            "reverse() with 0.0 should still write exactly 1 message, got {}",
            written.len()
        );
        assert!(
            (written[0].seconds - 0.0).abs() < f32::EPSILON,
            "message seconds should be 0.0, got {}",
            written[0].seconds
        );
    }
}
