//! Archetype definition types — RON-deserialized data structures.

use bevy::prelude::*;
use serde::Deserialize;

/// A breaker archetype definition loaded from a RON file.
///
/// Composes behaviors from a closed set of triggers and consequences.
/// Adding a new archetype = new RON file. Adding a new behavior type =
/// new trigger/consequence variant + handler.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ArchetypeDefinition {
    /// Display name of the archetype.
    pub name: String,
    /// Optional stat overrides applied on top of `BreakerDefaults`.
    pub stat_overrides: BreakerStatOverrides,
    /// Number of lives, if the archetype uses a life pool.
    pub life_pool: Option<u32>,
    /// Trigger→consequence bindings that define this archetype's behaviors.
    pub behaviors: Vec<BehaviorBinding>,
}

/// A single trigger→consequence binding within an archetype.
#[derive(Deserialize, Clone, Debug)]
pub struct BehaviorBinding {
    /// One or more triggers that activate this consequence.
    pub triggers: Vec<Trigger>,
    /// The consequence to fire when any of the triggers occur.
    pub consequence: Consequence,
}

/// Events that can trigger a behavior consequence.
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Trigger {
    /// Bolt fell below the breaker.
    BoltLost,
    /// Bump timed within the perfect window.
    PerfectBump,
    /// Bump pressed before the perfect zone.
    EarlyBump,
    /// Bump pressed after the bolt hit.
    LateBump,
    /// Forward bump window expired without bolt contact.
    BumpWhiff,
}

/// Actions that occur when a trigger fires.
#[derive(Deserialize, Clone, Debug)]
pub enum Consequence {
    /// Lose one life from the life pool.
    LoseLife,
    /// Multiply bolt speed (applied at init time via components).
    BoltSpeedBoost(f32),
    /// Subtract seconds from the node timer.
    TimePenalty(f32),
}

/// Optional overrides for `BreakerDefaults` fields.
///
/// Each `Some` field replaces the corresponding base value.
#[derive(Deserialize, Clone, Debug, Default)]
pub struct BreakerStatOverrides {
    /// Override breaker width.
    pub width: Option<f32>,
    /// Override breaker height.
    pub height: Option<f32>,
    /// Override maximum horizontal speed.
    pub max_speed: Option<f32>,
    /// Override horizontal acceleration.
    pub acceleration: Option<f32>,
    /// Override horizontal deceleration.
    pub deceleration: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aegis_ron_parses() {
        let ron_str = include_str!("../../../assets/archetypes/aegis.archetype.ron");
        let def: ArchetypeDefinition =
            ron::de::from_str(ron_str).expect("aegis archetype RON should parse");
        assert_eq!(def.name, "Aegis");
        assert_eq!(def.life_pool, Some(3));
        assert_eq!(def.behaviors.len(), 3);
    }

    #[test]
    fn chrono_ron_file_parses() {
        let ron_str = include_str!("../../../assets/archetypes/chrono.archetype.ron");
        let def: ArchetypeDefinition =
            ron::de::from_str(ron_str).expect("chrono archetype RON should parse");
        assert_eq!(def.name, "Chrono");
        assert!(def.life_pool.is_none());
        assert_eq!(def.behaviors.len(), 3);
        assert!(matches!(
            def.behaviors[0].consequence,
            Consequence::TimePenalty(t) if (t - 5.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn trigger_equality() {
        assert_eq!(Trigger::BoltLost, Trigger::BoltLost);
        assert_ne!(Trigger::BoltLost, Trigger::PerfectBump);
    }

    #[test]
    fn default_stat_overrides_are_all_none() {
        let overrides = BreakerStatOverrides::default();
        assert!(overrides.width.is_none());
        assert!(overrides.height.is_none());
        assert!(overrides.max_speed.is_none());
        assert!(overrides.acceleration.is_none());
        assert!(overrides.deceleration.is_none());
    }

    #[test]
    fn chrono_ron_parses() {
        let ron_str = r#"
        (
            name: "Chrono",
            stat_overrides: (),
            life_pool: None,
            behaviors: [
                (triggers: [BoltLost], consequence: TimePenalty(5.0)),
                (triggers: [PerfectBump], consequence: BoltSpeedBoost(1.5)),
                (triggers: [EarlyBump, LateBump], consequence: BoltSpeedBoost(1.1)),
            ],
        )
        "#;
        let def: ArchetypeDefinition =
            ron::de::from_str(ron_str).expect("chrono archetype RON should parse");
        assert_eq!(def.name, "Chrono");
        assert!(def.life_pool.is_none());
        assert_eq!(def.behaviors.len(), 3);
        assert!(matches!(
            def.behaviors[0].consequence,
            Consequence::TimePenalty(t) if (t - 5.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn multi_trigger_binding_parses() {
        let ron_str = r#"
        (
            name: "Test",
            stat_overrides: (),
            life_pool: None,
            behaviors: [
                (triggers: [EarlyBump, LateBump], consequence: BoltSpeedBoost(0.8)),
            ],
        )
        "#;
        let def: ArchetypeDefinition =
            ron::de::from_str(ron_str).expect("multi-trigger RON should parse");
        assert_eq!(def.behaviors[0].triggers.len(), 2);
    }
}
