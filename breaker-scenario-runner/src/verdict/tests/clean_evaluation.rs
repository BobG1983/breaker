//! Tests for clean evaluation scenarios that evaluate to Pass.

use super::{super::evaluation::*, helpers::*};

// -------------------------------------------------------------------------
// Behavior 3: Clean run evaluates to Pass
// -------------------------------------------------------------------------

#[test]
fn clean_run_evaluates_to_pass() {
    let mut verdict = ScenarioVerdict::default();
    let stats = make_healthy_stats();
    let definition = make_chaos_definition();

    verdict.evaluate(&[], &[], &stats, &definition);

    assert!(verdict.passed(), "clean run must evaluate to Pass");
    assert!(
        verdict.reasons.is_empty(),
        "clean run must produce no reasons, got: {:?}",
        verdict.reasons
    );
}

// -------------------------------------------------------------------------
// Behavior 9: Empty expected list with no violations evaluates to Pass
// -------------------------------------------------------------------------

#[test]
fn empty_expected_list_with_no_violations_evaluates_to_pass() {
    let mut verdict = ScenarioVerdict::default();
    let stats = make_healthy_stats();
    let mut definition = make_chaos_definition();
    definition.allowed_failures = Some(vec![]);

    verdict.evaluate(&[], &[], &stats, &definition);

    assert!(
        verdict.passed(),
        "Some([]) expected with no violations must evaluate to Pass"
    );
}

// -------------------------------------------------------------------------
// Behavior 16: Healthy stats produce no health reasons
// -------------------------------------------------------------------------

#[test]
fn healthy_stats_produce_no_health_reasons() {
    let mut verdict = ScenarioVerdict::default();
    let stats = make_healthy_stats();
    let definition = make_chaos_definition();

    verdict.evaluate(&[], &[], &stats, &definition);

    assert!(
        verdict.passed(),
        "healthy stats with no violations or logs must evaluate to Pass"
    );
    assert!(
        verdict.reasons.is_empty(),
        "healthy stats must produce no reasons, got: {:?}",
        verdict.reasons
    );
}
