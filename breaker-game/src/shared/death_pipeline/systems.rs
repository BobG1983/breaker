//! Death pipeline systems — damage application, death detection, and despawn processing.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::{
    damage_dealt::DamageDealt, dead::Dead, despawn_entity::DespawnEntity, game_entity::GameEntity,
    hp::Hp, kill_yourself::KillYourself, killed_by::KilledBy,
};

/// Processes `DamageDealt<T>` messages, decrements `Hp`, and sets `KilledBy` on
/// the killing blow. Uses `Without<Dead>` to skip entities already confirmed dead.
///
/// Generic over the entity marker type — monomorphized for Cell, Bolt, Wall, Breaker.
/// Query for entities with HP and kill-tracking — excludes already-dead entities.
type DamageTargetQuery<'w, 's, T> =
    Query<'w, 's, (&'static mut Hp, &'static mut KilledBy), (With<T>, Without<Dead>)>;

pub(crate) fn apply_damage<T: GameEntity>(
    mut reader: MessageReader<DamageDealt<T>>,
    mut query: DamageTargetQuery<T>,
) {
    for msg in reader.read() {
        let Ok((mut hp, mut killed_by)) = query.get_mut(msg.target) else {
            continue;
        };

        let was_positive = hp.current > 0.0;
        hp.current -= msg.amount;

        // Killing blow: Hp crossed from positive to <= 0.
        // First kill wins — do not overwrite if already set.
        if was_positive && hp.current <= 0.0 && killed_by.dealer.is_none() {
            killed_by.dealer = msg.dealer;
        }
    }
}

/// Detects entities of type `T` with `Hp <= 0` and sends `KillYourself<T>`.
///
/// Uses `Without<Dead>` to skip entities already confirmed dead by their domain
/// kill handler, preventing double-processing.
/// Query for entities that might have died — HP and kill-tracking, excludes already-dead.
type DeathDetectionQuery<'w, 's, T> =
    Query<'w, 's, (Entity, &'static KilledBy, &'static Hp), (With<T>, Without<Dead>)>;

pub(crate) fn detect_deaths<T: GameEntity>(
    query: DeathDetectionQuery<T>,
    mut writer: MessageWriter<KillYourself<T>>,
) {
    for (entity, killed_by, hp) in &query {
        if hp.current <= 0.0 {
            writer.write(KillYourself {
                victim:  entity,
                killer:  killed_by.dealer,
                _marker: PhantomData,
            });
        }
    }
}

