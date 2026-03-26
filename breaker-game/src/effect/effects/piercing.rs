//! Piercing chip effect observer — bolt passes through N cells.

use bevy::prelude::*;

use super::stack_u32;
use crate::{
    bolt::components::Bolt,
    chips::components::{Piercing, PiercingRemaining},
};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a piercing passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct PiercingApplied {
    /// Piercing count per stack.
    pub per_stack: u32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    /// Name of the chip that applied this effect.
    pub chip_name: String,
}

/// Query for bolts with optional piercing and active piercing tracking.
type PiercingQuery = (
    Entity,
    Option<&'static mut Piercing>,
    Option<&'static mut ActivePiercings>,
);

/// Observer: applies piercing stacking to all bolt entities.
///
/// Also pushes the piercing count onto each bolt's [`ActivePiercings`] vec
/// (if present) so that Until reversal can remove individual entries.
pub(crate) fn handle_piercing(
    trigger: On<PiercingApplied>,
    mut query: Query<PiercingQuery, With<Bolt>>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let per_stack = event.per_stack;
    let max_stacks = event.max_stacks;
    for (entity, mut existing, mut active_piercings) in &mut query {
        let had_piercing = existing.is_some();
        let piercing_before = existing.as_deref().map(|p| p.0);
        stack_u32(
            entity,
            existing.as_deref_mut().map(|c| &mut c.0),
            per_stack,
            max_stacks,
            &mut commands,
            Piercing,
        );
        if let Some(ref mut piercings) = active_piercings {
            piercings.0.push(per_stack);
        }
        if had_piercing {
            let piercing_after = existing.as_deref().map(|p| p.0);
            if piercing_after != piercing_before
                && let Some(stacked_count) = piercing_after
            {
                commands
                    .entity(entity)
                    .insert(PiercingRemaining(stacked_count));
            }
        } else {
            // stack_u32 inserted Piercing(per_stack) via deferred commands; mirror it
            commands.entity(entity).insert(PiercingRemaining(per_stack));
        }
    }
}

/// Per-bolt tracking of active piercing counts from individual chip stacks.
///
/// Each entry is a piercing count (e.g. 2 for "pierce 2 cells"). The
/// [`ActivePiercings::total`] method returns the sum of all entries, used
/// to recalculate `Piercing` and `PiercingRemaining` when stacks change.
/// Until reversal removes entries from the vec.
#[derive(Component, Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct ActivePiercings(pub Vec<u32>);

impl ActivePiercings {
    /// Returns the total piercing count: the sum of all active entries.
    ///
    /// Returns 0 if no entries are active (empty vec).
    pub fn total(&self) -> u32 {
        self.0.iter().copied().sum()
    }
}

/// Recalculates `Piercing` and `PiercingRemaining` from the sum of
/// [`ActivePiercings`] entries.
///
/// Runs after bridge dispatch and Until reversal to keep piercing values
/// consistent with the active stack vec.
pub(crate) fn apply_active_piercings(
    mut query: Query<(&mut Piercing, &mut PiercingRemaining, &ActivePiercings), With<Bolt>>,
) {
    for (mut piercing, mut remaining, active) in &mut query {
        let total = active.total();
        piercing.0 = total;
        remaining.0 = total;
    }
}

