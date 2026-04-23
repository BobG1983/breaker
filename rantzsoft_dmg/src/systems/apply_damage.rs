//! `apply_damage::<T>` — applies final damage to HP and attributes kills.
//!
//! Reads `DamageDealt<T>` messages (amounts already mutated by upstream
//! stages), decrements `Hp::current`, and — on the killing blow — inserts
//! `KilledBy { dealer }` via `Commands`. Runs in `DmgSystems::ApplyDamage`
//! inside `FixedUpdate`, chained strictly after `invulnerable_filter::<T>`.

use bevy::prelude::*;

use crate::{
    components::{Dead, Hp, KilledBy},
    messages::DamageDealt,
    traits::Dmgable,
};

/// Decrement HP by the final message amount. Issue `KilledBy` insertion
/// on the killing-blow message (the one that crosses `hp.current` from
/// positive to `<= 0.0`). First-kill-wins is intrinsic: subsequent
/// same-tick messages against the same target observe
/// `was_positive = false` because `hp.current` is already non-positive.
///
/// Targets that are already `Dead`, or do not carry the `T` marker, are
/// silently skipped via the query's `With<T>` + `Without<Dead>` filter.
/// Targets missing `Hp` are likewise skipped (query miss).
///
/// **Invulnerability is NOT filtered here.** The query shape deliberately
/// omits `Without<Invulnerable>` — upstream `invulnerable_filter::<T>`
/// (chained earlier in the same `DmgSystems::ApplyDamage` set) zeroes the
/// message amount for invulnerable targets before `apply_damage` sees it.
/// Do not add an `Invulnerable` filter to this query: the B153 contract
/// test pins the current shape.
pub(crate) fn apply_damage<T: Dmgable>(
    mut reader: MessageReader<DamageDealt<T>>,
    mut targets: Query<&mut Hp, (With<T>, Without<Dead>)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let Ok(mut hp) = targets.get_mut(msg.target) else {
            continue;
        };
        let was_positive = hp.current > 0.0;
        hp.current -= msg.amount;
        if was_positive && hp.current <= 0.0 {
            commands
                .entity(msg.target)
                .insert(KilledBy { dealer: msg.dealer });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use bevy::prelude::*;

    use super::*;
    use crate::{
        components::{Dead, Hp, Invulnerable, KilledBy},
        messages::DamageDealt,
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

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<DamageDealt<TestT>>();
        app.add_systems(FixedUpdate, apply_damage::<TestT>);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn enqueue(app: &mut App, msg: DamageDealt<TestT>) {
        app.world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .write(msg);
    }

    fn mk_msg(dealer: Option<Entity>, target: Entity, amount: f32) -> DamageDealt<TestT> {
        DamageDealt::<TestT> {
            dealer,
            target,
            amount,
            source: None,
            _marker: PhantomData,
        }
    }

    // ── Behavior 96: Hp decrements by msg.amount ──

    #[test]
    fn hp_decrements_by_msg_amount() {
        let mut app = test_app();
        let e = app.world_mut().spawn((TestT, Hp::new(30.0))).id();

        enqueue(&mut app, mk_msg(None, e, 10.0));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 20.0);
        assert!(app.world().get::<Dead>(e).is_none());
        assert!(app.world().get::<KilledBy>(e).is_none());
    }

    #[test]
    fn zero_amount_leaves_hp_unchanged_and_no_killed_by() {
        // Edge case 96a.
        let mut app = test_app();
        let e = app.world_mut().spawn((TestT, Hp::new(30.0))).id();

        enqueue(&mut app, mk_msg(None, e, 0.0));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 30.0);
        assert!(app.world().get::<KilledBy>(e).is_none());
    }

    // ── Behavior 97: killing blow inserts KilledBy { dealer: Some(e) } ──

    #[test]
    fn killing_blow_inserts_killed_by_with_dealer() {
        let mut app = test_app();
        let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();
        let dealer = app.world_mut().spawn_empty().id();

        enqueue(&mut app, mk_msg(Some(dealer), e, 10.0));
        tick(&mut app);

        let killed_by = app
            .world()
            .get::<KilledBy>(e)
            .expect("KilledBy should be inserted on killing blow");
        assert_eq!(killed_by.dealer, Some(dealer));
        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 0.0);
    }

    #[test]
    fn overkill_inserts_killed_by_and_leaves_negative_hp() {
        // Edge case 97a.
        let mut app = test_app();
        let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();
        let dealer = app.world_mut().spawn_empty().id();

        enqueue(&mut app, mk_msg(Some(dealer), e, 25.0));
        tick(&mut app);

        let killed_by = app.world().get::<KilledBy>(e).unwrap();
        assert_eq!(killed_by.dealer, Some(dealer));
        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, -15.0);
    }

    #[test]
    fn environmental_kill_inserts_killed_by_with_none_dealer() {
        // Edge case 97b.
        let mut app = test_app();
        let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();

        enqueue(&mut app, mk_msg(None, e, 10.0));
        tick(&mut app);

        let killed_by = app.world().get::<KilledBy>(e).unwrap();
        assert!(killed_by.dealer.is_none());
    }

    // ── Behavior 98: non-killing hit does NOT insert KilledBy ──

    #[test]
    fn non_killing_hit_does_not_insert_killed_by() {
        let mut app = test_app();
        let e = app.world_mut().spawn((TestT, Hp::new(30.0))).id();
        let dealer = app.world_mut().spawn_empty().id();

        enqueue(&mut app, mk_msg(Some(dealer), e, 10.0));
        tick(&mut app);

        assert!(app.world().get::<KilledBy>(e).is_none());
    }

    #[test]
    fn second_non_killing_hit_still_no_killed_by() {
        // Edge case 98a.
        let mut app = test_app();
        let e = app.world_mut().spawn((TestT, Hp::new(30.0))).id();
        let dealer = app.world_mut().spawn_empty().id();

        enqueue(&mut app, mk_msg(Some(dealer), e, 10.0));
        tick(&mut app);
        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 20.0);

        enqueue(&mut app, mk_msg(Some(dealer), e, 5.0));
        tick(&mut app);
        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 15.0);
        assert!(app.world().get::<KilledBy>(e).is_none());
    }

    // ── Behavior 99: entity with Dead marker is skipped ──

    #[test]
    fn dead_entity_is_skipped() {
        let mut app = test_app();
        let e = app.world_mut().spawn((TestT, Hp::new(10.0), Dead)).id();

        enqueue(&mut app, mk_msg(None, e, 5.0));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
        assert!(app.world().get::<KilledBy>(e).is_none());
    }

    #[test]
    fn dead_entity_with_low_hp_is_still_skipped() {
        // Edge case 99a.
        let mut app = test_app();
        let e = app.world_mut().spawn((TestT, Hp::new(5.0), Dead)).id();

        enqueue(&mut app, mk_msg(None, e, 10.0));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 5.0);
    }

    // ── Behavior 100: entity without Hp is silently skipped ──

    #[test]
    fn entity_without_hp_is_skipped() {
        let mut app = test_app();
        let e = app.world_mut().spawn(TestT).id();

        enqueue(&mut app, mk_msg(None, e, 5.0));
        tick(&mut app);

        assert!(app.world().get::<Hp>(e).is_none());
        assert!(app.world().get::<KilledBy>(e).is_none());
    }

    // ── Behavior 101: entity without T marker is skipped ──

    #[test]
    fn entity_without_t_marker_is_skipped() {
        let mut app = test_app();
        let e = app.world_mut().spawn((OtherT, Hp::new(10.0))).id();

        enqueue(&mut app, mk_msg(None, e, 10.0));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, 10.0);
        assert!(app.world().get::<KilledBy>(e).is_none());
    }

    // ── Behavior 102: first-kill-wins ──

    #[test]
    fn first_kill_wins_with_two_messages_same_tick() {
        let mut app = test_app();
        let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();
        let dealer_a = app.world_mut().spawn_empty().id();
        let dealer_b = app.world_mut().spawn_empty().id();

        enqueue(&mut app, mk_msg(Some(dealer_a), e, 10.0));
        enqueue(&mut app, mk_msg(Some(dealer_b), e, 5.0));
        tick(&mut app);

        let killed_by = app.world().get::<KilledBy>(e).unwrap();
        assert_eq!(killed_by.dealer, Some(dealer_a));
    }

    #[test]
    fn first_kill_wins_with_three_messages_same_tick() {
        // Edge case 102a.
        let mut app = test_app();
        let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();
        let a = app.world_mut().spawn_empty().id();
        let b = app.world_mut().spawn_empty().id();
        let c = app.world_mut().spawn_empty().id();

        enqueue(&mut app, mk_msg(Some(a), e, 10.0));
        enqueue(&mut app, mk_msg(Some(b), e, 5.0));
        enqueue(&mut app, mk_msg(Some(c), e, 2.0));
        tick(&mut app);

        let killed_by = app.world().get::<KilledBy>(e).unwrap();
        assert_eq!(killed_by.dealer, Some(a));
        assert_f32_eq(app.world().get::<Hp>(e).unwrap().current, -7.0);
    }

    #[test]
    fn first_kill_wins_when_first_is_non_killing() {
        // Edge case 102b.
        let mut app = test_app();
        let e = app.world_mut().spawn((TestT, Hp::new(10.0))).id();
        let a = app.world_mut().spawn_empty().id();
        let b = app.world_mut().spawn_empty().id();
        let c = app.world_mut().spawn_empty().id();

        enqueue(&mut app, mk_msg(Some(a), e, 3.0));
        enqueue(&mut app, mk_msg(Some(b), e, 7.0));
        enqueue(&mut app, mk_msg(Some(c), e, 5.0));
        tick(&mut app);

        let killed_by = app.world().get::<KilledBy>(e).unwrap();
        assert_eq!(killed_by.dealer, Some(b));
    }

    // ── Behavior 103: dealer copied verbatim ──

    #[test]
    fn killing_blow_copies_dealer_verbatim() {
        let mut app = test_app();
        let e = app.world_mut().spawn((TestT, Hp::new(1.0))).id();
        let dealer = app.world_mut().spawn_empty().id();

        enqueue(&mut app, mk_msg(Some(dealer), e, 1.0));
        tick(&mut app);

        let killed_by = app.world().get::<KilledBy>(e).unwrap();
        assert_eq!(killed_by.dealer, Some(dealer));
    }

    #[test]
    fn killing_blow_preserves_none_dealer() {
        // Edge case 103a.
        let mut app = test_app();
        let e = app.world_mut().spawn((TestT, Hp::new(1.0))).id();

        enqueue(&mut app, mk_msg(None, e, 1.0));
        tick(&mut app);

        let killed_by = app.world().get::<KilledBy>(e).unwrap();
        assert!(killed_by.dealer.is_none());
    }

    // ── Behavior 153: apply_damage's query does NOT filter Without<Invulnerable>.
    //     Tested in isolation (no invulnerable_filter upstream) ──

    #[test]
    fn apply_damage_applies_to_invulnerable_when_filter_bypassed() {
        // Bare app, no plugin, no register_dmgable — only apply_damage::<TestT>.
        let mut app = test_app();
        let victim = app
            .world_mut()
            .spawn((TestT, Hp::new(10.0), Invulnerable))
            .id();
        let dealer = app.world_mut().spawn_empty().id();

        enqueue(&mut app, mk_msg(Some(dealer), victim, 3.0));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 7.0);
        assert!(app.world().get::<KilledBy>(victim).is_none());
    }

    #[test]
    fn apply_damage_inserts_killed_by_on_invulnerable_when_filter_bypassed() {
        // Edge case 153a.
        let mut app = test_app();
        let victim = app
            .world_mut()
            .spawn((TestT, Hp::new(5.0), Invulnerable))
            .id();
        let dealer = app.world_mut().spawn_empty().id();

        enqueue(&mut app, mk_msg(Some(dealer), victim, 5.0));
        tick(&mut app);

        assert_f32_eq(app.world().get::<Hp>(victim).unwrap().current, 0.0);
        let killed_by = app.world().get::<KilledBy>(victim).unwrap();
        assert_eq!(killed_by.dealer, Some(dealer));
    }
}
