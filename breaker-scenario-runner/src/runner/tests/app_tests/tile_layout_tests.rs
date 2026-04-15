//! Tests for `apply_tile_layout` — positions window from `TileConfig` + Monitor.

use bevy::{
    prelude::*,
    window::{PrimaryMonitor, PrimaryWindow},
};

use crate::runner::{app::apply_tile_layout, tiling::TileConfig};

/// Helper: builds a minimal test app with `apply_tile_layout` in `Update`.
fn apply_tile_layout_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, apply_tile_layout);
    app
}

/// Spawns a `Monitor` entity with `PrimaryMonitor` marker and the given physical dimensions.
fn spawn_primary_monitor(app: &mut App, width: u32, height: u32) {
    app.world_mut().spawn((
        bevy::window::Monitor {
            name:                    None,
            physical_width:          width,
            physical_height:         height,
            physical_position:       IVec2::ZERO,
            scale_factor:            1.0,
            refresh_rate_millihertz: None,
            video_modes:             vec![],
        },
        PrimaryMonitor,
    ));
}

/// Spawns a `Window` entity with `PrimaryWindow` marker, returning its entity ID.
fn spawn_primary_window_tracked(app: &mut App, width: u32, height: u32) -> Entity {
    app.world_mut()
        .spawn((
            Window {
                resolution: bevy::window::WindowResolution::new(width, height),
                ..default()
            },
            PrimaryWindow,
        ))
        .id()
}

// -------------------------------------------------------------------------
// Spec Behavior 16: top-left tile in 2x2 on 1920x1080 monitor
// -------------------------------------------------------------------------

#[test]
fn apply_tile_layout_top_left_in_2x2_on_1920x1080() {
    let mut app = apply_tile_layout_app();
    app.insert_resource(TileConfig { index: 0, count: 4 });
    spawn_primary_monitor(&mut app, 1920, 1080);
    let win_entity = spawn_primary_window_tracked(&mut app, 1280, 720);

    app.update();

    let window = app
        .world()
        .get::<Window>(win_entity)
        .expect("window entity should exist");
    assert_eq!(
        window.position,
        WindowPosition::At(IVec2::new(0, 0)),
        "window position should be at (0, 0) for top-left tile"
    );
    assert!(
        (window.resolution.width() - 960.0).abs() < f32::EPSILON,
        "window width should be 960.0, got {}",
        window.resolution.width()
    );
    assert!(
        (window.resolution.height() - 540.0).abs() < f32::EPSILON,
        "window height should be 540.0, got {}",
        window.resolution.height()
    );
}

// -------------------------------------------------------------------------
// Spec Behavior 17: bottom-right tile in 2x2 on 1920x1080 monitor
// -------------------------------------------------------------------------

#[test]
fn apply_tile_layout_bottom_right_in_2x2_on_1920x1080() {
    let mut app = apply_tile_layout_app();
    app.insert_resource(TileConfig { index: 3, count: 4 });
    spawn_primary_monitor(&mut app, 1920, 1080);
    let win_entity = spawn_primary_window_tracked(&mut app, 1280, 720);

    app.update();

    let window = app
        .world()
        .get::<Window>(win_entity)
        .expect("window entity should exist");
    assert_eq!(
        window.position,
        WindowPosition::At(IVec2::new(960, 540)),
        "window position should be at (960, 540) for bottom-right tile"
    );
    assert!(
        (window.resolution.width() - 960.0).abs() < f32::EPSILON,
        "window width should be 960.0, got {}",
        window.resolution.width()
    );
    assert!(
        (window.resolution.height() - 540.0).abs() < f32::EPSILON,
        "window height should be 540.0, got {}",
        window.resolution.height()
    );
}

// -------------------------------------------------------------------------
// Spec Behavior 18: single scenario on 2560x1440 (full screen)
// -------------------------------------------------------------------------

