//! Window tiling utilities for parallel visual-mode scenario runs.
//!
//! Pure functions for computing grid dimensions and tile positions, plus
//! environment variable constants used to pass window geometry to subprocesses.

use bevy::{
    math::IVec2,
    window::{Window, WindowPosition, WindowResolution},
};

/// Default screen width in pixels (Full HD).
pub const DEFAULT_SCREEN_WIDTH: u32 = 1920;

/// Default screen height in pixels (Full HD).
pub const DEFAULT_SCREEN_HEIGHT: u32 = 1080;

/// Environment variable name for the window X position.
pub const ENV_WINDOW_X: &str = "SCENARIO_WINDOW_X";

/// Environment variable name for the window Y position.
pub const ENV_WINDOW_Y: &str = "SCENARIO_WINDOW_Y";

/// Environment variable name for the window width.
pub const ENV_WINDOW_W: &str = "SCENARIO_WINDOW_W";

/// Environment variable name for the window height.
pub const ENV_WINDOW_H: &str = "SCENARIO_WINDOW_H";

/// A tile's position and dimensions within a tiled screen layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TilePosition {
    /// Horizontal offset in pixels from the left edge of the screen.
    pub x:      u32,
    /// Vertical offset in pixels from the top edge of the screen.
    pub y:      u32,
    /// Width of the tile in pixels.
    pub width:  u32,
    /// Height of the tile in pixels.
    pub height: u32,
}

/// Returns `(cols, rows)` for a grid that can hold `total` scenarios.
///
/// Uses ceiling integer square root. Returns `(1, 1)` for 0 or 1.
#[must_use]
pub const fn grid_dimensions(total: usize) -> (usize, usize) {
    if total <= 1 {
        return (1, 1);
    }
    let mut cols = isqrt_ceil(total);
    if cols == 0 {
        cols = 1;
    }
    let rows = total.div_ceil(cols);
    (cols, rows)
}

/// Ceiling integer square root: smallest `n` such that `n * n >= val`.
const fn isqrt_ceil(val: usize) -> usize {
    if val == 0 {
        return 0;
    }
    let mut i = 1;
    while i * i < val {
        i += 1;
    }
    i
}

/// Returns the [`TilePosition`] for a given slot in a grid of `cols` x `rows`
/// tiles on a screen of `screen_w` x `screen_h` pixels.
///
/// Slots are numbered left-to-right, top-to-bottom (row-major order):
/// slot 0 = top-left, slot (cols-1) = top-right, slot cols = second row left.
///
/// # Panics
///
/// Panics if `cols` or `rows` is zero.
#[must_use]
pub const fn tile_position(
    slot: u32,
    cols: u32,
    rows: u32,
    screen_w: u32,
    screen_h: u32,
) -> TilePosition {
    let col = slot % cols;
    let row = slot / cols;
    let tile_w = screen_w / cols;
    let tile_h = screen_h / rows;
    TilePosition {
        x:      col * tile_w,
        y:      row * tile_h,
        width:  tile_w,
        height: tile_h,
    }
}

/// Returns environment variable key-value pairs for a given slot in a tiled grid.
///
/// Uses [`grid_dimensions`] and [`tile_position`] with [`DEFAULT_SCREEN_WIDTH`]
/// and [`DEFAULT_SCREEN_HEIGHT`] to compute the tile geometry, then returns the
/// four env var pairs that a subprocess needs to position its window.
#[must_use]
pub fn tile_env_vars(slot: usize, total: usize) -> Vec<(&'static str, String)> {
    let (cols, rows) = grid_dimensions(total);
    let tile = tile_position(
        u32::try_from(slot).unwrap_or(u32::MAX),
        u32::try_from(cols).unwrap_or(1),
        u32::try_from(rows).unwrap_or(1),
        DEFAULT_SCREEN_WIDTH,
        DEFAULT_SCREEN_HEIGHT,
    );
    vec![
        (ENV_WINDOW_X, tile.x.to_string()),
        (ENV_WINDOW_Y, tile.y.to_string()),
        (ENV_WINDOW_W, tile.width.to_string()),
        (ENV_WINDOW_H, tile.height.to_string()),
    ]
}

/// Reads tile position from environment variables through a dependency-injected getter.
///
/// Returns `Some(TilePosition)` if all four env var keys are present and parseable
/// as `u32`, otherwise returns `None`.
#[must_use]
pub fn parse_tile_env(getter: impl Fn(&str) -> Option<String>) -> Option<TilePosition> {
    let x = getter(ENV_WINDOW_X)?.parse::<u32>().ok()?;
    let y = getter(ENV_WINDOW_Y)?.parse::<u32>().ok()?;
    let width = getter(ENV_WINDOW_W)?.parse::<u32>().ok()?;
    let height = getter(ENV_WINDOW_H)?.parse::<u32>().ok()?;
    Some(TilePosition {
        x,
        y,
        width,
        height,
    })
}

/// Reads tile position from actual environment variables.
///
/// Thin wrapper around [`parse_tile_env`] using `std::env::var`.
#[must_use]
pub fn read_tile_env() -> Option<TilePosition> {
    parse_tile_env(|key| std::env::var(key).ok())
}

/// Converts a [`TilePosition`] into a Bevy [`Window`] with correct title,
/// position, and resolution.
#[must_use]
pub fn window_from_tile(tile: &TilePosition) -> Window {
    Window {
        title: "Scenario Runner".into(),
        position: WindowPosition::At(IVec2::new(tile.x.cast_signed(), tile.y.cast_signed())),
        resolution: WindowResolution::new(tile.width, tile.height),
        ..Default::default()
    }
}
