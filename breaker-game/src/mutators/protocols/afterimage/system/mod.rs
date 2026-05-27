//! Afterimage protocol — phantom-breaker + phantom-bolt.
//!
//! Design doc: `docs/design/protocols/afterimage.md`.
//!
//! Owns `AfterimageConfig`, the two runtime systems
//! (`afterimage_spawn_phantom_breaker`, `afterimage_spawn_phantom_bolt`),
//! and the `activate` / `wire` dispatch entry points. Uses the canonical
//! `crate::bolt::components` vocabulary — `Bolt::become_phantom` mutates
//! the real bolt in place, installing `PhantomBolt`, `PhantomDedupKey`,
//! `PhantomDamagedCells`, `Lifespan`, and
//! `LifetimeEndBehavior::RevertToNormalBolt`. Phantom-bolt lifetime
//! tick-down + expiry dispatch is delegated to `tick_bolt_lifespan` in
//! the bolt domain.
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
pub(crate) use activate::activate;
#[cfg(test)]
pub(crate) use config::AfterimageConfig;
pub(crate) use wire::wire;

#[cfg(test)]
pub(crate) use crate::bolt::components::PhantomBolt;
#[cfg(test)]
pub(crate) use crate::breaker::components::PhantomBreaker;
