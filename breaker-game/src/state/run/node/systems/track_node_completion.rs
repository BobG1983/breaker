//! System to track node completion by counting destroyed required cells.

use bevy::prelude::*;
use tracing::info;

use crate::{
    cells::components::{Cell, RequiredToClear},
    shared::death_pipeline::Destroyed,
    state::run::node::{ClearRemainingCount, messages::NodeCleared},
};

/// Reads [`Destroyed<Cell>`] messages and decrements [`ClearRemainingCount`]
/// when the victim had [`RequiredToClear`]. When the count reaches zero,
/// sends [`NodeCleared`].
///
/// Queries the victim's components directly because [`Destroyed<Cell>`] does
/// not carry a `was_required_to_clear` field. The victim is still alive when
/// [`Destroyed<Cell>`] is read — `Dead` is inserted but
/// `process_despawn_requests` has not yet run (deferred to `FixedPostUpdate`).
/// The query intentionally omits `Without<Dead>` because the victim carries
/// `Dead` at this schedule slot; filtering it out would exclude every victim.
pub(crate) fn track_node_completion(
    mut reader: MessageReader<Destroyed<Cell>>,
    required_query: Query<(), (With<Cell>, With<RequiredToClear>)>,
    mut remaining: ResMut<ClearRemainingCount>,
    mut writer: MessageWriter<NodeCleared>,
    mut fired: Local<bool>,
) {
    // Reset when a new node loads (resource re-inserted by init_clear_remaining).
    if remaining.is_changed() {
        *fired = false;
    }

    for msg in reader.read() {
        if required_query.contains(msg.victim) {
            remaining.remaining = remaining.remaining.saturating_sub(1);
        }
    }

    if remaining.remaining == 0 && !*fired {
        info!("node cleared");
        writer.write(NodeCleared);
        *fired = true;
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::*;
    use crate::{
        cells::components::{Cell, RequiredToClear},
        shared::{
            death_pipeline::{destroyed::Destroyed, hp::Hp, killed_by::KilledBy},
            test_utils::{TestAppBuilder, tick},
        },
    };

    #[derive(Resource, Default)]
    struct TestDestroyedMessages(Vec<Destroyed<Cell>>);

    fn enqueue_messages(
        msg_res: Res<TestDestroyedMessages>,
        mut writer: MessageWriter<Destroyed<Cell>>,
    ) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    #[derive(Resource, Default)]
    struct NodeClearedCount(u32);

    fn capture_node_cleared(
        mut reader: MessageReader<NodeCleared>,
        mut count: ResMut<NodeClearedCount>,
    ) {
        for _ in reader.read() {
            count.0 += 1;
        }
    }

    fn make_destroyed(victim: Entity) -> Destroyed<Cell> {
        Destroyed::<Cell> {
            victim,
            killer: None,
            victim_pos: Vec2::ZERO,
            killer_pos: None,
            _marker: PhantomData,
        }
    }

    fn test_app(remaining: u32) -> App {
        TestAppBuilder::new()
            .with_message::<Destroyed<Cell>>()
            .with_message::<NodeCleared>()
            .insert_resource(ClearRemainingCount { remaining })
            .with_resource::<TestDestroyedMessages>()
            .with_resource::<NodeClearedCount>()
            .with_system(
                FixedUpdate,
                (
                    enqueue_messages,
                    track_node_completion,
                    capture_node_cleared,
                )
                    .chain(),
            )
            .build()
    }

    // ── N1: track_node_completion decrements for a RequiredToClear cell ────

    #[test]
    fn decrements_clear_remaining_for_required_to_clear_cell() {
        let mut app = test_app(3);

        let cell_a = app
            .world_mut()
            .spawn((Cell, RequiredToClear, Hp::new(1.0), KilledBy::default()))
            .id();
        app.world_mut()
            .spawn((Cell, RequiredToClear, Hp::new(1.0), KilledBy::default()));
        app.world_mut()
            .spawn((Cell, RequiredToClear, Hp::new(1.0), KilledBy::default()));

        app.insert_resource(TestDestroyedMessages(vec![make_destroyed(cell_a)]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(
            count.remaining, 2,
            "ClearRemainingCount should decrement by 1 for a RequiredToClear victim"
        );
    }

    #[test]
    fn decrements_all_three_for_three_required_cells_in_same_tick() {
        let mut app = test_app(3);

        let cell_a = app
            .world_mut()
            .spawn((Cell, RequiredToClear, Hp::new(1.0), KilledBy::default()))
            .id();
        let cell_b = app
            .world_mut()
            .spawn((Cell, RequiredToClear, Hp::new(1.0), KilledBy::default()))
            .id();
        let cell_c = app
            .world_mut()
            .spawn((Cell, RequiredToClear, Hp::new(1.0), KilledBy::default()))
            .id();

        app.insert_resource(TestDestroyedMessages(vec![
            make_destroyed(cell_a),
            make_destroyed(cell_b),
            make_destroyed(cell_c),
        ]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(
            count.remaining, 0,
            "three RequiredToClear victims in one tick should drop remaining to 0"
        );

        let cleared = app.world().resource::<NodeClearedCount>();
        assert_eq!(
            cleared.0, 1,
            "NodeCleared should be emitted exactly once when remaining hits 0"
        );
    }

    // ── N2: track_node_completion ignores a cell without RequiredToClear ──

    #[test]
    fn does_not_decrement_for_cell_without_required_to_clear() {
        let mut app = test_app(3);

        let cell_decor = app
            .world_mut()
            .spawn((Cell, Hp::new(1.0), KilledBy::default()))
            .id();

        app.insert_resource(TestDestroyedMessages(vec![make_destroyed(cell_decor)]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(
            count.remaining, 3,
            "decor cell should NOT decrement ClearRemainingCount"
        );
    }

    #[test]
    fn mixed_batch_only_decrements_for_required_to_clear_cells() {
        let mut app = test_app(3);

        let cell_required = app
            .world_mut()
            .spawn((Cell, RequiredToClear, Hp::new(1.0), KilledBy::default()))
            .id();
        let cell_decor = app
            .world_mut()
            .spawn((Cell, Hp::new(1.0), KilledBy::default()))
            .id();

        app.insert_resource(TestDestroyedMessages(vec![
            make_destroyed(cell_required),
            make_destroyed(cell_decor),
        ]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(
            count.remaining, 2,
            "only the RequiredToClear victim should decrement, got {}",
            count.remaining
        );
    }

    #[test]
    fn missing_victim_entity_does_not_panic_and_does_not_decrement() {
        let mut app = test_app(3);

        // Never spawn this entity — the query-back must handle missing entities.
        let phantom = Entity::PLACEHOLDER;

        app.insert_resource(TestDestroyedMessages(vec![make_destroyed(phantom)]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(
            count.remaining, 3,
            "missing victim entity should be treated as not-required"
        );
    }

    // ── N3: NodeCleared emitted exactly once when count reaches zero ───────

    #[test]
    fn node_cleared_emitted_exactly_once_when_count_reaches_zero() {
        let mut app = test_app(1);

        let cell_last = app
            .world_mut()
            .spawn((Cell, RequiredToClear, Hp::new(1.0), KilledBy::default()))
            .id();

        // Tick 1: one destroyed message, remaining hits 0.
        app.insert_resource(TestDestroyedMessages(vec![make_destroyed(cell_last)]));
        tick(&mut app);

        let count = app.world().resource::<ClearRemainingCount>();
        assert_eq!(count.remaining, 0);
        let cleared = app.world().resource::<NodeClearedCount>();
        assert_eq!(
            cleared.0, 1,
            "NodeCleared should be emitted once after Tick 1"
        );

        // Tick 2: zero messages, remaining still 0 — NodeCleared should NOT re-fire.
        app.insert_resource(TestDestroyedMessages(vec![]));
        tick(&mut app);

        let cleared = app.world().resource::<NodeClearedCount>();
        assert_eq!(
            cleared.0, 1,
            "NodeCleared should still be 1 after Tick 2 (fired local guards re-emit)"
        );

        // Tick 3: another Destroyed<Cell> for a different RequiredToClear cell — still no re-fire.
        let cell_extra = app
            .world_mut()
            .spawn((Cell, RequiredToClear, Hp::new(1.0), KilledBy::default()))
            .id();
        app.insert_resource(TestDestroyedMessages(vec![make_destroyed(cell_extra)]));
        tick(&mut app);

        let cleared = app.world().resource::<NodeClearedCount>();
        assert_eq!(
            cleared.0, 1,
            "NodeCleared should still be 1 after Tick 3 (fired local guards re-emit)"
        );
    }
}
