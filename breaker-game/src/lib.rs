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
pub(crate) mod prelude;
pub mod shared;
pub mod state;

pub(crate) mod audio;
pub mod bolt;
pub mod breaker;
pub mod cells;
pub mod chips;
pub(crate) mod debug;
/// Data-driven trigger→effect pipeline (old domain — active until migration complete).
pub mod effect;
/// New effect domain (v3) — coexists with old domain during migration.
#[allow(
    dead_code,
    unused,
    missing_docs,
    reason = "effect_v3 — some stubs still unused during migration"
)]
pub mod effect_v3;
pub(crate) mod fx;
pub mod input;
pub mod walls;
