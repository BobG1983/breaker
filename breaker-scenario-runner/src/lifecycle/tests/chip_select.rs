use super::helpers::*;

// =========================================================================
// chip_selections Injection
// =========================================================================

/// First chip selection entry writes `ChipSelected` with the first name.
#[test]
fn chip_select_first_entry_writes_first_chip_selection() {
    let mut definition = make_scenario(100);
    definition.chip_selections = Some(vec!["Surge".to_owned(), "Piercing Shot".to_owned()]);

    let mut app = chip_select_app(definition);
    app.init_resource::<CapturedChipSelected>()
        .add_systems(Update, collect_chip_selected.after(auto_skip_chip_select));

    // ChipSelectionIndex starts at 0
    assert_eq!(app.world().resource::<ChipSelectionIndex>().0, 0);

    app.update();

    let captured = app.world().resource::<CapturedChipSelected>();
    assert_eq!(
        captured.0.len(),
        1,
        "expected 1 ChipSelected message, got {}",
        captured.0.len()
    );
    assert_eq!(
        captured.0[0].name, "Surge",
        "expected ChipSelected name == 'Surge', got '{}'",
        captured.0[0].name
    );

    let index = app.world().resource::<ChipSelectionIndex>();
    assert_eq!(
        index.0, 1,
        "expected ChipSelectionIndex == 1 after first chip select"
    );
}

/// Second chip selection entry writes `ChipSelected` with the second name.
#[test]
fn chip_select_second_entry_writes_second_chip_selection() {
    let mut definition = make_scenario(100);
    definition.chip_selections = Some(vec!["Surge".to_owned(), "Piercing Shot".to_owned()]);

    let mut app = chip_select_app(definition);
    // Start at index 1 (simulating first chip already selected)
    app.world_mut().resource_mut::<ChipSelectionIndex>().0 = 1;
    app.init_resource::<CapturedChipSelected>()
        .add_systems(Update, collect_chip_selected.after(auto_skip_chip_select));

    app.update();

    let captured = app.world().resource::<CapturedChipSelected>();
    assert_eq!(captured.0.len(), 1);
    assert_eq!(
        captured.0[0].name, "Piercing Shot",
        "expected ChipSelected name == 'Piercing Shot'"
    );

    let index = app.world().resource::<ChipSelectionIndex>();
    assert_eq!(index.0, 2);
}

/// Past end of `chip_selections` list: no message, still transitions.
#[test]
fn chip_select_past_end_no_message_still_transitions() {
    let mut definition = make_scenario(100);
    definition.chip_selections = Some(vec!["Surge".to_owned()]);

    let mut app = chip_select_app(definition);
    app.world_mut().resource_mut::<ChipSelectionIndex>().0 = 1;
    app.init_resource::<CapturedChipSelected>()
        .add_systems(Update, collect_chip_selected.after(auto_skip_chip_select));

    app.update();

    let captured = app.world().resource::<CapturedChipSelected>();
    assert!(
        captured.0.is_empty(),
        "expected no ChipSelected message past end of list, got {}",
        captured.0.len()
    );

    let index = app.world().resource::<ChipSelectionIndex>();
    assert_eq!(index.0, 1, "expected ChipSelectionIndex unchanged at 1");
}

/// When `chip_selections = None`, no message is written but transitions.
#[test]
fn chip_select_none_no_message_transitions() {
    let definition = make_scenario(100); // chip_selections is None by default

    let mut app = chip_select_app(definition);
    app.init_resource::<CapturedChipSelected>()
        .add_systems(Update, collect_chip_selected.after(auto_skip_chip_select));

    app.update();

    let captured = app.world().resource::<CapturedChipSelected>();
    assert!(
        captured.0.is_empty(),
        "expected no ChipSelected message when chip_selections is None"
    );
}

// =========================================================================
// ChipSelectionIndex Reset
// =========================================================================

/// `bypass_menu_to_playing` resets `ChipSelectionIndex` to 0.
#[test]
fn bypass_menu_to_playing_resets_chip_selection_index() {
    let mut definition = make_scenario(100);
    definition.breaker = "Aegis".to_owned();

    let mut app = bypass_app(definition);
    app.world_mut().resource_mut::<ChipSelectionIndex>().0 = 3;

    app.update();

    let index = app.world().resource::<ChipSelectionIndex>();
    assert_eq!(
        index.0, 0,
        "expected ChipSelectionIndex reset to 0, got {}",
        index.0
    );
}
