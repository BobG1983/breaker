//! First-failure screenshot tracking: detects new invariant violations,
//! formats screenshot output paths, and tracks which kinds have been captured.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};

use super::super::ViolationLog;
use crate::types::InvariantKind;

/// Tracks which [`InvariantKind`] values have already had a screenshot taken
/// during this scenario run, ensuring at most one screenshot per kind.
#[derive(Resource, Default)]
pub struct ScreenshotTracker {
    /// The set of invariant kinds that have already been captured.
    pub captured: HashSet<InvariantKind>,
}

impl ScreenshotTracker {
    /// Records that a screenshot has been taken for the given invariant kind.
    pub fn mark_captured(&mut self, kind: InvariantKind) {
        self.captured.insert(kind);
    }
}

/// Holds the scenario name (RON file stem), inserted by `run_scenario`.
#[derive(Resource)]
pub struct ScenarioName(pub String);

/// Holds the output directory path, inserted conditionally when running in visual mode.
#[derive(Resource)]
pub struct ScreenshotOutputDir(pub PathBuf);

/// Returns the set of [`InvariantKind`] values present in the [`ViolationLog`]
/// but not yet in `tracker.captured`. Pure function, no side effects.
#[must_use]
pub fn detect_new_violations(
    tracker: &ScreenshotTracker,
    log: &ViolationLog,
) -> HashSet<InvariantKind> {
    log.0
        .iter()
        .map(|entry| entry.invariant)
        .filter(|kind| !tracker.captured.contains(kind))
        .collect()
}

/// Constructs `<output_dir>/<scenario_name>-<kind:?>.png`.
#[must_use]
pub fn screenshot_path(output_dir: &Path, scenario_name: &str, kind: InvariantKind) -> PathBuf {
    output_dir.join(format!("{scenario_name}-{kind:?}.png"))
}

/// Detects new invariant violations and captures screenshots for each.
///
/// Early-returns when `ScreenshotOutputDir` or `ScenarioName` resources are
/// absent (headless mode). Otherwise, for each new violation kind not yet in
/// the tracker, creates the output directory, spawns a `Screenshot` entity
/// with a `save_to_disk` observer, and marks the kind as captured.
pub fn capture_violation_screenshots(
    mut commands: Commands,
    mut tracker: ResMut<ScreenshotTracker>,
    vl: Res<ViolationLog>,
    output_dir: Option<Res<ScreenshotOutputDir>>,
    scenario_name: Option<Res<ScenarioName>>,
) {
    let (Some(output_dir), Some(scenario_name)) = (output_dir, scenario_name) else {
        return;
    };

    let new_kinds = detect_new_violations(&tracker, &vl);
    if new_kinds.is_empty() {
        return;
    }

    for kind in &new_kinds {
        let path = screenshot_path(&output_dir.0, &scenario_name.0, *kind);

        drop(std::fs::create_dir_all(
            path.parent().unwrap_or_else(|| Path::new(".")),
        ));

        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.clone()));

        tracker.mark_captured(*kind);

        println!("Screenshot requested: {}", path.display());
    }
}
