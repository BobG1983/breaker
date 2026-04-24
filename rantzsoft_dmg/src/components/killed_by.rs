//! `KilledBy` kill-attribution component.

use bevy::prelude::*;

/// Post-mortem kill-attribution component.
///
/// Never default-spawned on live entities. Construct explicitly with
/// `KilledBy { killer: Some(e) }` for attributed kills or
/// `KilledBy { killer: None }` for environmental deaths.
#[derive(Component, Debug)]
pub struct KilledBy {
    /// The entity credited with the kill. `Some(entity)` for attributed
    /// kills, `None` for environmental deaths.
    pub killer: Option<Entity>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Behavior 12: KilledBy constructs explicitly with Some(entity) ──

    #[test]
    fn constructs_with_some_killer() {
        let e = Entity::PLACEHOLDER;
        let k = KilledBy { killer: Some(e) };
        assert_eq!(k.killer, Some(e));
    }

    // ── Behavior 13: KilledBy constructs with None for environmental kills ──

    #[test]
    fn constructs_with_none_killer() {
        let k = KilledBy { killer: None };
        assert!(k.killer.is_none());
    }

    // ── Behavior 14: KilledBy has NO Default impl (negative compile contract) ──
    //
    // The following single line is a documented negative contract. Leave it
    // commented. Uncommenting must cause the file to fail to compile with
    // E0599 (no method `default` on type `KilledBy`). If it ever compiles,
    // the Default contract has been broken.
    //
    // let _: KilledBy = KilledBy::default(); // Uncomment to verify KilledBy does NOT impl Default — must fail to compile.

    // ── Behavior 15: KilledBy derives Debug — formatter does not panic ──

    #[test]
    fn debug_format_contains_type_name_and_none() {
        let k = KilledBy { killer: None };
        let s = format!("{k:?}");
        assert!(s.contains("KilledBy"));
        assert!(s.contains("None"));
    }

    #[test]
    fn debug_format_with_some_killer_is_non_empty() {
        let k = KilledBy {
            killer: Some(Entity::PLACEHOLDER),
        };
        let s = format!("{k:?}");
        assert!(!s.is_empty());
    }

    // ── Behavior 16: KilledBy is a Bevy Component — spawnable and queryable ──

    #[test]
    fn spawns_as_component_in_bare_world() {
        let mut world = World::new();
        let e = world.spawn(KilledBy { killer: None }).id();
        let Some(k) = world.get::<KilledBy>(e) else {
            panic!("KilledBy component missing on spawned entity");
        };
        assert!(k.killer.is_none());
    }
}
