//! Typestate builder for test `App` instances.

use std::{marker::PhantomData, time::Duration};

use bevy::{
    audio::AudioPlugin,
    ecs::schedule::{IntoScheduleConfigs, ScheduleLabel},
    gilrs::GilrsPlugin,
    log::LogPlugin,
    prelude::*,
    render::{
        RenderPlugin,
        settings::{RenderCreation, WgpuSettings},
    },
    time::TimeUpdateStrategy,
    winit::WinitPlugin,
};
use rantzsoft_dmg::{RantzDmgAppExt, RantzDmgPlugin};
use rantzsoft_physics2d::plugin::RantzPhysics2dPlugin;

use super::{
    collector::{MessageCollector, clear_messages, collect_messages},
    effect_v3_infra::register_effect_v3_test_infrastructure,
};
use crate::{
    bolt::{components::Bolt, definition::BoltDefinition, registry::BoltRegistry},
    breaker::{components::Breaker, definition::BreakerDefinition, registry::BreakerRegistry},
    cells::{
        behaviors::survival::salvo::components::Salvo,
        components::Cell,
        definition::CellTypeDefinition,
        resources::{CellConfig, CellTypeRegistry},
    },
    effect_v3::EffectV3Plugin,
    shared::PlayfieldConfig,
    state::types::*,
    walls::components::Wall,
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

// ── Damage-pipeline typestate markers (W2) ──────────────────────────────────

/// Marker trait for the Damage pipeline dimension of `TestAppBuilder` typestate.
///
/// Mirrors [`StateStatus`]: `NoDmg` is the initial state, `WithDmg` is the state
/// after `with_dmg_pipeline()` (or `with_effects_pipeline()`) runs. Methods that
/// require a wired damage pipeline (e.g. `register_dmgable::<T>()`) live only on
/// the `WithDmg` impl block.
pub(crate) trait DmgStatus {}

/// Initial Dmg typestate — no `RantzDmgPlugin` installed.
pub(crate) struct NoDmg;
impl DmgStatus for NoDmg {}

/// After `with_dmg_pipeline()` or `with_effects_pipeline()` — damage pipeline
/// methods (like `register_dmgable`) are available.
pub(crate) struct WithDmg;
impl DmgStatus for WithDmg {}

// ── TestAppBuilder ─────────────────────────────────────────────────────────

/// Typestate builder for test `App` instances.
///
/// # W2 Behaviors 49/50 negative typestate contracts
///
/// The typestate enforces these at compile time via impl-block gating:
///
/// - `with_dmg_pipeline` lives on `impl<S> TestAppBuilder<S, NoDmg>` only.
///   Double-call `TestAppBuilder::new().with_dmg_pipeline().with_dmg_pipeline()`
///   fails to compile because the second call is on `<S, WithDmg>`, which has
///   no `with_dmg_pipeline` method.
/// - `register_dmgable` lives on `impl<S> TestAppBuilder<S, WithDmg>` only.
///   `TestAppBuilder::new().register_dmgable::<T>()` fails to compile because
///   the builder is `<NoStates, NoDmg>` there.
///
/// These negative contracts are exercised by the `structural_invariants`
/// module which greps for the expected impl-block shape.
pub(crate) struct TestAppBuilder<S: StateStatus = NoStates, D: DmgStatus = NoDmg> {
    app:    App,
    _state: PhantomData<S>,
    _dmg:   PhantomData<D>,
}

impl TestAppBuilder<NoStates, NoDmg> {
    /// Creates a new builder with headless `DefaultPlugins` registered and
    /// Bevy's `TimeUpdateStrategy` pinned to `ManualDuration(Duration::ZERO)`.
    ///
    /// Headless `DefaultPlugins` matches production more closely than
    /// `MinimalPlugins`: `StatesPlugin`, `AssetPlugin`, `InputPlugin`,
    /// `RenderPlugin`, `SpritePlugin`, etc. are all registered, so tests
    /// don't have to manually wire each one. `RenderPlugin` is configured
    /// with `backends: None` so no GPU is initialised, and `WinitPlugin` is
    /// disabled so no event loop / window is created.
    ///
    /// Pinning the time-update strategy is load-bearing for parallel-test
    /// determinism. `DefaultPlugins` registers `TimeUpdateStrategy::Automatic`
    /// by default, which calls `Instant::now()` on every `app.update()` and
    /// feeds wall-clock elapsed time into `Time<Fixed>::overstep`. Under
    /// parallel test load, the 4 `app.update()` calls in `in_state_*`
    /// transitions accumulate measurable overstep before the test fires its
    /// first `tick()`; `run_fixed_main_schedule` then drains that stray
    /// overstep by running `FixedUpdate` extra times. This manifests as the
    /// drift test reading `Vec2(1.1..1.4, 0.0)` instead of `(+0.1, 0.0)`, and
    /// as siphon `window_remaining` off by `1/64s`.
    ///
    /// `ManualDuration(Duration::ZERO)` pins virtual-time delta to zero per
    /// update, so real wall-clock time cannot leak into the fixed overstep.
    /// Test helpers remain the sole source of fixed-time advancement via
    /// `accumulate_overstep(dt)`.
    #[must_use]
    pub(crate) fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(
            DefaultPlugins
                .set(RenderPlugin {
                    synchronous_pipeline_compilation: true,
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: None,
                        ..default()
                    }),
                    ..default()
                })
                // No event loop / window in tests.
                .disable::<WinitPlugin>()
                // GilrsPlugin opens the host gamepad subsystem and panics in
                // test environments lacking a controller daemon (CI, sandbox).
                .disable::<GilrsPlugin>()
                // AudioPlugin tries to open an audio device.
                .disable::<AudioPlugin>()
                // LogPlugin installs a global tracing subscriber; the second
                // parallel test in a process panics on "global logger already
                // set".
                .disable::<LogPlugin>(),
        );
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        Self {
            app,
            _state: PhantomData,
            _dmg: PhantomData,
        }
    }
}

