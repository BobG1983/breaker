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
//! alongside the P2 core types. Systems, stacks, and the plugin arrive in
//! later phases as a `Dmgable`-driven pipeline.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "test assertions use unwrap/expect/panic"
    )
)]

mod components;
mod messages;
mod source_id;
mod traits;

pub use components::{Dead, HealCap, Hp, Invulnerable, KilledBy};
pub use messages::{DamageDealt, DespawnEntity, Destroyed, HealDealt, KillYourself};
pub use source_id::SourceId;
pub use traits::Dmgable;

#[cfg(test)]
mod tests {
    // ── Behavior 28: every P2 type is reachable via the crate root ──

    #[test]
    fn all_core_types_accessible() {
        use crate::*;

        // Dmgable: a test-local dummy type binds the trait through the
        // re-exported path.
        #[derive(bevy::prelude::Component)]
        struct TestT;
        impl Dmgable for TestT {}
        // Construct `TestT` so it is not flagged as dead code — the
        // `impl Dmgable for TestT {}` above is the reason it exists.
        let _ = TestT;

        // Each of the other six types is constructed and inspected here so
        // every imported name is used — no unused-import warnings, no
        // `#[allow(unused_imports)]`.
        let hp = Hp::new(3.0);
        assert!(
            (hp.current - 3.0).abs() < f32::EPSILON,
            "expected 3.0, got {}",
            hp.current
        );

        let _ = Dead;
        let _ = Invulnerable;

        let kb = KilledBy { dealer: None };
        assert!(kb.dealer.is_none());

        assert_ne!(HealCap::Starting, HealCap::Max);

        let source = SourceId::from("src:alpha");
        let source_again = SourceId::from("src:alpha");
        assert_eq!(source, source_again);
    }

    // ── Behavior 29: every new message is re-exported from the crate root
    //     (named imports) ──

    #[test]
    fn all_message_types_re_exported_from_crate_root() {
        use std::marker::PhantomData;

        use bevy::prelude::Entity;

        use crate::{
            DamageDealt, DespawnEntity, Destroyed, Dmgable, HealCap, HealDealt, KillYourself,
            SourceId,
        };

        #[derive(bevy::prelude::Component)]
        struct TestT;
        impl Dmgable for TestT {}

        drop(DamageDealt::<TestT> {
            dealer:  None,
            target:  Entity::PLACEHOLDER,
            amount:  1.0,
            source:  Some(SourceId::from("module:action")),
            _marker: PhantomData,
        });
        drop(HealDealt::<TestT> {
            healer:  None,
            target:  Entity::PLACEHOLDER,
            amount:  1.0,
            source:  None,
            cap:     HealCap::Starting,
            _marker: PhantomData,
        });
        let _ = KillYourself::<TestT> {
            victim:  Entity::PLACEHOLDER,
            killer:  None,
            _marker: PhantomData,
        };
        let _ = Destroyed::<TestT> {
            victim:     Entity::PLACEHOLDER,
            killer:     None,
            victim_pos: bevy::prelude::Vec2::ZERO,
            killer_pos: None,
            _marker:    PhantomData,
        };
        let _ = DespawnEntity {
            entity: Entity::PLACEHOLDER,
        };
    }

    #[test]
    fn all_message_types_re_exported_from_crate_root_via_glob() {
        // Edge case: glob import `use crate::*;` must also resolve all five
        // names (covers both Behavior 29's edge case and Behavior 30's
        // requirement that PhantomData<T> field is reachable from crate root
        // via struct-literal construction).
        use std::marker::PhantomData;

        use bevy::prelude::{Entity, Vec2};

        use crate::*;

        #[derive(bevy::prelude::Component)]
        struct TestT;
        impl Dmgable for TestT {}

        // Behavior 30: struct-literal construction of all four generic
        // messages with `_marker: PhantomData` proves the marker field is
        // reachable with `pub` visibility from the crate-root consumer site.
        drop(DamageDealt::<TestT> {
            dealer:  None,
            target:  Entity::PLACEHOLDER,
            amount:  1.0,
            source:  None,
            _marker: PhantomData,
        });
        drop(HealDealt::<TestT> {
            healer:  None,
            target:  Entity::PLACEHOLDER,
            amount:  1.0,
            source:  None,
            cap:     HealCap::Max,
            _marker: PhantomData,
        });
        let _ = KillYourself::<TestT> {
            victim:  Entity::PLACEHOLDER,
            killer:  None,
            _marker: PhantomData,
        };
        let _ = Destroyed::<TestT> {
            victim:     Entity::PLACEHOLDER,
            killer:     None,
            victim_pos: Vec2::ZERO,
            killer_pos: None,
            _marker:    PhantomData,
        };
        let _ = DespawnEntity {
            entity: Entity::PLACEHOLDER,
        };
    }
}
