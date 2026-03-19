//! Scenario lifecycle — state navigation, frame counter, and exit logic.
//!
//! [`ScenarioLifecycle`] is a Bevy plugin that:
//! - Bypasses menus: `OnEnter(GameState::MainMenu)` → immediately enters `Playing`
//! - Auto-skips chip selection: `OnEnter(GameState::ChipSelect)` → `NodeTransition`
//! - Counts fixed-update frames via [`ScenarioFrame`]
//! - Exits when `max_frames` is reached; optionally exits when the run ends
//!   naturally (controlled by [`ScenarioDefinition::allow_early_end`])

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use breaker::{
    bolt::{BoltSystems, components::Bolt},
    breaker::{BreakerSystems, components::Breaker},
    input::resources::InputActions,
    run::node::{ScenarioLayoutOverride, messages::SpawnNodeComplete, sets::NodeSystems},
    shared::{GameState, SelectedArchetype},
};

use crate::{
    input::InputDriver,
    invariants::{
        EntityLeakBaseline, PreviousGameState, ScenarioFrame, ScenarioPhysicsFrozen, ScenarioStats,
        ScenarioTagBolt, ScenarioTagBreaker, ViolationLog, check_bolt_count_reasonable,
        check_bolt_in_bounds, check_bolt_speed_in_range, check_breaker_in_bounds,
        check_breaker_position_clamped, check_no_entity_leaks, check_no_nan,
        check_physics_frozen_during_pause, check_timer_monotonically_decreasing,
        check_timer_non_negative, check_valid_breaker_state, check_valid_state_transitions,
    },
    types::{GameAction as ScenarioGameAction, ScenarioDefinition},
};

/// Query alias for breaker entities in [`apply_debug_setup`].
type BreakerDebugQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static mut Transform),
    (With<ScenarioTagBreaker>, Without<ScenarioTagBolt>),
>;

/// Loaded scenario configuration, inserted before the app runs.
#[derive(Resource)]
pub struct ScenarioConfig {
    /// The full scenario definition loaded from RON.
    pub definition: ScenarioDefinition,
}

/// Bevy resource wrapping an [`InputDriver`] for the current scenario run.
///
/// Inserted by [`init_scenario_input`] when the scenario starts.
/// Queried each fixed-update tick by [`inject_scenario_input`].
#[derive(Resource)]
pub struct ScenarioInputDriver(pub InputDriver);

/// Reads [`ScenarioConfig`] and inserts a [`ScenarioInputDriver`] into the world.
///
/// Runs once at scenario startup.
pub fn init_scenario_input(config: Res<ScenarioConfig>, mut commands: Commands) {
    let driver = InputDriver::from_strategy(&config.definition.input);
    commands.insert_resource(ScenarioInputDriver(driver));
}

/// Maps a scenario-crate [`ScenarioGameAction`] to the game-crate
/// [`breaker::input::resources::GameAction`].
const fn map_action(action: ScenarioGameAction) -> breaker::input::resources::GameAction {
    match action {
        ScenarioGameAction::MoveLeft => breaker::input::resources::GameAction::MoveLeft,
        ScenarioGameAction::MoveRight => breaker::input::resources::GameAction::MoveRight,
        ScenarioGameAction::Bump => breaker::input::resources::GameAction::Bump,
        ScenarioGameAction::DashLeft => breaker::input::resources::GameAction::DashLeft,
        ScenarioGameAction::DashRight => breaker::input::resources::GameAction::DashRight,
        ScenarioGameAction::MenuUp => breaker::input::resources::GameAction::MenuUp,
        ScenarioGameAction::MenuDown => breaker::input::resources::GameAction::MenuDown,
        ScenarioGameAction::MenuLeft => breaker::input::resources::GameAction::MenuLeft,
        ScenarioGameAction::MenuRight => breaker::input::resources::GameAction::MenuRight,
        ScenarioGameAction::MenuConfirm => breaker::input::resources::GameAction::MenuConfirm,
        ScenarioGameAction::TogglePause => breaker::input::resources::GameAction::TogglePause,
    }
}

