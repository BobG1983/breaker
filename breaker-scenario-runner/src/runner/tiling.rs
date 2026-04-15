//! Window tiling utilities for parallel visual-mode scenario runs.
//!
//! Pure functions for computing grid dimensions and tile positions, plus
//! environment variable constants used to pass tile config to subprocesses.

use bevy::prelude::Resource;

/// Default screen width in pixels (Full HD).
pub const DEFAULT_SCREEN_WIDTH: u32 = 1920;

/// Default screen height in pixels (Full HD).
pub const DEFAULT_SCREEN_HEIGHT: u32 = 1080;

/// Environment variable name for the tile index.
pub const ENV_TILE_INDEX: &str = "SCENARIO_TILE_INDEX";

/// Environment variable name for the tile count.
pub const ENV_TILE_COUNT: &str = "SCENARIO_TILE_COUNT";

/// Tile configuration for a child subprocess: which tile index out of how many total.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileConfig {
    /// The zero-based tile index for this child process.
    pub index: u32,
    /// The total number of tile slots across all child processes.
    pub count: u32,
}

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

/// Returns environment variable key-value pairs for passing tile config to a child process.
///
/// Returns a `Vec` of 2 pairs: `(ENV_TILE_INDEX, slot)` and `(ENV_TILE_COUNT, total)`.
#[must_use]
pub fn tile_config_env_vars(slot: usize, total: usize) -> Vec<(&'static str, String)> {
    vec![
        (ENV_TILE_INDEX, slot.to_string()),
        (ENV_TILE_COUNT, total.to_string()),
    ]
}

/// Parses a [`TileConfig`] from environment variables through a dependency-injected getter.
///
/// Returns `Some(TileConfig)` if both `ENV_TILE_INDEX` and `ENV_TILE_COUNT` are present,
/// parseable as `u32`, and `count > 0`. Otherwise returns `None`.
#[must_use]
pub fn parse_tile_config(getter: impl Fn(&str) -> Option<String>) -> Option<TileConfig> {
    let index = getter(ENV_TILE_INDEX)?.parse::<u32>().ok()?;
    let count = getter(ENV_TILE_COUNT)?.parse::<u32>().ok()?;
    if count == 0 {
        return None;
    }
    Some(TileConfig { index, count })
}

/// Reads tile config from actual environment variables.
///
/// Thin wrapper around [`parse_tile_config`] using `std::env::var`.
#[must_use]
pub fn read_tile_config() -> Option<TileConfig> {
    parse_tile_config(|key| std::env::var(key).ok())
}
