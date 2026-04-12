use std::path::{Path, PathBuf};

use super::super::{super::ViolationEntry, system::*};
use crate::{invariants::ViolationLog, types::InvariantKind};

// =========================================================================
// Pure function coverage extensions
// =========================================================================

/// `detect_new_violations` returns all `InvariantKind::ALL` variants when
/// tracker is empty and log has one entry per variant.
#[test]
fn detect_new_violations_returns_all_variants_from_empty_tracker() {
    let tracker = ScreenshotTracker::default();
    let log = ViolationLog(
        InvariantKind::ALL
            .iter()
            .enumerate()
            .map(|(i, &kind)| ViolationEntry {
                frame:     u32::try_from(i).expect("frame index fits u32"),
                invariant: kind,
                entity:    None,
                message:   format!("{kind:?}"),
            })
            .collect(),
    );

    let result = detect_new_violations(&tracker, &log);

    assert_eq!(
        result.len(),
        InvariantKind::ALL.len(),
        "must return all {} variants when tracker is empty, got {}",
        InvariantKind::ALL.len(),
        result.len()
    );
    for &kind in InvariantKind::ALL {
        assert!(result.contains(&kind), "result must contain {kind:?}");
    }
}

/// `screenshot_path` uses flat format with hyphen separator for long variant names.
#[test]
fn screenshot_path_flat_format_with_long_variant_name() {
    let result = screenshot_path(
        Path::new("/tmp/out/2026-04-07/0"),
        "self_test_bolt_in_bounds",
        InvariantKind::AabbMatchesEntityDimensions,
    );
    assert_eq!(
        result,
        PathBuf::from(
            "/tmp/out/2026-04-07/0/self_test_bolt_in_bounds-AabbMatchesEntityDimensions.png"
        ),
        "flat format must use hyphen separator with long variant name"
    );
}
