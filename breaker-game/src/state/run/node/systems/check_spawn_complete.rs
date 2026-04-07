//! Coordinator system that fires [`SpawnNodeComplete`] when all domain spawn
//! signals have been received.

use bevy::prelude::*;
use rantzsoft_stateflow::ChangeState;

use crate::{
    bolt::messages::BoltSpawned,
    breaker::messages::BreakerSpawned,
    state::{
        run::node::messages::{CellsSpawned, SpawnNodeComplete},
        types::NodeState,
    },
    walls::messages::WallsSpawned,
};

/// Tracks which domain spawn signals have been received this node as a bitfield.
#[derive(Default)]
pub(crate) struct SpawnChecklist(u8);

impl SpawnChecklist {
    const BOLT: u8 = 1 << 0;
    const BREAKER: u8 = 1 << 1;
    const CELLS: u8 = 1 << 2;
    const WALLS: u8 = 1 << 3;
    /// All domain signals combined; spawn is complete when all bits are set.
    const ALL: u8 = Self::BOLT | Self::BREAKER | Self::CELLS | Self::WALLS;

    const fn is_complete(&self) -> bool {
        self.0 & Self::ALL == Self::ALL
    }
}

/// Reads spawn signals from each domain and fires [`SpawnNodeComplete`] +
/// [`ChangeState<NodeState>`] when all have arrived. The `ChangeState` message
/// triggers the lifecycle route `Loading → AnimateIn`.
pub(crate) fn check_spawn_complete(
    mut checklist: Local<SpawnChecklist>,
    mut bolt_reader: MessageReader<BoltSpawned>,
    mut breaker_reader: MessageReader<BreakerSpawned>,
    mut cells_reader: MessageReader<CellsSpawned>,
    mut walls_reader: MessageReader<WallsSpawned>,
    mut spawn_writer: MessageWriter<SpawnNodeComplete>,
    mut state_writer: MessageWriter<ChangeState<NodeState>>,
) {
    for _ in bolt_reader.read() {
        checklist.0 |= SpawnChecklist::BOLT;
    }
    for _ in breaker_reader.read() {
        checklist.0 |= SpawnChecklist::BREAKER;
    }
    for _ in cells_reader.read() {
        checklist.0 |= SpawnChecklist::CELLS;
    }
    for _ in walls_reader.read() {
        checklist.0 |= SpawnChecklist::WALLS;
    }

    if checklist.is_complete() {
        spawn_writer.write(SpawnNodeComplete);
        state_writer.write(ChangeState::new());
        *checklist = SpawnChecklist::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltSpawned>()
            .add_message::<BreakerSpawned>()
            .add_message::<CellsSpawned>()
            .add_message::<WallsSpawned>()
            .add_message::<SpawnNodeComplete>()
            .add_message::<ChangeState<NodeState>>()
            .add_systems(Update, check_spawn_complete);
        app
    }

    fn send_all_signals(app: &mut App) {
        app.world_mut()
            .resource_mut::<Messages<BoltSpawned>>()
            .write(BoltSpawned);
        app.world_mut()
            .resource_mut::<Messages<BreakerSpawned>>()
            .write(BreakerSpawned);
        app.world_mut()
            .resource_mut::<Messages<CellsSpawned>>()
            .write(CellsSpawned);
        app.world_mut()
            .resource_mut::<Messages<WallsSpawned>>()
            .write(WallsSpawned);
    }

    #[test]
    fn fires_spawn_node_complete_when_all_signals_received() {
        let mut app = test_app();
        send_all_signals(&mut app);
        app.update();

        let messages = app.world().resource::<Messages<SpawnNodeComplete>>();
        assert!(
            messages.iter_current_update_messages().count() > 0,
            "SpawnNodeComplete must be sent when all signals received"
        );
    }

    #[test]
    fn does_not_fire_when_signals_incomplete() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<Messages<BoltSpawned>>()
            .write(BoltSpawned);
        app.world_mut()
            .resource_mut::<Messages<BreakerSpawned>>()
            .write(BreakerSpawned);
        app.update();

        let messages = app.world().resource::<Messages<SpawnNodeComplete>>();
        assert_eq!(
            messages.iter_current_update_messages().count(),
            0,
            "SpawnNodeComplete must not fire with incomplete signals"
        );
    }

    /// Each bit must be independently required — sending all-but-one must not fire.
    #[test]
    fn does_not_fire_when_missing_bolt() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<Messages<BreakerSpawned>>()
            .write(BreakerSpawned);
        app.world_mut()
            .resource_mut::<Messages<CellsSpawned>>()
            .write(CellsSpawned);
        app.world_mut()
            .resource_mut::<Messages<WallsSpawned>>()
            .write(WallsSpawned);
        app.update();

        let messages = app.world().resource::<Messages<SpawnNodeComplete>>();
        assert_eq!(messages.iter_current_update_messages().count(), 0);
    }

    #[test]
    fn does_not_fire_when_missing_cells() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<Messages<BoltSpawned>>()
            .write(BoltSpawned);
        app.world_mut()
            .resource_mut::<Messages<BreakerSpawned>>()
            .write(BreakerSpawned);
        app.world_mut()
            .resource_mut::<Messages<WallsSpawned>>()
            .write(WallsSpawned);
        app.update();

        let messages = app.world().resource::<Messages<SpawnNodeComplete>>();
        assert_eq!(messages.iter_current_update_messages().count(), 0);
    }

    #[test]
    fn does_not_fire_when_missing_walls() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<Messages<BoltSpawned>>()
            .write(BoltSpawned);
        app.world_mut()
            .resource_mut::<Messages<BreakerSpawned>>()
            .write(BreakerSpawned);
        app.world_mut()
            .resource_mut::<Messages<CellsSpawned>>()
            .write(CellsSpawned);
        app.update();

        let messages = app.world().resource::<Messages<SpawnNodeComplete>>();
        assert_eq!(messages.iter_current_update_messages().count(), 0);
    }

    #[test]
    fn fires_after_signals_arrive_across_frames() {
        let mut app = test_app();

        // Frame 1: bolt + breaker
        app.world_mut()
            .resource_mut::<Messages<BoltSpawned>>()
            .write(BoltSpawned);
        app.world_mut()
            .resource_mut::<Messages<BreakerSpawned>>()
            .write(BreakerSpawned);
        app.update();

        let messages = app.world().resource::<Messages<SpawnNodeComplete>>();
        assert_eq!(
            messages.iter_current_update_messages().count(),
            0,
            "should not fire after frame 1"
        );

        // Frame 2: cells + walls
        app.world_mut()
            .resource_mut::<Messages<CellsSpawned>>()
            .write(CellsSpawned);
        app.world_mut()
            .resource_mut::<Messages<WallsSpawned>>()
            .write(WallsSpawned);
        app.update();

        let messages = app.world().resource::<Messages<SpawnNodeComplete>>();
        assert!(
            messages.iter_current_update_messages().count() > 0,
            "SpawnNodeComplete must fire once all signals arrive across frames"
        );
    }

    #[test]
    fn fires_again_on_new_node_entry() {
        let mut app = test_app();
        send_all_signals(&mut app);
        app.update();

        // Verify first fire
        let messages = app.world().resource::<Messages<SpawnNodeComplete>>();
        assert!(messages.iter_current_update_messages().count() > 0);

        // Flush message persistence (2 frames)
        app.update();
        app.update();
        app.update();

        // Simulate new node entry — send all signals again
        send_all_signals(&mut app);
        app.update();

        let messages = app.world().resource::<Messages<SpawnNodeComplete>>();
        assert!(
            messages.iter_current_update_messages().count() > 0,
            "SpawnNodeComplete must fire again for a new node entry"
        );
    }

    /// When all spawn signals arrive, `check_spawn_complete` must write
    /// `ChangeState<NodeState>` so the lifecycle route `Loading → AnimateIn`
    /// fires.
    #[test]
    fn sends_change_state_when_all_signals_received() {
        use rantzsoft_stateflow::ChangeState;

        use crate::state::types::NodeState;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltSpawned>()
            .add_message::<BreakerSpawned>()
            .add_message::<CellsSpawned>()
            .add_message::<WallsSpawned>()
            .add_message::<SpawnNodeComplete>()
            .add_message::<ChangeState<NodeState>>()
            .add_systems(Update, check_spawn_complete);

        send_all_signals(&mut app);
        app.update();

        let messages = app.world().resource::<Messages<ChangeState<NodeState>>>();
        assert!(
            messages.iter_current_update_messages().count() > 0,
            "ChangeState<NodeState> must be sent to advance Loading → AnimateIn"
        );
    }
}
