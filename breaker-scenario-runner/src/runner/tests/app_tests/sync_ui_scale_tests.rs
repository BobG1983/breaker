//! Tests for `sync_ui_scale` — sets `UiScale` from primary window dimensions.

use bevy::{prelude::*, window::PrimaryWindow};

use crate::runner::app::sync_ui_scale;

/// Helper: builds a minimal test app with `sync_ui_scale` in `Update`.
///
/// Initializes `UiScale` to a sentinel value (99.0) so we can verify the
/// system actively writes the correct value. Without this, tests where the
/// expected output happens to equal the default (1.0) would pass against
/// a no-op stub.
fn sync_ui_scale_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(UiScale(99.0));
    app.add_systems(Update, sync_ui_scale);
    app
}

/// Spawns a `Window` entity with the given resolution and a `PrimaryWindow` marker.
fn spawn_primary_window(app: &mut App, width: u32, height: u32) {
    app.world_mut().spawn((
        Window {
            resolution: bevy::window::WindowResolution::new(width, height),
            ..default()
        },
        PrimaryWindow,
    ));
}

// -------------------------------------------------------------------------
// Behavior 20: full HD window produces ui_scale 1.0
// -------------------------------------------------------------------------

#[test]
fn sync_ui_scale_full_hd_window_produces_scale_1() {
    let mut app = sync_ui_scale_app();
    spawn_primary_window(&mut app, 1920, 1080);

    app.update();

    let ui_scale = app.world().resource::<UiScale>();
    assert!(
        (ui_scale.0 - 1.0).abs() < f32::EPSILON,
        "expected UiScale ~1.0 for 1920x1080, got {}",
        ui_scale.0
    );
}

// -------------------------------------------------------------------------
// Behavior 21: half-size window produces ui_scale 0.5
// -------------------------------------------------------------------------

#[test]
fn sync_ui_scale_half_size_window_produces_scale_0_5() {
    let mut app = sync_ui_scale_app();
    spawn_primary_window(&mut app, 960, 540);

    app.update();

    let ui_scale = app.world().resource::<UiScale>();
    assert!(
        (ui_scale.0 - 0.5).abs() < f32::EPSILON,
        "expected UiScale ~0.5 for 960x540, got {}",
        ui_scale.0
    );
}

// -------------------------------------------------------------------------
// Behavior 22: width-limited window uses width ratio
// -------------------------------------------------------------------------

#[test]
fn sync_ui_scale_width_limited_window_uses_width_ratio() {
    let mut app = sync_ui_scale_app();
    spawn_primary_window(&mut app, 960, 1080);

    app.update();

    let ui_scale = app.world().resource::<UiScale>();
    // min(960/1920, 1080/1080) = min(0.5, 1.0) = 0.5
    assert!(
        (ui_scale.0 - 0.5).abs() < f32::EPSILON,
        "expected UiScale ~0.5 for width-limited 960x1080, got {}",
        ui_scale.0
    );
}

// -------------------------------------------------------------------------
// Behavior 23: height-limited window uses height ratio
// -------------------------------------------------------------------------

#[test]
fn sync_ui_scale_height_limited_window_uses_height_ratio() {
    let mut app = sync_ui_scale_app();
    spawn_primary_window(&mut app, 1920, 540);

    app.update();

    let ui_scale = app.world().resource::<UiScale>();
    // min(1920/1920, 540/1080) = min(1.0, 0.5) = 0.5
    assert!(
        (ui_scale.0 - 0.5).abs() < f32::EPSILON,
        "expected UiScale ~0.5 for height-limited 1920x540, got {}",
        ui_scale.0
    );
}

// -------------------------------------------------------------------------
// Behavior 24: 4K window produces ui_scale 2.0
// -------------------------------------------------------------------------

#[test]
fn sync_ui_scale_4k_window_produces_scale_2() {
    let mut app = sync_ui_scale_app();
    spawn_primary_window(&mut app, 3840, 2160);

    app.update();

    let ui_scale = app.world().resource::<UiScale>();
    // min(3840/1920, 2160/1080) = min(2.0, 2.0) = 2.0
    assert!(
        (ui_scale.0 - 2.0).abs() < f32::EPSILON,
        "expected UiScale ~2.0 for 3840x2160, got {}",
        ui_scale.0
    );
}

// -------------------------------------------------------------------------
// Behavior 25: ultrawide window limited by smaller ratio
// -------------------------------------------------------------------------

#[test]
fn sync_ui_scale_ultrawide_window_limited_by_height() {
    let mut app = sync_ui_scale_app();
    spawn_primary_window(&mut app, 3840, 1080);

    app.update();

    let ui_scale = app.world().resource::<UiScale>();
    // min(3840/1920, 1080/1080) = min(2.0, 1.0) = 1.0
    assert!(
        (ui_scale.0 - 1.0).abs() < f32::EPSILON,
        "expected UiScale ~1.0 for ultrawide 3840x1080, got {}",
        ui_scale.0
    );
}

// -------------------------------------------------------------------------
// Behavior 26: no primary window does not panic, UiScale unchanged
// -------------------------------------------------------------------------

#[test]
fn sync_ui_scale_no_primary_window_does_not_panic_and_leaves_ui_scale_unchanged() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    // Insert UiScale manually per spec: "UiScale inserted manually with value 1.0"
    app.insert_resource(UiScale(1.0));
    app.add_systems(Update, sync_ui_scale);
    // No window spawned — the system should silently return.

    app.update();

    let ui_scale = app.world().resource::<UiScale>();
    assert!(
        (ui_scale.0 - 1.0).abs() < f32::EPSILON,
        "expected UiScale to remain 1.0 when no primary window exists, got {}",
        ui_scale.0
    );
}
