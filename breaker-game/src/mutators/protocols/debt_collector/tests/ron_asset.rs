//! Group K — RON asset structural check (Behaviors 55–56).
//!
//! Pins that `assets/protocols/debt_collector.protocol.ron` parses into a
//! `ProtocolDefinition` with `ProtocolKind::DebtCollector`, structural tuning
//! (finite, non-negative numeric fields), name `"Debt Collector"`, and
//! description / unlock tier values.

use crate::mutators::protocols::definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning};

// ── Behavior 55 — RON parses with correct tuning variant + structural checks ─-

#[test]
fn debt_collector_ron_asset_deserializes_to_protocol_definition() {
    let ron_str = include_str!("../../../../../assets/protocols/debt_collector.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("debt_collector.protocol.ron should parse");

    assert_eq!(
        def.kind(),
        ProtocolKind::DebtCollector,
        "parsed definition must report ProtocolKind::DebtCollector"
    );

    let ProtocolTuning::DebtCollector { stack_per_bump } = def.tuning.clone() else {
        panic!(
            "expected ProtocolTuning::DebtCollector, got {:?}",
            def.tuning
        );
    };

    assert!(
        stack_per_bump.is_finite() && stack_per_bump >= 0.0,
        "stack_per_bump must be finite and non-negative, got {stack_per_bump}"
    );
}

// ── Behavior 56 — name / description / unlock_tier pinned exactly ──────────-

#[test]
fn debt_collector_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/protocols/debt_collector.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("debt_collector.protocol.ron should parse");

    assert_eq!(
        def.name, "Debt Collector",
        "debt_collector name drift guard"
    );
    assert_eq!(
        def.description,
        "Early and Late bumps stack damage debt; a Perfect bump cashes it out on the next cell kill. Bolt loss resets the stack.",
        "debt_collector description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "debt_collector unlock_tier drift guard");
}
