//! System to track chip selections for the current run.

use bevy::prelude::*;

use crate::prelude::*;

/// Reads [`ChipSelected`] messages and pushes chip names to
/// [`RunStats::chips_collected`].
pub(crate) fn track_chips_collected(
    mut reader: MessageReader<ChipSelected>,
    mut stats: ResMut<RunStats>,
) {
    for msg in reader.read() {
        stats.chips_collected.push(msg.name.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct TestMessages(Vec<ChipSelected>);

    fn enqueue_messages(msg_res: Res<TestMessages>, mut writer: MessageWriter<ChipSelected>) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    /// Uses `Update` schedule (not `FixedUpdate`) since chip selection
    /// happens during the `ChipSelect` game state, not during physics.
    fn test_app() -> App {
        use crate::prelude::*;
        TestAppBuilder::new()
            .with_message::<ChipSelected>()
            .with_resource::<RunStats>()
            .with_system(Update, (enqueue_messages, track_chips_collected).chain())
            .build()
    }

    #[test]
    fn pushes_chip_name_on_selection() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![ChipSelected {
            name: "Piercing Shot".to_owned(),
        }]));
        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.chips_collected,
            vec!["Piercing Shot"],
            "selected chip name should be pushed to chips_collected"
        );
    }

    #[test]
    fn collects_multiple_chips_in_order() {
        let mut app = test_app();
        // Pre-populate with one chip already collected
        app.world_mut().resource_mut::<RunStats>().chips_collected = vec!["A".to_owned()];

        app.insert_resource(TestMessages(vec![ChipSelected {
            name: "B".to_owned(),
        }]));
        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.chips_collected,
            vec!["A", "B"],
            "chips should accumulate in order"
        );
    }
}
