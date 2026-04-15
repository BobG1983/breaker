// ════════════════════════════════════════════════════════════════════
// Tests use an **early-fire probe** to discriminate the bridge's
// schedule placement. Cross-schedule `.after` constraints silently
// no-op in Bevy 0.18 (they are not an error), so schedule introspection
// and ordering probes cannot reliably detect whether the bridge is in
// `OnExit(Playing)` or `OnEnter(Teardown)`.
//
// The probe: drive the state chain one step, `Playing → AnimateOut`.
// This runs `OnExit(Playing)` but NOT `OnEnter(Teardown)`. An entity
// with a `When(NodeEndOccurred, Fire(SpeedBoost))` tree has an
// `EffectStack<SpeedBoostConfig>` iff the bridge fired during
// `OnExit(Playing)` — i.e., the pre-fix wiring.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_stateflow::cleanup_on_exit;

use crate::{
    cells::components::Cell,
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::BoundEffects,
        triggers::node::register,
        types::{EffectType, Tree, Trigger},
    },
    shared::test_utils::TestAppBuilder,
    state::{
        run::node::sets::NodeSystems,
        types::{AppState, GameState, NodeState, RunState},
    },
};

// ── Helpers ─────────────────────────────────────────────────────

/// Builds a test `App` stopped at `RunState::Node` — `NodeState`
/// is at its default (`Loading`). Used directly by Behavior 8 so
/// fixture entities exist when `OnEnter(Playing)` runs, and as the
/// base for `build_test_app()` below.
///
/// Wires `cleanup_on_exit::<NodeState>` (mirrors production's
/// `state/run/node/plugin.rs:47`) and calls `register::register` —
/// the production wiring under test. `register` MUST run before
/// the state chain so `OnEnter(Playing)` picks up the bridges.
fn build_test_app_pre_playing() -> App {
    let mut app = TestAppBuilder::new().with_state_hierarchy().build();
    app.add_systems(
        OnEnter(NodeState::Teardown),
        cleanup_on_exit::<NodeState>.in_set(NodeSystems::Cleanup),
    );
    register::register(&mut app);

    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Run);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<RunState>>()
        .set(RunState::Node);
    app.update();

    app
}

/// Builds a test `App` driven all the way to `NodeState::Playing`.
fn build_test_app() -> App {
    let mut app = build_test_app_pre_playing();
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();
    app
}

/// Drives `NodeState` through `Playing → AnimateOut → Teardown`
/// via two sequential `NextState::<NodeState>` writes with an
/// `update()` after each. The two updates are load-bearing: the
/// first drains `OnExit(Playing) + OnEnter(AnimateOut)`, the
/// second drains `OnExit(AnimateOut) + OnEnter(Teardown)`.
fn drive_to_teardown(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Teardown);
    app.update();
}

/// Drives `NodeState` one step: `Playing → AnimateOut` via a
/// single `NextState::<NodeState>` write and one `update()`. This
/// runs `OnExit(NodeState::Playing)` but NOT
/// `OnEnter(NodeState::Teardown)`. Used by the early-fire RED
/// signal probes.
fn drive_to_animate_out(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();
}

/// Builds a `(source, When(trigger, Fire(SpeedBoost)))` tree entry
/// for use in `BoundEffects`.
fn speed_tree(source: &str, trigger: Trigger, multiplier: f32) -> (String, Tree) {
    (
        source.to_owned(),
        Tree::When(
            trigger,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(multiplier),
            }))),
        ),
    )
}

// ── Behavior 1: on_node_end_occurred does NOT fire during
//    OnExit(NodeState::Playing). This is the primary RED-signal
//    test. Under the buggy wiring, the bridge is registered in
//    OnExit(Playing) and fires while we transition from Playing →
//    AnimateOut. Under the fix it lives in OnEnter(Teardown) and
//    does not fire until we take the second step of the chain.

