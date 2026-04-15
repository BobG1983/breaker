//! Typestate builder for test `App` instances.

use std::marker::PhantomData;

use bevy::{
    ecs::schedule::{IntoScheduleConfigs, ScheduleLabel},
    prelude::*,
};
use rantzsoft_physics2d::plugin::RantzPhysics2dPlugin;

use super::collector::{MessageCollector, clear_messages, collect_messages};
use crate::{
    bolt::{definition::BoltDefinition, registry::BoltRegistry},
    breaker::{definition::BreakerDefinition, registry::BreakerRegistry},
    cells::{
        definition::CellTypeDefinition,
        resources::{CellConfig, CellTypeRegistry},
    },
    effect_v3::EffectV3Plugin,
    shared::{
        PlayfieldConfig,
        death_pipeline::{
            DeathPipelinePlugin, systems::tests::helpers::register_effect_v3_test_infrastructure,
        },
    },
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
    app:    App,
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
            app:    self.app,
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

    /// Registers the full effects pipeline: `DeathPipelinePlugin`,
    /// cross-domain messages + `GameRng`, and `EffectV3Plugin`.
    ///
    /// Order matters: `DeathPipelinePlugin` configures sets that
    /// `EffectV3Plugin` references, and `register_effect_v3_test_infrastructure`
    /// registers messages that both plugins' systems read.
    #[must_use]
    pub(crate) fn with_effects_pipeline(mut self) -> Self {
        self.app.add_plugins(DeathPipelinePlugin);
        register_effect_v3_test_infrastructure(&mut self.app);
        self.app.add_plugins(EffectV3Plugin);
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
