//! Group H — RON asset drift guard (Behaviors 57–58).
//!
//! Pins that `assets/protocols/reckless_dash.protocol.ron` parses into a
//! `ProtocolDefinition` with `ProtocolKind::RecklessDash`, authored tuning
//! `risky_zone_start: 0.7, damage_multiplier: 4.0, double_penalty: true`,
//! name `"Reckless Dash"`, and pinned description / unlock tier.
//!
//! NOTE: The shipped RON file currently has `risky_zone_start: 0.3` — the
//! design doc is the source of truth (0.7 = "last 30% is risky"). The
//! implementation spec author is authorised to update the RON to 0.7; until
//! that change lands, Behavior 57 FAILS at assertion time, which is the
//! correct RED signal.

use crate::protocol::definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning};

// ── Behavior 57 — RON parses with exact tuning variant + values ────────────-

#[test]
fn reckless_dash_ron_asset_deserializes_to_protocol_definition() {
    let ron_str = include_str!("../../../../../assets/protocols/reckless_dash.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("reckless_dash.protocol.ron should parse");

    assert_eq!(
        def.kind(),
        ProtocolKind::RecklessDash,
        "parsed definition must report ProtocolKind::RecklessDash"
    );

    let ProtocolTuning::RecklessDash {
        risky_zone_start,
        damage_multiplier,
        double_penalty,
    } = &def.tuning
    else {
        panic!(
            "expected ProtocolTuning::RecklessDash, got {:?}",
            def.tuning
        );
    };

    assert!(
        (risky_zone_start - 0.7).abs() < f32::EPSILON,
        "risky_zone_start expected 0.7 (design-doc source of truth — last 30% \
         of dash is risky), got {risky_zone_start}"
    );
    assert!(
        (damage_multiplier - 4.0).abs() < f32::EPSILON,
        "damage_multiplier expected 4.0 (RON file literal), got {damage_multiplier}"
    );
    assert!(
        *double_penalty,
        "double_penalty expected true (RON file literal), got {double_penalty}"
    );
}

// ── Behavior 58 — name / description / unlock_tier pinned exactly ──────────-

#[test]
fn reckless_dash_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/protocols/reckless_dash.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("reckless_dash.protocol.ron should parse");

    assert_eq!(def.name, "Reckless Dash", "reckless_dash name drift guard");
    assert_eq!(
        def.description,
        "Dashing into the risky zone massively boosts damage — at double the bolt-lost cost.",
        "reckless_dash description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "reckless_dash unlock_tier drift guard");
}
