//! Window positioning systems — tile layout and UI scale sync.

use bevy::{prelude::*, window::PrimaryWindow};
use tracing::warn;

use crate::runner::tiling;

/// Syncs the [`UiScale`] resource based on the primary window dimensions.
///
/// Formula: `ui_scale = min(window_width / 1920.0, window_height / 1080.0)`
///
/// This ensures all `Val::Px` and font sizes designed for 1920x1080 scale
/// correctly to the actual window size.
pub(crate) fn sync_ui_scale(
    mut ui_scale: ResMut<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = windows.single() {
        ui_scale.0 = (window.width() / 1920.0).min(window.height() / 1080.0);
    }
}

/// Applies tile layout to the primary window based on [`tiling::TileConfig`] and primary monitor dimensions.
///
/// Reads the tile config resource and primary monitor component, computes tile
/// position using [`tiling::grid_dimensions`] and [`tiling::tile_position`], then sets the
/// window's position and resolution. Removes `TileConfig` after applying (one-shot).
///
/// No-ops without consuming `TileConfig` when `PrimaryMonitor` or `PrimaryWindow`
/// is missing, so the system retries on the next frame once winit populates them.
/// No-ops silently (no retry) when `TileConfig` is absent.
pub(crate) fn apply_tile_layout(
    mut commands: Commands,
    tile_config: Option<Res<tiling::TileConfig>>,
    monitors: Query<&bevy::window::Monitor, With<bevy::window::PrimaryMonitor>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Some(tile_config) = tile_config else {
        return;
    };
    let Ok(monitor) = monitors.single() else {
        warn!("apply_tile_layout: PrimaryMonitor not available yet, will retry next frame");
        return;
    };
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let (cols, rows) = tiling::grid_dimensions(tile_config.count as usize);
    let tile = tiling::tile_position(
        tile_config.index,
        cols as u32,
        rows as u32,
        monitor.physical_width,
        monitor.physical_height,
    );
    window.resolution = bevy::window::WindowResolution::new(tile.width, tile.height);
    window.position = WindowPosition::At(IVec2::new(tile.x.cast_signed(), tile.y.cast_signed()));
    commands.remove_resource::<tiling::TileConfig>();
}
