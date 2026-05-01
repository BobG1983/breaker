//! Group I — RON asset structural check (Behaviors 62–63).
//!
//! Pins that `assets/protocols/echo_strike.protocol.ron` parses into a
//! `ProtocolDefinition` with `ProtocolKind::EchoStrike`, structural tuning
//! (finite, non-negative numeric fields; positive count), name `"Echo Strike"`,
//! and pinned description / unlock tier.

use crate::mutators::protocols::definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning};

// ── Behavior 62 — RON parses with correct tuning variant + structural checks ─-

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
    } = def.tuning.clone()
    else {
        panic!("expected ProtocolTuning::EchoStrike, got {:?}", def.tuning);
    };

    assert!(
        max_echoes > 0,
        "max_echoes must be positive, got {max_echoes}"
    );
    assert!(
        newest_fraction.is_finite() && newest_fraction >= 0.0,
        "newest_fraction must be finite and non-negative, got {newest_fraction}"
    );
    assert!(
        middle_fraction.is_finite() && middle_fraction >= 0.0,
        "middle_fraction must be finite and non-negative, got {middle_fraction}"
    );
    assert!(
        oldest_fraction.is_finite() && oldest_fraction >= 0.0,
        "oldest_fraction must be finite and non-negative, got {oldest_fraction}"
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
