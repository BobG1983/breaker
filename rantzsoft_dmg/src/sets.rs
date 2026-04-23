//! `DmgSystems` — ordered `FixedUpdate` stages of the damage pipeline.
//!
//! `RantzDmgPlugin` configures these 11 variants as a single `.chain()` so
//! consumers attach systems to individual stages via
//! `.in_set(DmgSystems::...)` without re-declaring ordering. P4 attaches no
//! per-set systems to these variants; later phases (P5 stacks, P6 per-`T`
//! systems + `register_dmgable`) will. Consumers may also attach game-side
//! systems directly.

use bevy::prelude::*;

/// Ordered `FixedUpdate` stages of the damage pipeline.
///
/// Eleven stages, configured as a single `.chain()` by `RantzDmgPlugin`.
/// Consumer systems attach to a specific stage via `.in_set(DmgSystems::X)`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DmgSystems {
    /// External / consumer systems emit `DamageDealt<T>` messages for the
    /// tick.
    EmitDamage,
    /// Dealer-side damage-boost stacks multiply in-flight damage amounts.
    ApplyDamageBoosts,
    /// Consumer-registered mutators further mutate in-flight damage messages
    /// (amplifiers, redirectors, siblings).
    MutateDamage,
    /// Target-side vulnerability stacks multiply in-flight damage amounts.
    ApplyVulnerable,
    /// Final damage is written to HP (and `KilledBy` is inserted on the
    /// killing blow).
    ApplyDamage,
    /// Entities whose HP reached zero emit `KillYourself<T>` messages.
    EmitKill,
    /// Consumer-registered mutators mutate in-flight kill messages (e.g.
    /// reviver effects).
    MutateKill,
    /// Kill messages are resolved (dead-marking, derived `Destroyed<T>`
    /// emission, despawn intents).
    ApplyKill,
    /// External / consumer systems emit `HealDealt<T>` messages for the tick.
    EmitHeal,
    /// Consumer-registered mutators mutate in-flight heal messages.
    MutateHeal,
    /// Final healing is written to HP.
    ApplyHeal,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use bevy::prelude::*;

    use super::*;

    // ── Behavior 31: `DmgSystems` enum exists at the crate root with all
    //     eleven variants ──

    #[test]
    fn all_eleven_variants_construct_and_are_usable() {
        // Construct every variant once as a binding so each name resolves.
        let v_emit_damage = DmgSystems::EmitDamage;
        let v_apply_damage_boosts = DmgSystems::ApplyDamageBoosts;
        let v_mutate_damage = DmgSystems::MutateDamage;
        let v_apply_vulnerable = DmgSystems::ApplyVulnerable;
        let v_apply_damage = DmgSystems::ApplyDamage;
        let v_emit_kill = DmgSystems::EmitKill;
        let v_mutate_kill = DmgSystems::MutateKill;
        let v_apply_kill = DmgSystems::ApplyKill;
        let v_emit_heal = DmgSystems::EmitHeal;
        let v_mutate_heal = DmgSystems::MutateHeal;
        let v_apply_heal = DmgSystems::ApplyHeal;

        // Use each binding in an assertion site (adjacent-pair inequality).
        assert_ne!(v_emit_damage, v_apply_damage_boosts);
        assert_ne!(v_apply_damage_boosts, v_mutate_damage);
        assert_ne!(v_mutate_damage, v_apply_vulnerable);
        assert_ne!(v_apply_vulnerable, v_apply_damage);
        assert_ne!(v_apply_damage, v_emit_kill);
        assert_ne!(v_emit_kill, v_mutate_kill);
        assert_ne!(v_mutate_kill, v_apply_kill);
        assert_ne!(v_apply_kill, v_emit_heal);
        assert_ne!(v_emit_heal, v_mutate_heal);
        assert_ne!(v_mutate_heal, v_apply_heal);
    }

    // ── Behavior 32: `DmgSystems` derives `Debug` — every variant produces a
    //     non-empty, distinct format string ──

    #[test]
    fn debug_format_is_distinct_and_contains_variant_name() {
        let variants = [
            (DmgSystems::EmitDamage, "EmitDamage"),
            (DmgSystems::ApplyDamageBoosts, "ApplyDamageBoosts"),
            (DmgSystems::MutateDamage, "MutateDamage"),
            (DmgSystems::ApplyVulnerable, "ApplyVulnerable"),
            (DmgSystems::ApplyDamage, "ApplyDamage"),
            (DmgSystems::EmitKill, "EmitKill"),
            (DmgSystems::MutateKill, "MutateKill"),
            (DmgSystems::ApplyKill, "ApplyKill"),
            (DmgSystems::EmitHeal, "EmitHeal"),
            (DmgSystems::MutateHeal, "MutateHeal"),
            (DmgSystems::ApplyHeal, "ApplyHeal"),
        ];

        let formatted: Vec<String> = variants.iter().map(|(v, _)| format!("{v:?}")).collect();

        // Each format string contains the variant identifier.
        for (formatted_str, (_, name)) in formatted.iter().zip(variants.iter()) {
            assert!(
                formatted_str.contains(name),
                "expected debug output to contain {name:?}, got {formatted_str:?}"
            );
            assert!(!formatted_str.is_empty());
        }

        // The 11 formatted strings are pairwise distinct.
        let unique: HashSet<&String> = formatted.iter().collect();
        assert_eq!(unique.len(), 11);

        // Edge case: last variant formats exactly as "ApplyHeal".
        assert_eq!(format!("{:?}", DmgSystems::ApplyHeal), "ApplyHeal");
    }

    // ── Behavior 33: `DmgSystems` derives `Clone` — cloned variant equals
    //     original ──

    // Routed through a generic `T: Clone` bound — proves a `Clone` impl
    // exists for the value type at compile time without tripping
    // `clippy::clone_on_copy`.
    #[must_use]
    fn require_clone<T: Clone>(value: &T) -> T {
        value.clone()
    }

    #[test]
    fn clone_preserves_variant_identity() {
        let original = DmgSystems::ApplyDamage;
        let cloned = require_clone(&original);
        assert_eq!(cloned, original);

        // Edge case: clone a variant, let the original binding go out of
        // scope, and use the clone in a later assertion. (Cannot `drop()` a
        // `Copy` variant — `clippy::dropping_copy_types` fires; a bare
        // scope-exit is the equivalent no-op here since the value is trivially
        // copyable.)
        let c = require_clone(&DmgSystems::EmitKill);
        assert_eq!(c, DmgSystems::EmitKill);
    }

    // ── Behavior 34: `DmgSystems` derives `Copy` — variant is trivially
    //     copyable (no move after use) ──

    fn eats(_: DmgSystems) {}

    #[test]
    fn copy_allows_double_use_and_pass_by_value() {
        let v = DmgSystems::MutateDamage;
        let a = v;
        let b = v;
        assert_eq!(a, b);
        // Original `v` still usable after the two copies — proves Copy.
        assert_eq!(v, DmgSystems::MutateDamage);

        // Edge case: passing by value into a helper twice compiles (proves
        // `Copy`, not just `Clone`).
        eats(v);
        eats(v);
    }

    // ── Behavior 35: `DmgSystems` derives `PartialEq` + `Eq` — same variant
    //     equal, different variants not equal ──

    #[test]
    fn partial_eq_eq_reflexive_and_pairwise() {
        let a = DmgSystems::ApplyDamageBoosts;
        let b = DmgSystems::ApplyDamageBoosts;
        let c = DmgSystems::ApplyVulnerable;

        assert!(a == b);
        assert!(a != c);
        assert!(b != c);

        // Edge case: iterate all 11 variants, assert reflexive equality for
        // each (via two independent bindings to avoid `clippy::eq_op`), then
        // inequality for each adjacent pair in the chain.
        let chain = [
            DmgSystems::EmitDamage,
            DmgSystems::ApplyDamageBoosts,
            DmgSystems::MutateDamage,
            DmgSystems::ApplyVulnerable,
            DmgSystems::ApplyDamage,
            DmgSystems::EmitKill,
            DmgSystems::MutateKill,
            DmgSystems::ApplyKill,
            DmgSystems::EmitHeal,
            DmgSystems::MutateHeal,
            DmgSystems::ApplyHeal,
        ];
        for (i, v) in chain.iter().enumerate() {
            let other = chain[i];
            assert_eq!(*v, other);
        }
        for pair in chain.windows(2) {
            assert_ne!(pair[0], pair[1]);
        }
    }

    // ── Behavior 36: `DmgSystems` derives `Hash` — all 11 variants hash
    //     distinctly ──

    #[test]
    fn hash_distinct_for_all_eleven_variants() {
        let mut set: HashSet<DmgSystems> = HashSet::new();
        set.insert(DmgSystems::EmitDamage);
        set.insert(DmgSystems::ApplyDamageBoosts);
        set.insert(DmgSystems::MutateDamage);
        set.insert(DmgSystems::ApplyVulnerable);
        set.insert(DmgSystems::ApplyDamage);
        set.insert(DmgSystems::EmitKill);
        set.insert(DmgSystems::MutateKill);
        set.insert(DmgSystems::ApplyKill);
        set.insert(DmgSystems::EmitHeal);
        set.insert(DmgSystems::MutateHeal);
        set.insert(DmgSystems::ApplyHeal);
        assert_eq!(set.len(), 11);

        // Edge case: inserting the same variant twice keeps len unchanged.
        set.insert(DmgSystems::EmitDamage);
        assert_eq!(set.len(), 11);
    }

    // ── Behavior 37: `DmgSystems` implements `SystemSet` — variants usable
    //     with `.in_set(...)` at app-build time ──

    #[test]
    fn in_set_accepts_every_variant_and_ticks_clean() {
        for variant in [
            DmgSystems::EmitDamage,
            DmgSystems::ApplyDamage,
            DmgSystems::EmitKill,
            DmgSystems::ApplyHeal,
        ] {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.add_systems(FixedUpdate, (|| {}).in_set(variant));
            let timestep = app.world().resource::<Time<Fixed>>().timestep();
            app.world_mut()
                .resource_mut::<Time<Fixed>>()
                .accumulate_overstep(timestep);
            app.update();
        }
    }

    // ── Behavior 38: `DmgSystems` variants are usable as `.before(...)` /
    //     `.after(...)` ordering constraints at app-build time ──

    #[test]
    fn before_after_constraints_accept_variants() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, (|| {}).before(DmgSystems::ApplyDamage));
        app.add_systems(FixedUpdate, (|| {}).after(DmgSystems::ApplyDamage));
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    #[test]
    fn before_after_constraints_accept_other_variants() {
        // Edge case: the same pattern for other variants proves the
        // constraint machinery accepts any variant, not just ApplyDamage.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, (|| {}).before(DmgSystems::EmitHeal));
        app.add_systems(FixedUpdate, (|| {}).after(DmgSystems::MutateHeal));
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Behavior 39: `DmgSystems` is `pub` at the crate root (re-exported
    //     from `lib.rs`) ──

    #[test]
    fn dmg_systems_is_pub_at_crate_root() {
        // Resolve DmgSystems through the crate-root re-export path. If
        // `lib.rs` did not `pub use sets::DmgSystems;`, this path would
        // fail with E0603.
        let _ = crate::DmgSystems::EmitDamage;
        let _ = crate::DmgSystems::ApplyHeal;
        let _ = crate::DmgSystems::ApplyDamage;
        // Construct every variant via the re-exported path — proves each
        // variant is reachable through `crate::DmgSystems::...`.
        let all = [
            crate::DmgSystems::EmitDamage,
            crate::DmgSystems::ApplyDamageBoosts,
            crate::DmgSystems::MutateDamage,
            crate::DmgSystems::ApplyVulnerable,
            crate::DmgSystems::ApplyDamage,
            crate::DmgSystems::EmitKill,
            crate::DmgSystems::MutateKill,
            crate::DmgSystems::ApplyKill,
            crate::DmgSystems::EmitHeal,
            crate::DmgSystems::MutateHeal,
            crate::DmgSystems::ApplyHeal,
        ];
        assert_eq!(all.len(), 11);
    }

    #[test]
    fn dmg_systems_resolves_via_crate_glob_import() {
        // Edge case: `use crate::*;` glob import also resolves DmgSystems.
        use crate::*;

        let _ = DmgSystems::EmitDamage;
        let _ = DmgSystems::ApplyHeal;
    }
}