/// Injects scenario-controlled actions into [`InputActions`] each fixed-update tick.
///
/// Reads [`ScenarioInputDriver`], queries the current [`ScenarioFrame`], maps the
/// scenario-crate [`crate::types::GameAction`] values to the game crate's
/// [`breaker::input::resources::GameAction`], and writes to [`InputActions`].
///
/// Uses `Option<ResMut<ScenarioInputDriver>>` so it does not panic if the resource
/// has not yet been inserted.
pub fn inject_scenario_input(
    mut driver: Option<ResMut<ScenarioInputDriver>>,
    frame: Res<ScenarioFrame>,
    mut actions: ResMut<InputActions>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    let Some(ref mut driver) = driver else {
        return;
    };

    let scenario_actions = driver.0.actions_for_frame(frame.0, true);
    let action_count = u32::try_from(scenario_actions.len()).unwrap_or(u32::MAX);

    for action in scenario_actions {
        actions.0.push(map_action(action));
    }

    if let Some(ref mut s) = stats {
        s.actions_injected += action_count;
    }
}

/// Plugin that drives the scenario lifecycle.
pub struct ScenarioLifecycle;

impl Plugin for ScenarioLifecycle {
    fn build(&self, app: &mut App) {
        let allow_early_end = app
            .world()
            .resource::<ScenarioConfig>()
            .definition
            .allow_early_end;

        app.init_resource::<ScenarioFrame>()
            .init_resource::<ViolationLog>()
            .init_resource::<PreviousGameState>()
            .init_resource::<EntityLeakBaseline>()
            .init_resource::<ScenarioStats>()
            .add_message::<SpawnNodeComplete>()
            .add_systems(OnEnter(GameState::MainMenu), bypass_menu_to_playing)
            .add_systems(OnEnter(GameState::ChipSelect), auto_skip_chip_select)
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    init_scenario_input,
                    ApplyDeferred,
                    tag_game_entities,
                    apply_debug_setup,
                )
                    .chain()
                    .after(BoltSystems::InitParams)
                    .after(BreakerSystems::Reset)
                    .after(NodeSystems::InitTimer),
            )
            // Input injection runs in FixedPreUpdate so it executes after
            // clear_input_actions (FixedPostUpdate of previous tick) and before
            // all FixedUpdate game systems that read InputActions.
            .add_systems(FixedPreUpdate, inject_scenario_input)
            .add_systems(
                FixedUpdate,
                (
                    (tick_scenario_frame, check_frame_limit)
                        .chain()
                        .before(breaker::breaker::sets::BreakerSystems::Move),
                    // Invariant checkers and frozen position enforcement must run
                    // BEFORE physics systems. Otherwise bolt_lost respawns OOB
                    // bolts before invariants can detect them.
                    (
                        enforce_frozen_positions,
                        check_bolt_in_bounds,
                        check_bolt_speed_in_range,
                        check_bolt_count_reasonable,
                        check_breaker_in_bounds,
                        check_no_nan,
                        check_timer_non_negative,
                        check_valid_state_transitions,
                        check_valid_breaker_state,
                        check_timer_monotonically_decreasing,
                        check_breaker_position_clamped,
                        check_physics_frozen_during_pause,
                        check_no_entity_leaks,
                    )
                        .after(tag_game_entities)
                        .before(breaker::physics::PhysicsSystems::BoltLost),
                    tag_game_entities,
                ),
            );

        if allow_early_end {
            // Normal: exit when run ends naturally.
            // Runs every frame while in RunEnd to avoid one-shot timing issues
            // where Winit misses a single AppExit message.
            app.add_systems(Update, exit_on_run_end.run_if(in_state(GameState::RunEnd)));
        } else {
            // Stress: restart when run ends, only max_frames triggers exit.
            app.add_systems(OnEnter(GameState::RunEnd), restart_run_on_end);
        }
    }
}

/// Sets the archetype and layout override, then immediately enters `Playing`.
///
/// This bypasses `RunSetup` entirely — the scenario controls which archetype
/// and layout are used without any user interaction.
fn bypass_menu_to_playing(
    config: Res<ScenarioConfig>,
    mut selected: ResMut<SelectedArchetype>,
    mut layout_override: ResMut<ScenarioLayoutOverride>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    selected.0.clone_from(&config.definition.breaker);
    layout_override.0 = Some(config.definition.layout.clone());
    next_state.set(GameState::Playing);
}

/// Transitions immediately to `NodeTransition`, skipping chip selection UI.
///
/// No chip is applied — the scenario runner does not model chip effects.
fn auto_skip_chip_select(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::NodeTransition);
}

