//! Shared test infrastructure: `TestAppBuilder`, `MessageCollector`, and `tick()`.
//!
//! This module provides composable building blocks for Bevy ECS integration tests.
//! All types are `#[cfg(test)]` and `pub(crate)` — available to any domain's tests.

use std::marker::PhantomData;

use bevy::{
    ecs::schedule::{IntoScheduleConfigs, ScheduleLabel},
    prelude::*,
};
use rantzsoft_physics2d::plugin::RantzPhysics2dPlugin;

use crate::{
    bolt::{definition::BoltDefinition, registry::BoltRegistry},
    breaker::{definition::BreakerDefinition, registry::BreakerRegistry},
    cells::{
        definition::CellTypeDefinition,
        resources::{CellConfig, CellTypeRegistry},
    },
    shared::PlayfieldConfig,
    state::types::*,
};

// ── Typestate markers ──────────────────────────────────────────────────────

/// Marker trait for `TestAppBuilder` typestate.
pub(crate) trait StateStatus {}

/// Initial typestate — no state hierarchy registered.
pub(crate) struct NoStates;
impl StateStatus for NoStates {}

/// After `with_state_hierarchy()` — state navigation methods available.
pub(crate) struct WithStates;
impl StateStatus for WithStates {}

// ── TestAppBuilder ─────────────────────────────────────────────────────────

/// Typestate builder for test `App` instances.
pub(crate) struct TestAppBuilder<S: StateStatus = NoStates> {
    app: App,
    _state: PhantomData<S>,
}

impl TestAppBuilder<NoStates> {
    /// Creates a new builder with `MinimalPlugins` registered.
    #[must_use]
    pub(crate) fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        Self {
            app,
            _state: PhantomData,
        }
    }

    /// Registers the full state hierarchy (`AppState` + all sub-states).
    #[must_use]
    pub(crate) fn with_state_hierarchy(mut self) -> TestAppBuilder<WithStates> {
        self.app.add_plugins(bevy::state::app::StatesPlugin);
        self.app.init_state::<AppState>();
        self.app.add_sub_state::<GameState>();
        self.app.add_sub_state::<RunState>();
        self.app.add_sub_state::<NodeState>();
        self.app.add_sub_state::<ChipSelectState>();
        self.app.add_sub_state::<RunEndState>();
        TestAppBuilder {
            app: self.app,
            _state: PhantomData,
        }
    }
}

impl TestAppBuilder<WithStates> {
    /// Drives the app into `NodeState::Playing` via four transitions:
    /// `AppState::Game` → `GameState::Run` → `RunState::Node` → `NodeState::Playing`.
    /// Each step sets `NextState` and calls `app.update()`.
    #[must_use]
    pub(crate) fn in_state_node_playing(mut self) -> Self {
        self.app
            .world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        self.app.update();
        self.app
            .world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Run);
        self.app.update();
        self.app
            .world_mut()
            .resource_mut::<NextState<RunState>>()
            .set(RunState::Node);
        self.app.update();
        self.app
            .world_mut()
            .resource_mut::<NextState<NodeState>>()
            .set(NodeState::Playing);
        self.app.update();
        self
    }

    /// Drives the app into `ChipSelectState::Selecting` via four transitions:
    /// `AppState::Game` → `GameState::Run` → `RunState::ChipSelect` → `ChipSelectState::Selecting`.
    /// Each step sets `NextState` and calls `app.update()`.
    #[must_use]
    pub(crate) fn in_state_chip_selecting(mut self) -> Self {
        self.app
            .world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        self.app.update();
        self.app
            .world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Run);
        self.app.update();
        self.app
            .world_mut()
            .resource_mut::<NextState<RunState>>()
            .set(RunState::ChipSelect);
        self.app.update();
        self.app
            .world_mut()
            .resource_mut::<NextState<ChipSelectState>>()
            .set(ChipSelectState::Selecting);
        self.app.update();
        self
    }
}

impl<S: StateStatus> TestAppBuilder<S> {
    /// Adds the `RantzPhysics2dPlugin`.
    #[must_use]
    pub(crate) fn with_physics(mut self) -> Self {
        self.app.add_plugins(RantzPhysics2dPlugin);
        self
    }

    /// Registers `PlayfieldConfig`, `CellConfig`, `Assets<Mesh>`, `Assets<ColorMaterial>`.
    #[must_use]
    pub(crate) fn with_playfield(mut self) -> Self {
        self.app.init_resource::<PlayfieldConfig>();
        self.app.init_resource::<CellConfig>();
        self.app.init_resource::<Assets<Mesh>>();
        self.app.init_resource::<Assets<ColorMaterial>>();
        self
    }

    /// Initializes a resource with its `Default` impl. Idempotent — does not
    /// overwrite an existing resource.
    #[must_use]
    pub(crate) fn with_resource<R: Resource + Default>(mut self) -> Self {
        self.app.init_resource::<R>();
        self
    }

    /// Inserts a concrete resource value.
    #[must_use]
    pub(crate) fn insert_resource<R: Resource>(mut self, resource: R) -> Self {
        self.app.insert_resource(resource);
        self
    }

    /// Registers a message type for sending and reading.
    #[must_use]
    pub(crate) fn with_message<M: Message>(mut self) -> Self {
        self.app.add_message::<M>();
        self
    }

    /// Registers a message type with automatic capture infrastructure.
    ///
    /// Idempotent: calling twice for the same message type is safe and does not
    /// duplicate the collector or systems.
    #[must_use]
    pub(crate) fn with_message_capture<M: Message + Clone>(mut self) -> Self {
        if self.app.world().contains_resource::<MessageCollector<M>>() {
            return self;
        }
        self.app.add_message::<M>();
        self.app.init_resource::<MessageCollector<M>>();
        self.app.add_systems(First, clear_messages::<M>);
        self.app.add_systems(Last, collect_messages::<M>);
        self
    }

