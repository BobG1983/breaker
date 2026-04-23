//! `Invulnerable` marker component.

use bevy::prelude::*;

/// Marker component — any entity carrying this component is filtered out of
/// damage application.
///
/// Producers insert/remove the marker freely; the damage pipeline only reads
/// its presence via `Without<Invulnerable>`.
#[derive(Component, Debug)]
pub struct Invulnerable;

#[cfg(test)]
mod tests {
    use super::*;

    // ── Behavior 10: Invulnerable is a Bevy Component — spawnable and queryable ──

    #[test]
    fn spawns_as_component_in_bare_world() {
        let mut world = World::new();
        let e = world.spawn(Invulnerable).id();
        let i = world.get::<Invulnerable>(e);
        assert!(i.is_some());
    }

    #[test]
    fn empty_spawn_does_not_get_invulnerable() {
        let mut world = World::new();
        let e = world.spawn_empty().id();
        assert!(world.get::<Invulnerable>(e).is_none());
    }

    // ── Behavior 11: Invulnerable derives Debug — formatter does not panic ──

    #[test]
    fn debug_format_contains_type_name() {
        let i = Invulnerable;
        let s = format!("{i:?}");
        assert!(s.contains("Invulnerable"));
    }
}
