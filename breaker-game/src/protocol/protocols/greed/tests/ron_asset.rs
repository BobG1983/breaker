//! Group F — RON asset shape (Behaviors 33–34).
//!
//! Pins that `assets/protocols/greed.protocol.ron` parses into a
//! `ProtocolDefinition` with the canonical Greed tuning value and the exact
//! name, description, and unlock tier — drift guard.

use crate::protocol::definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning};

// ── Behavior 33 — greed.protocol.ron parses with rarity_boost_per_skip: 0.05 ──

#[test]
fn greed_ron_asset_deserializes_to_protocol_definition() {
    let ron_str = include_str!("../../../../../assets/protocols/greed.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("greed.protocol.ron should parse");

    assert_eq!(
        def.kind(),
        ProtocolKind::Greed,
        "parsed definition must report ProtocolKind::Greed"
    );

    let ProtocolTuning::Greed {
        rarity_boost_per_skip,
    } = def.tuning.clone()
    else {
        panic!("expected ProtocolTuning::Greed, got {:?}", def.tuning);
    };

    assert!(
        (rarity_boost_per_skip - 0.05).abs() < f32::EPSILON,
        "rarity_boost_per_skip expected 0.05 (fractional), got {rarity_boost_per_skip}"
    );
}

// ── Behavior 34 — name / description / unlock_tier pinned exactly ───────────

#[test]
fn greed_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/protocols/greed.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("greed.protocol.ron should parse");

    // The production-defined `ProtocolDefinition` fields are `pub(crate)` so
    // these accesses are valid from within this crate's test module.
    assert_eq!(def.name, "Greed", "greed name drift guard");
    assert_eq!(
        def.description, "Skipping a chip offer boosts the rare-chip weight next visit.",
        "greed description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "greed unlock_tier drift guard");
}
