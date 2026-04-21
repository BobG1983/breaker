use super::{super::system::*, helpers::minimal_scenario};
use crate::types::ScenarioDefinition;

// -----------------------------------------------------------------------
// Behavior 3: Unused layout detection — finds unreferenced layout
// -----------------------------------------------------------------------

#[test]
fn unused_layout_detection_finds_unreferenced_layouts() {
    let scenarios = vec![(
        "corridor_test".to_owned(),
        minimal_scenario("Corridor", None),
    )];
    let self_test_names: Vec<String> = vec![];
    let layout_names = vec![
        "Corridor".to_owned(),
        "Fortress".to_owned(),
        "Scatter".to_owned(),
    ];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert_eq!(
        report.unused_layouts.len(),
        2,
        "expected 2 unused layouts, got: {:?}",
        report.unused_layouts
    );
    assert!(
        report.unused_layouts.contains(&"Fortress".to_owned()),
        "Fortress should be in unused_layouts"
    );
    assert!(
        report.unused_layouts.contains(&"Scatter".to_owned()),
        "Scatter should be in unused_layouts"
    );
    assert!(
        !report.unused_layouts.contains(&"Corridor".to_owned()),
        "Corridor is used and must not be in unused_layouts"
    );
}

#[test]
fn layout_name_comparison_is_case_insensitive() {
    // Scenario uses lowercase "corridor", layout file is "Corridor".
    let scenarios = vec![(
        "corridor_test".to_owned(),
        minimal_scenario("corridor", None),
    )];
    let self_test_names: Vec<String> = vec![];
    let layout_names = vec!["Corridor".to_owned()];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert!(
        report.unused_layouts.is_empty(),
        "case-insensitive match should mark Corridor as used, got unused: {:?}",
        report.unused_layouts
    );
}

#[test]
fn layout_name_comparison_normalizes_underscores_to_pascal_case() {
    // Layout file is "boss_arena" (snake_case), scenario uses "BossArena" (PascalCase).
    let scenarios = vec![("boss_test".to_owned(), minimal_scenario("BossArena", None))];
    let self_test_names: Vec<String> = vec![];
    let layout_names = vec!["boss_arena".to_owned()];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert!(
        report.unused_layouts.is_empty(),
        "boss_arena should match BossArena after normalization, got unused: {:?}",
        report.unused_layouts
    );
}

// -----------------------------------------------------------------------
// Behavior 4: All layouts used — empty unused list
// -----------------------------------------------------------------------

#[test]
fn all_layouts_used_produces_empty_unused_list() {
    let scenarios = vec![(
        "corridor_test".to_owned(),
        minimal_scenario("Corridor", None),
    )];
    let self_test_names: Vec<String> = vec![];
    let layout_names = vec!["Corridor".to_owned()];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert!(
        report.unused_layouts.is_empty(),
        "all layouts are used — unused_layouts should be empty, got: {:?}",
        report.unused_layouts
    );
}

// -----------------------------------------------------------------------
// Behavior 5 (spec): used_layouts populated with scenario counts
// -----------------------------------------------------------------------

#[test]
fn used_layouts_populated_with_scenario_counts() {
    let scenarios = vec![
        (
            "corridor_test".to_owned(),
            minimal_scenario("Corridor", None),
        ),
        (
            "corridor_chaos".to_owned(),
            minimal_scenario("Corridor", None),
        ),
        (
            "fortress_test".to_owned(),
            minimal_scenario("Fortress", None),
        ),
    ];
    let self_test_names: Vec<String> = vec![];
    let layout_names = vec![
        "Corridor".to_owned(),
        "Fortress".to_owned(),
        "Scatter".to_owned(),
    ];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert!(
        report.used_layouts.contains(&("Corridor".to_owned(), 2)),
        "used_layouts should contain (Corridor, 2), got: {:?}",
        report.used_layouts
    );
    assert!(
        report.used_layouts.contains(&("Fortress".to_owned(), 1)),
        "used_layouts should contain (Fortress, 1), got: {:?}",
        report.used_layouts
    );
    assert!(
        !report
            .used_layouts
            .iter()
            .any(|(name, _)| name == "Scatter"),
        "Scatter should not appear in used_layouts"
    );
    assert_eq!(
        report.unused_layouts,
        vec!["Scatter".to_owned()],
        "unused_layouts should contain only Scatter"
    );
}

