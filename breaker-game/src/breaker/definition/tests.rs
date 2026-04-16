use super::types::*;
use crate::effect_v3::types::{EntityKind, RootNode, StampTarget, Tree, Trigger};

// ── Behavior 1: BreakerDefinition parses RON with explicit bolt field ──

#[test]
fn breaker_definition_parses_ron_with_explicit_bolt_field() {
    let ron_str = r#"(
        name: "Aegis",
        bolt: "HeavyBolt",
        life_pool: Some(3),
        bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))),
        salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))),
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
        life_pool: Some(3),
        bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))),
        salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))),
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
        life_pool: None,
        bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))),
        salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))),
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
        life_pool: Some(3),
        bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))),
        salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))),
        effects: [],
    )"#;
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("RON with all other fields should parse");
    assert_eq!(
        def.bolt, "Bolt",
        "bolt should default to \"Bolt\" when omitted, even with all other fields present"
    );
}

// ── Behavior 3: Existing aegis.breaker.ron parses with bolt defaulting to "Bolt" ──

#[test]
fn aegis_breaker_ron_parses_with_bolt_defaulting_to_bolt() {
    let ron_str = include_str!("../../../assets/breakers/aegis.breaker.ron");
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("aegis.breaker.ron should parse");
    assert_eq!(def.name, "Aegis");
    assert_eq!(def.bolt, "Bolt");
    assert_eq!(def.life_pool, Some(3));
}

// ── Behavior 4: Existing chrono.breaker.ron parses with bolt defaulting to "Bolt" ──

#[test]
fn chrono_breaker_ron_parses_with_bolt_defaulting_to_bolt() {
    let ron_str = include_str!("../../../assets/breakers/chrono.breaker.ron");
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("chrono.breaker.ron should parse");
    assert_eq!(def.name, "Chrono");
    assert_eq!(def.bolt, "Bolt");
    assert_eq!(def.life_pool, None);
}

// ── Behavior 5: Existing prism.breaker.ron parses with bolt defaulting to "Bolt" ──

#[test]
fn prism_breaker_ron_parses_with_bolt_defaulting_to_bolt() {
    let ron_str = include_str!("../../../assets/breakers/prism.breaker.ron");
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("prism.breaker.ron should parse");
    assert_eq!(def.name, "Prism");
    assert_eq!(def.bolt, "Bolt");
    assert_eq!(def.life_pool, None);
}

// ── Behavior 6: BreakerDefinition clone preserves bolt field ──

#[test]
fn breaker_definition_clone_preserves_bolt_field() {
    let ron_str = r#"(name: "TestBreaker", bolt: "HeavyBolt", bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))), salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))), effects: [])"#;
    let def: BreakerDefinition = ron::de::from_str(ron_str).unwrap();
    let cloned = def.clone();
    assert_eq!(cloned.bolt, "HeavyBolt");
    assert_eq!(cloned.name, "TestBreaker");
    // Verify original is still intact after clone
    assert_eq!(def.bolt, "HeavyBolt");
}

#[test]
fn breaker_definition_clone_preserves_default_bolt_value() {
    let ron_str = r#"(name: "TestBreaker", bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))), salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))), effects: [])"#;
    let def: BreakerDefinition = ron::de::from_str(ron_str).unwrap();
    let cloned = def.clone();
    assert_eq!(cloned.bolt, "Bolt");
    // Verify original is still intact after clone
    assert_eq!(def.bolt, "Bolt");
}

// ==========================================================================
// Wave 6C: bolt_lost and salvo_hit required fields
// ==========================================================================

// ── Behavior 26: BreakerDefinition with bolt_lost field parses from RON ──

#[test]
fn breaker_definition_bolt_lost_parses_lose_life() {
    let ron_str = r#"(
        name: "TestBreaker",
        bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))),
        salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))),
        effects: [],
    )"#;
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("RON with bolt_lost should parse");
    assert!(
        matches!(
            def.bolt_lost,
            RootNode::Stamp(
                StampTarget::Breaker,
                Tree::When(Trigger::BoltLostOccurred, _)
            )
        ),
        "bolt_lost should be Stamp(Breaker, When(BoltLostOccurred, ...))"
    );
}

#[test]
fn breaker_definition_bolt_lost_round_trips() {
    let ron_str = r#"(
        name: "TestBreaker",
        bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))),
        salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))),
        effects: [],
    )"#;
    let def: BreakerDefinition = ron::de::from_str(ron_str).unwrap();
    let reserialized = ron::ser::to_string(&def.bolt_lost).expect("bolt_lost should serialize");
    let round_tripped: RootNode =
        ron::de::from_str(&reserialized).expect("bolt_lost should round-trip");
    assert_eq!(round_tripped, def.bolt_lost);
}

// ── Behavior 27: BreakerDefinition with salvo_hit field parses from RON ──

#[test]
fn breaker_definition_salvo_hit_parses_time_penalty() {
    let ron_str = r#"(
        name: "TestBreaker",
        bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))),
        salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(TimePenalty((seconds: 3.0))))),
        effects: [],
    )"#;
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("RON with salvo_hit TimePenalty should parse");
    assert!(
        matches!(
            def.salvo_hit,
            RootNode::Stamp(
                StampTarget::Breaker,
                Tree::When(Trigger::Impacted(EntityKind::Salvo), _)
            )
        ),
        "salvo_hit should be Stamp(Breaker, When(Impacted(Salvo), ...))"
    );
}

