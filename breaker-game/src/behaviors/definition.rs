//! Archetype definition types — RON-deserialized data structures.

use bevy::prelude::*;
use serde::Deserialize;

use crate::chips::definition::TriggerChain;

/// A breaker archetype definition loaded from a RON file.
///
/// Uses unified `TriggerChain` for all behavior bindings.
/// Adding a new archetype = new RON file. Adding a new behavior type =
/// new `TriggerChain` variant + handler.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub(crate) struct ArchetypeDefinition {
    /// Display name of the archetype.
    pub name: String,
    /// Optional stat overrides applied on top of `BreakerDefaults`.
    pub stat_overrides: BreakerStatOverrides,
    /// Number of lives, if the archetype uses a life pool.
    pub life_pool: Option<u32>,
    /// Chain fired when a bolt is lost.
    pub on_bolt_lost: Option<TriggerChain>,
    /// Chain fired on a perfect bump.
    pub on_perfect_bump: Option<TriggerChain>,
    /// Chain fired on an early bump.
    pub on_early_bump: Option<TriggerChain>,
    /// Chain fired on a late bump.
    pub on_late_bump: Option<TriggerChain>,
    /// Additional trigger chains (overclock-style multi-step chains).
    pub chains: Vec<TriggerChain>,
}

/// Optional overrides for `BreakerDefaults` fields.
///
/// Each `Some` field replaces the corresponding base value.
#[derive(Deserialize, Clone, Debug, Default)]
pub(crate) struct BreakerStatOverrides {
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
        let ron_str = include_str!("../../assets/archetypes/aegis.archetype.ron");
        let def: ArchetypeDefinition =
            ron::de::from_str(ron_str).expect("aegis archetype RON should parse");
        assert_eq!(def.name, "Aegis");
        assert_eq!(def.life_pool, Some(3));
        assert!(matches!(def.on_bolt_lost, Some(TriggerChain::LoseLife)));
        assert!(matches!(
            def.on_perfect_bump,
            Some(TriggerChain::SpeedBoost { multiplier, .. }) if (multiplier - 1.5).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn chrono_ron_file_parses() {
        let ron_str = include_str!("../../assets/archetypes/chrono.archetype.ron");
        let def: ArchetypeDefinition =
            ron::de::from_str(ron_str).expect("chrono archetype RON should parse");
        assert_eq!(def.name, "Chrono");
        assert!(def.life_pool.is_none());
        assert!(matches!(
            def.on_bolt_lost,
            Some(TriggerChain::TimePenalty { seconds }) if (seconds - 5.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn prism_ron_file_parses() {
        let ron_str = include_str!("../../assets/archetypes/prism.archetype.ron");
        let def: ArchetypeDefinition =
            ron::de::from_str(ron_str).expect("prism archetype RON should parse");
        assert_eq!(def.name, "Prism");
        assert!(def.life_pool.is_none());
        assert!(matches!(def.on_perfect_bump, Some(TriggerChain::SpawnBolt)));
        assert!(matches!(
            def.on_bolt_lost,
            Some(TriggerChain::TimePenalty { seconds }) if (seconds - 7.0).abs() < f32::EPSILON
        ));
        assert!(def.on_early_bump.is_none());
        assert!(def.on_late_bump.is_none());
    }

    #[test]
    fn prism_ron_parses() {
        let ron_str = r#"
        (
            name: "Prism",
            stat_overrides: (),
            life_pool: None,
            on_bolt_lost: None,
            on_perfect_bump: Some(SpawnBolt),
            on_early_bump: None,
            on_late_bump: None,
            chains: [],
        )
        "#;
        let def: ArchetypeDefinition =
            ron::de::from_str(ron_str).expect("prism archetype RON should parse");
        assert_eq!(def.name, "Prism");
        assert!(def.life_pool.is_none());
        assert!(matches!(def.on_perfect_bump, Some(TriggerChain::SpawnBolt)));
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
            on_bolt_lost: Some(TimePenalty(seconds: 5.0)),
            on_perfect_bump: Some(SpeedBoost(target: Bolt, multiplier: 1.5)),
            on_early_bump: Some(SpeedBoost(target: Bolt, multiplier: 1.1)),
            on_late_bump: Some(SpeedBoost(target: Bolt, multiplier: 1.1)),
            chains: [],
        )
        "#;
        let def: ArchetypeDefinition =
            ron::de::from_str(ron_str).expect("chrono archetype RON should parse");
        assert_eq!(def.name, "Chrono");
        assert!(def.life_pool.is_none());
        assert!(matches!(
            def.on_bolt_lost,
            Some(TriggerChain::TimePenalty { seconds }) if (seconds - 5.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn archetype_with_chains_parses() {
        let ron_str = r#"
        (
            name: "Test",
            stat_overrides: (),
            life_pool: None,
            on_bolt_lost: None,
            on_perfect_bump: None,
            on_early_bump: None,
            on_late_bump: None,
            chains: [
                OnPerfectBump([OnImpact(Cell, [Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0)])]),
            ],
        )
        "#;
        let def: ArchetypeDefinition =
            ron::de::from_str(ron_str).expect("archetype with chains should parse");
        assert_eq!(def.chains.len(), 1);
    }

    #[test]
    fn apply_stat_overrides_partial() {
        use super::super::init::apply_stat_overrides;
        use crate::breaker::resources::BreakerConfig;

        let mut config = BreakerConfig::default();
        let original_max_speed = config.max_speed;

        let overrides = BreakerStatOverrides {
            width: Some(200.0),
            height: Some(30.0),
            ..default()
        };

        apply_stat_overrides(&mut config, &overrides);

        assert!((config.width - 200.0).abs() < f32::EPSILON);
        assert!((config.height - 30.0).abs() < f32::EPSILON);
        assert!(
            (config.max_speed - original_max_speed).abs() < f32::EPSILON,
            "unset fields should remain unchanged"
        );
    }
}