// -----------------------------------------------------------------------
// Behavior 6 (spec): layout name normalization applies to used_layouts
// -----------------------------------------------------------------------

#[test]
fn used_layouts_counting_applies_normalization() {
    let scenarios = vec![
        (
            "corridor_lower".to_owned(),
            minimal_scenario("corridor", None),
        ),
        (
            "corridor_pascal".to_owned(),
            minimal_scenario("Corridor", None),
        ),
    ];
    let self_test_names: Vec<String> = vec![];
    let layout_names = vec!["Corridor".to_owned()];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert_eq!(
        report.used_layouts,
        vec![("Corridor".to_owned(), 2)],
        "both scenarios should match via normalization; name should be the layout_names entry"
    );
}

#[test]
fn used_layouts_normalization_strips_underscores() {
    // Layout file "boss_arena" (snake_case), scenario uses "BossArena" (PascalCase).
    let scenarios = vec![("boss_test".to_owned(), minimal_scenario("BossArena", None))];
    let self_test_names: Vec<String> = vec![];
    let layout_names = vec!["boss_arena".to_owned()];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert_eq!(
        report.used_layouts,
        vec![("boss_arena".to_owned(), 1)],
        "boss_arena should match BossArena after normalization"
    );
}

// -----------------------------------------------------------------------
// Behavior 7 (spec): all layouts used produces full used list
// -----------------------------------------------------------------------

#[test]
fn all_layouts_used_produces_full_used_list_and_empty_unused() {
    let scenarios = vec![(
        "corridor_test".to_owned(),
        minimal_scenario("Corridor", None),
    )];
    let self_test_names: Vec<String> = vec![];
    let layout_names = vec!["Corridor".to_owned()];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert_eq!(
        report.used_layouts,
        vec![("Corridor".to_owned(), 1)],
        "used_layouts should contain Corridor with count 1"
    );
    assert!(
        report.unused_layouts.is_empty(),
        "unused_layouts should be empty when all layouts are used"
    );
}

#[test]
fn no_layouts_produces_empty_used_and_unused() {
    let scenarios = vec![(
        "corridor_test".to_owned(),
        minimal_scenario("Corridor", None),
    )];
    let self_test_names: Vec<String> = vec![];
    let layout_names: Vec<String> = vec![];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert!(
        report.used_layouts.is_empty(),
        "used_layouts should be empty when no layout_names exist"
    );
    assert!(
        report.unused_layouts.is_empty(),
        "unused_layouts should be empty when no layout_names exist"
    );
}

// -----------------------------------------------------------------------
// Behavior 8 (spec): no scenarios means all layouts unused, none used
// -----------------------------------------------------------------------

#[test]
fn no_scenarios_means_all_layouts_unused_and_none_used() {
    let scenarios: Vec<(String, ScenarioDefinition)> = vec![];
    let self_test_names: Vec<String> = vec![];
    let layout_names = vec!["Corridor".to_owned(), "Fortress".to_owned()];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert!(
        report.used_layouts.is_empty(),
        "used_layouts should be empty when no scenarios exist"
    );
    assert_eq!(
        report.unused_layouts,
        vec!["Corridor".to_owned(), "Fortress".to_owned()],
        "all layouts should be unused when no scenarios exist"
    );
}

// -----------------------------------------------------------------------
// Behavior 9 (spec): used_layouts preserves layout_names ordering
// -----------------------------------------------------------------------

#[test]
fn used_layouts_preserves_layout_names_ordering() {
    let scenarios = vec![
        (
            "corridor_test".to_owned(),
            minimal_scenario("Corridor", None),
        ),
        (
            "fortress_test".to_owned(),
            minimal_scenario("Fortress", None),
        ),
    ];
    let self_test_names: Vec<String> = vec![];
    let layout_names = vec![
        "Fortress".to_owned(),
        "Corridor".to_owned(),
        "Scatter".to_owned(),
    ];

    let report = check_coverage(&scenarios, &self_test_names, &layout_names);

    assert_eq!(
        report.used_layouts,
        vec![("Fortress".to_owned(), 1), ("Corridor".to_owned(), 1),],
        "used_layouts should preserve layout_names ordering, not scenario discovery order"
    );
    assert_eq!(
        report.unused_layouts,
        vec!["Scatter".to_owned()],
        "unused_layouts should contain Scatter"
    );
}