impl<D: DmgStatus> TestAppBuilder<NoStates, D> {
    /// Registers the full state hierarchy (`AppState` + all sub-states).
    /// `StatesPlugin` itself is provided by `DefaultPlugins` (added in `new`).
    #[must_use]
    pub(crate) fn with_state_hierarchy(mut self) -> TestAppBuilder<WithStates, D> {
        self.app.init_state::<AppState>();
        self.app.add_sub_state::<GameState>();
        self.app.add_sub_state::<MenuState>();
        self.app.add_sub_state::<RunState>();
        self.app.add_sub_state::<NodeState>();
        self.app.add_sub_state::<ChipSelectState>();
        self.app.add_sub_state::<HazardSelectState>();
        self.app.add_sub_state::<RunEndState>();
        TestAppBuilder {
            app:    self.app,
            _state: PhantomData,
            _dmg:   PhantomData,
        }
    }
}

impl<D: DmgStatus> TestAppBuilder<WithStates, D> {
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

    /// Drives the app into `HazardSelectState::Selecting` via four transitions:
    /// `AppState::Game` → `GameState::Run` → `RunState::HazardSelect` → `HazardSelectState::Selecting`.
    /// Each step sets `NextState` and calls `app.update()`.
    #[must_use]
    pub(crate) fn in_state_hazard_selecting(mut self) -> Self {
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
            .set(RunState::HazardSelect);
        self.app.update();
        self.app
            .world_mut()
            .resource_mut::<NextState<HazardSelectState>>()
            .set(HazardSelectState::Selecting);
        self.app.update();
        self
    }
}

impl<S: StateStatus, D: DmgStatus> TestAppBuilder<S, D> {
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

// ── W2 Dmg-pipeline impls ────────────────────────────────────────────────

fn register_standard_dmgables(app: &mut App) {
    let _ = app
        .register_dmgable::<Bolt>()
        .register_dmgable::<Wall>()
        .register_dmgable::<Breaker>()
        .register_dmgable::<Salvo>()
        .register_dmgable::<Cell>();
}

impl<S: StateStatus> TestAppBuilder<S, NoDmg> {
    /// Installs `RantzDmgPlugin` and registers the standard dmgable types
    /// (`Bolt`, `Wall`, `Breaker`, `Salvo`, `Cell`), transitioning the
    /// typestate to `WithDmg`. After this call, `DamageDealt<T>` and related
    /// messages are initialized for all standard dmgable types.
    #[must_use]
    pub(crate) fn with_dmg_pipeline(mut self) -> TestAppBuilder<S, WithDmg> {
        self.app.add_plugins(RantzDmgPlugin);
        register_standard_dmgables(&mut self.app);
        TestAppBuilder {
            app:    self.app,
            _state: PhantomData,
            _dmg:   PhantomData,
        }
    }