    /// Creates an empty `BoltRegistry`.
    #[must_use]
    pub(crate) fn with_bolt_registry(mut self) -> Self {
        self.app.init_resource::<BoltRegistry>();
        self
    }

    /// Inserts a bolt definition into the registry (creating it if needed).
    #[must_use]
    pub(crate) fn with_bolt_registry_entry(mut self, name: &str, def: BoltDefinition) -> Self {
        self.app.init_resource::<BoltRegistry>();
        self.app
            .world_mut()
            .resource_mut::<BoltRegistry>()
            .insert(name.to_string(), def);
        self
    }

    /// Creates an empty `BreakerRegistry`.
    #[must_use]
    pub(crate) fn with_breaker_registry(mut self) -> Self {
        self.app.init_resource::<BreakerRegistry>();
        self
    }

    /// Inserts a breaker definition into the registry (creating it if needed).
    #[must_use]
    pub(crate) fn with_breaker_registry_entry(
        mut self,
        name: &str,
        def: BreakerDefinition,
    ) -> Self {
        self.app.init_resource::<BreakerRegistry>();
        self.app
            .world_mut()
            .resource_mut::<BreakerRegistry>()
            .insert(name.to_string(), def);
        self
    }

    /// Creates an empty `CellTypeRegistry`.
    #[must_use]
    pub(crate) fn with_cell_registry(mut self) -> Self {
        self.app.init_resource::<CellTypeRegistry>();
        self
    }

    /// Inserts a cell type definition into the registry (creating it if needed).
    #[must_use]
    pub(crate) fn with_cell_registry_entry(mut self, alias: &str, def: CellTypeDefinition) -> Self {
        self.app.init_resource::<CellTypeRegistry>();
        self.app
            .world_mut()
            .resource_mut::<CellTypeRegistry>()
            .insert(alias.to_string(), def);
        self
    }

    /// Adds a system to the specified schedule.
    #[must_use]
    pub(crate) fn with_system<M>(
        mut self,
        schedule: impl ScheduleLabel,
        system: impl IntoScheduleConfigs<bevy::ecs::system::ScheduleSystem, M>,
    ) -> Self {
        self.app.add_systems(schedule, system);
        self
    }

    /// Finalizes the builder and returns the `App`.
    pub(crate) fn build(self) -> App {
        self.app
    }
}

// ── MessageCollector ───────────────────────────────────────────────────────

/// Generic message collector resource. Captures messages for test assertions.
#[derive(Resource)]
pub(crate) struct MessageCollector<M: Message>(pub Vec<M>);

impl<M: Message> Default for MessageCollector<M> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<M: Message> MessageCollector<M> {
    /// Manually clears collected messages.
    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }
}

// ── Clear and collect systems ──────────────────────────────────────────────

/// Clears the `MessageCollector<M>` at the start of each update cycle.
fn clear_messages<M: Message>(mut collector: ResMut<MessageCollector<M>>) {
    collector.0.clear();
}

/// Reads messages from `MessageReader<M>` and pushes clones into `MessageCollector<M>`.
fn collect_messages<M: Message + Clone>(
    mut reader: MessageReader<M>,
    mut collector: ResMut<MessageCollector<M>>,
) {
    for msg in reader.read() {
        collector.0.push(msg.clone());
    }
}

// ── tick() ─────────────────────────────────────────────────────────────────

