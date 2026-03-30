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
pub mod screen;
pub mod shared;

pub(crate) mod audio;
pub mod bolt;
pub mod breaker;
pub mod cells;
pub mod chips;
pub(crate) mod debug;
/// Data-driven trigger→effect pipeline.
pub mod effect;
pub(crate) mod fx;
pub mod input;
pub mod run;
pub mod ui;
pub mod wall;
