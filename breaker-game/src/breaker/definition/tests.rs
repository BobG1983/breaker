use super::types::*;

// ── Behavior 1: BreakerDefinition parses RON with explicit bolt field ──

#[test]
fn breaker_definition_parses_ron_with_explicit_bolt_field() {
    let ron_str = r#"(
        name: "Aegis",
        bolt: "HeavyBolt",
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
    let ron_str = r#"(name: "TestBreaker", bolt: "HeavyBolt", effects: [])"#;
    let def: BreakerDefinition = ron::de::from_str(ron_str).unwrap();
    let cloned = def.clone();
    assert_eq!(cloned.bolt, "HeavyBolt");
    assert_eq!(cloned.name, "TestBreaker");
    // Verify original is still intact after clone
    assert_eq!(def.bolt, "HeavyBolt");
}

#[test]
fn breaker_definition_clone_preserves_default_bolt_value() {
    let ron_str = r#"(name: "TestBreaker", effects: [])"#;
    let def: BreakerDefinition = ron::de::from_str(ron_str).unwrap();
    let cloned = def.clone();
    assert_eq!(cloned.bolt, "Bolt");
    // Verify original is still intact after clone
    assert_eq!(def.bolt, "Bolt");
}
