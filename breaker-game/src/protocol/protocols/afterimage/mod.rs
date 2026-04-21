//! Afterimage protocol — phantom-breaker / phantom-bolt runtime.
//!
//! Design doc: `docs/todos/detail/mod-system-design/protocols/afterimage.md`.
//!
//! Owns `AfterimageConfig` (per-run tuning), `PhantomBreaker` /
//! `PhantomBreakerLifetime` components, the four runtime systems
//! (`afterimage_spawn_phantom_breaker`, `afterimage_tick_phantom_breaker`,
//! `afterimage_check_phantom_bounce`, `afterimage_spawn_phantom_bolt`),
//! and the `activate` / `register` dispatch entry points. Re-uses
//! `crate::effect_v3::effects::phantom_bolt::{PhantomBolt, PhantomLifetime,
//! PhantomOwner}` for the spawned phantom-bolt entity bundle, and delegates
//! phantom-bolt lifetime tick-down to the existing
//! `tick_phantom_lifetime` (registered by `EffectV3Plugin` via
//! `SpawnPhantomConfig::register`).
//!
//! Node-exit cleanup for afterimage-spawned phantom breakers and phantom
//! bolts is delegated to the stateflow `CleanupOnExit::<NodeState>`
//! handler attached at spawn time — afterimage does NOT define a custom
//! cleanup system.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{activate, register};
