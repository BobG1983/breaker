use std::path::{Path, PathBuf};

use super::execution::scenarios_dir;
use crate::types::ScenarioDefinition;

pub(super) fn collect_scenario_paths(scenario: Option<&str>, all: bool) -> Vec<PathBuf> {
    let dir = scenarios_dir();

    if all {
        collect_all_scenarios(&dir)
    } else if let Some(name) = scenario {
        find_scenario_by_name(&dir, name).map_or_else(
            || {
                eprintln!("Scenario '{name}' not found in {}", dir.display());
                vec![]
            },
            |p| vec![p],
        )
    } else {
        vec![]
    }
}

fn collect_all_scenarios(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    collect_scenarios_recursive(dir, &mut paths);
    paths.sort();
    paths
}

fn collect_scenarios_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_scenarios_recursive(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("ron")
            && path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".scenario.ron"))
        {
            out.push(path);
        }
    }
}

fn find_scenario_by_name(dir: &Path, name: &str) -> Option<PathBuf> {
    let target = format!("{name}.scenario.ron");
    let mut all = Vec::new();
    collect_scenarios_recursive(dir, &mut all);
    all.into_iter().find(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n == target)
    })
}

pub(super) fn scenario_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .trim_end_matches(".scenario")
        .to_owned()
}

/// Loads and deserializes a [`ScenarioDefinition`] from a `.scenario.ron` file.
///
/// Returns `None` if the file cannot be read or parsed (errors are printed to stderr).
#[must_use]
pub fn load_scenario(path: &Path) -> Option<ScenarioDefinition> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| eprintln!("Failed to read {}: {e}", path.display()))
        .ok()?;
    ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str(&content)
        .map_err(|e| eprintln!("Failed to parse {}: {e}", path.display()))
        .ok()
}
