//! System to track bolt losses for the current run.

use bevy::prelude::*;

use crate::{
    bolt::messages::BoltLost,
    run::resources::{HighlightTracker, RunStats},
};

/// Reads [`BoltLost`] messages and increments counters in [`RunStats`]
/// and [`HighlightTracker`].
pub(crate) fn track_bolts_lost(
    mut reader: MessageReader<BoltLost>,
    mut stats: ResMut<RunStats>,
    mut tracker: ResMut<HighlightTracker>,
) {
    for _msg in reader.read() {
        stats.bolts_lost += 1;
        tracker.node_bolts_lost += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct TestMessages(Vec<BoltLost>);

    fn enqueue_messages(msg_res: Res<TestMessages>, mut writer: MessageWriter<BoltLost>) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltLost>()
            .init_resource::<RunStats>()
            .init_resource::<HighlightTracker>()
            .add_systems(FixedUpdate, (enqueue_messages, track_bolts_lost).chain());
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    #[test]
    fn increments_bolts_lost_for_each_message() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![BoltLost, BoltLost]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.bolts_lost, 2,
            "2 BoltLost messages should increment bolts_lost to 2"
        );
    }

    #[test]
    fn increments_node_bolts_lost_in_tracker() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![BoltLost]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.node_bolts_lost, 1,
            "1 BoltLost should increment node_bolts_lost to 1"
        );
    }
}
