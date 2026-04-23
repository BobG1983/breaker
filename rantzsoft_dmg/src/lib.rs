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
pub use sets::DmgSystems;
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

    // ── Behavior 54: `DmgSystems` reachable through `use crate::*;` glob
    //     import ──

    #[test]
    fn dmg_systems_reachable_via_crate_glob_import() {
        use crate::*;

        // First variant reachable through the glob path.
        let first = DmgSystems::EmitDamage;
        // Edge case: last variant too — proves the glob doesn't cut off
        // midway.
        let last = DmgSystems::ApplyHeal;

        assert_ne!(first, last);
    }

    // ── Behavior 55: `RantzDmgPlugin` reachable through `use crate::*;`
    //     glob import ──

    #[test]
    fn rantz_dmg_plugin_reachable_via_crate_glob_import() {
        use bevy::prelude::*;

        use crate::*;

        let _ = RantzDmgPlugin;
        let _ = <RantzDmgPlugin as Default>::default();

        // Edge case: add it to a fresh App to prove Plugin trait resolves
        // through the glob re-export.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);
        app.update();
    }

    // ── Behavior 80: `DamageBoostStack` reachable via `use crate::*;` glob
    //     import ──

    #[test]
    fn damage_boost_stack_reachable_via_crate_glob_import() {
        use crate::*;
        let stack = DamageBoostStack::default();
        assert!(stack.is_empty());
    }

    // ── Behavior 81: `VulnerableStack` reachable via `use crate::*;` glob
    //     import ──

    #[test]
    fn vulnerable_stack_reachable_via_crate_glob_import() {
        use crate::*;
        let stack = VulnerableStack::default();
        assert!(stack.is_empty());
    }

    // ── Behavior 151: `RantzDmgAppExt` is reachable at crate root ──

    #[test]
    fn rantz_dmg_app_ext_reachable_at_crate_root() {
        use bevy::prelude::*;

        use crate::RantzDmgAppExt;

        #[derive(Component)]
        struct TestT;
        impl crate::Dmgable for TestT {}

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(crate::RantzDmgPlugin);
        let _ = app.register_dmgable::<TestT>();
    }

    // ── Behavior 152: `RantzDmgAppExt` is reachable via glob import ──

    #[test]
    fn rantz_dmg_app_ext_reachable_via_crate_glob_import() {
        use bevy::prelude::*;

        use crate::*;

        #[derive(Component)]
        struct TestT;
        impl Dmgable for TestT {}

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);
        let _ = app.register_dmgable::<TestT>();

        assert!(
            app.world()
                .contains_resource::<Messages<DamageDealt<TestT>>>()
        );
    }

    #[test]
    fn rantz_dmg_app_ext_chains_multiple_types_via_glob_import() {
        // Edge case 152a.
        use bevy::prelude::*;

        use crate::*;

        #[derive(Component)]
        struct A;
        impl Dmgable for A {}

        #[derive(Component)]
        struct B;
        impl Dmgable for B {}

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);
        let _ = app.register_dmgable::<A>().register_dmgable::<B>();

        assert!(app.world().contains_resource::<Messages<DamageDealt<A>>>());
        assert!(app.world().contains_resource::<Messages<HealDealt<A>>>());
        assert!(app.world().contains_resource::<Messages<KillYourself<A>>>());
        assert!(app.world().contains_resource::<Messages<Destroyed<A>>>());
        assert!(app.world().contains_resource::<Messages<DamageDealt<B>>>());
        assert!(app.world().contains_resource::<Messages<HealDealt<B>>>());
        assert!(app.world().contains_resource::<Messages<KillYourself<B>>>());
        assert!(app.world().contains_resource::<Messages<Destroyed<B>>>());
    }
}
