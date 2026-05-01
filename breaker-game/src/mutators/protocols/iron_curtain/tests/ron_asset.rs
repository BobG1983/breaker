//! Group E — RON asset structural check (Behaviors 27–28).
//!
//! Pins that `assets/protocols/iron_curtain.protocol.ron` parses into a
//! `ProtocolDefinition` with `ProtocolKind::IronCurtain`, structural tuning
//! (finite, non-negative numeric fields), name `"Iron Curtain"`, the authored
//! description, and `unlock_tier: 0`.

use crate::mutators::protocols::definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning};

// ── Behavior 27 — RON parses with correct tuning variant + structural checks ─-

#[test]
fn iron_curtain_ron_asset_deserializes_to_protocol_definition() {
    let ron_str = include_str!("../../../../../assets/protocols/iron_curtain.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("iron_curtain.protocol.ron should parse");

    assert_eq!(
        def.kind(),
        ProtocolKind::IronCurtain,
        "parsed definition must report ProtocolKind::IronCurtain"
    );

    let ProtocolTuning::IronCurtain {
        damage_fraction,
        falloff_start,
    } = def.tuning.clone()
    else {
        panic!("expected ProtocolTuning::IronCurtain, got {:?}", def.tuning);
    };

    assert!(
        damage_fraction.is_finite() && damage_fraction >= 0.0,
        "damage_fraction must be finite and non-negative, got {damage_fraction}"
    );
    assert!(
        falloff_start.is_finite() && falloff_start >= 0.0,
        "falloff_start must be finite and non-negative, got {falloff_start}"
    );
}

// ── Behavior 28 — name / description / unlock_tier pinned exactly ──────────-

#[test]
fn iron_curtain_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/protocols/iron_curtain.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("iron_curtain.protocol.ron should parse");

    assert_eq!(def.name, "Iron Curtain", "iron_curtain name drift guard");
    assert_eq!(
        def.description,
        "Losing a bolt triggers a damage wave — fractional damage with distance falloff toward farther cells.",
        "iron_curtain description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "iron_curtain unlock_tier drift guard");
}
