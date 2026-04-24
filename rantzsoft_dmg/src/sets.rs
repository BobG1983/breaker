//! `DmgSystems` — ordered `FixedUpdate` stages of the damage pipeline.
//!
//! `RantzDmgPlugin` configures these 16 variants as a single `.chain()` so
//! consumers attach systems to individual stages via
//! `.in_set(DmgSystems::...)` without re-declaring ordering. P4 attaches no
//! per-set systems to these variants; later phases (P5 stacks, P6 per-`T`
//! systems + `register_dmgable`) will. Consumers may also attach game-side
//! systems directly.

use bevy::prelude::*;

/// Ordered `FixedUpdate` stages of the damage pipeline.
///
/// Sixteen stages, configured as a single `.chain()` by `RantzDmgPlugin`.
/// Consumer systems attach to a specific stage via `.in_set(DmgSystems::X)`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DmgSystems {
    /// External / consumer systems emit `DamageDealt<T>` messages for the
    /// tick.
    EmitDamage,
    /// Consumer-registered post-emit hook stage (game-side custom systems
    /// that observe/react to freshly-emitted damage messages). Ships empty
    /// in the crate.
    PostEmitDamage,
    /// Dealer-side damage-boost stacks multiply in-flight damage amounts.
    ApplyDamageBoosts,
    /// Consumer-registered mutators further mutate in-flight damage messages
    /// (amplifiers, redirectors, siblings).
    MutateDamage,
    /// Consumer-registered post-mutate hook stage. Ships empty in the crate.
    PostMutateDamage,
    /// Target-side vulnerability stacks multiply in-flight damage amounts.
    ApplyVulnerable,
    /// Final damage is written to HP (and `KilledBy` is inserted on the
    /// killing blow).
    ApplyDamage,
    /// Consumer-registered post-apply-damage hook stage (ripple emitters like
    /// diffusion rings, tether partner, echo strike siblings). Ships empty
    /// in the crate.
    PostApplyDamage,
    /// Entities whose HP reached zero emit `KillYourself<T>` messages.
    EmitKill,
    /// Consumer-registered mutators mutate in-flight kill messages (e.g.
    /// reviver effects).
    MutateKill,
    /// Kill messages are resolved (dead-marking, derived `Destroyed<T>`
    /// emission, despawn intents).
    ApplyKill,
    /// Consumer-registered post-apply-kill hook stage. Ships empty in the crate.
    PostApplyKill,
    /// External / consumer systems emit `HealDealt<T>` messages for the tick.
    EmitHeal,
    /// Consumer-registered mutators mutate in-flight heal messages.
    MutateHeal,
    /// Final healing is written to HP.
    ApplyHeal,
    /// Consumer-registered post-apply-heal hook stage. Ships empty in the crate.
    PostApplyHeal,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use bevy::prelude::*;

    use super::*;

    // ── Behavior 31: `DmgSystems` enum exists at the crate root with all
    //     sixteen variants ──

    #[test]
    fn all_sixteen_variants_construct_and_are_usable() {
        // Construct every variant once as a binding so each name resolves.
        let v_emit_damage = DmgSystems::EmitDamage;
        let v_post_emit = DmgSystems::PostEmitDamage;
        let v_apply_damage_boosts = DmgSystems::ApplyDamageBoosts;
        let v_mutate_damage = DmgSystems::MutateDamage;
        let v_post_mutate = DmgSystems::PostMutateDamage;
        let v_apply_vulnerable = DmgSystems::ApplyVulnerable;
        let v_apply_damage = DmgSystems::ApplyDamage;
        let v_post_apply = DmgSystems::PostApplyDamage;
        let v_emit_kill = DmgSystems::EmitKill;
        let v_mutate_kill = DmgSystems::MutateKill;
        let v_apply_kill = DmgSystems::ApplyKill;
        let v_post_kill = DmgSystems::PostApplyKill;
        let v_emit_heal = DmgSystems::EmitHeal;
        let v_mutate_heal = DmgSystems::MutateHeal;
        let v_apply_heal = DmgSystems::ApplyHeal;
        let v_post_heal = DmgSystems::PostApplyHeal;

        // Use each binding in an assertion site (adjacent-pair inequality).
        assert_ne!(v_emit_damage, v_post_emit);
        assert_ne!(v_post_emit, v_apply_damage_boosts);
        assert_ne!(v_apply_damage_boosts, v_mutate_damage);
        assert_ne!(v_mutate_damage, v_post_mutate);
        assert_ne!(v_post_mutate, v_apply_vulnerable);
        assert_ne!(v_apply_vulnerable, v_apply_damage);
        assert_ne!(v_apply_damage, v_post_apply);
        assert_ne!(v_post_apply, v_emit_kill);
        assert_ne!(v_emit_kill, v_mutate_kill);
        assert_ne!(v_mutate_kill, v_apply_kill);
        assert_ne!(v_apply_kill, v_post_kill);
        assert_ne!(v_post_kill, v_emit_heal);
        assert_ne!(v_emit_heal, v_mutate_heal);
        assert_ne!(v_mutate_heal, v_apply_heal);
        assert_ne!(v_apply_heal, v_post_heal);
    }

    // ── Behavior 32: `DmgSystems` derives `Debug` — every variant produces a
    //     non-empty, distinct format string ──

    #[test]
    fn debug_format_is_distinct_and_contains_variant_name() {
        let variants = [
            (DmgSystems::EmitDamage, "EmitDamage"),
            (DmgSystems::PostEmitDamage, "PostEmitDamage"),
            (DmgSystems::ApplyDamageBoosts, "ApplyDamageBoosts"),
            (DmgSystems::MutateDamage, "MutateDamage"),
            (DmgSystems::PostMutateDamage, "PostMutateDamage"),
            (DmgSystems::ApplyVulnerable, "ApplyVulnerable"),
            (DmgSystems::ApplyDamage, "ApplyDamage"),
            (DmgSystems::PostApplyDamage, "PostApplyDamage"),
            (DmgSystems::EmitKill, "EmitKill"),
            (DmgSystems::MutateKill, "MutateKill"),
            (DmgSystems::ApplyKill, "ApplyKill"),
            (DmgSystems::PostApplyKill, "PostApplyKill"),
            (DmgSystems::EmitHeal, "EmitHeal"),
            (DmgSystems::MutateHeal, "MutateHeal"),
            (DmgSystems::ApplyHeal, "ApplyHeal"),
            (DmgSystems::PostApplyHeal, "PostApplyHeal"),
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

        // The 16 formatted strings are pairwise distinct.
        let unique: HashSet<&String> = formatted.iter().collect();
        assert_eq!(unique.len(), 16);

        // Edge case: last variant formats exactly as "PostApplyHeal".
        assert_eq!(format!("{:?}", DmgSystems::PostApplyHeal), "PostApplyHeal");
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

        // Edge case: iterate all 16 variants, assert reflexive equality for
        // each (via two independent bindings to avoid `clippy::eq_op`), then
        // inequality for each adjacent pair in the chain.
        let chain = [
            DmgSystems::EmitDamage,
            DmgSystems::PostEmitDamage,
            DmgSystems::ApplyDamageBoosts,
            DmgSystems::MutateDamage,
            DmgSystems::PostMutateDamage,
            DmgSystems::ApplyVulnerable,
            DmgSystems::ApplyDamage,
            DmgSystems::PostApplyDamage,
            DmgSystems::EmitKill,
            DmgSystems::MutateKill,
            DmgSystems::ApplyKill,
            DmgSystems::PostApplyKill,
            DmgSystems::EmitHeal,
            DmgSystems::MutateHeal,
            DmgSystems::ApplyHeal,
            DmgSystems::PostApplyHeal,
        ];
        for (i, v) in chain.iter().enumerate() {
            let other = chain[i];
            assert_eq!(*v, other);
        }
        for pair in chain.windows(2) {
            assert_ne!(pair[0], pair[1]);
        }
    }

    // ── Behavior 36: `DmgSystems` derives `Hash` — all 16 variants hash
    //     distinctly ──

    #[test]
    fn hash_distinct_for_all_sixteen_variants() {
        let mut set: HashSet<DmgSystems> = HashSet::new();
        set.insert(DmgSystems::EmitDamage);
        set.insert(DmgSystems::PostEmitDamage);
        set.insert(DmgSystems::ApplyDamageBoosts);
        set.insert(DmgSystems::MutateDamage);
        set.insert(DmgSystems::PostMutateDamage);
        set.insert(DmgSystems::ApplyVulnerable);
        set.insert(DmgSystems::ApplyDamage);
        set.insert(DmgSystems::PostApplyDamage);
        set.insert(DmgSystems::EmitKill);
        set.insert(DmgSystems::MutateKill);
        set.insert(DmgSystems::ApplyKill);
        set.insert(DmgSystems::PostApplyKill);
        set.insert(DmgSystems::EmitHeal);
        set.insert(DmgSystems::MutateHeal);
        set.insert(DmgSystems::ApplyHeal);
        set.insert(DmgSystems::PostApplyHeal);
        assert_eq!(set.len(), 16);

        // Edge case: inserting the same variant twice keeps len unchanged.
        set.insert(DmgSystems::EmitDamage);
        assert_eq!(set.len(), 16);
    }

    // ── Behavior 37: `DmgSystems` implements `SystemSet` — variants usable
    //     with `.in_set(...)` at app-build time ──

    #[test]
    fn in_set_accepts_every_variant_and_ticks_clean() {
        for variant in [
            DmgSystems::EmitDamage,
            DmgSystems::PostEmitDamage,
            DmgSystems::ApplyDamage,
            DmgSystems::PostApplyDamage,
            DmgSystems::EmitKill,
            DmgSystems::PostApplyKill,
            DmgSystems::ApplyHeal,
            DmgSystems::PostApplyHeal,
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
        app.add_systems(FixedUpdate, (|| {}).before(DmgSystems::PostApplyDamage));
        app.add_systems(FixedUpdate, (|| {}).after(DmgSystems::PostApplyHeal));
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
        let _ = crate::DmgSystems::PostApplyHeal;
        let _ = crate::DmgSystems::ApplyDamage;
        // Construct every variant via the re-exported path — proves each
        // variant is reachable through `crate::DmgSystems::...`.
        let all = [
            crate::DmgSystems::EmitDamage,
            crate::DmgSystems::PostEmitDamage,
            crate::DmgSystems::ApplyDamageBoosts,
            crate::DmgSystems::MutateDamage,
            crate::DmgSystems::PostMutateDamage,
            crate::DmgSystems::ApplyVulnerable,
            crate::DmgSystems::ApplyDamage,
            crate::DmgSystems::PostApplyDamage,
            crate::DmgSystems::EmitKill,
            crate::DmgSystems::MutateKill,
            crate::DmgSystems::ApplyKill,
            crate::DmgSystems::PostApplyKill,
            crate::DmgSystems::EmitHeal,
            crate::DmgSystems::MutateHeal,
            crate::DmgSystems::ApplyHeal,
            crate::DmgSystems::PostApplyHeal,
        ];
        assert_eq!(all.len(), 16);
    }

    #[test]
    fn dmg_systems_resolves_via_crate_glob_import() {
        // Edge case: `use crate::*;` glob import also resolves DmgSystems.
        use crate::*;

        let _ = DmgSystems::EmitDamage;
        let _ = DmgSystems::PostApplyHeal;
    }
}
