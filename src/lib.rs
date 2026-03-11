//! Brickbreaker — a roguelite Arkanoid clone.
//!
//! Library crate containing all game logic, organized as domain plugins.

pub mod app;
pub mod game;
pub mod shared;

pub(crate) mod audio;
pub(crate) mod bolt;
pub(crate) mod breaker;
pub(crate) mod cells;
pub(crate) mod debug;
pub(crate) mod physics;
pub(crate) mod run;
pub(crate) mod screen;
pub(crate) mod ui;
pub(crate) mod upgrades;
