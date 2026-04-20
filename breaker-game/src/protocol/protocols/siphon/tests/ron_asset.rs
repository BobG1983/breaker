//! Group J — RON asset drift guard (Behaviors 46–47).
//!
//! Pins that `assets/protocols/siphon.protocol.ron` parses into a
//! `ProtocolDefinition` with `ProtocolKind::Siphon`, the authored tuning
//! values `streak_window: 2.0` / `time_per_kill: 0.25`, the exact name,
//! description, and `unlock_tier: 0`.
//!
//! NOTE: The RON's `time_per_kill: 0.25` differs from the design-doc
//! canonical `0.5` used by all system-behavior tests — this is intentional.
//! These drift-guard tests read the file literal via `include_str!`, not the
//! `canonical_siphon_config()` helper.

use crate::protocol::definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning};

// ── Behavior 46 — RON parses with the exact tuning values ───────────────────

#[test]
fn siphon_ron_asset_deserializes_to_protocol_definition() {
    let ron_str = include_str!("../../../../../assets/protocols/siphon.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("siphon.protocol.ron should parse");

    assert_eq!(
        def.kind(),
        ProtocolKind::Siphon,
        "parsed definition must report ProtocolKind::Siphon"
    );

    let ProtocolTuning::Siphon {
        streak_window,
        time_per_kill,
    } = def.tuning.clone()
    else {
        panic!("expected ProtocolTuning::Siphon, got {:?}", def.tuning);
    };

    assert!(
        (streak_window - 2.0).abs() < f32::EPSILON,
        "streak_window expected 2.0, got {streak_window}"
    );
    assert!(
        (time_per_kill - 0.25).abs() < f32::EPSILON,
        "time_per_kill expected 0.25 (RON file literal), got {time_per_kill}"
    );
}

// ── Behavior 47 — name / description / unlock_tier pinned exactly ───────────

#[test]
fn siphon_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/protocols/siphon.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("siphon.protocol.ron should parse");

    assert_eq!(def.name, "Siphon", "siphon name drift guard");
    assert_eq!(
        def.description,
        "Chain cell kills within a short window to bleed time from the node timer.",
        "siphon description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "siphon unlock_tier drift guard");
}
