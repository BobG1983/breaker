use std::time::Duration;

use bevy::prelude::*;

use super::super::*;
use crate::state::types::{AppState, ChipSelectState, GameState, NodeState, RunEndState, RunState};

// ════════════════════════════════════════════════════════════════════
// Section A: TestAppBuilder Core Construction
// ════════════════════════════════════════════════════════════════════

// ── Behavior 1: new() returns a builder that produces a minimal app ──

#[test]
fn builder_new_produces_app_with_time_fixed_resource() {
    let app = TestAppBuilder::new().build();
    // MinimalPlugins provides Time<Fixed>; stub doesn't add MinimalPlugins,
    // so this should fail if the stub is a bare App::new().
    let time_fixed = app.world().get_resource::<Time<Fixed>>();
    assert!(
        time_fixed.is_some(),
        "App from TestAppBuilder::new().build() must have Time<Fixed> (MinimalPlugins)"
    );
}

#[test]
fn builder_new_time_fixed_has_default_timestep() {
    let app = TestAppBuilder::new().build();
    let time_fixed = app.world().resource::<Time<Fixed>>();
    let expected = Duration::from_secs_f64(1.0 / 64.0);
    assert_eq!(
        time_fixed.timestep(),
        expected,
        "Time<Fixed> timestep should be the Bevy default (1/64s), got {:?}",
        time_fixed.timestep()
    );
}

// ── Behavior 2: new() does not register states ──

#[test]
fn builder_new_does_not_register_app_state() {
    let app = TestAppBuilder::new().build();
    assert!(
        app.world().get_resource::<State<AppState>>().is_none(),
        "TestAppBuilder::new() should not register AppState"
    );
}

#[test]
fn builder_new_does_not_register_sub_states() {
    let app = TestAppBuilder::new().build();
    assert!(
        app.world().get_resource::<State<GameState>>().is_none(),
        "TestAppBuilder::new() should not register GameState"
    );
    assert!(
        app.world().get_resource::<State<RunState>>().is_none(),
        "TestAppBuilder::new() should not register RunState"
    );
    assert!(
        app.world().get_resource::<State<NodeState>>().is_none(),
        "TestAppBuilder::new() should not register NodeState"
    );
}

// ── Behavior 3: new() does not register messages ──

#[test]
fn builder_new_does_not_register_message_collector() {
    use crate::{cells::components::Cell, prelude::DamageDealt};

    let app = TestAppBuilder::new().build();
    assert!(
        app.world()
            .get_resource::<MessageCollector<DamageDealt<Cell>>>()
            .is_none(),
        "TestAppBuilder::new() should not register any MessageCollector"
    );
}

// ════════════════════════════════════════════════════════════════════
// Section B: State Hierarchy Registration
// ════════════════════════════════════════════════════════════════════

// ── Behavior 4: with_state_hierarchy() registers states ──

#[test]
fn with_state_hierarchy_registers_app_state() {
    let mut app = TestAppBuilder::new().with_state_hierarchy().build();
    app.update();
    let state = app.world().get_resource::<State<AppState>>();
    assert!(
        state.is_some(),
        "with_state_hierarchy() must register AppState"
    );
    assert_eq!(
        *state.unwrap().get(),
        AppState::Loading,
        "AppState should default to Loading"
    );
}

#[test]
fn with_state_hierarchy_sub_states_not_present_in_default_parent() {
    let mut app = TestAppBuilder::new().with_state_hierarchy().build();
    app.update();
    // GameState is a sub-state of AppState::Game, not AppState::Loading
    assert!(
        app.world().get_resource::<State<GameState>>().is_none(),
        "GameState should not be present when AppState is Loading"
    );
    assert!(
        app.world().get_resource::<State<RunState>>().is_none(),
        "RunState should not be present when AppState is Loading"
    );
    assert!(
        app.world().get_resource::<State<NodeState>>().is_none(),
        "NodeState should not be present when AppState is Loading"
    );
    assert!(
        app.world()
            .get_resource::<State<ChipSelectState>>()
            .is_none(),
        "ChipSelectState should not be present when AppState is Loading"
    );
    assert!(
        app.world().get_resource::<State<RunEndState>>().is_none(),
        "RunEndState should not be present when AppState is Loading"
    );
}

// ── Behavior 5: with_state_hierarchy() typestate transition ──

#[test]
fn with_state_hierarchy_enables_state_navigation_methods() {
    // This test verifies the typestate transition at compile time.
    // If it compiles, the test passes (in_state_node_playing is only on WithStates).
    let _app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
}

