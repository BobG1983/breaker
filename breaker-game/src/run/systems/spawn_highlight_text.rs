//! Spawns in-game text popups when highlight moments are detected.

use bevy::prelude::*;

use crate::{
    fx::FadeOut,
    run::{messages::HighlightTriggered, resources::HighlightKind},
    shared::CleanupOnNodeExit,
};

/// Spawns floating text for each [`HighlightTriggered`] message.
pub(crate) fn spawn_highlight_text(
    mut reader: MessageReader<HighlightTriggered>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let (text, color) = match msg.kind {
            HighlightKind::ClutchClear => ("CLUTCH CLEAR!", Color::srgb(0.0, 1.0, 1.0)),
            HighlightKind::MassDestruction => ("MASS DESTRUCTION!", Color::srgb(1.0, 0.6, 0.0)),
            HighlightKind::PerfectStreak => ("PERFECT STREAK!", Color::srgb(0.0, 1.0, 1.0)),
            HighlightKind::FastClear => ("SPEED CLEAR!", Color::srgb(0.0, 1.0, 1.0)),
            HighlightKind::FirstEvolution => ("FIRST EVOLUTION!", Color::srgb(1.0, 1.0, 0.0)),
            HighlightKind::NoDamageNode => ("FLAWLESS!", Color::srgb(0.0, 1.0, 0.4)),
            HighlightKind::CloseSave => ("CLOSE SAVE!", Color::srgb(0.0, 1.0, 0.4)),
            HighlightKind::SpeedDemon => ("SPEED DEMON!", Color::srgb(0.0, 1.0, 1.0)),
            HighlightKind::Untouchable => ("UNTOUCHABLE!", Color::srgb(0.0, 1.0, 1.0)),
            HighlightKind::ComboKing => ("COMBO KING!", Color::srgb(1.0, 0.6, 0.0)),
            HighlightKind::PinballWizard => ("PINBALL WIZARD!", Color::srgb(1.0, 0.6, 0.0)),
            HighlightKind::Comeback => ("COMEBACK!", Color::srgb(0.0, 1.0, 0.4)),
            HighlightKind::PerfectNode => ("PERFECT NODE!", Color::srgb(0.0, 1.0, 1.0)),
            HighlightKind::NailBiter => ("NAIL BITER!", Color::srgb(0.0, 1.0, 0.4)),
            HighlightKind::MostPowerfulEvolution => ("DEVASTATING!", Color::srgb(1.0, 0.6, 0.0)),
        };

        commands.spawn((
            Text2d::new(text),
            TextColor(color),
            TextFont::from_font_size(64.0),
            Transform::from_xyz(0.0, 100.0, 10.0),
            FadeOut {
                timer: 2.0,
                duration: 2.0,
            },
            CleanupOnNodeExit,
        ));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Text2d;

    use super::*;
    use crate::{
        fx::FadeOut,
        run::{messages::HighlightTriggered, resources::HighlightKind},
        shared::CleanupOnNodeExit,
    };

    #[derive(Resource)]
    struct TestHighlightMsg(Vec<HighlightTriggered>);

    fn enqueue_highlights(
        msg_res: Res<TestHighlightMsg>,
        mut writer: MessageWriter<HighlightTriggered>,
    ) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<HighlightTriggered>()
            .add_systems(
                Update,
                (
                    enqueue_highlights.before(spawn_highlight_text),
                    spawn_highlight_text,
                ),
            );
        app
    }

    /// Runs a single highlight kind through the system and asserts that:
    /// 1. A `Text2d` entity is spawned containing `expected_text`.
    /// 2. The entity has a `FadeOut` component.
    /// 3. The entity has a `CleanupOnNodeExit` component.
    fn assert_highlight_spawns_text(kind: HighlightKind, expected_text: &str) {
        let mut app = test_app();
        app.insert_resource(TestHighlightMsg(vec![HighlightTriggered { kind }]));
        app.update();

        let texts: Vec<String> = app
            .world_mut()
            .query::<&Text2d>()
            .iter(app.world())
            .map(|t| t.0.clone())
            .collect();

        assert!(
            texts.iter().any(|t| t.contains(expected_text)),
            "expected Text2d containing {expected_text:?}, found: {texts:?}"
        );

        // Verify FadeOut component exists on at least one Text2d entity
        let fade_count = app
            .world_mut()
            .query_filtered::<Entity, (With<Text2d>, With<FadeOut>)>()
            .iter(app.world())
            .count();
        assert!(
            fade_count > 0,
            "highlight text entity should have FadeOut component"
        );

        // Verify CleanupOnNodeExit component exists on at least one Text2d entity
        let cleanup_count = app
            .world_mut()
            .query_filtered::<Entity, (With<Text2d>, With<CleanupOnNodeExit>)>()
            .iter(app.world())
            .count();
        assert!(
            cleanup_count > 0,
            "highlight text entity should have CleanupOnNodeExit component"
        );
    }

    #[test]
    fn clutch_clear_spawns_text() {
        assert_highlight_spawns_text(HighlightKind::ClutchClear, "CLUTCH CLEAR!");
    }

    #[test]
    fn mass_destruction_spawns_text() {
        assert_highlight_spawns_text(HighlightKind::MassDestruction, "MASS DESTRUCTION!");
    }

    #[test]
    fn perfect_streak_spawns_text() {
        assert_highlight_spawns_text(HighlightKind::PerfectStreak, "PERFECT STREAK!");
    }

    #[test]
    fn fast_clear_spawns_text() {
        assert_highlight_spawns_text(HighlightKind::FastClear, "SPEED CLEAR!");
    }

    #[test]
    fn first_evolution_spawns_text() {
        assert_highlight_spawns_text(HighlightKind::FirstEvolution, "FIRST EVOLUTION!");
    }

    #[test]
    fn no_damage_node_spawns_text() {
        assert_highlight_spawns_text(HighlightKind::NoDamageNode, "FLAWLESS!");
    }

    #[test]
    fn close_save_spawns_text() {
        assert_highlight_spawns_text(HighlightKind::CloseSave, "CLOSE SAVE!");
    }

    #[test]
    fn speed_demon_spawns_text() {
        assert_highlight_spawns_text(HighlightKind::SpeedDemon, "SPEED DEMON!");
    }

    #[test]
    fn untouchable_spawns_text() {
        assert_highlight_spawns_text(HighlightKind::Untouchable, "UNTOUCHABLE!");
    }

    #[test]
    fn combo_king_spawns_text() {
        assert_highlight_spawns_text(HighlightKind::ComboKing, "COMBO KING!");
    }

    #[test]
    fn pinball_wizard_spawns_text() {
        assert_highlight_spawns_text(HighlightKind::PinballWizard, "PINBALL WIZARD!");
    }

    #[test]
    fn comeback_spawns_text() {
        assert_highlight_spawns_text(HighlightKind::Comeback, "COMEBACK!");
    }

    #[test]
    fn perfect_node_spawns_text() {
        assert_highlight_spawns_text(HighlightKind::PerfectNode, "PERFECT NODE!");
    }

    #[test]
    fn nail_biter_spawns_text() {
        assert_highlight_spawns_text(HighlightKind::NailBiter, "NAIL BITER!");
    }

    #[test]
    fn most_powerful_evolution_spawns_text() {
        assert_highlight_spawns_text(HighlightKind::MostPowerfulEvolution, "DEVASTATING!");
    }

    #[test]
    fn multiple_highlights_spawn_multiple_text_entities() {
        let mut app = test_app();
        app.insert_resource(TestHighlightMsg(vec![
            HighlightTriggered {
                kind: HighlightKind::ClutchClear,
            },
            HighlightTriggered {
                kind: HighlightKind::NoDamageNode,
            },
            HighlightTriggered {
                kind: HighlightKind::PerfectStreak,
            },
        ]));
        app.update();

        let count = app.world_mut().query::<&Text2d>().iter(app.world()).count();
        assert_eq!(
            count, 3,
            "each HighlightTriggered message should spawn one Text2d entity"
        );
    }

    #[test]
    fn no_highlights_spawns_no_text_entities() {
        let mut app = test_app();
        app.insert_resource(TestHighlightMsg(vec![]));
        app.update();

        let count = app.world_mut().query::<&Text2d>().iter(app.world()).count();
        assert_eq!(
            count, 0,
            "no HighlightTriggered messages should spawn no Text2d entities"
        );
    }
}
