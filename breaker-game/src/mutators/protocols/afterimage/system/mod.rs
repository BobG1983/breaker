//! Afterimage protocol — phantom-breaker + phantom-bounce + phantom-bolt.
//!
//! Design doc: `docs/design/protocols/afterimage.md`.
//!
//! Owns `AfterimageConfig`, `PhantomBreaker`, `PhantomBreakerLifetime`, the
//! four runtime systems (`afterimage_spawn_phantom_breaker`,
//! `afterimage_tick_phantom_breaker`, `afterimage_check_phantom_bounce`,
//! `afterimage_spawn_phantom_bolt`), and the `activate` / `wire`
//! dispatch entry points. Re-uses
//! `crate::effect_v3::effects::phantom_bolt::components::{PhantomBolt,
//! PhantomLifetime, PhantomOwner}` and delegates phantom-bolt lifetime
//! tick-down to the existing `tick_phantom_lifetime` registered by
//! `EffectV3Plugin` via `SpawnPhantomConfig::wire`.
//!
//! Node-exit cleanup for afterimage-owned phantom entities is delegated to
//! the `rantzsoft_stateflow` cleanup handler — every spawn attaches
//! `CleanupOnExit::<NodeState>::default()`, so no custom cleanup system is
//! required here.

mod activate;
mod check_phantom_bounce;
mod components;
mod config;
mod spawn_phantom_bolt;
mod spawn_phantom_breaker;
mod tick_phantom_breaker;
mod wire;

// External lib consumers (`protocol/protocols/mod.rs`) only touch `activate`
// and `wire`. The test-only re-exports are gated with `#[cfg(test)]` so
// clippy doesn't flag them as unused in the lib build — tests reach them via
// `super::super::system::{PhantomBreaker, AfterimageConfig, PhantomBolt, ...}`.
// The 4 system-fn names and `PhantomLifetime` / `PhantomOwner` are NOT
// re-exported because nothing consumes them by that path (wire.rs uses
// sibling-module paths; tests import `PhantomLifetime` / `PhantomOwner`
// directly from `crate::effect_v3::effects::phantom_bolt::components`).
pub(crate) use activate::activate;
#[cfg(test)]
pub(crate) use components::PhantomBreakerLifetime;
#[cfg(test)]
pub(crate) use config::AfterimageConfig;
pub(crate) use wire::wire;

#[cfg(test)]
pub(crate) use crate::breaker::components::PhantomBreaker;
#[cfg(test)]
pub(crate) use crate::effect_v3::effects::phantom_bolt::components::PhantomBolt;