// ════════════════════════════════════════════════════════════════════
// W2 Section — Dmg typestate dimension (Behaviors 47–52)
// ════════════════════════════════════════════════════════════════════

// ── W2 Behavior 47: DmgStatus trait and NoDmg/WithDmg marker types exist ──

#[test]
fn dmg_status_markers_exist() {
    use super::super::builder::{DmgStatus, NoDmg, WithDmg};

    // Both markers implement DmgStatus at the trait level; force the
    // bound by using a generic helper.
    const fn require_dmg_status<T: DmgStatus>() {}
    require_dmg_status::<NoDmg>();
    require_dmg_status::<WithDmg>();
}

// ── W2 Behavior 48: TestAppBuilder<S, D> has two type parameters with defaults ──

#[test]
fn test_app_builder_has_default_type_params_nostates_nodmg() {
    use super::super::builder::{NoDmg, NoStates};

    // Explicit binding with both type params spelled out.
    let _b: TestAppBuilder<NoStates, NoDmg> = TestAppBuilder::new();
    // Equivalent without turbofish — defaults cover both axes.
    let _c = TestAppBuilder::new();
}

// ── W2 Behavior 49: with_dmg_pipeline adds RantzDmgPlugin and transitions typestate ──

#[test]
fn with_dmg_pipeline_registers_despawn_entity_message() {
    use rantzsoft_dmg::DespawnEntity;

    let app = TestAppBuilder::new().with_dmg_pipeline().build();

    assert!(
        app.world().contains_resource::<Messages<DespawnEntity>>(),
        "with_dmg_pipeline() must add RantzDmgPlugin (observable via \
         Messages<DespawnEntity>)"
    );
}

#[test]
fn with_dmg_pipeline_returns_with_dmg_typestate() {
    use super::super::builder::{NoStates, WithDmg};

    let _b: TestAppBuilder<NoStates, WithDmg> = TestAppBuilder::new().with_dmg_pipeline();
}

// ── W2 Behavior 50: register_dmgable::<T> only on TestAppBuilder<S, WithDmg> ──

#[test]
fn register_dmgable_on_with_dmg_registers_messages() {
    use crate::{cells::components::Cell, prelude::DamageDealt};

    let app = TestAppBuilder::new()
        .with_dmg_pipeline()
        .register_dmgable::<Cell>()
        .build();

    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<Cell>>>(),
        "register_dmgable::<Cell>() on WithDmg must register Messages<DamageDealt<Cell>>"
    );
}

#[test]
fn register_dmgable_chains_multiple_types() {
    use crate::{bolt::components::Bolt, cells::components::Cell, prelude::DamageDealt};

    let app = TestAppBuilder::new()
        .with_dmg_pipeline()
        .register_dmgable::<Bolt>()
        .register_dmgable::<Cell>()
        .build();

    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<Bolt>>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<Cell>>>()
    );
}

// ── W2 Behavior 51: with_effects_pipeline registers Bolt/Wall/Breaker/Salvo/Cell ──

#[test]
fn with_effects_pipeline_registers_all_five_types() {
    use crate::{
        bolt::components::Bolt,
        breaker::components::Breaker,
        cells::{behaviors::survival::salvo::components::Salvo, components::Cell},
        prelude::DamageDealt,
        walls::components::Wall,
    };

    let app = TestAppBuilder::new().with_effects_pipeline().build();

    // Bolt, Wall, Breaker, Salvo, and Cell are all registered.
    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<Bolt>>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<Wall>>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<Breaker>>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<Salvo>>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<DamageDealt<Cell>>>(),
        "with_effects_pipeline() must register Cell for W2 (Cell joins the crate chain)"
    );
}

#[test]
fn with_effects_pipeline_transitions_to_with_dmg() {
    use super::super::builder::{NoStates, WithDmg};

    let _b: TestAppBuilder<NoStates, WithDmg> = TestAppBuilder::new().with_effects_pipeline();
}

#[test]
fn with_effects_pipeline_adds_rantz_dmg_plugin() {
    use rantzsoft_dmg::DespawnEntity;

    let app = TestAppBuilder::new().with_effects_pipeline().build();
    assert!(
        app.world().contains_resource::<Messages<DespawnEntity>>(),
        "with_effects_pipeline() implicitly adds RantzDmgPlugin"
    );
}

// Grep-style `include_str!` structural-invariant tests were removed per
// plan §M. The typestate itself enforces `with_dmg_pipeline` / `register_dmgable`
// availability — compile-fail doctests in `builder.rs` pin the negative
// paths; the positive paths are pinned by behavioral tests above.
