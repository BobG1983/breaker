//! `RantzDmgPlugin` — the damage-pipeline plugin skeleton.
//!
//! The plugin registers the non-generic `DespawnEntity` message, configures
//! the 11-variant `DmgSystems` chain under `FixedUpdate`, and schedules a
//! no-op `process_despawn_requests` stub in `FixedPostUpdate`. Per-`T`
//! generic messages (`DamageDealt<T>`, `HealDealt<T>`, `KillYourself<T>`,
//! `Destroyed<T>`) and per-`T` generic systems are registered later by
//! `register_dmgable::<T>` in P6 via `RantzDmgAppExt` — NOT here.

use bevy::prelude::*;

use crate::{messages::DespawnEntity, sets::DmgSystems, systems::process_despawn_requests};

/// The damage-pipeline plugin. Consumers add this once to get the ordered
/// `DmgSystems` chain and the `DespawnEntity` message registered.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RantzDmgPlugin;

impl Plugin for RantzDmgPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DespawnEntity>();

        app.configure_sets(
            FixedUpdate,
            (
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
            )
                .chain(),
        );

        app.add_systems(FixedPostUpdate, process_despawn_requests);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
    use crate::{
        Dmgable,
        messages::{DamageDealt, Destroyed, HealDealt, KillYourself},
        systems::process_despawn_requests,
    };

    // ── Behavior 40: `RantzDmgPlugin` is a unit struct at the crate root ──

    #[test]
    fn unit_struct_constructs_without_arguments() {
        let _ = RantzDmgPlugin;
        // Edge case: struct-literal form also compiles.
        let _ = RantzDmgPlugin {};
    }

    // ── Behavior 41: `RantzDmgPlugin` derives `Default` ──

    #[test]
    fn default_constructor_compiles() {
        // The `<_ as Default>::default()` form below exercises the Default
        // derive without tripping `clippy::default_constructed_unit_structs`
        // (which would fire on `RantzDmgPlugin::default()`).
        let _ = <RantzDmgPlugin as Default>::default();
    }

    // ── Behavior 42: `RantzDmgPlugin` derives `Debug`, `Clone`, `Copy`,
    //     `PartialEq`, `Eq`, `Hash` ──

    // Routed through a generic `T: Clone` bound — proves a `Clone` impl
    // exists for the value type at compile time without tripping
    // `clippy::clone_on_copy`.
    #[must_use]
    fn require_clone<T: Clone>(value: &T) -> T {
        value.clone()
    }

    #[test]
    fn derives_debug_clone_copy_partial_eq_eq_hash() {
        use std::collections::HashSet;

        let p = RantzDmgPlugin;

        // Debug — non-empty, contains the type name.
        let formatted = format!("{p:?}");
        assert!(!formatted.is_empty());
        assert!(
            formatted.contains("RantzDmgPlugin"),
            "expected debug output to contain RantzDmgPlugin, got {formatted:?}"
        );

        // Clone — returns an equal value.
        let cloned = require_clone(&p);
        assert_eq!(cloned, p);

        // Copy — double-use without move.
        let a = p;
        let b = p;
        assert_eq!(a, b);

        // PartialEq + Eq.
        assert_eq!(p, RantzDmgPlugin);

        // Hash.
        let mut set: HashSet<RantzDmgPlugin> = HashSet::new();
        set.insert(p);
        assert_eq!(set.len(), 1);

        // Edge case: clone compares equal.
        assert_eq!(p, require_clone(&p));
    }

    // ── Behavior 43: `RantzDmgPlugin` is `pub` at the crate root
    //     (re-exported from `lib.rs`) ──

    #[test]
    fn rantz_dmg_plugin_is_pub_at_crate_root() {
        // Resolve RantzDmgPlugin through the crate-root re-export path. If
        // `lib.rs` did not `pub use plugin::RantzDmgPlugin;`, this path
        // would fail with E0603.
        let _ = crate::RantzDmgPlugin;
        let _ = <crate::RantzDmgPlugin as Default>::default();
    }

    #[test]
    fn rantz_dmg_plugin_resolves_via_crate_glob_import() {
        // Edge case: `use crate::*;` glob import also resolves
        // RantzDmgPlugin.
        use crate::*;

        let _ = RantzDmgPlugin;
    }

    // ── Behavior 44: `RantzDmgPlugin` implements `bevy::prelude::Plugin` ──

    fn requires_plugin<P: Plugin>(_: P) {}

    #[test]
    fn impl_plugin_and_app_builds_and_ticks() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);
        app.update();

        // Edge case: the trait bound is satisfied at the type level, not
        // only via dyn dispatch.
        requires_plugin(RantzDmgPlugin);
    }

    // ── Behavior 45: Adding `RantzDmgPlugin` registers `DespawnEntity` as a
    //     Bevy message ──

    #[test]
    fn plugin_registers_despawn_entity_message() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);

        assert!(
            app.world().contains_resource::<Messages<DespawnEntity>>(),
            "RantzDmgPlugin should register Messages<DespawnEntity>"
        );
    }

    #[test]
    fn baseline_without_plugin_does_not_register_despawn_entity() {
        // Edge case: baseline without `RantzDmgPlugin` must NOT have the
        // resource — proves the plugin is what registers it, not default
        // Bevy behavior.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        assert!(
            !app.world().contains_resource::<Messages<DespawnEntity>>(),
            "baseline App without RantzDmgPlugin must not have Messages<DespawnEntity>"
        );
    }

    // ── Behavior 46: Adding `RantzDmgPlugin` does NOT register any per-`T`
    //     messages ──

    #[derive(Component)]
    struct TestT;
    impl Dmgable for TestT {}

    #[derive(Component)]
    struct TestU;
    impl Dmgable for TestU {}

    #[test]
    fn plugin_does_not_register_per_t_messages() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);

        // TestT — none of the four per-T messages are registered.
        assert!(
            !app.world()
                .contains_resource::<Messages<DamageDealt<TestT>>>()
        );
        assert!(
            !app.world()
                .contains_resource::<Messages<HealDealt<TestT>>>()
        );
        assert!(
            !app.world()
                .contains_resource::<Messages<KillYourself<TestT>>>()
        );
        assert!(
            !app.world()
                .contains_resource::<Messages<Destroyed<TestT>>>()
        );

        // Edge case: same for a second dummy type TestU — also all false.
        assert!(
            !app.world()
                .contains_resource::<Messages<DamageDealt<TestU>>>()
        );
        assert!(
            !app.world()
                .contains_resource::<Messages<HealDealt<TestU>>>()
        );
        assert!(
            !app.world()
                .contains_resource::<Messages<KillYourself<TestU>>>()
        );
        assert!(
            !app.world()
                .contains_resource::<Messages<Destroyed<TestU>>>()
        );
    }

    // ── Behavior 47: Adding `RantzDmgPlugin` configures all 11 `DmgSystems`
    //     variants as sets in `FixedUpdate` usable as `.before(...)` /
    //     `.after(...)` anchors ──

    #[test]
    fn all_eleven_sets_are_usable_as_before_and_after_anchors() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);

        let variants = [
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

        for v in variants {
            app.add_systems(FixedUpdate, (|| {}).after(v));
        }
        // Edge case: also add .before for each of the 11 — 22 no-op systems
        // total.
        for v in variants {
            app.add_systems(FixedUpdate, (|| {}).before(v));
        }

        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Behavior 48: Adding `RantzDmgPlugin` orders the 11 variants in the
    //     exact plan/detail chain order within `FixedUpdate` ──

    #[derive(Resource, Default)]
    struct ExecutionLog(Vec<DmgSystems>);

    fn record_emit_damage(mut log: ResMut<ExecutionLog>) {
        log.0.push(DmgSystems::EmitDamage);
    }
    fn record_apply_damage_boosts(mut log: ResMut<ExecutionLog>) {
        log.0.push(DmgSystems::ApplyDamageBoosts);
    }
    fn record_mutate_damage(mut log: ResMut<ExecutionLog>) {
        log.0.push(DmgSystems::MutateDamage);
    }
    fn record_apply_vulnerable(mut log: ResMut<ExecutionLog>) {
        log.0.push(DmgSystems::ApplyVulnerable);
    }
    fn record_apply_damage(mut log: ResMut<ExecutionLog>) {
        log.0.push(DmgSystems::ApplyDamage);
    }
    fn record_emit_kill(mut log: ResMut<ExecutionLog>) {
        log.0.push(DmgSystems::EmitKill);
    }
    fn record_mutate_kill(mut log: ResMut<ExecutionLog>) {
        log.0.push(DmgSystems::MutateKill);
    }
    fn record_apply_kill(mut log: ResMut<ExecutionLog>) {
        log.0.push(DmgSystems::ApplyKill);
    }
    fn record_emit_heal(mut log: ResMut<ExecutionLog>) {
        log.0.push(DmgSystems::EmitHeal);
    }
    fn record_mutate_heal(mut log: ResMut<ExecutionLog>) {
        log.0.push(DmgSystems::MutateHeal);
    }
    fn record_apply_heal(mut log: ResMut<ExecutionLog>) {
        log.0.push(DmgSystems::ApplyHeal);
    }

    fn build_app_with_recorders() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);
        app.init_resource::<ExecutionLog>();
        app.add_systems(
            FixedUpdate,
            (
                record_emit_damage.in_set(DmgSystems::EmitDamage),
                record_apply_damage_boosts.in_set(DmgSystems::ApplyDamageBoosts),
                record_mutate_damage.in_set(DmgSystems::MutateDamage),
                record_apply_vulnerable.in_set(DmgSystems::ApplyVulnerable),
                record_apply_damage.in_set(DmgSystems::ApplyDamage),
                record_emit_kill.in_set(DmgSystems::EmitKill),
                record_mutate_kill.in_set(DmgSystems::MutateKill),
                record_apply_kill.in_set(DmgSystems::ApplyKill),
                record_emit_heal.in_set(DmgSystems::EmitHeal),
                record_mutate_heal.in_set(DmgSystems::MutateHeal),
                record_apply_heal.in_set(DmgSystems::ApplyHeal),
            ),
        );
        app
    }

    #[test]
    fn chain_order_matches_plan_after_one_tick() {
        let mut app = build_app_with_recorders();

        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();

        let expected = vec![
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

        assert_eq!(
            app.world().resource::<ExecutionLog>().0,
            expected,
            "DmgSystems chain ran in the wrong order — RantzDmgPlugin's \
             `configure_sets(FixedUpdate, (...).chain())` must list the 11 \
             variants in the canonical Emit/Mutate/Apply plan order"
        );
    }

    #[test]
    fn chain_order_stable_across_two_ticks() {
        // Edge case: a second tick produces another 11 push events in the
        // same order. Clear the log after the first tick, run a second
        // tick, and assert the order is stable.
        let mut app = build_app_with_recorders();

        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();

        // Clear the log.
        app.world_mut().resource_mut::<ExecutionLog>().0.clear();

        // Second tick.
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();

        let expected = vec![
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

        assert_eq!(app.world().resource::<ExecutionLog>().0, expected);
    }

    // ── Behavior 49: Adding `RantzDmgPlugin` places the chain in
    //     `FixedUpdate`, not in `Update` ──

    #[test]
    fn chain_is_in_fixed_update_not_update() {
        let mut app = build_app_with_recorders();

        // app.update() WITHOUT accumulating overstep — FixedUpdate should
        // NOT run, and the recorders should not fire.
        app.update();

        assert!(
            app.world().resource::<ExecutionLog>().0.is_empty(),
            "ExecutionLog should be empty when FixedUpdate has no overstep \
             budget — recorders (and thus the DmgSystems sets) must be bound \
             to FixedUpdate, not Update"
        );

        // Positive control companion assertion: accumulate overstep and tick
        // once — now the 11 recorders should fire.
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();

        assert_eq!(
            app.world().resource::<ExecutionLog>().0.len(),
            11,
            "After one FixedUpdate tick, all 11 recorders should have fired"
        );
    }

    // ── Behavior 50: Adding `RantzDmgPlugin` adds `process_despawn_requests`
    //     to `FixedPostUpdate` ──

    #[derive(Resource, Default)]
    struct PreDespawnWitnessCount(u32);

    fn pre_despawn_witness(mut c: ResMut<PreDespawnWitnessCount>) {
        c.0 += 1;
    }

    #[test]
    fn process_despawn_requests_scheduled_in_fixed_post_update() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);
        app.init_resource::<PreDespawnWitnessCount>();
        app.add_systems(
            FixedPostUpdate,
            pre_despawn_witness.before(process_despawn_requests),
        );

        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();

        assert_eq!(
            app.world().resource::<PreDespawnWitnessCount>().0,
            1,
            "pre_despawn_witness.before(process_despawn_requests) should have \
             fired once — this proves process_despawn_requests exists in \
             FixedPostUpdate as an ordering anchor"
        );
    }

    #[test]
    fn plugin_tolerates_empty_despawn_queue() {
        // Edge case: plugin does not panic when ticked with zero
        // DespawnEntity messages pending.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);

        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    #[test]
    fn plugin_tolerates_single_despawn_message_in_queue() {
        // Edge case (stronger): write one `DespawnEntity` message, tick
        // once, assert no panic. We do NOT assert the message was consumed
        // — that is P6's job.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);

        app.world_mut()
            .resource_mut::<Messages<DespawnEntity>>()
            .write(DespawnEntity {
                entity: Entity::PLACEHOLDER,
            });

        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Behavior 51: RESERVED NUMBER GAP — no test ──
    //
    // Behavior 51 (duplicate-plugin add policy) is intentionally out of scope
    // for P4 per the test spec: Bevy's built-in duplicate-plugin handling is
    // not a P4-level decision. The number is preserved in the global
    // behavior ladder to avoid renumbering across phases.

    // ── Behavior 52: Adding `RantzDmgPlugin` to a headless app does not
    //     panic ──

    #[test]
    fn adding_plugin_to_headless_app_does_not_panic() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);
        app.update();
    }

    #[test]
    fn adding_plugin_to_headless_app_does_not_panic_with_fixed_tick() {
        // Edge case: tick once with accumulated FixedUpdate overstep.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);

        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Behavior 53: `RantzDmgPlugin` does not inject any systems into
    //     `FixedUpdate` beyond what consumers add ──

    #[derive(Resource, Default)]
    struct FixedTickCount(u32);

    fn fixed_tick_witness(mut c: ResMut<FixedTickCount>) {
        c.0 += 1;
    }

    #[test]
    fn plugin_injects_no_systems_into_fixed_update_beyond_consumer_additions() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);
        app.init_resource::<ExecutionLog>();
        app.init_resource::<FixedTickCount>();
        app.add_systems(FixedUpdate, fixed_tick_witness);

        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();

        assert_eq!(
            app.world().resource::<FixedTickCount>().0,
            1,
            "FixedUpdate must run exactly once (positive control)"
        );
        assert!(
            app.world().resource::<ExecutionLog>().0.is_empty(),
            "RantzDmgPlugin must not inject any recorder system that writes \
             to ExecutionLog — the 11 DmgSystems sets are empty until \
             consumers attach systems"
        );
    }

    #[test]
    fn plugin_injects_no_systems_across_two_fixed_update_ticks() {
        // Edge case: two ticks — FixedTickCount should be 2 and
        // ExecutionLog should still be empty.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);
        app.init_resource::<ExecutionLog>();
        app.init_resource::<FixedTickCount>();
        app.add_systems(FixedUpdate, fixed_tick_witness);

        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        // First tick.
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
        // Second tick — two separate accumulate+update pairs (Duration * f32
        // does not compile in Bevy 0.18).
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();

        assert_eq!(app.world().resource::<FixedTickCount>().0, 2);
        assert!(app.world().resource::<ExecutionLog>().0.is_empty());
    }
}
