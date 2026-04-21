//! Group I — RON asset drift guard (Behaviors 62–63).
//!
//! Pins that `assets/protocols/echo_strike.protocol.ron` parses into a
//! `ProtocolDefinition` with `ProtocolKind::EchoStrike`, authored tuning
//! `max_echoes: 3, newest_fraction: 0.5, middle_fraction: 0.25,
//! oldest_fraction: 0.125`, name `"Echo Strike"`, and pinned description /
//! unlock tier.
//!
//! NOTE: The RON's `oldest_fraction: 0.125` DIFFERS from the design-doc
//! canonical `0.1` used by all system-behavior tests — this is intentional.

use crate::protocol::definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning};

// ── Behavior 62 — RON parses with exact tuning variant + values ────────────-

#[test]
fn echo_strike_ron_asset_deserializes_to_protocol_definition() {
    let ron_str = include_str!("../../../../../assets/protocols/echo_strike.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("echo_strike.protocol.ron should parse");

    assert_eq!(
        def.kind(),
        ProtocolKind::EchoStrike,
        "parsed definition must report ProtocolKind::EchoStrike"
    );

    let ProtocolTuning::EchoStrike {
        max_echoes,
        newest_fraction,
        middle_fraction,
        oldest_fraction,
    } = &def.tuning
    else {
        panic!("expected ProtocolTuning::EchoStrike, got {:?}", def.tuning);
    };

    assert_eq!(
        *max_echoes, 3,
        "max_echoes expected 3 (RON file literal), got {max_echoes}"
    );
    assert!(
        (newest_fraction - 0.5).abs() < f32::EPSILON,
        "newest_fraction expected 0.5 (RON file literal), got {newest_fraction}"
    );
    assert!(
        (middle_fraction - 0.25).abs() < f32::EPSILON,
        "middle_fraction expected 0.25 (RON file literal), got {middle_fraction}"
    );
    assert!(
        (oldest_fraction - 0.125).abs() < f32::EPSILON,
        "oldest_fraction expected 0.125 (RON file literal), got {oldest_fraction}"
    );
}

// ── Behavior 63 — name / description / unlock_tier pinned exactly ──────────-

#[test]
fn echo_strike_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/protocols/echo_strike.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("echo_strike.protocol.ron should parse");

    assert_eq!(def.name, "Echo Strike", "echo_strike name drift guard");
    assert_eq!(
        def.description,
        "Perfect bumps prime a bolt; its next cell hits echo fractional damage to the three most recent targets.",
        "echo_strike description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "echo_strike unlock_tier drift guard");
}