/// Processes `DespawnEntity` messages — despawns entities via `try_despawn`.
///
/// This is the ONLY system that despawns entities in the death pipeline.
/// Runs in `FixedPostUpdate`.
pub(crate) fn process_despawn_requests(
    mut reader: MessageReader<DespawnEntity>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        commands.entity(msg.entity).try_despawn();
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use bevy::prelude::*;

    use super::{apply_damage, detect_deaths, process_despawn_requests};
    use crate::shared::{
        death_pipeline::{
            damage_dealt::DamageDealt, dead::Dead, despawn_entity::DespawnEntity,
            game_entity::GameEntity, hp::Hp, kill_yourself::KillYourself, killed_by::KilledBy,
        },
        test_utils::{MessageCollector, TestAppBuilder, tick},
    };

    // ── Test-only entity type ────────────────────────────────────────────

    #[derive(Component, Default, Clone)]
    struct TestEntity;

    impl GameEntity for TestEntity {}

    // ── Test helpers ─────────────────────────────────────────────────────

    /// Resource to hold damage messages that should be enqueued before the system runs.
    #[derive(Resource, Default)]
    struct PendingDamage(Vec<DamageDealt<TestEntity>>);

    /// System that writes `DamageDealt<TestEntity>` from the `PendingDamage` resource.
    fn enqueue_damage(
        pending: Res<PendingDamage>,
        mut writer: MessageWriter<DamageDealt<TestEntity>>,
    ) {
        for msg in &pending.0 {
            writer.write(msg.clone());
        }
    }

    /// Resource to hold despawn messages that should be enqueued before the system runs.
    #[derive(Resource, Default)]
    struct PendingDespawns(Vec<DespawnEntity>);

    /// System that writes `DespawnEntity` from the `PendingDespawns` resource.
    fn enqueue_despawns(pending: Res<PendingDespawns>, mut writer: MessageWriter<DespawnEntity>) {
        for msg in &pending.0 {
            writer.write(msg.clone());
        }
    }

    fn build_apply_damage_app() -> App {
        TestAppBuilder::new()
            .with_message::<DamageDealt<TestEntity>>()
            .with_resource::<PendingDamage>()
            .with_system(
                FixedUpdate,
                enqueue_damage.before(apply_damage::<TestEntity>),
            )
            .with_system(FixedUpdate, apply_damage::<TestEntity>)
            .build()
    }

    fn build_detect_deaths_app() -> App {
        TestAppBuilder::new()
            .with_message_capture::<KillYourself<TestEntity>>()
            .with_system(FixedUpdate, detect_deaths::<TestEntity>)
            .build()
    }

    fn build_despawn_app() -> App {
        TestAppBuilder::new()
            .with_message::<DespawnEntity>()
            .with_resource::<PendingDespawns>()
            .with_system(
                FixedUpdate,
                enqueue_despawns.before(process_despawn_requests),
            )
            .with_system(FixedUpdate, process_despawn_requests)
            .build()
    }

    fn spawn_test_entity(app: &mut App, hp_value: f32) -> Entity {
        app.world_mut()
            .spawn((TestEntity, Hp::new(hp_value), KilledBy::default()))
            .id()
    }

    fn damage_msg(target: Entity, amount: f32, dealer: Option<Entity>) -> DamageDealt<TestEntity> {
        DamageDealt {
            dealer,
            target,
            amount,
            source_chip: None,
            _marker: PhantomData,
        }
    }

    // =====================================================================
    // apply_damage tests
    // =====================================================================

    #[test]
    fn apply_damage_reduces_hp() {
        let mut app = build_apply_damage_app();
        let entity = spawn_test_entity(&mut app, 30.0);

        app.insert_resource(PendingDamage(vec![damage_msg(entity, 10.0, None)]));
        tick(&mut app);

        let hp = app.world().get::<Hp>(entity).unwrap();
        assert!(
            (hp.current - 20.0).abs() < f32::EPSILON,
            "Hp should be 20.0 after 10 damage to 30-HP entity, got {}",
            hp.current
        );
    }

    #[test]
    fn apply_damage_inserts_dead_marker_is_not_its_job() {
        // apply_damage does NOT insert Dead — that's the domain kill handler's job.
        // This test confirms that after apply_damage reduces Hp to 0, Dead is NOT present.
        let mut app = build_apply_damage_app();
        let entity = spawn_test_entity(&mut app, 10.0);

        app.insert_resource(PendingDamage(vec![damage_msg(entity, 10.0, None)]));
        tick(&mut app);

        assert!(
            app.world().get::<Dead>(entity).is_none(),
            "apply_damage should NOT insert Dead — that is the domain kill handler's job"
        );
    }

    #[test]
    fn apply_damage_sets_killed_by_on_killing_blow() {
        let mut app = build_apply_damage_app();
        let entity = spawn_test_entity(&mut app, 10.0);
        let dealer = app.world_mut().spawn_empty().id();

        app.insert_resource(PendingDamage(vec![damage_msg(entity, 10.0, Some(dealer))]));
        tick(&mut app);

        let killed_by = app.world().get::<KilledBy>(entity).unwrap();
        assert_eq!(
            killed_by.dealer,
            Some(dealer),
            "KilledBy should record the dealer on the killing blow"
        );
    }

    #[test]
    fn apply_damage_sets_killed_by_when_dealer_is_none() {
        // Environmental death: dealer is None, but it's still the killing blow.
        let mut app = build_apply_damage_app();
        let entity = spawn_test_entity(&mut app, 10.0);

        app.insert_resource(PendingDamage(vec![damage_msg(entity, 10.0, None)]));
        tick(&mut app);

        let killed_by = app.world().get::<KilledBy>(entity).unwrap();
        assert_eq!(
            killed_by.dealer, None,
            "KilledBy.dealer should remain None for environmental kills"
        );
    }

    #[test]
    fn apply_damage_does_not_set_killed_by_when_hp_stays_positive() {
        let mut app = build_apply_damage_app();
        let entity = spawn_test_entity(&mut app, 30.0);
        let dealer = app.world_mut().spawn_empty().id();

        app.insert_resource(PendingDamage(vec![damage_msg(entity, 10.0, Some(dealer))]));
        tick(&mut app);

        let killed_by = app.world().get::<KilledBy>(entity).unwrap();
        assert_eq!(
            killed_by.dealer, None,
            "KilledBy should not be set when Hp is still positive"
        );
    }

    #[test]
    fn apply_damage_skips_entity_already_dead() {
        let mut app = build_apply_damage_app();
        let entity = app
            .world_mut()
            .spawn((TestEntity, Hp::new(10.0), KilledBy::default(), Dead))
            .id();

        app.insert_resource(PendingDamage(vec![damage_msg(entity, 5.0, None)]));
        tick(&mut app);

        let hp = app.world().get::<Hp>(entity).unwrap();
        assert!(
            (hp.current - 10.0).abs() < f32::EPSILON,
            "Dead entity's Hp should remain unchanged at 10.0, got {}",
            hp.current
        );
    }

    #[test]
    fn apply_damage_skips_entity_without_hp() {
        // Entity with TestEntity but no Hp — system should silently skip.
        let mut app = build_apply_damage_app();
        let entity = app.world_mut().spawn(TestEntity).id();

        app.insert_resource(PendingDamage(vec![damage_msg(entity, 5.0, None)]));
        // Should not panic
        tick(&mut app);

        assert!(
            app.world().get::<Hp>(entity).is_none(),
            "Entity without Hp should remain without Hp"
        );
    }

    #[test]
    fn apply_damage_first_kill_wins() {
        // Two damage messages in the same frame both cross Hp to <= 0.
        // First kill wins — KilledBy records the first dealer.
        let mut app = build_apply_damage_app();
        let entity = spawn_test_entity(&mut app, 10.0);
        let dealer_a = app.world_mut().spawn_empty().id();
        let dealer_b = app.world_mut().spawn_empty().id();

        app.insert_resource(PendingDamage(vec![
            damage_msg(entity, 10.0, Some(dealer_a)),
            damage_msg(entity, 5.0, Some(dealer_b)),
        ]));
        tick(&mut app);

        let killed_by = app.world().get::<KilledBy>(entity).unwrap();
        assert_eq!(
            killed_by.dealer,
            Some(dealer_a),
            "First kill should win — dealer_a dealt the killing blow"
        );
    }

    #[test]
    fn apply_damage_multiple_messages_accumulate() {
        let mut app = build_apply_damage_app();
        let entity = spawn_test_entity(&mut app, 30.0);

        app.insert_resource(PendingDamage(vec![
            damage_msg(entity, 10.0, None),
            damage_msg(entity, 8.0, None),
        ]));
        tick(&mut app);

        let hp = app.world().get::<Hp>(entity).unwrap();
        assert!(
            (hp.current - 12.0).abs() < f32::EPSILON,
            "Hp should be 12.0 after 10+8 damage to 30-HP entity, got {}",
            hp.current
        );
    }

    #[test]
    fn apply_damage_overkill_sets_negative_hp() {
        let mut app = build_apply_damage_app();
        let entity = spawn_test_entity(&mut app, 10.0);

        app.insert_resource(PendingDamage(vec![damage_msg(entity, 25.0, None)]));
        tick(&mut app);

        let hp = app.world().get::<Hp>(entity).unwrap();
        assert!(
            hp.current < 0.0,
            "Hp should go negative on overkill, got {}",
            hp.current
        );
        assert!(
            (hp.current - (-15.0)).abs() < f32::EPSILON,
            "Hp should be -15.0 after 25 damage to 10-HP entity, got {}",
            hp.current
        );
    }

    // =====================================================================
    // detect_deaths tests
    // =====================================================================

    #[test]
    fn detect_deaths_sends_kill_yourself_when_hp_zero() {
        let mut app = build_detect_deaths_app();
        let entity = app
            .world_mut()
            .spawn((
                TestEntity,
                Hp {
                    current:  0.0,
                    starting: 10.0,
                    max:      None,
                },
                KilledBy::default(),
            ))
            .id();

        tick(&mut app);

        let collector = app
            .world()
            .resource::<MessageCollector<KillYourself<TestEntity>>>();
        assert_eq!(
            collector.0.len(),
            1,
            "detect_deaths should send exactly one KillYourself message"
        );
        assert_eq!(
            collector.0[0].victim, entity,
            "KillYourself victim should be the entity with Hp <= 0"
        );
    }

    #[test]
    fn detect_deaths_sends_kill_yourself_when_hp_negative() {
        let mut app = build_detect_deaths_app();
        let entity = app
            .world_mut()
            .spawn((
                TestEntity,
                Hp {
                    current:  -5.0,
                    starting: 10.0,
                    max:      None,
                },
                KilledBy::default(),
            ))
            .id();

        tick(&mut app);

        let collector = app
            .world()
            .resource::<MessageCollector<KillYourself<TestEntity>>>();
        assert_eq!(collector.0.len(), 1);
        assert_eq!(collector.0[0].victim, entity);
    }

    #[test]
    fn detect_deaths_does_not_send_for_positive_hp() {
        let mut app = build_detect_deaths_app();
        app.world_mut()
            .spawn((TestEntity, Hp::new(10.0), KilledBy::default()));

        tick(&mut app);

        let collector = app
            .world()
            .resource::<MessageCollector<KillYourself<TestEntity>>>();
        assert!(
            collector.0.is_empty(),
            "detect_deaths should not send KillYourself for entities with positive Hp"
        );
    }

    #[test]
    fn detect_deaths_skips_dead_entities() {
        let mut app = build_detect_deaths_app();
        app.world_mut().spawn((
            TestEntity,
            Hp {
                current:  0.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
            Dead,
        ));

        tick(&mut app);

        let collector = app
            .world()
            .resource::<MessageCollector<KillYourself<TestEntity>>>();
        assert!(
            collector.0.is_empty(),
            "detect_deaths should skip entities with Dead marker"
        );
    }

    #[test]
    fn detect_deaths_includes_killer_from_killed_by() {
        let mut app = build_detect_deaths_app();
        let dealer = app.world_mut().spawn_empty().id();
        app.world_mut().spawn((
            TestEntity,
            Hp {
                current:  0.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy {
                dealer: Some(dealer),
            },
        ));

        tick(&mut app);

        let collector = app
            .world()
            .resource::<MessageCollector<KillYourself<TestEntity>>>();
        assert_eq!(collector.0.len(), 1);
        assert_eq!(
            collector.0[0].killer,
            Some(dealer),
            "KillYourself.killer should carry the dealer from KilledBy"
        );
    }

    // =====================================================================
    // process_despawn_requests tests
    // =====================================================================

    #[test]
    fn process_despawn_requests_despawns_entity() {
        let mut app = build_despawn_app();
        let entity = app.world_mut().spawn_empty().id();

        app.insert_resource(PendingDespawns(vec![DespawnEntity { entity }]));
        tick(&mut app);

        assert!(
            app.world().get_entity(entity).is_err(),
            "Entity should be despawned after process_despawn_requests"
        );
    }

    #[test]
    fn process_despawn_requests_handles_already_despawned() {
        // try_despawn should not panic if entity is already gone.
        let mut app = build_despawn_app();
        let entity = app.world_mut().spawn_empty().id();
        app.world_mut().despawn(entity);

        app.insert_resource(PendingDespawns(vec![DespawnEntity { entity }]));
        // Should not panic
        tick(&mut app);
    }

    #[test]
    fn process_despawn_requests_handles_multiple() {
        let mut app = build_despawn_app();
        let entity_a = app.world_mut().spawn_empty().id();
        let entity_b = app.world_mut().spawn_empty().id();

        app.insert_resource(PendingDespawns(vec![
            DespawnEntity { entity: entity_a },
            DespawnEntity { entity: entity_b },
        ]));
        tick(&mut app);

        assert!(
            app.world().get_entity(entity_a).is_err(),
            "Entity A should be despawned"
        );
        assert!(
            app.world().get_entity(entity_b).is_err(),
            "Entity B should be despawned"
        );
    }

    #[test]
    fn process_despawn_requests_duplicate_entity_does_not_panic() {
        // Same entity in two messages — try_despawn handles the second gracefully.
        let mut app = build_despawn_app();
        let entity = app.world_mut().spawn_empty().id();

        app.insert_resource(PendingDespawns(vec![
            DespawnEntity { entity },
            DespawnEntity { entity },
        ]));
        // Should not panic
        tick(&mut app);

        assert!(
            app.world().get_entity(entity).is_err(),
            "Entity should be despawned"
        );
    }
}