#[test]
fn on_node_end_occurred_does_not_fire_on_exit_playing() {
    let mut app = build_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "test_chip",
            Trigger::NodeEndOccurred,
            1.5,
        )]))
        .id();

    // Drive ONLY Playing → AnimateOut. This runs
    // OnExit(NodeState::Playing) but NOT OnEnter(Teardown).
    drive_to_animate_out(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "on_node_end_occurred must NOT run in OnExit(Playing). After only driving \
         Playing → AnimateOut (one update, no Teardown transition), the \
         EffectStack should still be absent. Pre-fix the bridge lives in \
         OnExit(Playing) and will erroneously create the stack here."
    );
}

// ── Behavior 2: on_node_end_occurred walks effect trees on
//    BoundEffects entities. Post-fix contract — the bridge must
//    still dispatch the trigger after the move.

#[test]
fn on_node_end_occurred_walks_effect_trees_on_bound_entities() {
    let mut app = build_test_app();

    let e1 = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "test_chip",
            Trigger::NodeEndOccurred,
            1.5,
        )]))
        .id();

    drive_to_teardown(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(e1)
        .expect("EffectStack should exist on E1 after full transition to Teardown");
    assert_eq!(
        stack.len(),
        1,
        "E1 should have exactly one effect fired by on_node_end_occurred"
    );
}

#[test]
fn on_node_end_occurred_walks_all_bound_entities() {
    let mut app = build_test_app();

    let e1 = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "test_chip",
            Trigger::NodeEndOccurred,
            1.5,
        )]))
        .id();
    let e2 = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "test_chip",
            Trigger::NodeEndOccurred,
            1.5,
        )]))
        .id();

    drive_to_teardown(&mut app);

    let stack_e1 = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(e1)
        .expect("EffectStack should exist on E1");
    assert_eq!(stack_e1.len(), 1);

    let stack_e2 = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(e2)
        .expect("EffectStack should exist on E2");
    assert_eq!(
        stack_e2.len(),
        1,
        "bridge must iterate all BoundEffects entities, not just the first"
    );
}

// ── Behavior 3: on_node_end_occurred fires only once per
//    Teardown transition (OnEnter is schedule-driven, not Update-gated).

#[test]
fn on_node_end_occurred_fires_only_once_per_teardown_transition() {
    let mut app = build_test_app();

    let e1 = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "test_chip",
            Trigger::NodeEndOccurred,
            1.5,
        )]))
        .id();

    drive_to_teardown(&mut app);

    // An extra update while state remains in Teardown must NOT
    // re-run the OnEnter schedule.
    app.update();

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(e1)
        .expect("EffectStack should exist on E1 after Teardown transition");
    assert_eq!(
        stack.len(),
        1,
        "on_node_end_occurred must fire exactly once per state transition \
         (OnEnter-driven, not Update-gated)"
    );
}

// ── Behavior 4: cells have been despawned before the bridge
//    fires. Under the fix, `on_node_end_occurred` is ordered
//    `.after(cleanup_on_exit::<NodeState>)` in OnEnter(Teardown),
//    so no Cell entities remain when the bridge (which creates
//    EffectStack on BoundEffects entities) runs. We use the same
//    early-fire probe used in Behavior 1 to discriminate: after
//    ONLY Playing → AnimateOut, no EffectStack should exist
//    (because the bridge hasn't run yet). After the full chain
//    the stack exists AND cells are gone.