#[test]
fn apply_tile_layout_single_scenario_on_2560x1440_fills_screen() {
    let mut app = apply_tile_layout_app();
    app.insert_resource(TileConfig { index: 0, count: 1 });
    spawn_primary_monitor(&mut app, 2560, 1440);
    let win_entity = spawn_primary_window_tracked(&mut app, 1280, 720);

    app.update();

    let window = app
        .world()
        .get::<Window>(win_entity)
        .expect("window entity should exist");
    assert_eq!(
        window.position,
        WindowPosition::At(IVec2::new(0, 0)),
        "single tile fills entire screen, position at origin"
    );
    assert!(
        (window.resolution.width() - 2560.0).abs() < f32::EPSILON,
        "window width should be 2560.0, got {}",
        window.resolution.width()
    );
    assert!(
        (window.resolution.height() - 1440.0).abs() < f32::EPSILON,
        "window height should be 1440.0, got {}",
        window.resolution.height()
    );
}

// -------------------------------------------------------------------------
// Spec Behavior 19: adapts to non-standard monitor 2560x1440, index 1
// -------------------------------------------------------------------------

#[test]
fn apply_tile_layout_adapts_to_2560x1440_monitor() {
    let mut app = apply_tile_layout_app();
    app.insert_resource(TileConfig { index: 1, count: 4 });
    spawn_primary_monitor(&mut app, 2560, 1440);
    let win_entity = spawn_primary_window_tracked(&mut app, 1280, 720);

    app.update();

    let window = app
        .world()
        .get::<Window>(win_entity)
        .expect("window entity should exist");
    assert_eq!(
        window.position,
        WindowPosition::At(IVec2::new(1280, 0)),
        "tile index 1 in 2x2 on 2560x1440 should be at (1280, 0)"
    );
    assert!(
        (window.resolution.width() - 1280.0).abs() < f32::EPSILON,
        "window width should be 1280.0, got {}",
        window.resolution.width()
    );
    assert!(
        (window.resolution.height() - 720.0).abs() < f32::EPSILON,
        "window height should be 720.0, got {}",
        window.resolution.height()
    );
}

// -------------------------------------------------------------------------
// Spec Behavior 20: adapts to 4K monitor (3840x2160), 3x3 grid
// -------------------------------------------------------------------------

#[test]
fn apply_tile_layout_adapts_to_4k_monitor_3x3_grid() {
    let mut app = apply_tile_layout_app();
    app.insert_resource(TileConfig { index: 2, count: 9 });
    spawn_primary_monitor(&mut app, 3840, 2160);
    let win_entity = spawn_primary_window_tracked(&mut app, 1280, 720);

    app.update();

    let window = app
        .world()
        .get::<Window>(win_entity)
        .expect("window entity should exist");
    assert_eq!(
        window.position,
        WindowPosition::At(IVec2::new(2560, 0)),
        "tile index 2 in 3x3 on 3840x2160 should be at (2560, 0)"
    );
    assert!(
        (window.resolution.width() - 1280.0).abs() < f32::EPSILON,
        "window width should be 1280.0, got {}",
        window.resolution.width()
    );
    assert!(
        (window.resolution.height() - 720.0).abs() < f32::EPSILON,
        "window height should be 720.0, got {}",
        window.resolution.height()
    );
}

// -------------------------------------------------------------------------
// Spec Behavior 21: removes TileConfig resource after applying
// -------------------------------------------------------------------------

#[test]
fn apply_tile_layout_removes_tile_config_after_applying() {
    let mut app = apply_tile_layout_app();
    app.insert_resource(TileConfig { index: 0, count: 4 });
    spawn_primary_monitor(&mut app, 1920, 1080);
    spawn_primary_window_tracked(&mut app, 1280, 720);

    app.update();

    assert!(
        app.world().get_resource::<TileConfig>().is_none(),
        "TileConfig resource should be removed after apply_tile_layout runs"
    );
}

// -------------------------------------------------------------------------
// Spec Behavior 22: does not re-apply on second update (one-shot)
// -------------------------------------------------------------------------

