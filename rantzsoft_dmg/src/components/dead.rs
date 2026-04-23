//! `Dead` marker component for entities confirmed dead by a kill handler.

use bevy::prelude::*;

/// Marker component inserted on an entity confirmed dead by a kill handler.
///
/// Queries use `Without<Dead>` to skip entities that are already dying.
/// The entity itself is not removed by inserting `Dead`; a later system
/// is responsible for actual despawn.
#[derive(Component, Debug)]
pub struct Dead;

#[cfg(test)]
mod tests {
    use super::*;

    // ── Behavior 8: Dead is a Bevy Component — spawnable and queryable ──

    #[test]
    fn spawns_as_component_in_bare_world() {
        let mut world = World::new();
        let e = world.spawn(Dead).id();
        let d = world.get::<Dead>(e);
        assert!(d.is_some());
    }

    #[test]
    fn empty_spawn_does_not_get_dead() {
        let mut world = World::new();
        let e = world.spawn_empty().id();
        assert!(world.get::<Dead>(e).is_none());
    }

    // ── Behavior 9: Dead derives Debug — formatter does not panic ──

    #[test]
    fn debug_format_contains_type_name() {
        let d = Dead;
        let s = format!("{d:?}");
        assert!(s.contains("Dead"));
    }
}