/// Increments [`ScenarioFrame`] by 1 each fixed-update tick.
///
/// Also updates [`ScenarioStats::max_frame`] when the stats resource is present.
fn tick_scenario_frame(mut frame: ResMut<ScenarioFrame>, mut stats: Option<ResMut<ScenarioStats>>) {
    frame.0 += 1;
    if let Some(ref mut s) = stats {
        s.max_frame = frame.0;
    }
}

/// Sends [`AppExit::Success`] when [`ScenarioFrame`] reaches `max_frames`.
fn check_frame_limit(
    frame: Res<ScenarioFrame>,
    config: Res<ScenarioConfig>,
    mut exits: MessageWriter<AppExit>,
) {
    if frame.0 >= config.definition.max_frames {
        exits.write(AppExit::Success);
    }
}

/// Sends [`AppExit::Success`] when the run ends naturally.
///
/// Runs every frame while in `RunEnd` (not as a one-shot `OnEnter`) so that
/// the Winit event loop reliably sees the exit message on macOS.
fn exit_on_run_end(mut exits: MessageWriter<AppExit>) {
    exits.write(AppExit::Success);
}

/// Redirects `RunEnd` back to `MainMenu` (which `bypass_menu_to_playing`
/// sends to `Playing`). Used when `allow_early_end` is false so the
/// scenario runs for the full `max_frames` frame budget.
fn restart_run_on_end(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::MainMenu);
}

/// Applies debug overrides from [`ScenarioConfig`] to tagged bolt and breaker entities.
///
/// For each entity tagged with [`ScenarioTagBolt`] or [`ScenarioTagBreaker`],
/// applies position teleports from [`crate::types::DebugSetup`] (z coordinate is
/// preserved). When `disable_physics` is true, inserts [`ScenarioPhysicsFrozen`]
/// on both bolts and breakers with the post-teleport position as the frozen target.
pub fn apply_debug_setup(
    config: Res<ScenarioConfig>,
    mut bolt_query: Query<(Entity, &mut Transform), With<ScenarioTagBolt>>,
    mut breaker_query: BreakerDebugQuery,
    mut commands: Commands,
) {
    let Some(setup) = config.definition.debug_setup.as_ref() else {
        return;
    };

    for (entity, mut transform) in &mut bolt_query {
        if let Some((x, y)) = setup.bolt_position {
            transform.translation.x = x;
            transform.translation.y = y;
        }

        if setup.disable_physics {
            commands.entity(entity).insert(ScenarioPhysicsFrozen {
                target: transform.translation,
            });
        }
    }

    for (entity, mut transform) in &mut breaker_query {
        if let Some((x, y)) = setup.breaker_position {
            transform.translation.x = x;
            transform.translation.y = y;
        }

        if setup.disable_physics {
            commands.entity(entity).insert(ScenarioPhysicsFrozen {
                target: transform.translation,
            });
        }
    }
}

/// Resets each entity with [`ScenarioPhysicsFrozen`] back to its pinned `target` every tick.
///
/// Prevents physics systems from moving entities that should be stationary during
/// a self-test scenario. Runs after physics in `FixedUpdate`.
pub fn enforce_frozen_positions(mut frozen: Query<(&ScenarioPhysicsFrozen, &mut Transform)>) {
    for (pinned, mut transform) in &mut frozen {
        transform.translation = pinned.target;
    }
}

/// Tags game entities with scenario marker components for invariant checking.
///
/// Finds all untagged [`Bolt`] entities and inserts [`ScenarioTagBolt`].
/// Finds all untagged [`Breaker`] entities and inserts [`ScenarioTagBreaker`].
/// Runs in `OnEnter(GameState::Playing)` before [`apply_debug_setup`].
///
/// Also sets [`ScenarioStats::entered_playing`] to `true` when the stats resource is present.
pub fn tag_game_entities(
    bolt_query: Query<Entity, (With<Bolt>, Without<ScenarioTagBolt>)>,
    breaker_query: Query<Entity, (With<Breaker>, Without<ScenarioTagBreaker>)>,
    mut commands: Commands,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    let mut bolts_tagged = 0u32;
    let mut breakers_tagged = 0u32;

    for entity in &bolt_query {
        commands.entity(entity).insert(ScenarioTagBolt);
        bolts_tagged += 1;
    }
    for entity in &breaker_query {
        commands.entity(entity).insert(ScenarioTagBreaker);
        breakers_tagged += 1;
    }

    if let Some(ref mut s) = stats {
        s.entered_playing = true;
        s.bolts_tagged += bolts_tagged;
        s.breakers_tagged += breakers_tagged;
    }
}
