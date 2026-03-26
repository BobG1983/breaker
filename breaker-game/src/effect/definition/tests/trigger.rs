use super::super::*;

// =========================================================================
// C7 Wave 1 Part B: Trigger enum new variants (behaviors 12-16)
// =========================================================================

#[test]
fn trigger_time_expires_constructs_and_clones() {
    let t = Trigger::TimeExpires(3.0);
    let cloned = t;
    assert_eq!(t, cloned);
    assert_eq!(t, Trigger::TimeExpires(3.0));
}

#[test]
fn trigger_time_expires_zero_is_valid() {
    let t = Trigger::TimeExpires(0.0);
    assert_eq!(t, Trigger::TimeExpires(0.0));
}

#[test]
fn trigger_on_death_constructs() {
    let t = Trigger::Death;
    assert!(matches!(t, Trigger::Death));
}

#[test]
fn trigger_on_death_distinct_from_bolt_lost() {
    assert_ne!(Trigger::Death, Trigger::BoltLost);
}

#[test]
fn trigger_ron_time_expires() {
    let t: Trigger = ron::de::from_str("TimeExpires(3.0)").expect("TimeExpires RON should parse");
    assert_eq!(t, Trigger::TimeExpires(3.0));
}

#[test]
fn trigger_ron_time_expires_zero() {
    let t: Trigger =
        ron::de::from_str("TimeExpires(0.0)").expect("TimeExpires(0.0) RON should parse");
    assert_eq!(t, Trigger::TimeExpires(0.0));
}

#[test]
fn trigger_ron_on_death() {
    let t: Trigger = ron::de::from_str("OnDeath").expect("OnDeath RON should parse");
    assert_eq!(t, Trigger::Death);
}

// =========================================================================
// C7 Wave 2a: OnNodeTimerThreshold RON deserialization (behavior 12)
// =========================================================================

#[test]
fn trigger_ron_on_node_timer_threshold() {
    let t: Trigger = ron::de::from_str("OnNodeTimerThreshold(0.25)")
        .expect("OnNodeTimerThreshold(0.25) RON should parse");
    assert_eq!(t, Trigger::NodeTimerThreshold(0.25));
}

#[test]
fn trigger_ron_on_node_timer_threshold_zero() {
    let t: Trigger = ron::de::from_str("OnNodeTimerThreshold(0.0)")
        .expect("OnNodeTimerThreshold(0.0) RON should parse");
    assert_eq!(t, Trigger::NodeTimerThreshold(0.0));
}

#[test]
fn trigger_ron_on_node_timer_threshold_one() {
    let t: Trigger = ron::de::from_str("OnNodeTimerThreshold(1.0)")
        .expect("OnNodeTimerThreshold(1.0) RON should parse");
    assert_eq!(t, Trigger::NodeTimerThreshold(1.0));
}

#[test]
fn trigger_ron_invalid_variant_fails() {
    let result = ron::de::from_str::<Trigger>("OnGameEnd");
    assert!(
        result.is_err(),
        "invalid trigger variant should fail to parse"
    );
}

#[test]
fn trigger_enum_has_all_patterns() {
    let triggers = [
        Trigger::PerfectBump,
        Trigger::Bump,
        Trigger::EarlyBump,
        Trigger::LateBump,
        Trigger::BumpWhiff,
        Trigger::Impact(ImpactTarget::Cell),
        Trigger::Impact(ImpactTarget::Breaker),
        Trigger::Impact(ImpactTarget::Wall),
        Trigger::CellDestroyed,
        Trigger::BoltLost,
        Trigger::Death,
        Trigger::NoBump,
        Trigger::PerfectBumped,
        Trigger::Bumped,
        Trigger::EarlyBumped,
        Trigger::LateBumped,
        Trigger::Impacted(ImpactTarget::Cell),
        Trigger::Impacted(ImpactTarget::Wall),
        Trigger::Impacted(ImpactTarget::Breaker),
        Trigger::Died,
        Trigger::DestroyedCell,
        Trigger::TimeExpires(1.0),
        Trigger::NodeTimerThreshold(0.25),
    ];
    assert_eq!(
        triggers.len(),
        23,
        "all 23 active trigger patterns (Selected deleted, 5 new targeted triggers added)"
    );
}

#[test]
fn trigger_is_copy_but_not_eq() {
    // Verify Copy works (f32 is Copy)
    let t = Trigger::TimeExpires(3.0);
    let copied = t; // Copy, not move
    let also = t; // still usable — proves Copy
    assert_eq!(copied, also);

    // Eq is NOT derived because f32 doesn't implement Eq.
    // We can only verify PartialEq works:
    assert_eq!(t, t);
}

// =========================================================================
// New targeted trigger variants — RON deserialization
// =========================================================================

#[test]
fn impacted_cell_deserializes_from_ron() {
    let t: Trigger = ron::de::from_str("Impacted(Cell)").expect("Impacted(Cell) RON should parse");
    assert_eq!(t, Trigger::Impacted(ImpactTarget::Cell));
}

#[test]
fn impacted_wall_deserializes_from_ron() {
    let t: Trigger = ron::de::from_str("Impacted(Wall)").expect("Impacted(Wall) RON should parse");
    assert_eq!(t, Trigger::Impacted(ImpactTarget::Wall));
}

#[test]
fn impacted_breaker_deserializes_from_ron() {
    let t: Trigger =
        ron::de::from_str("Impacted(Breaker)").expect("Impacted(Breaker) RON should parse");
    assert_eq!(t, Trigger::Impacted(ImpactTarget::Breaker));
}

#[test]
fn died_deserializes_from_ron() {
    let t: Trigger = ron::de::from_str("Died").expect("Died RON should parse");
    assert_eq!(t, Trigger::Died);
}

#[test]
fn destroyed_cell_deserializes_from_ron() {
    let t: Trigger = ron::de::from_str("DestroyedCell").expect("DestroyedCell RON should parse");
    assert_eq!(t, Trigger::DestroyedCell);
}

#[test]
fn selected_ron_fails_to_parse() {
    let result = ron::de::from_str::<Trigger>("OnSelected");
    assert!(
        result.is_err(),
        "OnSelected should fail to parse — Selected variant has been removed"
    );
}

// =========================================================================
// New targeted trigger variants — distinctness
// =========================================================================

#[test]
fn impacted_is_distinct_from_impact() {
    assert_ne!(
        Trigger::Impacted(ImpactTarget::Cell),
        Trigger::Impact(ImpactTarget::Cell),
        "Impacted(Cell) must be distinct from Impact(Cell)"
    );
}

#[test]
fn died_is_distinct_from_death() {
    assert_ne!(
        Trigger::Died,
        Trigger::Death,
        "Died must be distinct from Death"
    );
}

#[test]
fn destroyed_cell_is_distinct_from_cell_destroyed() {
    assert_ne!(
        Trigger::DestroyedCell,
        Trigger::CellDestroyed,
        "DestroyedCell must be distinct from CellDestroyed"
    );
}
