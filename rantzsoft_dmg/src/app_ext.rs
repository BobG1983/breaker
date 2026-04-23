//! `RantzDmgAppExt` — extension trait registering per-`T: Dmgable`
//! messages and systems.
//!
//! The root `RantzDmgPlugin` performs zero per-`T` work — it only
//! registers `DespawnEntity`, configures the 11-set chain, and schedules
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

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use bevy::prelude::*;
    use rantzsoft_spatial2d::components::Position2D;

    use super::*;
    use crate::{
        HealCap, Hp, KilledBy, RantzDmgPlugin, VulnerableStack,
        components::{DamageBoostStack, Invulnerable},
        messages::{DamageDealt, DespawnEntity, Destroyed, HealDealt, KillYourself},
    };

    #[derive(Component)]
    struct TestT;
    impl Dmgable for TestT {}

    #[derive(Component)]
    struct OtherT;
    impl Dmgable for OtherT {}

    #[track_caller]
    fn assert_f32_eq(actual: f32, expected: f32) {
        if expected.is_infinite() {
            assert!(
                actual.is_infinite() && actual.is_sign_positive() == expected.is_sign_positive(),
                "expected {expected}, got {actual}"
            );
        } else {
            assert!(
                (actual - expected).abs() < f32::EPSILON,
                "expected {expected}, got {actual}"
            );
        }
    }

    fn app_with_plugin() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzDmgPlugin);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Behavior 139: register_dmgable::<TestT> adds Messages<DamageDealt<TestT>> ──

    #[test]
    fn register_dmgable_adds_damage_dealt_resource() {
        let mut app = app_with_plugin();
        let _ = app.register_dmgable::<TestT>();
        assert!(
            app.world()
                .contains_resource::<Messages<DamageDealt<TestT>>>()
        );
    }

    // ── Behavior 140: register_dmgable adds HealDealt, KillYourself, Destroyed ──

    #[test]
    fn register_dmgable_adds_all_four_per_t_resources() {
        let mut app = app_with_plugin();
        let _ = app.register_dmgable::<TestT>();
        assert!(
            app.world()
                .contains_resource::<Messages<DamageDealt<TestT>>>()
        );
        assert!(
            app.world()
                .contains_resource::<Messages<HealDealt<TestT>>>()
        );
        assert!(
            app.world()
                .contains_resource::<Messages<KillYourself<TestT>>>()
        );
        assert!(
            app.world()
                .contains_resource::<Messages<Destroyed<TestT>>>()
        );
    }

    #[test]
    fn without_register_dmgable_none_of_the_four_resources_exist() {
        // Edge case 140a.
        let app = app_with_plugin();
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
    }

    // ── Behavior 141: returns &mut Self for chaining ──

    #[test]
    fn returns_mut_self_for_chaining() {
        let mut app = app_with_plugin();
        let _ = app.register_dmgable::<TestT>().register_dmgable::<OtherT>();

        assert!(
            app.world()
                .contains_resource::<Messages<DamageDealt<TestT>>>()
        );
        assert!(
            app.world()
                .contains_resource::<Messages<DamageDealt<OtherT>>>()
        );
    }

    // ── Behavior 142: per-T queues are isolated ──

    #[test]
    fn per_t_queues_are_isolated() {
        let mut app = app_with_plugin();
        let _ = app.register_dmgable::<TestT>().register_dmgable::<OtherT>();

        app.world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .write(DamageDealt::<TestT> {
                dealer:  None,
                target:  Entity::PLACEHOLDER,
                amount:  1.0,
                source:  None,
                _marker: PhantomData,
            });
        tick(&mut app);

        let drained_t = app
            .world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .drain()
            .count();
        let drained_u = app
            .world_mut()
            .resource_mut::<Messages<DamageDealt<OtherT>>>()
            .drain()
            .count();
        assert_eq!(drained_t, 1);
        assert_eq!(drained_u, 0);
    }

    // ── Behavior 143: single call registers all four resources ──

    #[test]
    fn single_call_registers_all_four_resources() {
        let mut app = app_with_plugin();
        let _ = app.register_dmgable::<TestT>();

        assert!(
            app.world()
                .contains_resource::<Messages<DamageDealt<TestT>>>()
        );
        assert!(
            app.world()
                .contains_resource::<Messages<HealDealt<TestT>>>()
        );
        assert!(
            app.world()
                .contains_resource::<Messages<KillYourself<TestT>>>()
        );
        assert!(
            app.world()
                .contains_resource::<Messages<Destroyed<TestT>>>()
        );
    }

    #[test]
    fn single_call_chains_with_unrelated_add_message() {
        // Edge case 143a.
        #[derive(Message)]
        struct Unrelated;

        let mut app = app_with_plugin();
        let _ = app.register_dmgable::<TestT>().add_message::<Unrelated>();

        assert!(
            app.world()
                .contains_resource::<Messages<DamageDealt<TestT>>>()
        );
        assert!(app.world().contains_resource::<Messages<Unrelated>>());
    }

    // ── Behavior 144: apply_damage_boosts::<TestT> attached to ApplyDamageBoosts ──

    #[test]
    fn apply_damage_boosts_runs_in_pipeline() {
        let mut app = app_with_plugin();
        let _ = app.register_dmgable::<TestT>();

        let dealer = app
            .world_mut()
            .spawn({
                let mut s = DamageBoostStack::default();
                s.add_one_shot(2.0);
                s
            })
            .id();
        let target = app.world_mut().spawn((TestT, Hp::new(10.0))).id();

        app.world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .write(DamageDealt::<TestT> {
                dealer: Some(dealer),
                target,
                amount: 5.0,
                source: None,
                _marker: PhantomData,
            });
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(target).unwrap().current, 0.0);
        let killed_by = app
            .world()
            .get::<KilledBy>(target)
            .expect("KilledBy expected on killing blow");
        assert_eq!(killed_by.dealer, Some(dealer));
    }

    #[test]
    fn invulnerable_filter_zeroes_damage_within_apply_damage_set() {
        // Edge case 144a.
        let mut app = app_with_plugin();
        let _ = app.register_dmgable::<TestT>();

        let dealer = app
            .world_mut()
            .spawn({
                let mut s = DamageBoostStack::default();
                s.add_one_shot(100.0);
                s
            })
            .id();
        let target = app
            .world_mut()
            .spawn((TestT, Hp::new(10.0), Invulnerable))
            .id();

        app.world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .write(DamageDealt::<TestT> {
                dealer: Some(dealer),
                target,
                amount: 5.0,
                source: None,
                _marker: PhantomData,
            });
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(target).unwrap().current, 10.0);
    }

    // ── Behavior 145: apply_vulnerable::<TestT> attached to ApplyVulnerable ──

    #[test]
    fn apply_vulnerable_runs_in_pipeline() {
        let mut app = app_with_plugin();
        let _ = app.register_dmgable::<TestT>();

        let target = app
            .world_mut()
            .spawn((TestT, Hp::new(10.0), {
                let mut s = VulnerableStack::default();
                s.add_one_shot(2.0);
                s
            }))
            .id();

        app.world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .write(DamageDealt::<TestT> {
                dealer: None,
                target,
                amount: 5.0,
                source: None,
                _marker: PhantomData,
            });
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(target).unwrap().current, 0.0);
        assert!(app.world().get::<KilledBy>(target).is_some());
    }

    // ── Behavior 146: detect_deaths::<TestT> attached to EmitKill ──

    #[test]
    fn detect_deaths_runs_in_pipeline() {
        let mut app = app_with_plugin();
        let _ = app.register_dmgable::<TestT>();

        let victim = app
            .world_mut()
            .spawn((TestT, Hp::new(0.0), Position2D(Vec2::new(1.0, 2.0))))
            .id();

        tick(&mut app);

        // A nonempty Destroyed queue proves the full pipeline executed:
        // detect_deaths emitted KillYourself → handle_kill consumed it
        // and wrote Destroyed. (The victim itself is despawned in the
        // same tick by process_despawn_requests in FixedPostUpdate, so
        // the entity is no longer present to query for Dead.)
        let destroyed_count = app
            .world_mut()
            .resource_mut::<Messages<Destroyed<TestT>>>()
            .drain()
            .count();
        assert!(destroyed_count > 0);
        assert!(app.world().get_entity(victim).is_err());
    }

    // ── Behavior 147: handle_kill::<TestT> attached to ApplyKill ──

    #[test]
    fn handle_kill_runs_in_pipeline() {
        let mut app = app_with_plugin();
        let _ = app.register_dmgable::<TestT>();

        let victim = app
            .world_mut()
            .spawn((TestT, Position2D(Vec2::new(10.0, 20.0)), Hp::new(0.0)))
            .id();

        tick(&mut app);

        // The entity is despawned in FixedPostUpdate (same tick), so we
        // prove handle_kill ran by checking the Destroyed message it
        // emitted plus the post-despawn absence of the victim entity.
        let destroyed: Vec<Destroyed<TestT>> = app
            .world_mut()
            .resource_mut::<Messages<Destroyed<TestT>>>()
            .drain()
            .collect();
        assert_eq!(destroyed.len(), 1);
        assert_f32_eq(destroyed[0].victim_pos.x, 10.0);
        assert_f32_eq(destroyed[0].victim_pos.y, 20.0);
        assert!(app.world().get_entity(victim).is_err());
    }

    // ── Behavior 148: apply_heal::<TestT> attached to ApplyHeal ──

    #[test]
    fn apply_heal_runs_in_pipeline() {
        let mut app = app_with_plugin();
        let _ = app.register_dmgable::<TestT>();

        let target = app
            .world_mut()
            .spawn((
                TestT,
                Hp {
                    current:  5.0,
                    starting: 10.0,
                    max:      None,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Messages<HealDealt<TestT>>>()
            .write(HealDealt::<TestT> {
                healer: None,
                target,
                amount: 3.0,
                source: None,
                cap: HealCap::Max,
                _marker: PhantomData,
            });
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(target).unwrap().current, 8.0);
    }

    // ── Behavior 149: TestT and OtherT are independent ──

    #[test]
    fn test_t_and_other_t_pipelines_are_independent() {
        use std::marker::PhantomData;

        let mut app = app_with_plugin();
        let _ = app.register_dmgable::<TestT>().register_dmgable::<OtherT>();

        let target = app.world_mut().spawn((OtherT, Hp::new(10.0))).id();

        // Send message in TestT queue referencing an OtherT target — skipped.
        app.world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .write(DamageDealt::<TestT> {
                dealer: None,
                target,
                amount: 10.0,
                source: None,
                _marker: PhantomData,
            });
        // Send via OtherT queue — applied.
        app.world_mut()
            .resource_mut::<Messages<DamageDealt<OtherT>>>()
            .write(DamageDealt::<OtherT> {
                dealer: None,
                target,
                amount: 10.0,
                source: None,
                _marker: PhantomData,
            });
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(target).unwrap().current, 0.0);
    }

    // ── Behavior 150: end-to-end pipeline ──

    #[test]
    fn end_to_end_kill_and_despawn() {
        let mut app = app_with_plugin();
        let _ = app.register_dmgable::<TestT>();

        let victim = app
            .world_mut()
            .spawn((TestT, Hp::new(5.0), Position2D(Vec2::new(1.0, 2.0))))
            .id();
        let dealer = app.world_mut().spawn_empty().id();

        app.world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .write(DamageDealt::<TestT> {
                dealer:  Some(dealer),
                target:  victim,
                amount:  5.0,
                source:  None,
                _marker: PhantomData,
            });
        tick(&mut app);

        // Destroyed message drained and describes the victim.
        let destroyed: Vec<Destroyed<TestT>> = app
            .world_mut()
            .resource_mut::<Messages<Destroyed<TestT>>>()
            .drain()
            .collect();
        assert_eq!(destroyed.len(), 1);
        assert_eq!(destroyed[0].victim, victim);
        assert_eq!(destroyed[0].killer, Some(dealer));
        assert_f32_eq(destroyed[0].victim_pos.x, 1.0);
        assert_f32_eq(destroyed[0].victim_pos.y, 2.0);
        assert!(destroyed[0].killer_pos.is_none());

        // Entity actually despawned by process_despawn_requests in
        // FixedPostUpdate (same update() call).
        assert!(app.world().get_entity(victim).is_err());
    }

    #[test]
    fn end_to_end_kill_with_positioned_dealer_records_killer_pos() {
        // Edge case 150a.
        let mut app = app_with_plugin();
        let _ = app.register_dmgable::<TestT>();

        let victim = app
            .world_mut()
            .spawn((TestT, Hp::new(5.0), Position2D(Vec2::new(1.0, 2.0))))
            .id();
        let dealer = app
            .world_mut()
            .spawn(Position2D(Vec2::new(50.0, 60.0)))
            .id();

        app.world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .write(DamageDealt::<TestT> {
                dealer:  Some(dealer),
                target:  victim,
                amount:  5.0,
                source:  None,
                _marker: PhantomData,
            });
        tick(&mut app);

        let destroyed: Vec<Destroyed<TestT>> = app
            .world_mut()
            .resource_mut::<Messages<Destroyed<TestT>>>()
            .drain()
            .collect();
        assert_eq!(destroyed.len(), 1);
        let kp = destroyed[0].killer_pos.unwrap();
        assert_f32_eq(kp.x, 50.0);
        assert_f32_eq(kp.y, 60.0);
    }

    // Keep DespawnEntity import exercised — the end-to-end tests verify
    // despawn behavior indirectly; this assertion keeps the type visible.
    #[test]
    fn despawn_entity_message_exists_after_plugin_add() {
        let app = app_with_plugin();
        assert!(app.world().contains_resource::<Messages<DespawnEntity>>());
    }
}
