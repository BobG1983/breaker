//! System to initialize the clear-remaining count at node start.

use bevy::prelude::*;

use crate::{cells::components::RequiredToClear, state::run::node::ClearRemainingCount};

/// Counts all [`RequiredToClear`] entities and inserts [`ClearRemainingCount`].
///
/// Runs on `OnEnter(GameState::Playing)`, after `CellsSystems::Spawn`.
pub fn init_clear_remaining(query: Query<(), With<RequiredToClear>>, mut commands: Commands) {
    let remaining = u32::try_from(query.iter().count()).unwrap_or(0);
    commands.insert_resource(ClearRemainingCount { remaining });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::components::{Cell, RequiredToClear};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Startup, init_clear_remaining);
        app
    }

    #[test]
    fn counts_required_to_clear_entities() {
        let mut app = test_app();
        app.world_mut().spawn((Cell, RequiredToClear));
        app.world_mut().spawn((Cell, RequiredToClear));
        app.world_mut().spawn((Cell, RequiredToClear));
        app.world_mut().spawn(Cell); // not required
        app.update();

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(count.remaining, 3);
    }

    #[test]
    fn zero_when_no_required_cells() {
        let mut app = test_app();
        app.world_mut().spawn(Cell);
        app.update();

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(count.remaining, 0);
    }
}
