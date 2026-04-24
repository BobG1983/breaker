//! `RantzDmgAppExt` — extension trait registering per-`T: Dmgable`
//! messages and systems.
//!
//! The root `RantzDmgPlugin` performs zero per-`T` work — it only
//! registers `DespawnEntity`, configures the 16-set chain, and schedules
//! `process_despawn_requests`. Per-`T` wiring (four message queues and
//! six systems) is the consumer's job via `App::register_dmgable::<T>()`.
//!
//! ## Caller responsibility — call exactly once per `T`
//!
//! `register_dmgable::<T>()` does NOT track per-`T` state. Calling it
//! twice for the same `T` registers the six systems twice, causing them
//! to execute twice per tick. The caller is responsible for invoking
//! `register_dmgable::<T>()` **exactly once per `T`**.

use bevy::prelude::*;

use crate::{
    messages::{DamageDealt, Destroyed, HealDealt, KillYourself},
    sets::DmgSystems,
    systems::{
        apply_damage, apply_damage_boosts, apply_heal, apply_vulnerable, detect_deaths,
        handle_kill, invulnerable_filter,
    },
    traits::Dmgable,
};

/// Register a `Dmgable` type with the damage pipeline. Adds the four
/// per-`T` message queues and the six per-`T` `FixedUpdate` systems,
/// each bound to its appropriate `DmgSystems` set.
///
/// # Caller responsibility
///
/// Call exactly once per `T`. Calling twice causes double-execution of
/// the registered systems (duplicated damage, duplicated kill messages,
/// doubled one-shot stack drains). This trait does NOT track prior
/// registrations.
pub trait RantzDmgAppExt {
    /// Register `T` with the pipeline. See trait docs.
    ///
    /// # Caller responsibility
    ///
    /// Call exactly once per `T`. Calling twice causes double-execution
    /// of the registered systems.
    #[must_use = "register_dmgable returns &mut Self for fluent chaining; ignoring the return value is almost always a mistake"]
    fn register_dmgable<T: Dmgable>(&mut self) -> &mut Self;
}

impl RantzDmgAppExt for App {
    fn register_dmgable<T: Dmgable>(&mut self) -> &mut Self {
        self.add_message::<DamageDealt<T>>()
            .add_message::<HealDealt<T>>()
            .add_message::<KillYourself<T>>()
            .add_message::<Destroyed<T>>()
            .add_systems(
                FixedUpdate,
                (
                    apply_damage_boosts::<T>.in_set(DmgSystems::ApplyDamageBoosts),
                    apply_vulnerable::<T>.in_set(DmgSystems::ApplyVulnerable),
                    (invulnerable_filter::<T>, apply_damage::<T>)
                        .chain()
                        .in_set(DmgSystems::ApplyDamage),
                    detect_deaths::<T>.in_set(DmgSystems::EmitKill),
                    handle_kill::<T>.in_set(DmgSystems::ApplyKill),
                    apply_heal::<T>.in_set(DmgSystems::ApplyHeal),
                ),
            )
    }
}
