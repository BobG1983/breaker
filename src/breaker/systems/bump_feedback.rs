//! Visual text feedback for bump grades.

use bevy::prelude::*;

use crate::{
    bolt::components::FadeOut,
    breaker::{
        components::Breaker,
        messages::{BumpGrade, BumpPerformed},
    },
    shared::CleanupOnNodeExit,
};

/// Fade duration for bump grade text (seconds).
const FADE_DURATION: f32 = 0.8;

/// Spawns floating text near the breaker for each bump grade.
pub fn spawn_bump_grade_text(
    mut reader: MessageReader<BumpPerformed>,
    mut commands: Commands,
    breaker_query: Query<&Transform, With<Breaker>>,
) {
    let Ok(breaker_tf) = breaker_query.single() else {
        return;
    };

    for performed in reader.read() {
        let (text, font_size, color) = match performed.grade {
            BumpGrade::Early => ("EARLY", 24.0, Color::srgb(1.0, 0.7, 0.2)),
            BumpGrade::Late => ("LATE", 24.0, Color::srgb(1.0, 0.7, 0.2)),
            BumpGrade::Perfect => ("PERFECT", 36.0, Color::linear_rgb(0.5, 4.0, 5.0)),
        };

        let pos = Vec3::new(
            breaker_tf.translation.x,
            breaker_tf.translation.y + 40.0,
            10.0,
        );

        commands.spawn((
            Text2d::new(text),
            TextColor(color),
            TextFont::from_font_size(font_size),
            Transform::from_translation(pos),
            FadeOut {
                timer: FADE_DURATION,
                duration: FADE_DURATION,
            },
            CleanupOnNodeExit,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::components::FadeOut;

    #[derive(Resource)]
    struct TestBumpMsg(Option<BumpPerformed>);

    fn enqueue_bump(msg_res: Res<TestBumpMsg>, mut writer: MessageWriter<BumpPerformed>) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<BumpPerformed>();
        app.add_systems(
            Update,
            (
                enqueue_bump.before(spawn_bump_grade_text),
                spawn_bump_grade_text,
            ),
        );
        app
    }

    fn spawn_breaker(app: &mut App) {
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(0.0, -250.0, 0.0)));
    }

    #[test]
    fn perfect_bump_spawns_text() {
        let mut app = test_app();
        spawn_breaker(&mut app);
        app.insert_resource(TestBumpMsg(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
        })));
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<FadeOut>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "perfect bump should spawn feedback text");
    }

    #[test]
    fn early_spawns_text() {
        let mut app = test_app();
        spawn_breaker(&mut app);
        app.insert_resource(TestBumpMsg(Some(BumpPerformed {
            grade: BumpGrade::Early,
        })));
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<FadeOut>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "early bump should spawn feedback text");
    }

    #[test]
    fn late_spawns_text() {
        let mut app = test_app();
        spawn_breaker(&mut app);
        app.insert_resource(TestBumpMsg(Some(BumpPerformed {
            grade: BumpGrade::Late,
        })));
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<FadeOut>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "late bump should spawn feedback text");
    }
}
