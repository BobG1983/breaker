//! Visual feedback when the bolt is lost — spawns a "BOLT LOST" text that fades out.

use bevy::prelude::*;

use crate::{bolt::components::FadeOut, physics::messages::BoltLost, shared::CleanupOnNodeExit};

/// Duration of the "BOLT LOST" text fade in seconds.
const FADE_DURATION: f32 = 1.5;

/// Spawns a "BOLT LOST" text entity when a [`BoltLost`] message is received.
pub fn spawn_bolt_lost_text(mut reader: MessageReader<BoltLost>, mut commands: Commands) {
    for _ in reader.read() {
        commands.spawn((
            Text2d::new("BOLT LOST"),
            TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
            TextFont::from_font_size(86.0),
            Transform::from_xyz(0.0, 0.0, 10.0),
            FadeOut {
                timer: FADE_DURATION,
                duration: FADE_DURATION,
            },
            CleanupOnNodeExit,
        ));
    }
}

/// Ticks [`FadeOut`] timers and adjusts text alpha. Despawns when finished.
pub fn animate_fade_out(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut FadeOut, &mut TextColor)>,
) {
    let dt = time.delta_secs();
    for (entity, mut fade, mut color) in &mut query {
        fade.timer -= dt;
        if fade.timer <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let t = fade.timer / fade.duration;
        // Quadratic ease-out: alpha = t * t
        let alpha = t * t;
        color.0 = color.0.with_alpha(alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct TriggerLost(bool);

    fn enqueue_lost(trigger: Res<TriggerLost>, mut writer: MessageWriter<BoltLost>) {
        if trigger.0 {
            writer.write(BoltLost);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<BoltLost>();
        app.add_systems(
            Update,
            (
                enqueue_lost.before(spawn_bolt_lost_text),
                spawn_bolt_lost_text,
                animate_fade_out.after(spawn_bolt_lost_text),
            ),
        );
        app
    }

    #[test]
    fn text_entity_spawns_on_bolt_lost() {
        let mut app = test_app();
        app.insert_resource(TriggerLost(true));
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<FadeOut>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "should spawn one fade-out text entity");
    }

    #[test]
    fn text_entity_not_spawned_without_message() {
        let mut app = test_app();
        app.insert_resource(TriggerLost(false));
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<FadeOut>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "should not spawn text without BoltLost");
    }

    #[test]
    fn text_despawns_when_timer_expires() {
        let mut app = test_app();
        app.insert_resource(TriggerLost(true));
        app.update();

        // Stop triggering new messages
        app.insert_resource(TriggerLost(false));

        // Manually expire the timer so the next update despawns it
        for mut fade in app
            .world_mut()
            .query::<&mut FadeOut>()
            .iter_mut(app.world_mut())
        {
            fade.timer = 0.0;
        }

        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<FadeOut>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "text should be despawned when timer expires");
    }
}
