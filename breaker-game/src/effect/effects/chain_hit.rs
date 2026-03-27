//! Chain hit chip effect observer — bolt chains to N additional cells on hit.

use bevy::prelude::*;

use super::stack_u32;
use crate::{bolt::components::Bolt, chips::components::ChainHit};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a chain hit passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct ChainHitApplied {
    /// Chain hit count per stack.
    pub per_stack: u32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    // FUTURE: may be used for upcoming phases
    // /// Name of the chip that applied this effect.
    // pub chip_name: String,
}

/// Observer: applies chain hit stacking to all bolt entities.
pub(crate) fn handle_chain_hit(
    trigger: On<ChainHitApplied>,
    mut query: Query<(Entity, Option<&mut ChainHit>), With<Bolt>>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let per_stack = event.per_stack;
    let max_stacks = event.max_stacks;
    for (entity, mut existing) in &mut query {
        stack_u32(
            entity,
            existing.as_deref_mut().map(|c| &mut c.0),
            per_stack,
            max_stacks,
            &mut commands,
            ChainHit,
        );
    }
}

/// Registers all observers and systems for the chain hit effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_chain_hit);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_chain_hit);
        app
    }

    #[test]
    fn inserts_chain_hit_on_bolt() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(ChainHitApplied {
            per_stack: 2,
            max_stacks: 3,
        });
        app.world_mut().flush();

        let c = app.world().entity(bolt).get::<ChainHit>().unwrap();
        assert_eq!(c.0, 2);
    }

    #[test]
    fn stacks_chain_hit() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, ChainHit(2))).id();

        app.world_mut().commands().trigger(ChainHitApplied {
            per_stack: 2,
            max_stacks: 3,
        });
        app.world_mut().flush();

        let c = app.world().entity(bolt).get::<ChainHit>().unwrap();
        assert_eq!(c.0, 4, "ChainHit should stack from 2 to 4");
    }

    #[test]
    fn respects_max_stacks_chain_hit() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, ChainHit(6))).id();

        app.world_mut().commands().trigger(ChainHitApplied {
            per_stack: 2,
            max_stacks: 3,
        });
        app.world_mut().flush();

        let c = app.world().entity(bolt).get::<ChainHit>().unwrap();
        assert_eq!(
            c.0, 6,
            "ChainHit should not exceed max_stacks cap, got {}",
            c.0
        );
    }
}
