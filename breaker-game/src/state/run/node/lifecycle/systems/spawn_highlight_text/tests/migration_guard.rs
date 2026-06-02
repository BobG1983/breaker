// B9 — spawn_highlight_text/system.rs contains no GameRng reference
#[test]
fn spawn_highlight_text_system_has_no_game_rng_reference() {
    let source = include_str!("../system.rs");
    assert!(
        !source.contains("GameRng"),
        "spawn_highlight_text/system.rs must not reference GameRng after FxRng migration"
    );
}

// B11 — helpers.rs migration verified structurally by B7 in helpers.rs;
// the include_str guard here checks that the mod.rs wiring file contains no GameRng.
// (helpers.rs itself contains GameRng in the B7 absence assertion — that is expected.)
#[test]
fn spawn_highlight_text_mod_has_no_game_rng_reference() {
    let source = include_str!("mod.rs");
    assert!(
        !source.contains("GameRng"),
        "spawn_highlight_text/tests/mod.rs must not reference GameRng"
    );
}