#[test]
fn apply_tile_layout_does_not_reapply_on_second_update() {
    let mut app = apply_tile_layout_app();
    app.insert_resource(TileConfig { index: 0, count: 4 });
    spawn_primary_monitor(&mut app, 1920, 1080);
    let win_entity = spawn_primary_window_tracked(&mut app, 1280, 720);

    // First update: system applies tile layout and removes TileConfig.
    app.update();

    // Manually change window position to verify the system does not overwrite it.
    let mut entity_mut = app.world_mut().entity_mut(win_entity);
    let mut window = entity_mut.get_mut::<Window>().expect("window should exist");
    window.position = WindowPosition::Centered(bevy::window::MonitorSelection::Current);

    // Second update: system should no-op because TileConfig was removed.
    app.update();

    let window = app
        .world()
        .get::<Window>(win_entity)
        .expect("window entity should exist");
    assert_eq!(
        window.position,
        WindowPosition::Centered(bevy::window::MonitorSelection::Current),
        "window position should remain Centered after second update (system is one-shot)"
    );
}

// -------------------------------------------------------------------------
// Spec Behavior 23: no-ops when no TileConfig resource exists
// -------------------------------------------------------------------------

#[test]
fn apply_tile_layout_noops_when_no_tile_config() {
    let mut app = apply_tile_layout_app();
    // No TileConfig inserted
    spawn_primary_monitor(&mut app, 1920, 1080);
    let win_entity = spawn_primary_window_tracked(&mut app, 1280, 720);

    app.update();

    let window = app
        .world()
        .get::<Window>(win_entity)
        .expect("window entity should exist");
    // Default window position is Automatic, default resolution is 1280x720
    assert_eq!(
        window.position,
        WindowPosition::Automatic,
        "window position should remain default Automatic when no TileConfig"
    );
    assert!(
        (window.resolution.width() - 1280.0).abs() < f32::EPSILON,
        "window width should remain default 1280.0 when no TileConfig, got {}",
        window.resolution.width()
    );
    assert!(
        (window.resolution.height() - 720.0).abs() < f32::EPSILON,
        "window height should remain default 720.0 when no TileConfig, got {}",
        window.resolution.height()
    );
}

// -------------------------------------------------------------------------
// Spec Behavior 24: no-ops when no PrimaryMonitor entity exists
// -------------------------------------------------------------------------

#[test]
fn apply_tile_layout_noops_when_no_primary_monitor() {
    let mut app = apply_tile_layout_app();
    app.insert_resource(TileConfig { index: 0, count: 4 });
    // No Monitor/PrimaryMonitor spawned
    let win_entity = spawn_primary_window_tracked(&mut app, 1280, 720);

    app.update();

    let window = app
        .world()
        .get::<Window>(win_entity)
        .expect("window entity should exist");
    assert_eq!(
        window.position,
        WindowPosition::Automatic,
        "window position should remain default when no PrimaryMonitor"
    );
    // TileConfig should NOT be consumed when monitor is missing
    assert!(
        app.world().get_resource::<TileConfig>().is_some(),
        "TileConfig resource should still exist when PrimaryMonitor is missing"
    );
}

// -------------------------------------------------------------------------
// Spec Behavior 25: no-ops when no PrimaryWindow entity exists
// -------------------------------------------------------------------------

#[test]
fn apply_tile_layout_noops_when_no_primary_window() {
    let mut app = apply_tile_layout_app();
    app.insert_resource(TileConfig { index: 0, count: 4 });
    spawn_primary_monitor(&mut app, 1920, 1080);
    // No Window/PrimaryWindow spawned

    // Should not panic
    app.update();

    // TileConfig should still exist (not consumed because window couldn't be updated)
    assert!(
        app.world().get_resource::<TileConfig>().is_some(),
        "TileConfig resource should still exist when PrimaryWindow is missing"
    );
}

// -------------------------------------------------------------------------
// Spec Behavior 26: does not panic with out-of-bounds tile index
// -------------------------------------------------------------------------

#[test]
fn apply_tile_layout_does_not_panic_with_out_of_bounds_tile_index() {
    let mut app = apply_tile_layout_app();
    app.insert_resource(TileConfig { index: 5, count: 4 });
    spawn_primary_monitor(&mut app, 1920, 1080);
    spawn_primary_window_tracked(&mut app, 1280, 720);

    // Must not panic
    app.update();

    // TileConfig should be consumed (system ran, even with invalid index)
    assert!(
        app.world().get_resource::<TileConfig>().is_none(),
        "TileConfig resource should be consumed even with out-of-bounds index"
    );
}
