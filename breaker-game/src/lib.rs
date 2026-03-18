//! Brickbreaker — a roguelite Arkanoid clone.
//!
//! Library crate containing all game logic, organized as domain plugins.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "test assertions use unwrap/expect/panic"
    )
)]

pub mod app;
pub mod game;
pub(crate) mod screen;
pub mod shared;

pub(crate) mod audio;
pub(crate) mod behaviors;
pub mod bolt;
pub mod breaker;
pub(crate) mod cells;
pub(crate) mod chips;
pub(crate) mod debug;
pub(crate) mod fx;
pub mod input;
pub(crate) mod interpolate;
pub mod physics;
pub mod run;
pub(crate) mod ui;
pub(crate) mod wall;
