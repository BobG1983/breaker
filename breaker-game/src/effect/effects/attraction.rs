//! Attraction chip effect observer — pulls nearby cells toward the bolt.

use bevy::prelude::*;

use super::stack_f32;
use crate::{bolt::components::Bolt, effect::definition::AttractionType};

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Attraction force magnitude pulling nearby cells toward the bolt.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct AttractionForce(pub f32);

/// Tracks per-type attraction state on a bolt entity.
#[derive(Component, Debug, Clone, Default)]
pub(crate) struct ActiveAttractions {
    /// Individual attraction entries by type.
    pub entries: Vec<AttractionEntry>,
}

/// A single attraction type entry with force magnitude and active state.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AttractionEntry {
    /// Which type of entity this attraction targets.
    pub attraction_type: AttractionType,
    /// Force magnitude for this attraction.
    pub force: f32,
    /// Whether this attraction is currently active.
    pub active: bool,
}

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when an attraction passive effect is applied via chip selection.
#[derive(Event, Clone, Debug)]
pub(crate) struct AttractionApplied {
    /// The type of entity attraction targets.
    pub attraction_type: AttractionType,
    /// Attraction force per stack.
    pub per_stack: f32,
    /// Maximum number of stacks allowed.
    pub max_stacks: u32,
    // FUTURE: may be used for upcoming phases
    // /// Name of the chip that applied this effect.
    // pub chip_name: String,
}

type AttractionQuery = (
    Entity,
    Option<&'static mut AttractionForce>,
    Option<&'static mut ActiveAttractions>,
);

/// Observer: applies attraction force stacking to all bolt entities and
/// stamps `ActiveAttractions` with per-type tracking.
pub(crate) fn handle_attraction(
    trigger: On<AttractionApplied>,
    mut query: Query<AttractionQuery, With<Bolt>>,
    mut commands: Commands,
) {
    let event = trigger.event();
    let per_stack = event.per_stack;
    let max_stacks = event.max_stacks;
    let attraction_type = event.attraction_type;
    for (entity, mut existing, active) in &mut query {
        stack_f32(
            entity,
            existing.as_deref_mut().map(|c| &mut c.0),
            per_stack,
            max_stacks,
            &mut commands,
            AttractionForce,
        );

        // Track per-type attraction state on the bolt.
        if let Some(mut aa) = active {
            if let Some(entry) = aa
                .entries
                .iter_mut()
                .find(|e| e.attraction_type == attraction_type)
            {
                entry.force += per_stack;
            } else {
                aa.entries.push(AttractionEntry {
                    attraction_type,
                    force: per_stack,
                    active: true,
                });
            }
        } else {
            commands.entity(entity).insert(ActiveAttractions {
                entries: vec![AttractionEntry {
                    attraction_type,
                    force: per_stack,
                    active: true,
                }],
            });
        }
    }
}

