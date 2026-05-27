//! Afterimage protocol — phantom-breaker / phantom-bolt runtime.
//!
//! Design doc: `docs/design/protocols/afterimage.md`.
//!
//! Owns `AfterimageConfig` (per-run tuning) and the runtime systems
//! (`afterimage_spawn_phantom_breaker`, `afterimage_spawn_phantom_bolt`),
//! plus the `activate` / `wire` dispatch entry points.
//!
//! Phantom-bolt semantics use the canonical `crate::bolt::components`
//! vocabulary — `Bolt::become_phantom` mutates the real bolt in place,
//! installing `PhantomBolt`, `PhantomDedupKey`, `PhantomDamagedCells`,
//! `Lifespan`, and `LifetimeEndBehavior::RevertToNormalBolt`. Lifetime
//! tick-down + expiry dispatch is delegated to `tick_bolt_lifespan` in the
//! bolt domain.
//!
//! Node-exit cleanup for afterimage-spawned phantom breakers is delegated
//! to the stateflow `CleanupOnExit::<NodeState>` handler attached at spawn
//! time — afterimage does NOT define a custom cleanup system.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{activate, wire};
