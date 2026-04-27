//! Generic 2D damage primitives for Bevy 0.18 games.
//!
//! This crate provides reusable damage building blocks — HP tracking,
//! dead-state marking, invulnerability windows, kill attribution, heal
//! ceilings, and source identifiers — intended for any 2D Bevy game. It
//! obeys the zero-game-knowledge contract documented in
//! `.claude/rules/rantzsoft-crates.md`: no game-specific vocabulary,
//! entities, or assumptions leak into this crate.
//!
//! P3 ships the generic damage-pipeline messages (`DamageDealt<T>`,
//! `HealDealt<T>`, `KillYourself<T>`, `Destroyed<T>`, `DespawnEntity`)
//! alongside the P2 core types. P4 adds the `DmgSystems` `SystemSet` enum
//! and the `RantzDmgPlugin` skeleton. P5 adds the two damage-stack
//! components — `DamageBoostStack` and `VulnerableStack` — as
//! private-field, `Vec`-backed, append-semantic containers keyed by
//! `SourceId`. P6 adds the seven per-`T` pipeline systems
//! (`apply_damage_boosts`, `apply_vulnerable`, `invulnerable_filter`,
//! `apply_damage`, `detect_deaths`, `handle_kill`, `apply_heal`), the real
//! `process_despawn_requests` body, and the public `RantzDmgAppExt`
//! extension trait whose `register_dmgable::<T>()` method wires the
//! per-`T` message queues and systems for a given `Dmgable` type.
//!
//! # W2 Behavior 13: per-`T` system items are NOT reachable from outside
//!
//! External consumers must reach the damage pipeline through
//! `RantzDmgAppExt::register_dmgable::<T>()`, never by importing individual
//! per-`T` system functions.
//!
//! ```compile_fail
//! use bevy::prelude::*;
//! use rantzsoft_dmg::Dmgable;
//! #[derive(Component)]
//! struct TestT;
//! impl Dmgable for TestT {}
//! let _ = rantzsoft_dmg::apply_damage::<TestT>;
//! ```
//!
//! ```compile_fail
//! use bevy::prelude::*;
//! use rantzsoft_dmg::Dmgable;
//! #[derive(Component)]
//! struct TestT;
//! impl Dmgable for TestT {}
//! let _ = rantzsoft_dmg::apply_damage_boosts::<TestT>;
//! ```
//!
//! ```compile_fail
//! use bevy::prelude::*;
//! use rantzsoft_dmg::Dmgable;
//! #[derive(Component)]
//! struct TestT;
//! impl Dmgable for TestT {}
//! let _ = rantzsoft_dmg::apply_heal::<TestT>;
//! ```
//!
//! ```compile_fail
//! use bevy::prelude::*;
//! use rantzsoft_dmg::Dmgable;
//! #[derive(Component)]
//! struct TestT;
//! impl Dmgable for TestT {}
//! let _ = rantzsoft_dmg::apply_vulnerable::<TestT>;
//! ```
//!
//! ```compile_fail
//! use bevy::prelude::*;
//! use rantzsoft_dmg::Dmgable;
//! #[derive(Component)]
//! struct TestT;
//! impl Dmgable for TestT {}
//! let _ = rantzsoft_dmg::detect_deaths::<TestT>;
//! ```
//!
//! ```compile_fail
//! use bevy::prelude::*;
//! use rantzsoft_dmg::Dmgable;
//! #[derive(Component)]
//! struct TestT;
//! impl Dmgable for TestT {}
//! let _ = rantzsoft_dmg::handle_kill::<TestT>;
//! ```
//!
//! ```compile_fail
//! use bevy::prelude::*;
//! use rantzsoft_dmg::Dmgable;
//! #[derive(Component)]
//! struct TestT;
//! impl Dmgable for TestT {}
//! let _ = rantzsoft_dmg::invulnerable_filter::<TestT>;
//! ```
//!
//! The positive-access contract for `register_dmgable::<T>()` is covered
//! by `rantzsoft_dmg/src/app_ext/tests.rs` as an in-process behavioral
//! test. A module-level doctest was considered but is incompatible with
//! `bevy/dynamic_linking` (the ephemeral doctest binary can't locate
//! `libstd` at the merged-doctest runtime stage, even with `no_run`).

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "test assertions use unwrap/expect/panic"
    )
)]

mod app_ext;
mod components;
mod messages;
mod plugin;
mod preview;
mod sets;
mod source_id;
mod systems;
mod traits;

pub use app_ext::RantzDmgAppExt;
pub use components::{
    DamageBoostStack, Dead, HealCap, Hp, Invulnerable, KilledBy, VulnerableStack,
};
pub use messages::{DamageDealt, DespawnEntity, Destroyed, HealDealt, KillYourself};
pub use plugin::RantzDmgPlugin;
pub use preview::preview_damage;
pub use sets::DmgSystems;
pub use source_id::SourceId;
// Per-`T` pipeline systems are NOT re-exported at the crate root. External
// consumers must reach them via `RantzDmgAppExt::register_dmgable::<T>()`.
// See the compile_fail doctests at the top of this file.
pub use traits::Dmgable;

#[cfg(test)]
mod crate_root_tests;
