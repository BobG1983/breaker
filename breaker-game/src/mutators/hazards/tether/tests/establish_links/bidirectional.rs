//! Behavior 24 — links are mutual (both endpoints carry `TetherLink`).

use super::{
    super::helpers::{
        add_tether_stacks, build_establish_tether_app, canonical_tether_config, enter_playing,
        install_tether_config, spawn_cell_row,
    },
    link_helpers::links_map,
};

#[test]
fn links_are_bidirectional() {
    let mut app = build_establish_tether_app(42);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);
    let _cells = spawn_cell_row(&mut app, 10, 50.0);
    enter_playing(&mut app);

    let map = links_map(&mut app);
    assert!(!map.is_empty(), "expected at least one TetherLink");
    for (a, b) in &map {
        let back = map.get(b).copied();
        assert_eq!(
            back,
            Some(*a),
            "TetherLink on {a:?} points to {b:?} but reverse points to {back:?}"
        );
    }
}