    /// Full effects pipeline: `RantzDmgPlugin`, per-`T` registrations
    /// (`Bolt`, `Wall`, `Breaker`, `Salvo`, `Cell`), and `EffectV3Plugin`.
    /// Transitions typestate to `WithDmg`.
    #[must_use]
    pub(crate) fn with_effects_pipeline(mut self) -> TestAppBuilder<S, WithDmg> {
        self.app.add_plugins(RantzDmgPlugin);
        register_standard_dmgables(&mut self.app);
        register_effect_v3_test_infrastructure(&mut self.app);
        self.app.add_plugins(EffectV3Plugin);
        TestAppBuilder {
            app:    self.app,
            _state: PhantomData,
            _dmg:   PhantomData,
        }
    }

    /// Preset for protocol scheduling tests: bundles the standard physics +
    /// playfield + registries + protocol/input resources + effects pipeline
    /// chain that every protocol's `*_scheduling_app()` builds. Callers add
    /// the protocol-specific config + `register(...)` themselves.
    ///
    /// Callers must establish state hierarchy first
    /// (`.with_state_hierarchy().in_state_node_playing()`) — protocol
    /// `register(...)` systems are gated on `NodeState::Playing` and require
    /// the state graph to exist.
    ///
    /// Equivalent to:
    /// ```ignore
    /// .with_physics()
    /// .with_playfield()
    /// .with_bolt_registry()
    /// .with_breaker_registry()
    /// .with_cell_registry()
    /// .with_resource::<ActiveProtocols>()
    /// .with_resource::<InputActions>()
    /// .with_effects_pipeline()
    /// ```
    #[must_use]
    pub(crate) fn with_protocol_scaffolding(self) -> TestAppBuilder<S, WithDmg> {
        self.with_physics()
            .with_playfield()
            .with_bolt_registry()
            .with_breaker_registry()
            .with_cell_registry()
            .with_resource::<crate::mutators::protocols::resources::ActiveProtocols>()
            .with_resource::<crate::input::resources::InputActions>()
            .with_effects_pipeline()
    }
}

// ── Cross-domain test-system bridges ──────────────────────────────────────

/// Registers `update_previous_dash_state` and `handle_bolt_lost` in their
/// `BreakerSystems` sets. Use when building a test app that needs the
/// bolt-loss and dash-state-tracking pipeline without the full `BreakerPlugin`
/// overhead.
///
/// Lives here (not in a domain-specific test module) because cross-domain
/// tests must not import bare function pointers from `breaker::systems` —
/// `shared::test_utils` is the accepted bridge for that wiring.
#[cfg(test)]
pub(crate) fn add_breaker_transition_systems(app: &mut App) {
    use crate::breaker::{
        sets::BreakerSystems,
        systems::{handle_bolt_lost, update_previous_dash_state},
    };
    app.add_systems(
        FixedUpdate,
        (
            update_previous_dash_state
                .in_set(BreakerSystems::UpdatePreviousState)
                .after(BreakerSystems::UpdateState),
            handle_bolt_lost.in_set(BreakerSystems::HandleBoltLost),
        ),
    );
}

#[cfg(test)]
mod tests {
    // Behavior 7 (FAILS at RED): builder.rs doc-comment for `with_effects_pipeline`
    // must not claim it registers the old monolithic RNG type — after Wave 2D cleanup
    // of effect_v3_infra.rs, the builder no longer inserts it, making the docstring
    // stale. Writer-code removes the stale phrase from the comment at GREEN.
    //
    // Self-referential guard: concat! split to avoid detecting this comment.
    const FORBIDDEN_DOC: &str = concat!("cross-domain `Game", "Rng`");

    #[test]
    fn builder_rs_with_effects_pipeline_doc_does_not_claim_game_rng_registration() {
        let source = include_str!("builder.rs");
        assert!(
            !source.contains(FORBIDDEN_DOC),
            "shared/test_utils/builder.rs: the `with_effects_pipeline` docstring must not \
             claim the old monolithic RNG type is registered — after Wave 2D cleanup of \
             effect_v3_infra.rs, `register_effect_v3_test_infrastructure` no longer inserts it"
        );
    }
}
