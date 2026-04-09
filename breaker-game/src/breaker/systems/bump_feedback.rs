//! Visual text feedback for bump grades.

use bevy::prelude::*;

use crate::{
    breaker::messages::{BumpGrade, BumpWhiffed},
    fx::FadeOut,
    prelude::*,
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
            BumpGrade::Early => ("EARLY", 43.0, Color::srgb(1.0, 0.7, 0.2)),
            BumpGrade::Late => ("LATE", 43.0, Color::srgb(1.0, 0.7, 0.2)),
            BumpGrade::Perfect => ("PERFECT", 65.0, Color::linear_rgb(0.5, 4.0, 5.0)),
        };

        let pos = Vec3::new(
            breaker_tf.translation.x,
            breaker_tf.translation.y + 72.0,
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
            CleanupOnExit::<NodeState>::default(),
        ));
    }
}

/// Spawns floating "WHIFF" text near the breaker when a bump window expires.
pub fn spawn_whiff_text(
    mut reader: MessageReader<BumpWhiffed>,
    mut commands: Commands,
    breaker_query: Query<&Transform, With<Breaker>>,
) {
    let Ok(breaker_tf) = breaker_query.single() else {
        return;
    };

    for _whiff in reader.read() {
        let pos = Vec3::new(
            breaker_tf.translation.x,
            breaker_tf.translation.y + 72.0,
            10.0,
        );

        commands.spawn((
            Text2d::new("WHIFF"),
            TextColor(Color::srgb(0.5, 0.5, 0.5)),
            TextFont::from_font_size(36.0),
            Transform::from_translation(pos),
            FadeOut {
                timer: FADE_DURATION,
                duration: FADE_DURATION,
            },
            CleanupOnExit::<NodeState>::default(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fx::FadeOut;

    #[derive(Resource)]
    struct TestBumpMsg(Option<BumpPerformed>);

    fn enqueue_bump(msg_res: Res<TestBumpMsg>, mut writer: MessageWriter<BumpPerformed>) {
        if let Some(msg) = msg_res.0.clone() {
            writer.write(msg);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpPerformed>()
            .add_systems(
                FixedUpdate,
                (
                    enqueue_bump.before(spawn_bump_grade_text),
                    spawn_bump_grade_text,
                ),
            );
        app
    }

    use crate::shared::test_utils::tick;

    fn spawn_breaker(app: &mut App) {
        use crate::breaker::definition::BreakerDefinition;
        let def = BreakerDefinition::default();
        let world = app.world_mut();
        let entity = Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut world.commands());
        world.flush();
        app.world_mut()
            .entity_mut(entity)
            .insert(Transform::from_xyz(0.0, -450.0, 0.0));
    }

    #[test]
    fn perfect_bump_spawns_text() {
        let mut app = test_app();
        spawn_breaker(&mut app);
        app.insert_resource(TestBumpMsg(Some(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: None,
            breaker: Entity::PLACEHOLDER,
        })));
        tick(&mut app);

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
            bolt: None,
            breaker: Entity::PLACEHOLDER,
        })));
        tick(&mut app);

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
            bolt: None,
            breaker: Entity::PLACEHOLDER,
        })));
        tick(&mut app);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<FadeOut>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "late bump should spawn feedback text");
    }

    #[derive(Resource)]
    struct TestWhiffMsg(bool);

    fn enqueue_whiff(msg_res: Res<TestWhiffMsg>, mut writer: MessageWriter<BumpWhiffed>) {
        if msg_res.0 {
            writer.write(BumpWhiffed);
        }
    }

    #[test]
    fn whiff_spawns_text() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BumpWhiffed>()
            .insert_resource(TestWhiffMsg(true))
            .add_systems(
                FixedUpdate,
                (enqueue_whiff.before(spawn_whiff_text), spawn_whiff_text),
            );
        spawn_breaker(&mut app);
        tick(&mut app);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<FadeOut>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "whiff should spawn feedback text");
    }
}
