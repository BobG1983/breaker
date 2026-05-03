//! Afterimage protocol — phantom-breaker + phantom-bolt.
//!
//! Design doc: `docs/design/protocols/afterimage.md`.
//!
//! Owns `AfterimageConfig`, the two runtime systems
//! (`afterimage_spawn_phantom_breaker`, `afterimage_spawn_phantom_bolt`),
//! and the `activate` / `wire` dispatch entry points. Re-uses
//! `crate::effect_v3::effects::phantom_bolt::components::{PhantomBolt,
//! PhantomLifetime, PhantomOwner}` and delegates phantom-bolt lifetime
//! tick-down to the existing `tick_phantom_lifetime` registered by
//! `EffectV3Plugin` via `SpawnPhantomConfig::wire`.
//!
//! Phantom-breaker lifetime ticking is delegated to
//! `tick_phantom_breaker_lifespan` (acting on the canonical `Lifespan`
//! component the builder inserts via `.phantom(...)`), and bolt-vs-phantom
//! collisions are handled by the standard `bolt_breaker_collision` path —
//! the afterimage domain no longer owns a bounce-check system.
//!
//! Node-exit cleanup for afterimage-owned phantom entities is delegated to
//! the `rantzsoft_stateflow` cleanup handler — every spawn attaches
//! `CleanupOnExit::<NodeState>::default()`, so no custom cleanup system is
//! required here.

mod activate;
mod config;
mod spawn_phantom_bolt;
mod spawn_phantom_breaker;
mod wire;

// External lib consumers (`protocol/protocols/mod.rs`) only touch `activate`
// and `wire`. The test-only re-exports are gated with `#[cfg(test)]` so
// clippy doesn't flag them as unused in the lib build — tests reach them via
// `super::super::system::{PhantomBreaker, AfterimageConfig, PhantomBolt, ...}`.
// The system-fn names and `PhantomLifetime` / `PhantomOwner` are NOT
// re-exported because nothing consumes them by that path (wire.rs uses
// sibling-module paths; tests import `PhantomLifetime` / `PhantomOwner`
// directly from `crate::effect_v3::effects::phantom_bolt::components`).
pub(crate) use activate::activate;
#[cfg(test)]
pub(crate) use config::AfterimageConfig;
pub(crate) use wire::wire;

#[cfg(test)]
pub(crate) use crate::breaker::components::PhantomBreaker;
#[cfg(test)]
pub(crate) use crate::effect_v3::effects::phantom_bolt::components::PhantomBolt;