#[test]
fn on_node_end_occurred_fires_after_cells_are_cleaned_up() {
    let mut app = build_test_app();

    // 3 Cell entities — each auto-requires CleanupOnExit<NodeState>
    // via the `#[require]` attribute on Cell.
    app.world_mut().spawn(Cell);
    app.world_mut().spawn(Cell);
    app.world_mut().spawn(Cell);

    let e1 = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "test_chip",
            Trigger::NodeEndOccurred,
            1.5,
        )]))
        .id();

    // Sanity: three cells are alive while state is Playing.
    let mut cell_query = app.world_mut().query::<&Cell>();
    let cells_in_playing = cell_query.iter(app.world()).count();
    assert_eq!(cells_in_playing, 3, "3 Cell entities expected in Playing");

    // First step of the chain: Playing → AnimateOut. Under the
    // fix the bridge has NOT fired yet (it lives in
    // OnEnter(Teardown)). Under the bug the bridge runs here.
    drive_to_animate_out(&mut app);
    let stack_after_animate_out = app.world().get::<EffectStack<SpeedBoostConfig>>(e1);
    assert!(
        stack_after_animate_out.is_none(),
        "Bridge must not fire during OnExit(Playing). Under the bug it does, \
         creating an EffectStack here — which is exactly the ordering problem \
         Wave 2b fixes (the bridge would fire while Cell entities still exist)."
    );

    // Second step: AnimateOut → Teardown. Under the fix this
    // runs `cleanup_on_exit::<NodeState>` (despawning all 3
    // Cells) and THEN `on_node_end_occurred` (creating the
    // EffectStack on E1).
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Teardown);
    app.update();

    // Cells have been despawned by cleanup_on_exit.
    let mut remaining_query = app.world_mut().query::<&Cell>();
    let cells_remaining = remaining_query.iter(app.world()).count();
    assert_eq!(
        cells_remaining, 0,
        "cleanup_on_exit::<NodeState> must despawn all Cell entities by the \
         time OnEnter(Teardown) completes"
    );

    // Bridge has fired exactly once, observable on E1.
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(e1)
        .expect("EffectStack should exist on E1 after OnEnter(Teardown)");
    assert_eq!(stack.len(), 1);
}

// ── Behavior 5: on_node_end_occurred does not fire if there are
//    no BoundEffects entities (empty bound_query).

#[test]
fn on_node_end_occurred_is_noop_without_bound_entities() {
    let mut app = build_test_app();

    // No BoundEffects entities — just a plain empty entity.
    let e1 = app.world_mut().spawn_empty().id();

    drive_to_teardown(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(e1);
    assert!(
        stack.is_none(),
        "E1 has no BoundEffects — no EffectStack should be created"
    );
}

// ── Behavior 6: on_node_end_occurred walks all trees in a
//    multi-tree BoundEffects entity (the `for (source, tree) in
//    trees` loop inside walk_effects).

#[test]
fn on_node_end_occurred_walks_multiple_trees_on_one_entity() {
    let mut app = build_test_app();

    let e1 = app
        .world_mut()
        .spawn(BoundEffects(vec![
            speed_tree("chip_a", Trigger::NodeEndOccurred, 1.5),
            speed_tree("chip_b", Trigger::NodeEndOccurred, 2.0),
        ]))
        .id();

    drive_to_teardown(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(e1)
        .expect("EffectStack should exist after multi-tree walk");
    assert_eq!(
        stack.len(),
        2,
        "each of the two trees should contribute one stack entry"
    );
}

// ── Behavior 7: on_node_end_occurred does NOT fire for a
//    When-gate trigger that doesn't match NodeEndOccurred.

#[test]
fn on_node_end_occurred_does_not_fire_non_matching_gates() {
    let mut app = build_test_app();

    // BoltLostOccurred gate, NOT NodeEndOccurred
    let e1 = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "test_chip",
            Trigger::BoltLostOccurred,
            1.5,
        )]))
        .id();

    drive_to_teardown(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(e1);
    assert!(
        stack.is_none(),
        "BoltLostOccurred gate must not match NodeEndOccurred trigger passed \
         by the bridge — confirms the bridge dispatches the correct trigger"
    );
}

// ── Behavior 8: on_node_start_occurred (the sibling bridge) is
//    NOT moved — it still fires on OnEnter(NodeState::Playing).
//    This is a regression guard against accidentally moving the
//    start bridge alongside the end bridge.

#[test]
fn on_node_start_occurred_still_fires_on_enter_playing() {
    let mut app = build_test_app_pre_playing();

    // Spawn the fixture BEFORE driving to Playing so the
    // OnEnter(Playing) schedule observes it.
    let e1 = app
        .world_mut()
        .spawn(BoundEffects(vec![speed_tree(
            "test_chip",
            Trigger::NodeStartOccurred,
            1.5,
        )]))
        .id();

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Playing);
    app.update();

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(e1)
        .expect("EffectStack should exist on E1 after OnEnter(Playing)");
    assert_eq!(
        stack.len(),
        1,
        "on_node_start_occurred must still fire on OnEnter(NodeState::Playing) \
         — only on_node_end_occurred is being moved in this wave"
    );
}
