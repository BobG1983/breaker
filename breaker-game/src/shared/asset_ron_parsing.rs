//! Cross-domain RON file parsing smoke tests — validates that every RON asset
//! in `assets/chips/`, `assets/bolts/`, `assets/breakers/`, `assets/cells/`,
//! and `assets/walls/` deserializes into its matching Rust type without error.
//!
//! Closes the "silent failure" hole flagged in the `effect_v3` audit (P0-1):
//! the asset pipeline would previously fail to load individual RON files at
//! runtime with no test coverage catching it. These tests use the same
//! `deserialize_ron` helper that `RonAssetLoader` uses (with the `IMPLICIT_SOME`
//! RON extension enabled) and panic loudly listing every file+error on failure.
//!
//! Node RON files (`assets/nodes/`) are covered separately by
//! `state/run/node/definition/tests/ron_parsing.rs` which also runs layout
//! validation against the cell registry.

use std::fs;

use rantzsoft_defaults::loader::deserialize_ron;
use serde::de::DeserializeOwned;

use crate::{
    bolt::definition::BoltDefinition,
    breaker::definition::BreakerDefinition,
    cells::definition::CellTypeDefinition,
    chips::definition::{ChipTemplate, EvolutionTemplate},
    walls::definition::WallDefinition,
};

/// Deserializes every `.ron` file in `dir` as `T` and collects failures.
///
/// Returns `(successful_count, failures)` where each failure is a formatted
/// `"  path: error"` line suitable for joining into a panic message.
fn parse_all_in_dir<T: DeserializeOwned>(dir: &str) -> (usize, Vec<String>) {
    let mut count = 0;
    let mut failures = Vec::new();
    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("failed to read directory {dir}: {e}"));
    for entry in entries {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) == Some("ron") {
            let content = fs::read(&path).unwrap();
            match deserialize_ron::<T>(&content) {
                Ok(_) => count += 1,
                Err(e) => failures.push(format!("  {}: {e}", path.display())),
            }
        }
    }
    (count, failures)
}

/// Asserts that every `.ron` file in `dir` deserializes as `T`, accumulating
/// all failures into a single panic message.
fn assert_all_parse<T: DeserializeOwned>(dir: &str, label: &str) {
    let (count, failures) = parse_all_in_dir::<T>(dir);
    assert!(
        count > 0 || !failures.is_empty(),
        "no .ron files found in {dir}"
    );
    assert!(
        failures.is_empty(),
        "{} {label} files failed to parse:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn all_chip_rons_parse() {
    assert_all_parse::<ChipTemplate>(
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/chips/standard"),
        "chip",
    );
}

#[test]
fn all_evolution_rons_parse() {
    assert_all_parse::<EvolutionTemplate>(
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/chips/evolutions"),
        "evolution",
    );
}

#[test]
fn all_bolt_rons_parse() {
    assert_all_parse::<BoltDefinition>(
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/bolts"),
        "bolt",
    );
}

#[test]
fn all_breaker_rons_parse() {
    assert_all_parse::<BreakerDefinition>(
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/breakers"),
        "breaker",
    );
}

#[test]
fn all_cell_rons_parse() {
    assert_all_parse::<CellTypeDefinition>(
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/cells"),
        "cell",
    );
}

#[test]
fn all_wall_rons_parse() {
    assert_all_parse::<WallDefinition>(
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/walls"),
        "wall",
    );
}