/// Registers all observers and systems for the piercing effect.
pub(crate) fn register(app: &mut App) {
    use crate::{
        effect::{effect_nodes::until, sets::EffectSystems},
        shared::PlayingState,
    };

    app.add_observer(handle_piercing);

    // Piercing recalculation — after bridge and Until reversal
    app.add_systems(
        FixedUpdate,
        apply_active_piercings
            .after(EffectSystems::Bridge)
            .after(until::tick_until_timers)
            .after(until::check_until_triggers)
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chips::components::PiercingRemaining;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_piercing);
        app
    }

    #[test]
    fn inserts_piercing_on_bolt() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(PiercingApplied {
            per_stack: 1,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let p = app.world().entity(bolt).get::<Piercing>().unwrap();
        assert_eq!(p.0, 1);
    }

    #[test]
    fn stacks_piercing() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, Piercing(1))).id();

        app.world_mut().commands().trigger(PiercingApplied {
            per_stack: 1,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let p = app.world().entity(bolt).get::<Piercing>().unwrap();
        assert_eq!(p.0, 2);
    }

    #[test]
    fn respects_max_stacks() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, Piercing(3))).id();

        app.world_mut().commands().trigger(PiercingApplied {
            per_stack: 1,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let p = app.world().entity(bolt).get::<Piercing>().unwrap();
        assert_eq!(p.0, 3);
    }

    // --- PiercingRemaining tests ---

    #[test]
    fn inserts_piercing_remaining_on_first_application() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(PiercingApplied {
            per_stack: 2,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let p = app.world().entity(bolt).get::<Piercing>().unwrap();
        assert_eq!(p.0, 2);
        let pr = app.world().entity(bolt).get::<PiercingRemaining>().unwrap();
        assert_eq!(pr.0, 2);
    }

    #[test]
    fn stacking_updates_piercing_remaining_to_new_value() {
        let mut app = test_app();
        let bolt = app
            .world_mut()
            .spawn((Bolt, Piercing(1), PiercingRemaining(0)))
            .id();

        app.world_mut().commands().trigger(PiercingApplied {
            per_stack: 1,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let p = app.world().entity(bolt).get::<Piercing>().unwrap();
        assert_eq!(p.0, 2);
        let pr = app.world().entity(bolt).get::<PiercingRemaining>().unwrap();
        assert_eq!(pr.0, 2);
    }

    #[test]
    fn max_stacks_leaves_piercing_remaining_unchanged() {
        let mut app = test_app();
        let bolt = app
            .world_mut()
            .spawn((Bolt, Piercing(3), PiercingRemaining(1)))
            .id();

        app.world_mut().commands().trigger(PiercingApplied {
            per_stack: 1,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let p = app.world().entity(bolt).get::<Piercing>().unwrap();
        assert_eq!(p.0, 3);
        let pr = app.world().entity(bolt).get::<PiercingRemaining>().unwrap();
        assert_eq!(pr.0, 1);
    }

    #[test]
    fn stacking_with_partial_remaining_refreshes_remaining() {
        let mut app = test_app();
        let bolt = app
            .world_mut()
            .spawn((Bolt, Piercing(1), PiercingRemaining(1)))
            .id();

        app.world_mut().commands().trigger(PiercingApplied {
            per_stack: 1,
            max_stacks: 3,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let p = app.world().entity(bolt).get::<Piercing>().unwrap();
        assert_eq!(p.0, 2, "Piercing should stack from 1 to 2");

        let pr = app.world().entity(bolt).get::<PiercingRemaining>().unwrap();
        assert_eq!(
            pr.0, 2,
            "PiercingRemaining should refresh to new Piercing total (2)"
        );
    }

    // =========================================================================
    // ActivePiercings — vec-based piercing management
    // =========================================================================

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // --- Test 1: handle_piercing pushes to ActivePiercings ---

    #[test]
    fn handle_piercing_pushes_to_active_piercings() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, ActivePiercings(vec![]))).id();

        app.world_mut().commands().trigger(PiercingApplied {
            per_stack: 2,
            max_stacks: 5,
            chip_name: String::new(),
        });
        app.world_mut().flush();

        let piercings = app
            .world()
            .entity(bolt)
            .get::<ActivePiercings>()
            .expect("bolt should have ActivePiercings");
        assert_eq!(
            piercings.0,
            vec![2],
            "ActivePiercings should contain [2] after PiercingApplied, got {:?}",
            piercings.0
        );
    }

    // --- Test 2: ActivePiercings::total returns sum ---

    #[test]
    fn active_piercings_total_returns_sum() {
        let piercings = ActivePiercings(vec![2, 1, 3]);
        let total = piercings.total();
        assert_eq!(total, 6, "total of [2, 1, 3] should be 6, got {total}");
    }

    // --- Test 3: apply_active_piercings recalculates from vec ---

    #[test]
    fn apply_active_piercings_recalculates_from_vec() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, apply_active_piercings);

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Piercing(99),
                PiercingRemaining(99),
                ActivePiercings(vec![2, 1]),
            ))
            .id();

        tick(&mut app);

        let p = app.world().entity(bolt).get::<Piercing>().unwrap();
        assert_eq!(
            p.0, 3,
            "Piercing should be recalculated to 3 (sum of [2, 1]), got {}",
            p.0
        );
        let pr = app.world().entity(bolt).get::<PiercingRemaining>().unwrap();
        assert_eq!(
            pr.0, 3,
            "PiercingRemaining should be recalculated to 3, got {}",
            pr.0
        );
    }

    // --- Test 4: apply_active_piercings empty vec sets zero ---

    #[test]
    fn apply_active_piercings_empty_vec_sets_zero() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, apply_active_piercings);

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Piercing(5),
                PiercingRemaining(3),
                ActivePiercings(vec![]),
            ))
            .id();

        tick(&mut app);

        let p = app.world().entity(bolt).get::<Piercing>().unwrap();
        assert_eq!(
            p.0, 0,
            "Piercing should be 0 with empty ActivePiercings, got {}",
            p.0
        );
    }
}
