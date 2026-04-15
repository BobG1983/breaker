use super::{super::core::*, helpers::default_playfield};
use crate::{prelude::*, walls::components::Wall};

// ── Behavior 1: Wall::builder() returns a builder in the unconfigured side state ──

#[test]
fn wall_builder_returns_unconfigured_builder() {
    let _builder: WallBuilder<NoSide> = Wall::builder();
}

#[test]
fn wall_builder_twice_produces_independent_builders() {
    let pf = default_playfield();
    let builder_a = Wall::builder();
    let builder_b = Wall::builder();
    let _a: WallBuilder<Left> = builder_a.left(&pf);
    let _b: WallBuilder<Right> = builder_b.right(&pf);
}

// ── Behavior 2: .left() transitions S from NoSide to Left ──

#[test]
fn left_transitions_to_left() {
    let pf = default_playfield();
    let _builder: WallBuilder<Left> = Wall::builder().left(&pf);
}

#[test]
fn left_stores_playfield_left_and_half_height() {
    let pf = default_playfield();
    let builder = Wall::builder().left(&pf);
    assert!(
        (builder.side.playfield_left - (-400.0)).abs() < f32::EPSILON,
        "playfield_left should be -400.0, got {}",
        builder.side.playfield_left
    );
    assert!(
        (builder.side.half_height - 300.0).abs() < f32::EPSILON,
        "half_height should be 300.0, got {}",
        builder.side.half_height
    );
}

#[test]
fn left_stores_custom_playfield_width() {
    let pf = PlayfieldConfig {
        width: 1000.0,
        ..default_playfield()
    };
    let builder = Wall::builder().left(&pf);
    assert!(
        (builder.side.playfield_left - (-500.0)).abs() < f32::EPSILON,
        "playfield_left should be -500.0 for width 1000.0, got {}",
        builder.side.playfield_left
    );
    assert!(
        (builder.side.half_height - 300.0).abs() < f32::EPSILON,
        "half_height should be 300.0"
    );
}

// ── Behavior 3: .right() transitions S from NoSide to Right ──

#[test]
fn right_transitions_to_right() {
    let pf = default_playfield();
    let _builder: WallBuilder<Right> = Wall::builder().right(&pf);
}

#[test]
fn right_stores_playfield_right_and_half_height() {
    let pf = default_playfield();
    let builder = Wall::builder().right(&pf);
    assert!(
        (builder.side.playfield_right - 400.0).abs() < f32::EPSILON,
        "playfield_right should be 400.0, got {}",
        builder.side.playfield_right
    );
    assert!(
        (builder.side.half_height - 300.0).abs() < f32::EPSILON,
        "half_height should be 300.0"
    );
}

#[test]
fn right_stores_custom_playfield_width() {
    let pf = PlayfieldConfig {
        width: 1000.0,
        ..default_playfield()
    };
    let builder = Wall::builder().right(&pf);
    assert!(
        (builder.side.playfield_right - 500.0).abs() < f32::EPSILON,
        "playfield_right should be 500.0 for width 1000.0, got {}",
        builder.side.playfield_right
    );
}

// ── Behavior 4: .ceiling() transitions S from NoSide to Ceiling ──

#[test]
fn ceiling_transitions_to_ceiling() {
    let pf = default_playfield();
    let _builder: WallBuilder<Ceiling> = Wall::builder().ceiling(&pf);
}

#[test]
fn ceiling_stores_playfield_top_and_half_width() {
    let pf = default_playfield();
    let builder = Wall::builder().ceiling(&pf);
    assert!(
        (builder.side.playfield_top - 300.0).abs() < f32::EPSILON,
        "playfield_top should be 300.0, got {}",
        builder.side.playfield_top
    );
    assert!(
        (builder.side.half_width - 400.0).abs() < f32::EPSILON,
        "half_width should be 400.0"
    );
}

#[test]
fn ceiling_stores_custom_playfield_height() {
    let pf = PlayfieldConfig {
        height: 1080.0,
        ..default_playfield()
    };
    let builder = Wall::builder().ceiling(&pf);
    assert!(
        (builder.side.playfield_top - 540.0).abs() < f32::EPSILON,
        "playfield_top should be 540.0 for height 1080.0, got {}",
        builder.side.playfield_top
    );
}

// ── Behavior 5: .floor() transitions S from NoSide to Floor ──

#[test]
fn floor_transitions_to_floor() {
    let pf = default_playfield();
    let _builder: WallBuilder<Floor> = Wall::builder().floor(&pf);
}

#[test]
fn floor_stores_playfield_bottom_and_half_width() {
    let pf = default_playfield();
    let builder = Wall::builder().floor(&pf);
    assert!(
        (builder.side.playfield_bottom - (-300.0)).abs() < f32::EPSILON,
        "playfield_bottom should be -300.0, got {}",
        builder.side.playfield_bottom
    );
    assert!(
        (builder.side.half_width - 400.0).abs() < f32::EPSILON,
        "half_width should be 400.0"
    );
}

#[test]
fn floor_stores_custom_playfield_height() {
    let pf = PlayfieldConfig {
        height: 1080.0,
        ..default_playfield()
    };
    let builder = Wall::builder().floor(&pf);
    assert!(
        (builder.side.playfield_bottom - (-540.0)).abs() < f32::EPSILON,
        "playfield_bottom should be -540.0 for height 1080.0, got {}",
        builder.side.playfield_bottom
    );
}