#[test]
fn breaker_definition_salvo_hit_parses_lose_life() {
    let ron_str = r#"(
        name: "TestBreaker",
        bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))),
        salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))),
        effects: [],
    )"#;
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("RON with salvo_hit LoseLife should parse");
    assert!(
        matches!(
            def.salvo_hit,
            RootNode::Stamp(
                StampTarget::Breaker,
                Tree::When(Trigger::Impacted(EntityKind::Salvo), _)
            )
        ),
        "salvo_hit with LoseLife is also valid"
    );
}

// ── Behavior 28: BreakerDefinition without bolt_lost fails to parse ──

#[test]
fn breaker_definition_without_bolt_lost_fails_to_parse() {
    let ron_str = r#"(
        name: "TestBreaker",
        salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))),
        effects: [],
    )"#;
    let result = ron::de::from_str::<BreakerDefinition>(ron_str);
    assert!(
        result.is_err(),
        "RON without bolt_lost should fail to parse (required field)"
    );
}

// ── Behavior 29: BreakerDefinition without salvo_hit fails to parse ──

#[test]
fn breaker_definition_without_salvo_hit_fails_to_parse() {
    let ron_str = r#"(
        name: "TestBreaker",
        bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))),
        effects: [],
    )"#;
    let result = ron::de::from_str::<BreakerDefinition>(ron_str);
    assert!(
        result.is_err(),
        "RON without salvo_hit should fail to parse (required field)"
    );
}

// ── Behavior 30: aegis.breaker.ron parses with bolt_lost and salvo_hit ──

#[test]
fn aegis_breaker_ron_parses_with_bolt_lost_and_salvo_hit() {
    let ron_str = include_str!("../../../assets/breakers/aegis.breaker.ron");
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("aegis.breaker.ron should parse with new fields");
    assert_eq!(def.name, "Aegis");
    assert_eq!(def.life_pool, Some(3));
    assert!(
        matches!(def.bolt_lost, RootNode::Stamp(StampTarget::Breaker, _)),
        "bolt_lost should be a valid Stamp RootNode"
    );
    assert!(
        matches!(def.salvo_hit, RootNode::Stamp(StampTarget::Breaker, _)),
        "salvo_hit should be a valid Stamp RootNode"
    );
}

// ── Behavior 31: chrono.breaker.ron parses with bolt_lost and salvo_hit ──

#[test]
fn chrono_breaker_ron_parses_with_bolt_lost_and_salvo_hit() {
    let ron_str = include_str!("../../../assets/breakers/chrono.breaker.ron");
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("chrono.breaker.ron should parse with new fields");
    assert_eq!(def.name, "Chrono");
    assert_eq!(def.life_pool, None);
    assert!(
        matches!(def.bolt_lost, RootNode::Stamp(StampTarget::Breaker, _)),
        "bolt_lost should be a valid Stamp RootNode"
    );
    assert!(
        matches!(def.salvo_hit, RootNode::Stamp(StampTarget::Breaker, _)),
        "salvo_hit should be a valid Stamp RootNode"
    );
}

// ── Behavior 32: prism.breaker.ron parses with bolt_lost and salvo_hit ──

#[test]
fn prism_breaker_ron_parses_with_bolt_lost_and_salvo_hit() {
    let ron_str = include_str!("../../../assets/breakers/prism.breaker.ron");
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("prism.breaker.ron should parse with new fields");
    assert_eq!(def.name, "Prism");
    assert!(
        matches!(def.bolt_lost, RootNode::Stamp(StampTarget::Breaker, _)),
        "bolt_lost should be a valid Stamp RootNode"
    );
    assert!(
        matches!(def.salvo_hit, RootNode::Stamp(StampTarget::Breaker, _)),
        "salvo_hit should be a valid Stamp RootNode"
    );
}

// ── Behavior 33: BreakerDefinition Default impl includes bolt_lost and salvo_hit ──

#[test]
fn breaker_definition_default_has_bolt_lost_and_salvo_hit() {
    let def = BreakerDefinition::default();
    // Both fields must be valid RootNodes (compile-time guarantee from the Default impl)
    assert!(
        matches!(def.bolt_lost, RootNode::Stamp(_, _)),
        "bolt_lost default should be a Stamp RootNode"
    );
    assert!(
        matches!(def.salvo_hit, RootNode::Stamp(_, _)),
        "salvo_hit default should be a Stamp RootNode"
    );
}

// ── Behavior 34: BreakerDefinition clone preserves bolt_lost and salvo_hit ──

#[test]
fn breaker_definition_clone_preserves_bolt_lost_and_salvo_hit() {
    let ron_str = r#"(
        name: "TestBreaker",
        bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(TimePenalty((seconds: 5.0))))),
        salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(TimePenalty((seconds: 3.0))))),
        effects: [],
    )"#;
    let def: BreakerDefinition = ron::de::from_str(ron_str).unwrap();
    let cloned = def.clone();
    assert_eq!(cloned.bolt_lost, def.bolt_lost);
    assert_eq!(cloned.salvo_hit, def.salvo_hit);
    // Original still intact
    assert!(matches!(def.bolt_lost, RootNode::Stamp(_, _)));
}
