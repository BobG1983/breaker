//! `invulnerable_filter::<T>` — zeros damage amount when the target is
//! `Invulnerable`.
//!
//! Reads `DamageDealt<T>` messages; if `msg.target` carries the
//! `Invulnerable` marker, sets `msg.amount = 0.0`. Runs inside
//! `DmgSystems::ApplyDamage` strictly before `apply_damage::<T>` via
//! `.chain()`.

use bevy::prelude::*;

use crate::{components::Invulnerable, messages::DamageDealt, traits::Dmgable};

/// Zero `msg.amount` for any `DamageDealt<T>` whose target carries
/// `Invulnerable`. Runs strictly before `apply_damage::<T>` within
/// `DmgSystems::ApplyDamage`.
pub(crate) fn invulnerable_filter<T: Dmgable>(
    mut reader: MessageMutator<DamageDealt<T>>,
    invuln: Query<(), With<Invulnerable>>,
) {
    for msg in reader.read() {
        if invuln.get(msg.target).is_ok() {
            msg.amount = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use bevy::prelude::*;

    use super::*;
    use crate::{components::Invulnerable, messages::DamageDealt};

    #[derive(Component)]
    struct TestT;
    impl Dmgable for TestT {}

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
        app.add_systems(FixedUpdate, invulnerable_filter::<TestT>);
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

    fn drain_messages(app: &mut App) -> Vec<DamageDealt<TestT>> {
        app.world_mut()
            .resource_mut::<Messages<DamageDealt<TestT>>>()
            .drain()
            .collect()
    }

    fn mk_msg(target: Entity, amount: f32) -> DamageDealt<TestT> {
        DamageDealt::<TestT> {
            dealer: None,
            attributed_to: None,
            target,
            amount,
            source: None,
            _marker: PhantomData,
        }
    }

    // ── Behavior 92: Invulnerable target → amount set to 0.0 ──

    #[test]
    fn invulnerable_target_zeroes_amount() {
        let mut app = test_app();
        let target = app.world_mut().spawn((TestT, Invulnerable)).id();

        enqueue(&mut app, mk_msg(target, 1000.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 0.0);
    }

    #[test]
    fn invulnerable_target_zeroes_negative_amount() {
        // Edge case 92a: prior negative amount is still set to 0.0.
        let mut app = test_app();
        let target = app.world_mut().spawn((TestT, Invulnerable)).id();

        enqueue(&mut app, mk_msg(target, -5.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 0.0);
    }

    #[test]
    fn invulnerable_target_zeroes_infinity_amount_no_nan() {
        // Edge case 92b: f32::INFINITY → 0.0 (no NaN).
        let mut app = test_app();
        let target = app.world_mut().spawn((TestT, Invulnerable)).id();

        enqueue(&mut app, mk_msg(target, f32::INFINITY));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 0.0);
        assert!(!drained[0].amount.is_nan());
    }

    #[test]
    fn invulnerable_target_zeroes_already_zero_amount() {
        // Edge case 92c.
        let mut app = test_app();
        let target = app.world_mut().spawn((TestT, Invulnerable)).id();

        enqueue(&mut app, mk_msg(target, 0.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 0.0);
    }

    // ── Behavior 93: non-Invulnerable target is unchanged ──

    #[test]
    fn non_invulnerable_target_passes_through() {
        let mut app = test_app();
        let target = app.world_mut().spawn(TestT).id();

        enqueue(&mut app, mk_msg(target, 7.5));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 7.5);
    }

    #[test]
    fn invulnerable_without_testt_still_filtered() {
        // Edge case 93a: filter keys on With<Invulnerable> alone (not on T).
        let mut app = test_app();
        let target = app.world_mut().spawn(Invulnerable).id();

        enqueue(&mut app, mk_msg(target, 7.5));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 0.0);
    }

    // ── Behavior 94: mixed batch ──

    #[test]
    fn mixed_batch_filters_correctly() {
        let mut app = test_app();
        let inv = app.world_mut().spawn((TestT, Invulnerable)).id();
        let vuln = app.world_mut().spawn(TestT).id();

        enqueue(&mut app, mk_msg(inv, 5.0));
        enqueue(&mut app, mk_msg(vuln, 5.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 2);
        assert_f32_eq(drained[0].amount, 0.0);
        assert_f32_eq(drained[1].amount, 5.0);
    }

    #[test]
    fn mixed_batch_reversed_order_still_filters_correctly() {
        // Edge case 94a.
        let mut app = test_app();
        let inv = app.world_mut().spawn((TestT, Invulnerable)).id();
        let vuln = app.world_mut().spawn(TestT).id();

        enqueue(&mut app, mk_msg(vuln, 5.0));
        enqueue(&mut app, mk_msg(inv, 5.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 2);
        assert_f32_eq(drained[0].amount, 5.0);
        assert_f32_eq(drained[1].amount, 0.0);
    }

    // ── Behavior 95: despawned target is pass-through ──

    #[test]
    fn despawned_target_passes_through() {
        let mut app = test_app();
        let target = app.world_mut().spawn((TestT, Invulnerable)).id();
        app.world_mut().despawn(target);

        enqueue(&mut app, mk_msg(target, 9.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 9.0);
    }
}
