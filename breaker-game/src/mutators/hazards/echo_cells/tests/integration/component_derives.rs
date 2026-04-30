use bevy::prelude::*;

use super::super::{super::system::*, helpers::*};

// ── F. Component derive pins ──────────────────────────────────────────

// Static trait-bound assertions — compile-time pins for Clone/Copy/Default
// without triggering clippy::clone_on_copy (which fires on `.clone()` of
// Copy types). Calling the zero-body fn is a no-op at runtime; the trait
// bound is checked at monomorphization.
const fn assert_clone<T: Clone>() {}
const fn assert_copy<T: Copy>() {}
const fn assert_default<T: Default>() {}

// Behavior 51 — PendingGhost derives Copy + Clone.
#[test]
fn pending_ghost_is_clone_copy() {
    assert_clone::<PendingGhost>();
    assert_copy::<PendingGhost>();
    let a = PendingGhost {
        position: Vec2::new(1.0, 2.0),
        timer:    0.5,
    };
    let b = a; // copy, not move — a remains usable below.
    let c = a;
    assert_eq!(a.position, b.position);
    assert!((a.timer - c.timer).abs() < f32::EPSILON);
}

// Behavior 52 — GhostCell derives Default + Clone + Copy.
#[test]
fn ghost_cell_is_clone_copy_default() {
    assert_clone::<GhostCell>();
    assert_copy::<GhostCell>();
    assert_default::<GhostCell>();
    // Copy semantics pin: `a` remains usable after being copied twice.
    let a = GhostCell;
    let (b, c) = (a, a);
    let _ = b;
    let _ = c;
    // Default-constructor pin: `GhostCell::default()` returns the
    // canonical unit value (use the bare struct to avoid
    // `clippy::default_constructed_unit_structs`).
    let _: GhostCell = GhostCell;
}

// Behavior 53 — EchoCellsConfig derives Clone + Copy.
#[test]
fn echo_cells_config_is_clone_copy() {
    assert_clone::<EchoCellsConfig>();
    assert_copy::<EchoCellsConfig>();
    let cfg = canonical_config();
    let dup = cfg; // copy, not move — cfg remains usable below.
    assert!((dup.delay_secs - cfg.delay_secs).abs() < f32::EPSILON);
    assert!((cfg.base_hp - 1.0).abs() < f32::EPSILON);
}