/// Registers all observers and systems for the attraction effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_attraction);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_observer(handle_attraction);
        app
    }

    #[test]
    fn inserts_attraction_force_on_bolt() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();
        // Non-bolt entity should NOT receive the component.
        let non_bolt = app.world_mut().spawn_empty().id();

        app.world_mut().commands().trigger(AttractionApplied {
            attraction_type: crate::effect::definition::AttractionType::Cell,
            per_stack: 8.0,
            max_stacks: 3,
        });
        app.world_mut().flush();

        let a = app
            .world()
            .entity(bolt)
            .get::<AttractionForce>()
            .expect("bolt should have AttractionForce after Attraction effect");
        assert!((a.0 - 8.0).abs() < f32::EPSILON);

        assert!(
            app.world()
                .entity(non_bolt)
                .get::<AttractionForce>()
                .is_none(),
            "non-bolt entity should NOT receive AttractionForce"
        );
    }

    #[test]
    fn stacks_attraction_force() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn((Bolt, AttractionForce(8.0))).id();

        app.world_mut().commands().trigger(AttractionApplied {
            attraction_type: crate::effect::definition::AttractionType::Cell,
            per_stack: 8.0,
            max_stacks: 3,
        });
        app.world_mut().flush();

        let a = app.world().entity(bolt).get::<AttractionForce>().unwrap();
        assert!(
            (a.0 - 16.0).abs() < f32::EPSILON,
            "AttractionForce should stack from 8.0 to 16.0, got {}",
            a.0
        );
    }

    #[test]
    fn respects_max_stacks_attraction_force() {
        let mut app = test_app();
        // 3 stacks at 8.0 per stack = 24.0, which is at the cap of max_stacks: 3.
        let bolt = app.world_mut().spawn((Bolt, AttractionForce(24.0))).id();

        app.world_mut().commands().trigger(AttractionApplied {
            attraction_type: crate::effect::definition::AttractionType::Cell,
            per_stack: 8.0,
            max_stacks: 3,
        });
        app.world_mut().flush();

        let a = app.world().entity(bolt).get::<AttractionForce>().unwrap();
        assert!(
            (a.0 - 24.0).abs() < f32::EPSILON,
            "AttractionForce should not exceed max_stacks cap, got {}",
            a.0
        );
    }

    // =========================================================================
    // Part A: handle_attraction stamps ActiveAttractions with type tracking
    // =========================================================================

    #[test]
    fn handle_attraction_stamps_active_attractions_with_applied_type() {
        let mut app = test_app();
        let bolt = app.world_mut().spawn(Bolt).id();

        app.world_mut().commands().trigger(AttractionApplied {
            attraction_type: AttractionType::Cell,
            per_stack: 8.0,
            max_stacks: 3,
        });
        app.world_mut().flush();

        let aa = app
            .world()
            .entity(bolt)
            .get::<ActiveAttractions>()
            .expect("bolt should have ActiveAttractions after AttractionApplied fires");
        assert_eq!(
            aa.entries.len(),
            1,
            "should have exactly 1 entry, got {}",
            aa.entries.len()
        );
        assert_eq!(
            aa.entries[0].attraction_type,
            AttractionType::Cell,
            "entry type should be Cell"
        );
        assert!(
            (aa.entries[0].force - 8.0).abs() < f32::EPSILON,
            "entry force should be 8.0, got {}",
            aa.entries[0].force
        );
        assert!(
            aa.entries[0].active,
            "newly inserted entry should be active"
        );
    }

    #[test]
    fn handle_attraction_adds_new_entry_for_different_type() {
        let mut app = test_app();
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                ActiveAttractions {
                    entries: vec![AttractionEntry {
                        attraction_type: AttractionType::Cell,
                        force: 8.0,
                        active: true,
                    }],
                },
            ))
            .id();

        app.world_mut().commands().trigger(AttractionApplied {
            attraction_type: AttractionType::Wall,
            per_stack: 4.0,
            max_stacks: 3,
        });
        app.world_mut().flush();

        let aa = app
            .world()
            .entity(bolt)
            .get::<ActiveAttractions>()
            .expect("bolt should still have ActiveAttractions");
        assert_eq!(
            aa.entries.len(),
            2,
            "should have 2 entries (Cell + Wall), got {}",
            aa.entries.len()
        );
        // Verify both entries present
        let cell_entry = aa
            .entries
            .iter()
            .find(|e| e.attraction_type == AttractionType::Cell);
        let wall_entry = aa
            .entries
            .iter()
            .find(|e| e.attraction_type == AttractionType::Wall);
        assert!(cell_entry.is_some(), "Cell entry should still exist");
        assert!(wall_entry.is_some(), "Wall entry should be added");
        assert!(
            (cell_entry.unwrap().force - 8.0).abs() < f32::EPSILON,
            "Cell force should remain 8.0"
        );
        assert!(
            (wall_entry.unwrap().force - 4.0).abs() < f32::EPSILON,
            "Wall force should be 4.0"
        );
    }

    #[test]
    fn handle_attraction_stacks_force_on_same_type() {
        let mut app = test_app();
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                ActiveAttractions {
                    entries: vec![AttractionEntry {
                        attraction_type: AttractionType::Cell,
                        force: 8.0,
                        active: true,
                    }],
                },
            ))
            .id();

        app.world_mut().commands().trigger(AttractionApplied {
            attraction_type: AttractionType::Cell,
            per_stack: 8.0,
            max_stacks: 3,
        });
        app.world_mut().flush();

        let aa = app
            .world()
            .entity(bolt)
            .get::<ActiveAttractions>()
            .expect("bolt should have ActiveAttractions");
        assert_eq!(
            aa.entries.len(),
            1,
            "should still have exactly 1 entry (no duplicate), got {}",
            aa.entries.len()
        );
        assert!(
            (aa.entries[0].force - 16.0).abs() < f32::EPSILON,
            "Cell force should stack from 8.0 to 16.0, got {}",
            aa.entries[0].force
        );
    }
}
