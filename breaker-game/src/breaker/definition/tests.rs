use ordered_float::OrderedFloat;

use super::types::*;
use crate::{
    breaker::components::BoltLossBehavior,
    effect_v3::types::{EffectType, EntityKind, RootNode, StampTarget, Tree, Trigger},
};

// ── Behavior 1: BreakerDefinition parses RON with explicit bolt field ──

#[test]
fn breaker_definition_parses_ron_with_explicit_bolt_field() {
    let ron_str = r#"(
        name: "Aegis",
        bolt: "HeavyBolt",
        life_pool: Some(3),
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

// ── Behavior 6: BreakerDefinition clone preserves bolt field ──

#[test]
fn breaker_definition_clone_preserves_bolt_field() {
    let ron_str = r#"(name: "TestBreaker", bolt: "HeavyBolt", salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))), effects: [])"#;
    let def: BreakerDefinition = ron::de::from_str(ron_str).unwrap();
    let cloned = def.clone();
    assert_eq!(cloned.bolt, "HeavyBolt");
    assert_eq!(cloned.name, "TestBreaker");
    // Verify original is still intact after clone
    assert_eq!(def.bolt, "HeavyBolt");
}

#[test]
fn breaker_definition_clone_preserves_default_bolt_value() {
    let ron_str = r#"(name: "TestBreaker", salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))), effects: [])"#;
    let def: BreakerDefinition = ron::de::from_str(ron_str).unwrap();
    let cloned = def.clone();
    assert_eq!(cloned.bolt, "Bolt");
    // Verify original is still intact after clone
    assert_eq!(def.bolt, "Bolt");
}

// ==========================================================================
// Wave 6C: salvo_hit required field
// ==========================================================================

// ── Behavior 27: BreakerDefinition with salvo_hit field parses from RON ──

#[test]
fn breaker_definition_salvo_hit_parses_time_penalty() {
    let ron_str = r#"(
        name: "TestBreaker",
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

// ── Behavior 29: BreakerDefinition without salvo_hit fails to parse ──

#[test]
fn breaker_definition_without_salvo_hit_fails_to_parse() {
    let ron_str = r#"(
        name: "TestBreaker",
        effects: [],
    )"#;
    let result = ron::de::from_str::<BreakerDefinition>(ron_str);
    assert!(
        result.is_err(),
        "RON without salvo_hit should fail to parse (required field)"
    );
}

// ==========================================================================
// Wave 3: BoltLossBehavior — definition wiring + RON migration
// ==========================================================================

// ── Behavior 13: BreakerDefinition::default().bolt_loss_behavior == LifeLoss(1) ──

#[test]
fn breaker_definition_default_has_bolt_loss_behavior_life_loss_one() {
    let def = BreakerDefinition::default();
    assert_eq!(
        def.bolt_loss_behavior,
        BoltLossBehavior::LifeLoss(1),
        "BreakerDefinition::default().bolt_loss_behavior must be LifeLoss(1)",
    );
}

// ── Behavior 14a: aegis.breaker.ron deserializes with bolt_loss_behavior == LifeLoss(1) ──

#[test]
fn aegis_breaker_ron_field_value_is_life_loss_one() {
    let ron_str = include_str!("../../../assets/breakers/aegis.breaker.ron");
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("aegis.breaker.ron should parse");
    assert_eq!(def.name, "Aegis");
    assert_eq!(
        def.bolt_loss_behavior,
        BoltLossBehavior::LifeLoss(1),
        "aegis.breaker.ron should produce bolt_loss_behavior == LifeLoss(1) \
         (either explicit in the file post-migration, or via #[serde(default)])",
    );
}

// ── Behavior 14b: aegis.breaker.ron does NOT contain `bolt_lost:` ──

#[test]
fn aegis_breaker_ron_does_not_contain_bolt_lost_field() {
    let ron_str = include_str!("../../../assets/breakers/aegis.breaker.ron");
    assert!(
        !ron_str.contains("bolt_lost:"),
        "aegis.breaker.ron must NOT contain `bolt_lost:` (legacy field). \
         Migration: replace `bolt_lost: ...` with `bolt_loss_behavior: LifeLoss(1)` \
         (or omit — default is LifeLoss(1)).",
    );
}

// ── Behavior 15a: chrono.breaker.ron deserializes with bolt_loss_behavior == TimeLoss(5.0) ──

#[test]
fn chrono_breaker_ron_field_value_is_time_loss_five() {
    let ron_str = include_str!("../../../assets/breakers/chrono.breaker.ron");
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("chrono.breaker.ron should parse");
    assert_eq!(def.name, "Chrono");
    assert_eq!(
        def.bolt_loss_behavior,
        BoltLossBehavior::TimeLoss(5.0),
        "chrono.breaker.ron should produce bolt_loss_behavior == TimeLoss(5.0). \
         Migration: add `bolt_loss_behavior: TimeLoss(5.0)` to the RON.",
    );
}

// ── Behavior 15b: chrono.breaker.ron does NOT contain `bolt_lost:` ──

#[test]
fn chrono_breaker_ron_does_not_contain_bolt_lost_field() {
    let ron_str = include_str!("../../../assets/breakers/chrono.breaker.ron");
    assert!(
        !ron_str.contains("bolt_lost:"),
        "chrono.breaker.ron must NOT contain `bolt_lost:` (legacy field). \
         Migration: replace `bolt_lost: ...` with `bolt_loss_behavior: TimeLoss(5.0)`.",
    );
}

// ── Behavior 16: prism.breaker.ron deserializes correctly ──

#[test]
fn prism_breaker_ron_deserializes_correctly() {
    let ron_str = include_str!("../../../assets/breakers/prism.breaker.ron");
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("prism.breaker.ron should parse");
    assert_eq!(def.name, "Prism");
    assert_eq!(
        def.life_pool, None,
        "prism.breaker.ron must have life_pool: None",
    );
    assert_eq!(
        def.bolt_loss_behavior,
        BoltLossBehavior::TimeLoss(7.0),
        "prism.breaker.ron must have bolt_loss_behavior: TimeLoss(7.0)",
    );
}

#[test]
fn prism_breaker_ron_salvo_hit_is_time_penalty_seven_seconds() {
    let ron_str = include_str!("../../../assets/breakers/prism.breaker.ron");
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("prism.breaker.ron should parse");
    let RootNode::Stamp(StampTarget::Breaker, ref outer_tree) = def.salvo_hit else {
        panic!(
            "prism.breaker.ron salvo_hit must be Stamp(Breaker, ...), got {:?}",
            def.salvo_hit
        );
    };
    let Tree::When(Trigger::Impacted(EntityKind::Salvo), ref inner_tree) = *outer_tree else {
        panic!(
            "prism.breaker.ron salvo_hit inner tree must be When(Impacted(Salvo), ...), \
             got {outer_tree:?}"
        );
    };
    let Tree::Fire(EffectType::TimePenalty(ref cfg)) = **inner_tree else {
        panic!(
            "prism.breaker.ron salvo_hit leaf must be Fire(TimePenalty(...)), got {inner_tree:?}"
        );
    };
    assert_eq!(
        cfg.seconds,
        OrderedFloat(7.0),
        "prism.breaker.ron salvo_hit TimePenalty seconds must be 7.0, got {:?}",
        cfg.seconds,
    );
}

#[test]
fn prism_breaker_ron_effects_contains_spawn_bolts_on_perfect_bump() {
    let ron_str = include_str!("../../../assets/breakers/prism.breaker.ron");
    let def: BreakerDefinition =
        ron::de::from_str(ron_str).expect("prism.breaker.ron should parse");
    assert_eq!(
        def.effects.len(),
        1,
        "prism.breaker.ron must have exactly 1 effect entry",
    );
    let RootNode::Stamp(StampTarget::Bolt, ref outer_tree) = def.effects[0] else {
        panic!(
            "prism.breaker.ron effects[0] must be Stamp(Bolt, ...), got {:?}",
            def.effects[0]
        );
    };
    let Tree::When(Trigger::PerfectBumpOccurred, ref inner_tree) = *outer_tree else {
        panic!(
            "prism.breaker.ron effects[0] inner tree must be \
             When(PerfectBumpOccurred, ...), got {outer_tree:?}"
        );
    };
    assert!(
        matches!(**inner_tree, Tree::Fire(EffectType::SpawnBolts(_))),
        "prism.breaker.ron effects[0] leaf must be Fire(SpawnBolts(...)), got {inner_tree:?}",
    );
}

#[test]
fn prism_breaker_ron_does_not_contain_legacy_bolt_lost_field() {
    let ron_str = include_str!("../../../assets/breakers/prism.breaker.ron");
    assert!(
        !ron_str.contains("bolt_lost:"),
        "prism.breaker.ron must NOT contain `bolt_lost:` (legacy field). \
         Use `bolt_loss_behavior: TimeLoss(7.0)` instead.",
    );
}
