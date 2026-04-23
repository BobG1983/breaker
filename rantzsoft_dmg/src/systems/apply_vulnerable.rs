//! `apply_vulnerable::<T>` — target-side vulnerability multiplier stage.
//!
//! Reads `DamageDealt<T>` messages, looks up the target's
//! `VulnerableStack`, and multiplies `msg.amount` by BOTH the persistent-
//! lane aggregate AND the one-shot-lane aggregate (which drains the
//! one-shots as a side effect). Runs in `DmgSystems::ApplyVulnerable`
//! inside `FixedUpdate`.

use bevy::prelude::*;

use crate::{components::VulnerableStack, messages::DamageDealt, traits::Dmgable};

/// For each pending `DamageDealt<T>`, multiply `msg.amount` by the
/// target's `aggregate_persistent() * aggregate_and_consume_one_shots()`.
/// Messages whose target has no stack component are passed through
/// unchanged.
pub(crate) fn apply_vulnerable<T: Dmgable>(
    mut reader: MessageMutator<DamageDealt<T>>,
    mut stacks: Query<&mut VulnerableStack>,
) {
    for msg in reader.read() {
        if let Ok(mut stack) = stacks.get_mut(msg.target) {
            msg.amount *= stack.aggregate_persistent() * stack.aggregate_and_consume_one_shots();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use bevy::prelude::*;

    use super::*;
    use crate::{
        SourceId,
        components::{DamageBoostStack, VulnerableStack},
        messages::DamageDealt,
    };

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
        app.add_systems(FixedUpdate, apply_vulnerable::<TestT>);
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

    fn mk_msg(dealer: Option<Entity>, target: Entity, amount: f32) -> DamageDealt<TestT> {
        DamageDealt::<TestT> {
            dealer,
            target,
            amount,
            source: None,
            _marker: PhantomData,
        }
    }

    // ── Behavior 87: persistent-only VulnerableStack multiplies by aggregate ──

    #[test]
    fn persistent_only_multiplies_and_retains() {
        let mut app = test_app();
        let target = app
            .world_mut()
            .spawn({
                let mut s = VulnerableStack::default();
                s.add(SourceId::from("src:a"), 2.0);
                s
            })
            .id();

        enqueue(&mut app, mk_msg(None, target, 4.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 8.0);

        let stack = app.world().get::<VulnerableStack>(target).unwrap();
        assert_f32_eq(stack.aggregate_persistent(), 2.0);
    }

    #[test]
    fn persistent_multi_entry_multiplies_by_product() {
        // Edge case 87a.
        let mut app = test_app();
        let target = app
            .world_mut()
            .spawn({
                let mut s = VulnerableStack::default();
                s.add(SourceId::from("src:a"), 2.0);
                s.add(SourceId::from("src:b"), 3.0);
                s
            })
            .id();

        enqueue(&mut app, mk_msg(None, target, 1.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 6.0);
    }

    #[test]
    fn persistent_applies_to_each_of_two_messages_same_tick() {
        // Edge case 87b.
        let mut app = test_app();
        let target = app
            .world_mut()
            .spawn({
                let mut s = VulnerableStack::default();
                s.add(SourceId::from("src:a"), 2.0);
                s
            })
            .id();

        enqueue(&mut app, mk_msg(None, target, 4.0));
        enqueue(&mut app, mk_msg(None, target, 4.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 2);
        assert_f32_eq(drained[0].amount, 8.0);
        assert_f32_eq(drained[1].amount, 8.0);
    }

    // ── Behavior 88: one-shot-only multiplies and drains ──

    #[test]
    fn one_shot_only_multiplies_and_drains() {
        let mut app = test_app();
        let target = app
            .world_mut()
            .spawn({
                let mut s = VulnerableStack::default();
                s.add_one_shot(2.5);
                s
            })
            .id();

        enqueue(&mut app, mk_msg(None, target, 4.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 10.0);

        let mut stack = app.world_mut().get_mut::<VulnerableStack>(target).unwrap();
        assert_f32_eq(stack.aggregate_and_consume_one_shots(), 1.0);
    }

    #[test]
    fn one_shot_multi_entry_multiplies_by_product() {
        // Edge case 88a.
        let mut app = test_app();
        let target = app
            .world_mut()
            .spawn({
                let mut s = VulnerableStack::default();
                s.add_one_shot(2.0);
                s.add_one_shot(3.0);
                s
            })
            .id();

        enqueue(&mut app, mk_msg(None, target, 1.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 6.0);
    }

    #[test]
    fn one_shot_drained_by_first_message_is_identity_for_second_same_tick() {
        // Edge case 88b.
        let mut app = test_app();
        let target = app
            .world_mut()
            .spawn({
                let mut s = VulnerableStack::default();
                s.add_one_shot(2.0);
                s
            })
            .id();

        enqueue(&mut app, mk_msg(None, target, 5.0));
        enqueue(&mut app, mk_msg(None, target, 5.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 2);
        assert_f32_eq(drained[0].amount, 10.0);
        assert_f32_eq(drained[1].amount, 5.0);
    }

    // ── Behavior 89: both lanes — product, one-shots drain, persistent stays ──

    #[test]
    fn both_lanes_populated_multiplies_by_product() {
        let mut app = test_app();
        let target = app
            .world_mut()
            .spawn({
                let mut s = VulnerableStack::default();
                s.add(SourceId::from("src:x"), 2.0);
                s.add_one_shot(3.0);
                s
            })
            .id();

        enqueue(&mut app, mk_msg(None, target, 10.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 60.0);

        let mut stack = app.world_mut().get_mut::<VulnerableStack>(target).unwrap();
        assert_f32_eq(stack.aggregate_persistent(), 2.0);
        assert_f32_eq(stack.aggregate_and_consume_one_shots(), 1.0);
    }

    #[test]
    fn both_lanes_multi_entry_multiplies_by_product_of_products() {
        // Edge case 89a.
        let mut app = test_app();
        let target = app
            .world_mut()
            .spawn({
                let mut s = VulnerableStack::default();
                s.add(SourceId::from("src:a"), 2.0);
                s.add(SourceId::from("src:b"), 5.0);
                s.add_one_shot(3.0);
                s.add_one_shot(4.0);
                s
            })
            .id();

        enqueue(&mut app, mk_msg(None, target, 1.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 120.0);

        let stack = app.world().get::<VulnerableStack>(target).unwrap();
        assert_f32_eq(stack.aggregate_persistent(), 10.0);
    }

    #[test]
    fn both_lanes_two_messages_only_first_gets_one_shots() {
        // Edge case 89b.
        let mut app = test_app();
        let target = app
            .world_mut()
            .spawn({
                let mut s = VulnerableStack::default();
                s.add(SourceId::from("src:a"), 2.0);
                s.add_one_shot(3.0);
                s
            })
            .id();

        enqueue(&mut app, mk_msg(None, target, 10.0));
        enqueue(&mut app, mk_msg(None, target, 10.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 2);
        assert_f32_eq(drained[0].amount, 60.0);
        assert_f32_eq(drained[1].amount, 20.0);
    }

    // ── Behavior 90: empty stack is identity; no stack pass-through; despawned
    //     target pass-through; multiple messages on empty stack identity ──

    #[test]
    fn empty_stack_is_identity() {
        let mut app = test_app();
        let target = app.world_mut().spawn(VulnerableStack::default()).id();

        enqueue(&mut app, mk_msg(None, target, 7.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 7.0);
    }

    #[test]
    fn target_without_stack_passes_through() {
        // Edge case 90a.
        let mut app = test_app();
        let target = app.world_mut().spawn_empty().id();

        enqueue(&mut app, mk_msg(None, target, 5.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 5.0);
    }

    #[test]
    fn despawned_target_passes_through() {
        // Edge case 90b.
        let mut app = test_app();
        let target = app.world_mut().spawn_empty().id();
        app.world_mut().despawn(target);

        enqueue(&mut app, mk_msg(None, target, 9.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 9.0);
    }

    #[test]
    fn multiple_messages_empty_stack_identity() {
        // Edge case 90c.
        let mut app = test_app();
        let target = app.world_mut().spawn(VulnerableStack::default()).id();

        enqueue(&mut app, mk_msg(None, target, 7.0));
        enqueue(&mut app, mk_msg(None, target, 3.5));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 2);
        assert_f32_eq(drained[0].amount, 7.0);
        assert_f32_eq(drained[1].amount, 3.5);
    }

    // ── Behavior 91: dealer's boost stack does NOT apply here; target keys
    //     this system ──

    #[test]
    fn dealer_damage_boost_stack_is_ignored() {
        let mut app = test_app();
        let dealer = app
            .world_mut()
            .spawn({
                let mut s = DamageBoostStack::default();
                s.add(SourceId::from("src:a"), 10.0);
                s.add_one_shot(10.0);
                s
            })
            .id();
        let target = app.world_mut().spawn_empty().id();

        enqueue(&mut app, mk_msg(Some(dealer), target, 3.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 3.0);

        // Dealer's stack untouched.
        let mut dealer_stack = app.world_mut().get_mut::<DamageBoostStack>(dealer).unwrap();
        assert_f32_eq(dealer_stack.aggregate_persistent(), 10.0);
        assert_f32_eq(dealer_stack.aggregate_and_consume_one_shots(), 10.0);
    }

    #[test]
    fn target_persistent_only_multiplies_when_dealer_has_no_boost_stack() {
        // Edge case 91a.
        let mut app = test_app();
        let dealer = app.world_mut().spawn_empty().id();
        let target = app
            .world_mut()
            .spawn({
                let mut s = VulnerableStack::default();
                s.add(SourceId::from("src:a"), 10.0);
                s
            })
            .id();

        enqueue(&mut app, mk_msg(Some(dealer), target, 3.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 30.0);
    }

    #[test]
    fn target_one_shot_only_multiplies_when_dealer_has_no_boost_stack() {
        // Edge case 91b.
        let mut app = test_app();
        let dealer = app.world_mut().spawn_empty().id();
        let target = app
            .world_mut()
            .spawn({
                let mut s = VulnerableStack::default();
                s.add_one_shot(10.0);
                s
            })
            .id();

        enqueue(&mut app, mk_msg(Some(dealer), target, 3.0));
        tick(&mut app);

        let drained = drain_messages(&mut app);
        assert_eq!(drained.len(), 1);
        assert_f32_eq(drained[0].amount, 30.0);
    }
}
