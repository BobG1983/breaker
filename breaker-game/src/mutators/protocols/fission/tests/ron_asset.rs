//! Group K — RON asset + constants drift guards (Behaviors 39-40).
//!
//! Pins that `assets/protocols/fission.protocol.ron` parses into a
//! `ProtocolDefinition` with `ProtocolKind::Fission`, authored
//! `kills_per_split: 10`, name `"Fission"`, description, and `unlock_tier`.
//! Also pins the exact value of `FISSION_DIVERGENCE_ANGLE_RAD` against
//! `15.0_f32.to_radians()` (drift guard).

use super::super::system::FISSION_DIVERGENCE_ANGLE_RAD;
use crate::mutators::protocols::definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning};

// ── Behavior 39 — FISSION_DIVERGENCE_ANGLE_RAD drift guard ─────────────────-

#[test]
fn fission_divergence_angle_is_fifteen_degrees_in_radians() {
    let expected = 15.0_f32.to_radians();
    let delta = (FISSION_DIVERGENCE_ANGLE_RAD - expected).abs();
    assert!(
        delta < f32::EPSILON,
        "FISSION_DIVERGENCE_ANGLE_RAD drift guard: must equal 15.0_f32.to_radians() ({expected}); \
         got {FISSION_DIVERGENCE_ANGLE_RAD}, delta {delta}"
    );
}

// ── Behavior 40 — RON asset parses to Fission tuning kills_per_split: 10 ───-

#[test]
fn fission_ron_asset_deserializes_to_protocol_definition() {
    let ron_str = include_str!("../../../../../assets/protocols/fission.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("fission.protocol.ron should parse");

    assert_eq!(
        def.kind(),
        ProtocolKind::Fission,
        "parsed definition must report ProtocolKind::Fission"
    );

    let ProtocolTuning::Fission { kills_per_split } = def.tuning.clone() else {
        panic!("expected ProtocolTuning::Fission, got {:?}", def.tuning);
    };

    assert_eq!(
        kills_per_split, 10,
        "kills_per_split expected 10 (RON file literal); got {kills_per_split}"
    );
}

// ── Behavior 40 (edge case) — name / description / unlock_tier pinned ──────-

#[test]
fn fission_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/protocols/fission.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("fission.protocol.ron should parse");

    assert_eq!(def.name, "Fission", "fission name drift guard");
    assert_eq!(
        def.description, "Every Nth cell kill splits the active bolt into two.",
        "fission description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "fission unlock_tier drift guard");
}
