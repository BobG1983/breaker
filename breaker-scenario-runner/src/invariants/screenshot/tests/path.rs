use std::path::{Path, PathBuf};

use super::super::system::*;
use crate::types::InvariantKind;

// =========================================================================
// screenshot_path
// =========================================================================

/// `screenshot_path` formats path as `<output_dir>/<scenario_name>-<Kind:?>.png`.
#[test]
fn screenshot_path_formats_with_output_dir_scenario_name_and_kind_debug() {
    let result = screenshot_path(
        Path::new("/tmp/breaker-scenario-runner/2026-04-06/0"),
        "aegis_chaos",
        InvariantKind::BoltInBounds,
    );
    assert_eq!(
        result,
        PathBuf::from("/tmp/breaker-scenario-runner/2026-04-06/0/aegis_chaos-BoltInBounds.png"),
        "path must be <output_dir>/<scenario_name>-<Kind:?>.png"
    );
}

/// `screenshot_path` uses the Debug format of `InvariantKind` for the
/// filename, including long variant names without truncation.
#[test]
fn screenshot_path_uses_debug_format_for_long_variant_name() {
    let result = screenshot_path(
        Path::new("/tmp/out/2026-04-06/1"),
        "bolt_speed_stress",
        InvariantKind::TimerMonotonicallyDecreasing,
    );
    assert_eq!(
        result,
        PathBuf::from("/tmp/out/2026-04-06/1/bolt_speed_stress-TimerMonotonicallyDecreasing.png"),
        "long variant name must not be truncated or mangled"
    );
}

/// `screenshot_path` preserves scenario names with underscores exactly.
#[test]
fn screenshot_path_preserves_underscored_scenario_name() {
    let result = screenshot_path(
        Path::new("/tmp/breaker-scenario-runner/2026-04-06/0"),
        "self_test_bolt_in_bounds",
        InvariantKind::NoNaN,
    );
    assert_eq!(
        result,
        PathBuf::from(
            "/tmp/breaker-scenario-runner/2026-04-06/0/self_test_bolt_in_bounds-NoNaN.png"
        ),
        "scenario name with underscores must be preserved exactly"
    );
}

/// `screenshot_path` handles minimal (short) path components correctly.
#[test]
fn screenshot_path_handles_minimal_output_dir() {
    let result = screenshot_path(Path::new("/out"), "test", InvariantKind::NoEntityLeaks);
    assert_eq!(
        result,
        PathBuf::from("/out/test-NoEntityLeaks.png"),
        "very short path components must still produce valid output"
    );
}
