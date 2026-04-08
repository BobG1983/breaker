//! Section A: Entry Point, Section B: Typestate Dimension Transitions,
//! Section C: Dimension Ordering Independence

use bevy::prelude::*;

use crate::cells::{builder::core::*, components::Cell};

// ── Section A: Entry Point ──────────────────────────────────────────────────

// Behavior 1: Cell::builder() returns a builder in the fully-unconfigured state
#[test]
fn cell_builder_returns_unconfigured_builder() {
    let _builder: CellBuilder<NoPosition, NoDimensions, NoHealth, Unvisual> = Cell::builder();
    // Type annotation compiles successfully — that is the assertion.
}

// Behavior 1 edge case: calling Cell::builder() twice produces independent builders
#[test]
fn cell_builder_twice_produces_independent_builders() {
    let builder_a = Cell::builder();
    let builder_b = Cell::builder();
    // Both builders are independent — modifying one does not affect the other.
    let _a = builder_a.position(Vec2::new(1.0, 2.0));
    let _b = builder_b.position(Vec2::new(3.0, 4.0));
}

// ── Section B: Typestate Dimension Transitions ──────────────────────────────

// Behavior 3: .position(pos) transitions Position dimension
#[test]
fn position_transitions_to_has_position() {
    let _builder: CellBuilder<HasPosition, NoDimensions, NoHealth, Unvisual> =
        Cell::builder().position(Vec2::new(100.0, 250.0));
}

// Behavior 3 edge case: zero position is valid
#[test]
fn position_accepts_zero() {
    let _builder: CellBuilder<HasPosition, NoDimensions, NoHealth, Unvisual> =
        Cell::builder().position(Vec2::ZERO);
}

// Behavior 3 edge case: negative coordinates
#[test]
fn position_accepts_negative_coordinates() {
    let _builder: CellBuilder<HasPosition, NoDimensions, NoHealth, Unvisual> =
        Cell::builder().position(Vec2::new(-200.0, -100.0));
}

// Behavior 5: .dimensions(w, h) transitions Dimensions dimension
#[test]
fn dimensions_transitions_to_has_dimensions() {
    let _builder: CellBuilder<NoPosition, HasDimensions, NoHealth, Unvisual> =
        Cell::builder().dimensions(70.0, 24.0);
}

// Behavior 5 edge case: zero dimensions
#[test]
fn dimensions_accepts_zero() {
    let _builder: CellBuilder<NoPosition, HasDimensions, NoHealth, Unvisual> =
        Cell::builder().dimensions(0.0, 0.0);
}

// Behavior 7: .hp(value) transitions Health dimension
#[test]
fn hp_transitions_to_has_health() {
    let _builder: CellBuilder<NoPosition, NoDimensions, HasHealth, Unvisual> =
        Cell::builder().hp(20.0);
}

// Behavior 7 edge case: tiny positive HP
#[test]
fn hp_accepts_tiny_positive() {
    let _builder: CellBuilder<NoPosition, NoDimensions, HasHealth, Unvisual> =
        Cell::builder().hp(0.001);
}

// Behavior 9: .headless() transitions Visual dimension
#[test]
fn headless_transitions_to_headless() {
    let _builder: CellBuilder<NoPosition, NoDimensions, NoHealth, Headless> =
        Cell::builder().headless();
}

// Behavior 10: .rendered(meshes, materials) transitions Visual dimension
#[test]
fn rendered_transitions_to_rendered() {
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<ColorMaterial>::default();
    let _builder: CellBuilder<NoPosition, NoDimensions, NoHealth, Rendered> =
        Cell::builder().rendered(&mut meshes, &mut materials);
}

// ── Section C: Dimension Ordering Independence ──────────────────────────────

// Behavior 11: Dimensions can be set in any order
#[test]
fn ordering_hp_position_dimensions_headless() {
    let _builder = Cell::builder()
        .hp(20.0)
        .position(Vec2::new(50.0, 100.0))
        .dimensions(70.0, 24.0)
        .headless();
    // Compiles — assertion is the type annotation
}

// Behavior 11 edge case: reversed order
#[test]
fn ordering_dimensions_hp_position_headless() {
    let _builder = Cell::builder()
        .dimensions(70.0, 24.0)
        .hp(20.0)
        .position(Vec2::new(50.0, 100.0))
        .headless();
}

// Behavior 12: All six orderings of Position, Dimensions, Health compile
#[test]
fn ordering_position_dimensions_hp() {
    let _builder = Cell::builder()
        .position(Vec2::ZERO)
        .dimensions(70.0, 24.0)
        .hp(20.0)
        .headless();
}

#[test]
fn ordering_position_hp_dimensions() {
    let _builder = Cell::builder()
        .position(Vec2::ZERO)
        .hp(20.0)
        .dimensions(70.0, 24.0)
        .headless();
}

#[test]
fn ordering_dimensions_position_hp() {
    let _builder = Cell::builder()
        .dimensions(70.0, 24.0)
        .position(Vec2::ZERO)
        .hp(20.0)
        .headless();
}

#[test]
fn ordering_hp_dimensions_position() {
    let _builder = Cell::builder()
        .hp(20.0)
        .dimensions(70.0, 24.0)
        .position(Vec2::ZERO)
        .headless();
}