/// Advances exactly one `FixedUpdate` timestep by accumulating overstep then updating.
pub(crate) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::{
        bolt::{
            definition::BoltDefinition,
            messages::{BoltImpactCell, BoltLost},
            registry::BoltRegistry,
        },
        breaker::{definition::BreakerDefinition, registry::BreakerRegistry},
        cells::{
            definition::{CellTypeDefinition, Toughness},
            messages::DamageCell,
            resources::{CellConfig, CellTypeRegistry},
        },
        shared::{playfield::PlayfieldConfig, rng::GameRng},
        state::types::{AppState, ChipSelectState, GameState, NodeState, RunEndState, RunState},
    };

    // ── Test helper types ──────────────────────────────────────────────

    #[derive(Resource, Default)]
    struct Counter(u32);

    #[derive(Resource, Default)]
    struct Order(String);

    #[derive(Resource)]
    struct ShouldSend(bool);

    fn increment(mut counter: ResMut<Counter>) {
        counter.0 += 1;
    }

    fn damage_sender_system(mut writer: MessageWriter<DamageCell>) {
        writer.write(DamageCell {
            cell: Entity::PLACEHOLDER,
            damage: 25.0,
            source_chip: None,
        });
    }

    fn conditional_damage_sender(flag: Res<ShouldSend>, mut writer: MessageWriter<DamageCell>) {
        if flag.0 {
            writer.write(DamageCell {
                cell: Entity::PLACEHOLDER,
                damage: 10.0,
                source_chip: None,
            });
        }
    }

    fn triple_damage_sender(mut writer: MessageWriter<DamageCell>) {
        for i in 0_i16..3 {
            writer.write(DamageCell {
                cell: Entity::PLACEHOLDER,
                damage: f32::from(i + 1),
                source_chip: None,
            });
        }
    }

    fn damage_and_bolt_lost_sender(
        mut damage_writer: MessageWriter<DamageCell>,
        mut bolt_lost_writer: MessageWriter<BoltLost>,
    ) {
        damage_writer.write(DamageCell {
            cell: Entity::PLACEHOLDER,
            damage: 5.0,
            source_chip: None,
        });
        bolt_lost_writer.write(BoltLost);
    }

    fn first_system(mut order: ResMut<Order>) {
        if !order.0.is_empty() {
            order.0.push(',');
        }
        order.0.push_str("first");
    }

    fn second_system(mut order: ResMut<Order>) {
        if !order.0.is_empty() {
            order.0.push(',');
        }
        order.0.push_str("second");
    }

    /// Helper: constructs a `BoltDefinition` with all required fields.
    fn make_bolt_definition(name: &str, base_speed: f32) -> BoltDefinition {
        BoltDefinition {
            name: name.to_string(),
            base_speed,
            min_speed: 200.0,
            max_speed: 800.0,
            radius: 8.0,
            base_damage: 10.0,
            effects: vec![],
            color_rgb: [6.0, 5.0, 0.5],
            min_angle_horizontal: 5.0,
            min_angle_vertical: 5.0,
            min_radius: None,
            max_radius: None,
        }
    }

    /// Helper: constructs a `CellTypeDefinition` with all required fields.
    fn make_cell_definition(alias: &str) -> CellTypeDefinition {
        CellTypeDefinition {
            id: alias.to_lowercase(),
            alias: alias.to_string(),
            toughness: Toughness::Standard,
            color_rgb: [1.0, 1.0, 1.0],
            required_to_clear: true,
            damage_hdr_base: 2.0,
            damage_green_min: 0.1,
            damage_blue_range: 0.5,
            damage_blue_base: 0.2,
            behaviors: None,
            effects: None,
        }
    }

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
        let app = TestAppBuilder::new().build();
        assert!(
            app.world()
                .get_resource::<MessageCollector<DamageCell>>()
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
    // Section C: State Navigation — in_state_node_playing()
    // ════════════════════════════════════════════════════════════════════

    // ── Behavior 6: in_state_node_playing() drives into NodeState::Playing ──

    #[test]
    fn in_state_node_playing_sets_app_state_to_game() {
        let app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .build();
        assert_eq!(
            *app.world().resource::<State<AppState>>().get(),
            AppState::Game,
            "in_state_node_playing() must set AppState::Game"
        );
    }

    #[test]
    fn in_state_node_playing_sets_game_state_to_run() {
        let app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .build();
        assert_eq!(
            *app.world().resource::<State<GameState>>().get(),
            GameState::Run,
            "in_state_node_playing() must set GameState::Run"
        );
    }

    #[test]
    fn in_state_node_playing_sets_run_state_to_node() {
        let app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .build();
        assert_eq!(
            *app.world().resource::<State<RunState>>().get(),
            RunState::Node,
            "in_state_node_playing() must set RunState::Node"
        );
    }

    #[test]
    fn in_state_node_playing_sets_node_state_to_playing() {
        let app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .build();
        assert_eq!(
            *app.world().resource::<State<NodeState>>().get(),
            NodeState::Playing,
            "in_state_node_playing() must set NodeState::Playing"
        );
    }

    #[test]
    fn in_state_node_playing_chip_select_state_not_present() {
        let app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .build();
        assert!(
            app.world()
                .get_resource::<State<ChipSelectState>>()
                .is_none(),
            "ChipSelectState should not exist when RunState is Node"
        );
    }

    // ── Behavior 7: state-gated system executes after in_state_node_playing ──

    #[test]
    fn state_gated_system_runs_in_node_playing() {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_resource::<Counter>()
            .with_system(FixedUpdate, increment.run_if(in_state(NodeState::Playing)))
            .build();
        tick(&mut app);
        assert_eq!(
            app.world().resource::<Counter>().0,
            1,
            "System gated on NodeState::Playing should execute after in_state_node_playing()"
        );
    }

    #[test]
    fn system_gated_on_wrong_state_does_not_run() {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_resource::<Counter>()
            .with_system(FixedUpdate, increment.run_if(in_state(NodeState::Loading)))
            .build();
        tick(&mut app);
        assert_eq!(
            app.world().resource::<Counter>().0,
            0,
            "System gated on NodeState::Loading should NOT run when in NodeState::Playing"
        );
    }

    // ════════════════════════════════════════════════════════════════════
    // Section D: State Navigation — in_state_chip_selecting()
    // ════════════════════════════════════════════════════════════════════

    // ── Behavior 8: in_state_chip_selecting() drives into ChipSelectState::Selecting ──

    #[test]
    fn in_state_chip_selecting_sets_app_state_to_game() {
        let app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_chip_selecting()
            .build();
        assert_eq!(
            *app.world().resource::<State<AppState>>().get(),
            AppState::Game,
            "in_state_chip_selecting() must set AppState::Game"
        );
    }

    #[test]
    fn in_state_chip_selecting_sets_game_state_to_run() {
        let app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_chip_selecting()
            .build();
        assert_eq!(
            *app.world().resource::<State<GameState>>().get(),
            GameState::Run,
            "in_state_chip_selecting() must set GameState::Run"
        );
    }

    #[test]
    fn in_state_chip_selecting_sets_run_state_to_chip_select() {
        let app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_chip_selecting()
            .build();
        assert_eq!(
            *app.world().resource::<State<RunState>>().get(),
            RunState::ChipSelect,
            "in_state_chip_selecting() must set RunState::ChipSelect"
        );
    }

    #[test]
    fn in_state_chip_selecting_sets_chip_select_state_to_selecting() {
        let app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_chip_selecting()
            .build();
        assert_eq!(
            *app.world().resource::<State<ChipSelectState>>().get(),
            ChipSelectState::Selecting,
            "in_state_chip_selecting() must set ChipSelectState::Selecting"
        );
    }

    #[test]
    fn in_state_chip_selecting_node_state_not_present() {
        let app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_chip_selecting()
            .build();
        assert!(
            app.world().get_resource::<State<NodeState>>().is_none(),
            "NodeState should not exist when RunState is ChipSelect"
        );
    }

    // ── Behavior 9: state-gated system executes after in_state_chip_selecting ──

    #[test]
    fn state_gated_system_runs_in_chip_selecting() {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_chip_selecting()
            .with_resource::<Counter>()
            .with_system(
                Update,
                increment.run_if(in_state(ChipSelectState::Selecting)),
            )
            .build();
        app.update();
        assert_eq!(
            app.world().resource::<Counter>().0,
            1,
            "System gated on ChipSelectState::Selecting should execute"
        );
    }

    #[test]
    fn system_gated_on_chip_select_loading_does_not_run() {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_chip_selecting()
            .with_resource::<Counter>()
            .with_system(Update, increment.run_if(in_state(ChipSelectState::Loading)))
            .build();
        app.update();
        assert_eq!(
            app.world().resource::<Counter>().0,
            0,
            "System gated on ChipSelectState::Loading should NOT run when in Selecting"
        );
    }

    // ── Behavior 9b: chaining navigations — last one wins ──

    #[test]
    fn chained_navigation_last_wins() {
        let app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .in_state_chip_selecting()
            .build();
        assert_eq!(
            *app.world().resource::<State<RunState>>().get(),
            RunState::ChipSelect,
            "Last navigation (chip_selecting) should win over earlier (node_playing)"
        );
        assert_eq!(
            *app.world().resource::<State<ChipSelectState>>().get(),
            ChipSelectState::Selecting,
        );
        assert!(
            app.world().get_resource::<State<NodeState>>().is_none(),
            "NodeState should not exist after navigating away from RunState::Node"
        );
    }

    // ════════════════════════════════════════════════════════════════════
    // Section E: with_physics()
    // ════════════════════════════════════════════════════════════════════

    // ── Behavior 10: with_physics() adds RantzPhysics2dPlugin ──

    #[test]
    fn with_physics_adds_collision_quadtree() {
        let app = TestAppBuilder::new().with_physics().build();
        assert!(
            app.world()
                .get_resource::<rantzsoft_physics2d::resources::CollisionQuadtree>()
                .is_some(),
            "with_physics() must add CollisionQuadtree resource"
        );
    }

    #[test]
    fn with_physics_works_with_state_hierarchy_and_navigation() {
        let app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_physics()
            .build();
        assert!(
            app.world()
                .get_resource::<rantzsoft_physics2d::resources::CollisionQuadtree>()
                .is_some(),
            "with_physics() should work alongside state hierarchy"
        );
    }

    // ════════════════════════════════════════════════════════════════════
    // Section F: with_playfield()
    // ════════════════════════════════════════════════════════════════════

    // ── Behavior 11: with_playfield() registers config resources ──

    #[test]
    fn with_playfield_registers_playfield_config() {
        let app = TestAppBuilder::new().with_playfield().build();
        let config = app.world().get_resource::<PlayfieldConfig>();
        assert!(
            config.is_some(),
            "with_playfield() must register PlayfieldConfig"
        );
        let config = config.unwrap();
        assert!(
            (config.width - 800.0).abs() < f32::EPSILON,
            "PlayfieldConfig default width should be 800.0, got {}",
            config.width
        );
        assert!(
            (config.height - 600.0).abs() < f32::EPSILON,
            "PlayfieldConfig default height should be 600.0, got {}",
            config.height
        );
    }

    #[test]
    fn with_playfield_registers_cell_config() {
        let app = TestAppBuilder::new().with_playfield().build();
        let config = app.world().get_resource::<CellConfig>();
        assert!(
            config.is_some(),
            "with_playfield() must register CellConfig"
        );
        let config = config.unwrap();
        assert!(
            (config.width - 70.0).abs() < f32::EPSILON,
            "CellConfig default width should be 70.0, got {}",
            config.width
        );
        assert!(
            (config.height - 24.0).abs() < f32::EPSILON,
            "CellConfig default height should be 24.0, got {}",
            config.height
        );
    }

    #[test]
    fn with_playfield_registers_mesh_and_color_material_assets() {
        let app = TestAppBuilder::new().with_playfield().build();
        assert!(
            app.world().get_resource::<Assets<Mesh>>().is_some(),
            "with_playfield() must register Assets<Mesh>"
        );
        assert!(
            app.world()
                .get_resource::<Assets<ColorMaterial>>()
                .is_some(),
            "with_playfield() must register Assets<ColorMaterial>"
        );
    }

    #[test]
    fn with_playfield_overwrite_with_insert_resource() {
        let app = TestAppBuilder::new()
            .with_playfield()
            .insert_resource(PlayfieldConfig {
                width: 400.0,
                ..Default::default()
            })
            .build();
        assert!(
            (app.world().resource::<PlayfieldConfig>().width - 400.0).abs() < f32::EPSILON,
            "insert_resource after with_playfield should overwrite PlayfieldConfig"
        );
    }

    // ════════════════════════════════════════════════════════════════════
    // Section G: with_resource() and insert_resource()
    // ════════════════════════════════════════════════════════════════════

    // ── Behavior 12: with_resource() initializes from Default ──

    #[test]
    fn with_resource_initializes_default() {
        let app = TestAppBuilder::new()
            .with_resource::<PlayfieldConfig>()
            .build();
        assert!(
            (app.world().resource::<PlayfieldConfig>().width - 800.0).abs() < f32::EPSILON,
            "with_resource::<PlayfieldConfig>() should init with Default (width 800.0)"
        );
    }

    #[test]
    fn with_resource_called_twice_does_not_panic() {
        let app = TestAppBuilder::new()
            .with_resource::<PlayfieldConfig>()
            .with_resource::<PlayfieldConfig>()
            .build();
        assert!(
            app.world().get_resource::<PlayfieldConfig>().is_some(),
            "calling with_resource twice should be idempotent"
        );
    }

    // ── Behavior 13: insert_resource() inserts concrete value ──

    #[test]
    fn insert_resource_inserts_concrete_value() {
        let app = TestAppBuilder::new()
            .insert_resource(PlayfieldConfig {
                width: 1920.0,
                height: 1080.0,
                ..Default::default()
            })
            .build();
        let config = app.world().resource::<PlayfieldConfig>();
        assert!(
            (config.width - 1920.0).abs() < f32::EPSILON,
            "insert_resource should set width to 1920.0, got {}",
            config.width
        );
        assert!(
            (config.height - 1080.0).abs() < f32::EPSILON,
            "insert_resource should set height to 1080.0, got {}",
            config.height
        );
    }

    #[test]
    fn insert_resource_overwrites_with_resource() {
        let app = TestAppBuilder::new()
            .with_resource::<PlayfieldConfig>()
            .insert_resource(PlayfieldConfig {
                width: 42.0,
                ..Default::default()
            })
            .build();
        assert!(
            (app.world().resource::<PlayfieldConfig>().width - 42.0).abs() < f32::EPSILON,
            "insert_resource after with_resource should overwrite to 42.0"
        );
    }

    // ════════════════════════════════════════════════════════════════════
    // Section H: with_message()
    // ════════════════════════════════════════════════════════════════════

    // ── Behavior 14: with_message() registers a message type ──

    /// Helper: system that reads `DamageCell` messages and counts them in a resource.
    #[derive(Resource, Default)]
    struct DamageCount(usize);

    fn count_damage_messages(
        mut reader: MessageReader<DamageCell>,
        mut count: ResMut<DamageCount>,
    ) {
        for _msg in reader.read() {
            count.0 += 1;
        }
    }

    fn damage_sender_10(mut writer: MessageWriter<DamageCell>) {
        writer.write(DamageCell {
            cell: Entity::PLACEHOLDER,
            damage: 10.0,
            source_chip: None,
        });
    }

    #[test]
    fn with_message_enables_message_send_and_read() {
        let mut app = TestAppBuilder::new()
            .with_message::<DamageCell>()
            .with_resource::<DamageCount>()
            .with_system(FixedUpdate, damage_sender_10)
            .with_system(FixedUpdate, count_damage_messages.after(damage_sender_10))
            .build();
        tick(&mut app);
        assert_eq!(
            app.world().resource::<DamageCount>().0,
            1,
            "with_message should enable sending and reading DamageCell messages"
        );
    }

    #[test]
    fn with_message_does_not_add_collector() {
        let app = TestAppBuilder::new().with_message::<DamageCell>().build();
        assert!(
            app.world()
                .get_resource::<MessageCollector<DamageCell>>()
                .is_none(),
            "with_message should NOT add a MessageCollector"
        );
    }

    // ════════════════════════════════════════════════════════════════════
    // Section I: MessageCollector and with_message_capture()
    // ════════════════════════════════════════════════════════════════════

    // ── Behavior 15: with_message_capture() registers collector ──

    #[test]
    fn with_message_capture_registers_collector() {
        let app = TestAppBuilder::new()
            .with_message_capture::<DamageCell>()
            .build();
        let collector = app.world().get_resource::<MessageCollector<DamageCell>>();
        assert!(
            collector.is_some(),
            "with_message_capture must register MessageCollector<DamageCell>"
        );
        assert_eq!(
            collector.unwrap().0.len(),
            0,
            "MessageCollector should start empty"
        );
    }

    // ── Behavior 16: MessageCollector captures messages ──

    #[test]
    fn message_collector_captures_messages_during_tick() {
        let mut app = TestAppBuilder::new()
            .with_message_capture::<DamageCell>()
            .with_system(FixedUpdate, damage_sender_system)
            .build();
        tick(&mut app);
        let collector = app.world().resource::<MessageCollector<DamageCell>>();
        assert_eq!(
            collector.0.len(),
            1,
            "MessageCollector should capture 1 message after tick"
        );
        assert!(
            (collector.0[0].damage - 25.0).abs() < f32::EPSILON,
            "Captured message damage should be 25.0, got {}",
            collector.0[0].damage
        );
    }

    #[test]
    fn message_collector_captures_multiple_messages_per_tick() {
        let mut app = TestAppBuilder::new()
            .with_message_capture::<DamageCell>()
            .with_system(FixedUpdate, triple_damage_sender)
            .build();
        tick(&mut app);
        assert_eq!(
            app.world()
                .resource::<MessageCollector<DamageCell>>()
                .0
                .len(),
            3,
            "MessageCollector should capture all 3 messages sent in one tick"
        );
    }

    // ── Behavior 17: MessageCollector auto-clears at start of each tick ──

    #[test]
    fn message_collector_auto_clears_between_ticks() {
        let mut app = TestAppBuilder::new()
            .with_message_capture::<DamageCell>()
            .insert_resource(ShouldSend(true))
            .with_system(FixedUpdate, conditional_damage_sender)
            .build();

        // First tick — flag is true, 1 message
        tick(&mut app);
        assert_eq!(
            app.world()
                .resource::<MessageCollector<DamageCell>>()
                .0
                .len(),
            1,
            "First tick: should have 1 message"
        );

        // Second tick — flag is false, no messages sent
        app.world_mut().resource_mut::<ShouldSend>().0 = false;
        tick(&mut app);
        assert_eq!(
            app.world()
                .resource::<MessageCollector<DamageCell>>()
                .0
                .len(),
            0,
            "Second tick: auto-clear should empty collector when no messages sent"
        );

        // Third tick — flag back to true
        app.world_mut().resource_mut::<ShouldSend>().0 = true;
        tick(&mut app);
        assert_eq!(
            app.world()
                .resource::<MessageCollector<DamageCell>>()
                .0
                .len(),
            1,
            "Third tick: collector should have 1 message again (clear-then-collect repeatable)"
        );
    }

    // ── Behavior 18: MessageCollector::clear() manual reset ──

    #[test]
    fn message_collector_manual_clear() {
        let mut app = TestAppBuilder::new()
            .with_message_capture::<DamageCell>()
            .with_system(FixedUpdate, damage_sender_system)
            .build();
        tick(&mut app);
        assert_eq!(
            app.world()
                .resource::<MessageCollector<DamageCell>>()
                .0
                .len(),
            1,
        );

        app.world_mut()
            .resource_mut::<MessageCollector<DamageCell>>()
            .clear();
        assert_eq!(
            app.world()
                .resource::<MessageCollector<DamageCell>>()
                .0
                .len(),
            0,
            "clear() should empty the collector"
        );
    }

    #[test]
    fn message_collector_clear_on_empty_does_not_panic() {
        let mut app = TestAppBuilder::new()
            .with_message_capture::<DamageCell>()
            .build();
        app.world_mut()
            .resource_mut::<MessageCollector<DamageCell>>()
            .clear();
    }

    // ── Behavior 19: Multiple collectors coexist ──

    #[test]
    fn multiple_message_collectors_coexist() {
        let mut app = TestAppBuilder::new()
            .with_message_capture::<DamageCell>()
            .with_message_capture::<BoltLost>()
            .with_system(FixedUpdate, damage_and_bolt_lost_sender)
            .build();
        tick(&mut app);
        assert_eq!(
            app.world()
                .resource::<MessageCollector<DamageCell>>()
                .0
                .len(),
            1,
            "DamageCell collector should have 1 message"
        );
        assert_eq!(
            app.world().resource::<MessageCollector<BoltLost>>().0.len(),
            1,
            "BoltLost collector should have 1 message"
        );
    }

    #[test]
    fn clearing_one_collector_does_not_affect_other() {
        let mut app = TestAppBuilder::new()
            .with_message_capture::<DamageCell>()
            .with_message_capture::<BoltLost>()
            .with_system(FixedUpdate, damage_and_bolt_lost_sender)
            .build();
        tick(&mut app);

        app.world_mut()
            .resource_mut::<MessageCollector<DamageCell>>()
            .clear();
        assert_eq!(
            app.world()
                .resource::<MessageCollector<DamageCell>>()
                .0
                .len(),
            0,
            "DamageCell collector should be empty after clear"
        );
        assert_eq!(
            app.world().resource::<MessageCollector<BoltLost>>().0.len(),
            1,
            "BoltLost collector should be unaffected by clearing DamageCell"
        );
    }

    // ── Behavior 20: accumulation pattern across ticks ──

    #[test]
    fn message_collector_per_tick_isolation() {
        let mut app = TestAppBuilder::new()
            .with_message_capture::<DamageCell>()
            .with_system(FixedUpdate, damage_sender_system)
            .build();

        let mut running_total = 0usize;
        for i in 1..=3 {
            tick(&mut app);
            let count = app
                .world()
                .resource::<MessageCollector<DamageCell>>()
                .0
                .len();
            running_total += count;
            assert_eq!(
                count, 1,
                "Tick {i}: collector should have exactly 1 message (auto-clear isolates ticks)"
            );
        }
        assert_eq!(running_total, 3, "Running total over 3 ticks should be 3");
    }

    // ── Behavior 20b: with_message_capture called twice is idempotent ──

    #[test]
    fn with_message_capture_twice_is_idempotent() {
        let mut app = TestAppBuilder::new()
            .with_message_capture::<DamageCell>()
            .with_message_capture::<DamageCell>()
            .with_system(FixedUpdate, damage_sender_system)
            .build();
        tick(&mut app);
        assert_eq!(
            app.world()
                .resource::<MessageCollector<DamageCell>>()
                .0
                .len(),
            1,
            "Double with_message_capture should not duplicate messages (len should be 1, not 2)"
        );
    }

    #[test]
    fn with_message_then_message_capture_does_not_panic() {
        let mut app = TestAppBuilder::new()
            .with_message::<DamageCell>()
            .with_message_capture::<DamageCell>()
            .with_system(FixedUpdate, damage_sender_system)
            .build();
        tick(&mut app);
        assert_eq!(
            app.world()
                .resource::<MessageCollector<DamageCell>>()
                .0
                .len(),
            1,
            "with_message followed by with_message_capture should capture messages normally"
        );
    }

    // ════════════════════════════════════════════════════════════════════
    // Section J: Registry Methods — Bolt
    // ════════════════════════════════════════════════════════════════════

    // ── Behavior 21: with_bolt_registry() creates empty registry ──

    #[test]
    fn with_bolt_registry_creates_empty_registry() {
        let app = TestAppBuilder::new().with_bolt_registry().build();
        let registry = app.world().get_resource::<BoltRegistry>();
        assert!(
            registry.is_some(),
            "with_bolt_registry() must register BoltRegistry"
        );
        assert!(
            registry.unwrap().is_empty(),
            "BoltRegistry should start empty"
        );
    }

    #[test]
    fn with_bolt_registry_twice_does_not_panic() {
        let app = TestAppBuilder::new()
            .with_bolt_registry()
            .with_bolt_registry()
            .build();
        assert!(app.world().get_resource::<BoltRegistry>().is_some());
    }

    // ── Behavior 22: with_bolt_registry_entry() inserts definition ──

    #[test]
    fn with_bolt_registry_entry_creates_registry_and_inserts() {
        let def = make_bolt_definition("Bolt", 400.0);
        let app = TestAppBuilder::new()
            .with_bolt_registry_entry("Bolt", def)
            .build();
        let registry = app.world().resource::<BoltRegistry>();
        assert!(
            registry.get("Bolt").is_some(),
            "BoltRegistry should contain 'Bolt' after with_bolt_registry_entry"
        );
        assert!(
            (registry.get("Bolt").unwrap().base_speed - 400.0).abs() < f32::EPSILON,
            "Bolt base_speed should be 400.0"
        );
    }

    #[test]
    fn with_bolt_registry_entry_multiple_entries() {
        let def_a = make_bolt_definition("A", 300.0);
        let def_b = make_bolt_definition("B", 500.0);
        let app = TestAppBuilder::new()
            .with_bolt_registry_entry("A", def_a)
            .with_bolt_registry_entry("B", def_b)
            .build();
        let registry = app.world().resource::<BoltRegistry>();
        assert_eq!(
            registry.len(),
            2,
            "Registry should have 2 entries after inserting A and B"
        );
    }

    // ── Behavior 23: with_bolt_registry_entry() overwrites same name ──

    #[test]
    fn with_bolt_registry_entry_overwrites_same_name() {
        let def1 = make_bolt_definition("Bolt", 400.0);
        let def2 = make_bolt_definition("Bolt", 600.0);
        let app = TestAppBuilder::new()
            .with_bolt_registry_entry("Bolt", def1)
            .with_bolt_registry_entry("Bolt", def2)
            .build();
        let registry = app.world().resource::<BoltRegistry>();
        assert!(
            (registry.get("Bolt").unwrap().base_speed - 600.0).abs() < f32::EPSILON,
            "Second entry should overwrite first (base_speed should be 600.0)"
        );
        assert_eq!(
            registry.len(),
            1,
            "Registry should have 1 entry (not 2) after overwrite"
        );
    }

    // ════════════════════════════════════════════════════════════════════
    // Section K: Registry Methods — Breaker
    // ════════════════════════════════════════════════════════════════════

    // ── Behavior 24: with_breaker_registry() creates empty registry ──

    #[test]
    fn with_breaker_registry_creates_empty_registry() {
        let app = TestAppBuilder::new().with_breaker_registry().build();
        let registry = app.world().get_resource::<BreakerRegistry>();
        assert!(
            registry.is_some(),
            "with_breaker_registry() must register BreakerRegistry"
        );
        assert!(
            registry.unwrap().is_empty(),
            "BreakerRegistry should start empty"
        );
    }

    // ── Behavior 25: with_breaker_registry_entry() inserts definition ──

    #[test]
    fn with_breaker_registry_entry_creates_and_inserts() {
        let def = BreakerDefinition {
            name: "Aegis".to_string(),
            life_pool: None,
            effects: vec![],
            ..Default::default()
        };
        let app = TestAppBuilder::new()
            .with_breaker_registry_entry("Aegis", def)
            .build();
        let registry = app.world().resource::<BreakerRegistry>();
        assert!(
            registry.get("Aegis").is_some(),
            "BreakerRegistry should contain 'Aegis' after with_breaker_registry_entry"
        );
    }

    #[test]
    fn with_breaker_registry_entry_without_prior_registry() {
        // Calling entry without with_breaker_registry should auto-create
        let def = BreakerDefinition {
            name: "Vortex".to_string(),
            ..Default::default()
        };
        let app = TestAppBuilder::new()
            .with_breaker_registry_entry("Vortex", def)
            .build();
        assert!(
            app.world()
                .resource::<BreakerRegistry>()
                .get("Vortex")
                .is_some(),
            "with_breaker_registry_entry should auto-create the registry"
        );
    }

    // ════════════════════════════════════════════════════════════════════
    // Section L: Registry Methods — Cell
    // ════════════════════════════════════════════════════════════════════

    // ── Behavior 26: with_cell_registry() creates empty registry ──

    #[test]
    fn with_cell_registry_creates_empty_registry() {
        let app = TestAppBuilder::new().with_cell_registry().build();
        let registry = app.world().get_resource::<CellTypeRegistry>();
        assert!(
            registry.is_some(),
            "with_cell_registry() must register CellTypeRegistry"
        );
        assert_eq!(
            registry.unwrap().len(),
            0,
            "CellTypeRegistry should start with 0 entries"
        );
    }

    // ── Behavior 27: with_cell_registry_entry() inserts definition ──

    #[test]
    fn with_cell_registry_entry_creates_and_inserts() {
        let def = make_cell_definition("S");
        let app = TestAppBuilder::new()
            .with_cell_registry_entry("S", def)
            .build();
        let registry = app.world().resource::<CellTypeRegistry>();
        assert!(
            registry.get("S").is_some(),
            "CellTypeRegistry should contain 'S' after with_cell_registry_entry"
        );
    }

    #[test]
    fn with_cell_registry_entry_multiple_aliases() {
        let def_s = make_cell_definition("S");
        let def_t = make_cell_definition("T");
        let app = TestAppBuilder::new()
            .with_cell_registry_entry("S", def_s)
            .with_cell_registry_entry("T", def_t)
            .build();
        let registry = app.world().resource::<CellTypeRegistry>();
        assert!(registry.get("S").is_some(), "should contain S");
        assert!(registry.get("T").is_some(), "should contain T");
    }

    // ════════════════════════════════════════════════════════════════════
    // Section M: with_system()
    // ════════════════════════════════════════════════════════════════════

    // ── Behavior 28: with_system() adds system to schedule ──

    #[test]
    fn with_system_adds_fixed_update_system() {
        let mut app = TestAppBuilder::new()
            .with_resource::<Counter>()
            .with_system(FixedUpdate, increment)
            .build();
        tick(&mut app);
        assert_eq!(
            app.world().resource::<Counter>().0,
            1,
            "System added via with_system(FixedUpdate, ...) should run on tick()"
        );
    }

    #[test]
    fn with_system_update_schedule() {
        let mut app = TestAppBuilder::new()
            .with_resource::<Counter>()
            .with_system(Update, increment)
            .build();
        app.update();
        assert_eq!(
            app.world().resource::<Counter>().0,
            1,
            "System added to Update should run on app.update()"
        );
    }

    // ── Behavior 29: with_system() supports ordering ──

    #[test]
    fn with_system_ordering_after() {
        let mut app = TestAppBuilder::new()
            .with_resource::<Order>()
            .with_system(
                FixedUpdate,
                (first_system, second_system.after(first_system)),
            )
            .build();
        tick(&mut app);
        assert_eq!(
            app.world().resource::<Order>().0,
            "first,second",
            "second_system should run after first_system"
        );
    }

    #[test]
    fn with_system_ordering_reversed() {
        let mut app = TestAppBuilder::new()
            .with_resource::<Order>()
            .with_system(
                FixedUpdate,
                (first_system.after(second_system), second_system),
            )
            .build();
        tick(&mut app);
        assert_eq!(
            app.world().resource::<Order>().0,
            "second,first",
            "first_system.after(second_system) should reverse execution order"
        );
    }

    // ════════════════════════════════════════════════════════════════════
    // Section N: build()
    // ════════════════════════════════════════════════════════════════════

    // ── Behavior 30: build() without state navigation doesn't run systems ──

    #[test]
    fn build_without_update_does_not_run_systems() {
        let app = TestAppBuilder::new().with_resource::<Counter>().build();
        assert_eq!(
            app.world().resource::<Counter>().0,
            0,
            "No systems should have run — counter should be 0"
        );
    }

    // ── Behavior 31: build() after state navigation has app in target state ──

    #[test]
    fn build_after_state_navigation_is_in_target_state_immediately() {
        let app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .build();
        // Immediately after build — no additional app.update() or tick()
        assert_eq!(
            *app.world().resource::<State<NodeState>>().get(),
            NodeState::Playing,
            "After build(), app should already be in NodeState::Playing without extra update"
        );
    }

    // ════════════════════════════════════════════════════════════════════
    // Section O: tick()
    // ════════════════════════════════════════════════════════════════════

    // ── Behavior 32: tick() advances exactly one FixedUpdate timestep ──

    #[test]
    fn tick_advances_one_fixed_update() {
        let mut app = TestAppBuilder::new()
            .with_resource::<Counter>()
            .with_system(FixedUpdate, increment)
            .build();
        tick(&mut app);
        assert_eq!(
            app.world().resource::<Counter>().0,
            1,
            "tick() should advance exactly one FixedUpdate (counter = 1)"
        );
    }

    #[test]
    fn tick_five_times_increments_five() {
        let mut app = TestAppBuilder::new()
            .with_resource::<Counter>()
            .with_system(FixedUpdate, increment)
            .build();
        for _ in 0..5 {
            tick(&mut app);
        }
        assert_eq!(
            app.world().resource::<Counter>().0,
            5,
            "5 ticks should increment counter to 5"
        );
    }

    // ── Behavior 33: tick() reads configured timestep (not hardcoded) ──

    #[test]
    fn tick_reads_app_timestep_not_hardcoded() {
        let mut app = TestAppBuilder::new()
            .with_resource::<Counter>()
            .with_system(FixedUpdate, increment)
            .build();
        // Change the timestep to a non-default value
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .set_timestep(std::time::Duration::from_millis(100));
        // tick() should still advance exactly one FixedUpdate (it reads from the resource)
        tick(&mut app);
        assert_eq!(
            app.world().resource::<Counter>().0,
            1,
            "tick() should trigger exactly one FixedUpdate even with a custom timestep"
        );
    }

    #[test]
    fn raw_update_without_overstep_does_not_run_fixed_update() {
        let mut app = TestAppBuilder::new()
            .with_resource::<Counter>()
            .with_system(FixedUpdate, increment)
            .build();
        // Calling app.update() directly without accumulating overstep
        app.update();
        assert_eq!(
            app.world().resource::<Counter>().0,
            0,
            "app.update() without overstep accumulation should NOT run FixedUpdate"
        );
    }

    // ── Behavior 34b: tick() on systemless app does not panic ──

    #[test]
    fn tick_on_empty_app_does_not_panic() {
        let mut app = TestAppBuilder::new().build();
        tick(&mut app);
        // No panic = pass
    }

    #[test]
    fn tick_multiple_times_on_empty_app_does_not_panic() {
        let mut app = TestAppBuilder::new().build();
        for _ in 0..3 {
            tick(&mut app);
        }
        // No panic = pass
    }

    // ════════════════════════════════════════════════════════════════════
    // Section P: Builder Method Chaining
    // ════════════════════════════════════════════════════════════════════

    // ── Behavior 35: all builder methods chain fluently ──

    #[test]
    fn maximal_builder_chain_compiles_and_builds() {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_physics()
            .with_playfield()
            .with_resource::<GameRng>()
            .insert_resource(PlayfieldConfig {
                width: 1920.0,
                ..Default::default()
            })
            .with_message::<BoltImpactCell>()
            .with_message_capture::<DamageCell>()
            .with_bolt_registry()
            .with_breaker_registry()
            .with_cell_registry()
            .with_system(FixedUpdate, increment)
            .build();

        // Verify critical resources
        assert!(
            app.world().get_resource::<State<NodeState>>().is_some(),
            "State<NodeState> should be present in maximal chain"
        );
        assert!(
            app.world().get_resource::<PlayfieldConfig>().is_some(),
            "PlayfieldConfig should be present"
        );
        assert!(
            (app.world().resource::<PlayfieldConfig>().width - 1920.0).abs() < f32::EPSILON,
            "insert_resource should have overridden with_playfield's default to 1920.0"
        );
        assert!(
            app.world().get_resource::<GameRng>().is_some(),
            "GameRng should be present"
        );
        assert!(
            app.world().get_resource::<BoltRegistry>().is_some(),
            "BoltRegistry should be present"
        );
        assert!(
            app.world().get_resource::<BreakerRegistry>().is_some(),
            "BreakerRegistry should be present"
        );
        assert!(
            app.world().get_resource::<CellTypeRegistry>().is_some(),
            "CellTypeRegistry should be present"
        );
        assert!(
            app.world()
                .get_resource::<MessageCollector<DamageCell>>()
                .is_some(),
            "MessageCollector<DamageCell> should be present"
        );

        // Verify the app actually works
        app.insert_resource(Counter::default());
        tick(&mut app);
    }
}
