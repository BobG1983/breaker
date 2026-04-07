//! Window tiling utilities for parallel visual-mode scenario runs.
//!
//! Pure functions for computing grid dimensions and tile positions, plus
//! environment variable constants used to pass window geometry to subprocesses.

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
    pub x: u32,
    /// Vertical offset in pixels from the top edge of the screen.
    pub y: u32,
    /// Width of the tile in pixels.
    pub width: u32,
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
        x: col * tile_w,
        y: row * tile_h,
        width: tile_w,
        height: tile_h,
    }
}
