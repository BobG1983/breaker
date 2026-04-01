//! Breaker definition — RON-deserialized breaker data.

use bevy::prelude::*;
use serde::Deserialize;

use crate::effect::RootEffect;

/// Default value for `bolt` when omitted from RON.
fn default_bolt_name() -> String {
    "Bolt".to_owned()
}

/// A breaker definition loaded from a RON file.
///
/// Uses `RootEffect` for all behavior bindings. Each top-level effect is
/// wrapped in `On(target, ...)` to specify the target entity.
/// Adding a new breaker = new RON file. Adding a new behavior type =
/// new `Effect` variant + handler.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct BreakerDefinition {
    /// Display name of the breaker.
    pub name: String,
    /// Name of the bolt definition this breaker uses.
    #[serde(default = "default_bolt_name")]
    pub bolt: String,
    /// Optional stat overrides applied on top of `BreakerDefaults`.
    pub stat_overrides: BreakerStatOverrides,
    /// Number of lives, if the breaker uses a life pool.
    pub life_pool: Option<u32>,
    /// All effect chains for this breaker, each scoped to a target entity.
    pub effects: Vec<RootEffect>,
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

    // ── Behavior 1: BreakerDefinition parses RON with explicit bolt field ──

    #[test]
    fn breaker_definition_parses_ron_with_explicit_bolt_field() {
        let ron_str = r#"(
            name: "Aegis",
            bolt: "HeavyBolt",
            stat_overrides: (),
            life_pool: Some(3),
            effects: [],
        )"#;
        let def: BreakerDefinition =
            ron::de::from_str(ron_str).expect("RON with explicit bolt field should parse");
        assert_eq!(def.bolt, "HeavyBolt");
        assert_eq!(def.name, "Aegis");
        assert_eq!(def.life_pool, Some(3));
    }

    #[test]
    fn breaker_definition_parses_ron_with_empty_bolt_field() {
        let ron_str = r#"(
            name: "Aegis",
            bolt: "",
            stat_overrides: (),
            life_pool: Some(3),
            effects: [],
        )"#;
        let def: BreakerDefinition =
            ron::de::from_str(ron_str).expect("RON with empty bolt field should parse");
        assert_eq!(def.bolt, "");
    }

    // ── Behavior 2: BreakerDefinition serde default for bolt field is "Bolt" ──

    #[test]
    fn breaker_definition_defaults_bolt_to_bolt_when_omitted() {
        let ron_str = r#"(
            name: "Chrono",
            stat_overrides: (),
            life_pool: None,
            effects: [],
        )"#;
        let def: BreakerDefinition =
            ron::de::from_str(ron_str).expect("RON without bolt field should parse");
        assert_eq!(def.bolt, "Bolt");
    }

    #[test]
    fn breaker_definition_defaults_bolt_with_all_other_fields_present() {
        let ron_str = r#"(
            name: "Aegis",
            stat_overrides: (
                width: Some(200.0),
                height: Some(30.0),
            ),
            life_pool: Some(3),
            effects: [],
        )"#;
        let def: BreakerDefinition =
            ron::de::from_str(ron_str).expect("RON with all other fields should parse");
        assert_eq!(
            def.bolt, "Bolt",
            "bolt should default to \"Bolt\" when omitted, even with all other fields present"
        );
    }

    // ── Behavior 3: Existing aegis.bdef.ron parses with bolt defaulting to "Bolt" ──

    #[test]
    fn aegis_bdef_ron_parses_with_bolt_defaulting_to_bolt() {
        let ron_str = include_str!("../../assets/breakers/aegis.bdef.ron");
        let def: BreakerDefinition =
            ron::de::from_str(ron_str).expect("aegis.bdef.ron should parse");
        assert_eq!(def.name, "Aegis");
        assert_eq!(def.bolt, "Bolt");
        assert_eq!(def.life_pool, Some(3));
    }

    // ── Behavior 4: Existing chrono.bdef.ron parses with bolt defaulting to "Bolt" ──

    #[test]
    fn chrono_bdef_ron_parses_with_bolt_defaulting_to_bolt() {
        let ron_str = include_str!("../../assets/breakers/chrono.bdef.ron");
        let def: BreakerDefinition =
            ron::de::from_str(ron_str).expect("chrono.bdef.ron should parse");
        assert_eq!(def.name, "Chrono");
        assert_eq!(def.bolt, "Bolt");
        assert_eq!(def.life_pool, None);
    }

    // ── Behavior 5: Existing prism.bdef.ron parses with bolt defaulting to "Bolt" ──

    #[test]
    fn prism_bdef_ron_parses_with_bolt_defaulting_to_bolt() {
        let ron_str = include_str!("../../assets/breakers/prism.bdef.ron");
        let def: BreakerDefinition =
            ron::de::from_str(ron_str).expect("prism.bdef.ron should parse");
        assert_eq!(def.name, "Prism");
        assert_eq!(def.bolt, "Bolt");
        assert_eq!(def.life_pool, None);
    }

    // ── Behavior 6: BreakerDefinition clone preserves bolt field ──

    #[test]
    fn breaker_definition_clone_preserves_bolt_field() {
        let def = BreakerDefinition {
            name: "TestBreaker".to_owned(),
            bolt: "HeavyBolt".to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            effects: vec![],
        };
        let cloned = def.clone();
        assert_eq!(cloned.bolt, "HeavyBolt");
        assert_eq!(cloned.name, "TestBreaker");
        // Verify original is still intact after clone
        assert_eq!(def.bolt, "HeavyBolt");
    }

    #[test]
    fn breaker_definition_clone_preserves_default_bolt_value() {
        let def = BreakerDefinition {
            name: "TestBreaker".to_owned(),
            bolt: "Bolt".to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            effects: vec![],
        };
        let cloned = def.clone();
        assert_eq!(cloned.bolt, "Bolt");
        // Verify original is still intact after clone
        assert_eq!(def.bolt, "Bolt");
    }
}
