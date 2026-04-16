//! `EntityKind` — classifies entity types for trigger matching.

use serde::{Deserialize, Serialize};

/// Classifies an entity type for trigger and participant matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityKind {
    /// Matches cell entities.
    Cell,
    /// Matches bolt entities.
    Bolt,
    /// Matches wall entities.
    Wall,
    /// Matches breaker entities.
    Breaker,
    /// Matches salvo (turret projectile) entities.
    Salvo,
    /// Matches any entity type.
    Any,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Behavior 1: EntityKind::Salvo serializes to RON and back ──

    #[test]
    fn entity_kind_salvo_ron_round_trip() {
        let original = EntityKind::Salvo;
        let ron_str = ron::ser::to_string(&original).expect("serialize should succeed");
        let deserialized: EntityKind =
            ron::de::from_str(&ron_str).expect("deserialize should succeed");
        assert_eq!(deserialized, EntityKind::Salvo);
    }

    #[test]
    fn entity_kind_salvo_is_distinct_from_any() {
        assert_ne!(
            EntityKind::Salvo,
            EntityKind::Any,
            "Salvo and Any must be distinct variants"
        );
    }

    // ── Behavior 2: EntityKind::Salvo equality and hashing ──

    #[test]
    fn entity_kind_salvo_equals_itself() {
        let a = EntityKind::Salvo;
        let b = EntityKind::Salvo;
        assert_eq!(a, b);
    }

    #[test]
    fn entity_kind_salvo_is_not_bolt() {
        assert_ne!(
            EntityKind::Salvo,
            EntityKind::Bolt,
            "Salvo and Bolt must be distinct"
        );
    }

    // ── Behavior 3: EntityKind::Salvo debug output ──

    #[test]
    fn entity_kind_salvo_debug_contains_salvo() {
        let debug_str = format!("{:?}", EntityKind::Salvo);
        assert!(
            debug_str.contains("Salvo"),
            "debug output should contain 'Salvo', got: {debug_str}"
        );
    }
}
