//! Scenario lifecycle — state navigation, frame counter, and exit logic.
//!
//! [`ScenarioLifecycle`] is a Bevy plugin that:
//! - Bypasses menus: `OnEnter(GameState::MainMenu)` → immediately enters `Playing`
//! - Auto-skips chip selection: `OnEnter(GameState::ChipSelect)` → `TransitionIn`
//! - Counts fixed-update frames via [`ScenarioFrame`]
//! - Exits when `max_frames` is reached; optionally exits when the run ends
//!   naturally (controlled by [`ScenarioDefinition::allow_early_end`])

#[cfg(test)]
mod tests;

use bevy::{ecs::system::SystemParam, prelude::*};
use breaker::{
    behaviors::ActiveChains,
    bolt::{BoltSystems, components::Bolt},
    breaker::{
        BreakerSystems,
        components::{Breaker, BreakerState},
        systems::update_breaker_state,
    },
    chips::inventory::ChipInventory,
    input::resources::InputActions,
    run::{
        RunStats,
        node::{
            ScenarioLayoutOverride, messages::SpawnNodeComplete, resources::NodeTimer,
            sets::NodeSystems,
        },
    },
    screen::chip_select::{ChipOffering, ChipOffers},
    shared::{GameState, PlayingState, RunSeed, SelectedArchetype},
};
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use crate::{
    input::InputDriver,
    invariants::{
        EntityLeakBaseline, PreviousGameState, ScenarioFrame, ScenarioPhysicsFrozen, ScenarioStats,
        ScenarioTagBolt, ScenarioTagBreaker, ViolationLog, check_bolt_count_reasonable,
        check_bolt_in_bounds, check_bolt_speed_in_range, check_breaker_in_bounds,
        check_breaker_position_clamped, check_chip_stacks_consistent,
        check_maxed_chip_never_offered, check_no_entity_leaks, check_no_nan,
        check_offering_no_duplicates, check_physics_frozen_during_pause, check_run_stats_monotonic,
        check_timer_monotonically_decreasing, check_timer_non_negative, check_valid_breaker_state,
        check_valid_state_transitions,
    },
    types::{
        ForcedGameState, GameAction as ScenarioGameAction, MutationKind, RunStatCounter,
        ScenarioBreakerState, ScenarioDefinition,
    },
};

/// Query alias for bolt entities in [`apply_debug_setup`] and [`deferred_debug_setup`].
type BoltDebugQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static mut Position2D, &'static mut Velocity2D),
    With<ScenarioTagBolt>,
>;

/// Query alias for breaker entities in [`apply_debug_setup`] and [`deferred_debug_setup`].
type BreakerDebugQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static mut Position2D),
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
                    ApplyDeferred,
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
                        .run_if(entered_playing)
                        .before(breaker::breaker::sets::BreakerSystems::Move),
                    // Invariant checkers and frozen position enforcement must run
                    // BEFORE physics systems. Otherwise bolt_lost respawns OOB
                    // bolts before invariants can detect them.
                    //
                    // Gated on entered_playing: during Loading/MainMenu, entities
                    // may not be fully initialized (especially under parallel I/O
                    // contention). Checkers only fire once Playing has been entered.
                    (
                        enforce_frozen_positions,
                        apply_debug_frame_mutations,
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
                        check_offering_no_duplicates,
                        check_maxed_chip_never_offered,
                        check_chip_stacks_consistent,
                        check_run_stats_monotonic,
                    )
                        .chain()
                        .run_if(|stats: Option<Res<ScenarioStats>>| {
                            stats.is_some_and(|s| s.entered_playing)
                        })
                        .after(deferred_debug_setup)
                        .after(tag_game_entities)
                        .after(update_breaker_state)
                        .before(breaker::bolt::BoltSystems::BoltLost),
                    tag_game_entities,
                    deferred_debug_setup.after(tag_game_entities),
                    mark_entered_playing_on_spawn_complete,
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
    mut run_seed: ResMut<RunSeed>,
    mut active_overclocks: Option<ResMut<ActiveChains>>,
) {
    selected.0.clone_from(&config.definition.breaker);
    layout_override.0 = Some(config.definition.layout.clone());
    // Scenarios always use deterministic seed (default 0 when not specified)
    run_seed.0 = Some(config.definition.seed.unwrap_or(0));
    if let Some(ref overclocks) = config.definition.initial_overclocks
        && let Some(ref mut active) = active_overclocks
    {
        active.0 = overclocks.iter().cloned().map(|c| (None, c)).collect();
    }
    next_state.set(GameState::Playing);
}

/// Transitions immediately to `TransitionIn`, skipping chip selection UI.
///
/// No chip is applied — the scenario runner does not model chip effects.
fn auto_skip_chip_select(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::TransitionIn);
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

/// Maps a [`ForcedGameState`] to the game crate's [`GameState`].
///
/// Used by [`apply_debug_setup`] to translate the RON-serializable enum
/// into the Bevy state enum.
pub(crate) fn map_forced_game_state(forced: ForcedGameState) -> GameState {
    match forced {
        ForcedGameState::Loading => GameState::Loading,
        ForcedGameState::MainMenu => GameState::MainMenu,
        ForcedGameState::RunSetup => GameState::RunSetup,
        ForcedGameState::Playing => GameState::Playing,
        ForcedGameState::TransitionOut => GameState::TransitionOut,
        ForcedGameState::ChipSelect => GameState::ChipSelect,
        ForcedGameState::TransitionIn => GameState::TransitionIn,
        ForcedGameState::RunEnd => GameState::RunEnd,
        ForcedGameState::MetaProgression => GameState::MetaProgression,
    }
}

/// Applies debug overrides from [`ScenarioConfig`] to tagged bolt and breaker entities.
///
/// For each entity tagged with [`ScenarioTagBolt`] or [`ScenarioTagBreaker`],
/// applies position teleports from [`crate::types::DebugSetup`]. When
/// `disable_physics` is true, inserts [`ScenarioPhysicsFrozen`] on both bolts
/// and breakers with the post-teleport position as the frozen target.
///
/// Also handles `bolt_velocity`, `extra_tagged_bolts`, `node_timer_remaining`,
/// and `force_previous_game_state` overrides.
pub fn apply_debug_setup(
    config: Res<ScenarioConfig>,
    mut bolt_query: BoltDebugQuery,
    mut breaker_query: BreakerDebugQuery,
    mut commands: Commands,
    node_timer: Option<ResMut<NodeTimer>>,
    mut previous_state: Option<ResMut<PreviousGameState>>,
) {
    let Some(setup) = config.definition.debug_setup.as_ref() else {
        return;
    };

    for (entity, mut position, mut velocity) in &mut bolt_query {
        if let Some((x, y)) = setup.bolt_position {
            position.0.x = x;
            position.0.y = y;
        }

        if let Some((vx, vy)) = setup.bolt_velocity {
            velocity.0 = Vec2::new(vx, vy);
        }

        if setup.disable_physics {
            commands
                .entity(entity)
                .insert(ScenarioPhysicsFrozen { target: position.0 });
        }
    }

    for (entity, mut position) in &mut breaker_query {
        if let Some((x, y)) = setup.breaker_position {
            position.0.x = x;
            position.0.y = y;
        }

        if setup.disable_physics {
            commands
                .entity(entity)
                .insert(ScenarioPhysicsFrozen { target: position.0 });
        }
    }

    if let Some(count) = setup.extra_tagged_bolts {
        for _ in 0..count {
            commands.spawn(ScenarioTagBolt);
        }
    }

    if let Some(remaining) = setup.node_timer_remaining
        && let Some(mut timer) = node_timer
    {
        timer.remaining = remaining;
    }

    if let Some(forced) = setup.force_previous_game_state
        && let Some(ref mut prev) = previous_state
    {
        prev.0 = Some(map_forced_game_state(forced));
    }
}

/// Deferred fallback for [`apply_debug_setup`] — runs once in `FixedUpdate` after
/// [`tag_game_entities`] to catch entities that were not yet spawned when the
/// `OnEnter(GameState::Playing)` version of `apply_debug_setup` ran.
///
/// Under heavy parallel I/O contention (45+ scenarios loading simultaneously),
/// the `OnEnter` schedule can execute `apply_debug_setup` before spawn systems
/// have flushed their deferred commands, leaving 0 tagged entities to process.
/// This system re-applies the entity-dependent parts of debug setup (position
/// overrides, velocity overrides, and physics freeze) on the first `FixedUpdate`
/// tick where tagged entities exist.
///
/// Uses a [`Local<bool>`] guard so it fires at most once per app lifetime.
/// Non-entity parts (extra tagged bolts, timer override, forced previous state)
/// are handled by the `OnEnter` version and are not repeated here.
pub fn deferred_debug_setup(
    mut done: Local<bool>,
    config: Res<ScenarioConfig>,
    mut bolt_query: BoltDebugQuery,
    mut breaker_query: BreakerDebugQuery,
    mut commands: Commands,
) {
    if *done {
        return;
    }

    let Some(setup) = config.definition.debug_setup.as_ref() else {
        *done = true;
        return;
    };

    // Wait until at least one tagged entity exists before applying.
    if bolt_query.is_empty() && breaker_query.is_empty() {
        return;
    }

    for (entity, mut position, mut velocity) in &mut bolt_query {
        if let Some((x, y)) = setup.bolt_position {
            position.0.x = x;
            position.0.y = y;
        }

        if let Some((vx, vy)) = setup.bolt_velocity {
            velocity.0 = Vec2::new(vx, vy);
        }

        if setup.disable_physics {
            commands
                .entity(entity)
                .insert(ScenarioPhysicsFrozen { target: position.0 });
        }
    }

    for (entity, mut position) in &mut breaker_query {
        if let Some((x, y)) = setup.breaker_position {
            position.0.x = x;
            position.0.y = y;
        }

        if setup.disable_physics {
            commands
                .entity(entity)
                .insert(ScenarioPhysicsFrozen { target: position.0 });
        }
    }

    *done = true;
}

/// Resets each entity with [`ScenarioPhysicsFrozen`] back to its pinned `target` every tick.
///
/// Prevents physics systems from moving entities that should be stationary during
/// a self-test scenario. Runs after physics in `FixedUpdate`.
pub fn enforce_frozen_positions(mut frozen: Query<(&ScenarioPhysicsFrozen, &mut Position2D)>) {
    for (pinned, mut position) in &mut frozen {
        position.0 = pinned.target;
    }
}

/// Tags game entities with scenario marker components for invariant checking.
///
/// Finds all untagged [`Bolt`] entities and inserts [`ScenarioTagBolt`].
/// Finds all untagged [`Breaker`] entities and inserts [`ScenarioTagBreaker`].
/// Runs in `OnEnter(GameState::Playing)` before [`apply_debug_setup`].
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
        s.bolts_tagged += bolts_tagged;
        s.breakers_tagged += breakers_tagged;
    }
}

/// Maps a [`ScenarioBreakerState`] to the game crate's [`BreakerState`].
///
/// Used by [`apply_debug_frame_mutations`] to translate the RON-serializable
/// enum into the Bevy component enum.
#[must_use]
pub(crate) fn map_scenario_breaker_state(state: ScenarioBreakerState) -> BreakerState {
    match state {
        ScenarioBreakerState::Idle => BreakerState::Idle,
        ScenarioBreakerState::Dashing => BreakerState::Dashing,
        ScenarioBreakerState::Braking => BreakerState::Braking,
        ScenarioBreakerState::Settling => BreakerState::Settling,
    }
}

/// Grouped system parameters for pause toggle control.
///
/// Extracted to keep [`apply_debug_frame_mutations`] under the 7-argument clippy limit.
#[derive(SystemParam)]
pub struct PauseControl<'w> {
    /// Current [`PlayingState`] — only present when [`GameState::Playing`] is active.
    state: Option<Res<'w, State<PlayingState>>>,
    /// [`NextState`] writer for toggling pause.
    next: Option<ResMut<'w, NextState<PlayingState>>>,
}

/// Grouped system parameters for mutation targets that need additional game state.
///
/// Extracted to keep [`apply_debug_frame_mutations`] under the 7-argument clippy limit.
#[derive(SystemParam)]
pub struct MutationTargets<'w, 's> {
    /// [`RunStats`] resource — absent before a run starts.
    run_stats: Option<ResMut<'w, RunStats>>,
    /// [`ChipInventory`] resource — absent before a run starts.
    chip_inventory: Option<ResMut<'w, ChipInventory>>,
    /// [`ChipOffers`] resource — present only during [`GameState::ChipSelect`].
    chip_offers: Option<ResMut<'w, ChipOffers>>,
    /// [`Commands`] for inserting resources when the optional resource is absent.
    commands: Commands<'w, 's>,
}

/// Applies per-frame mutations from [`ScenarioConfig`] at matching frames.
///
/// Reads `frame_mutations` from the scenario definition. For each mutation
/// whose frame matches [`ScenarioFrame`], applies the corresponding state
/// change (breaker state override, timer override, entity spawn, bolt
/// teleport, pause toggle, run stat decrement, or chip inventory injection).
pub fn apply_debug_frame_mutations(
    config: Res<ScenarioConfig>,
    frame: Res<ScenarioFrame>,
    mut breakers: Query<&mut BreakerState, With<ScenarioTagBreaker>>,
    mut bolts: Query<&mut Position2D, With<ScenarioTagBolt>>,
    mut node_timer: Option<ResMut<NodeTimer>>,
    mut pause: PauseControl,
    mut targets: MutationTargets,
) {
    let Some(ref mutations) = config.definition.frame_mutations else {
        return;
    };

    for mutation in mutations {
        if mutation.frame != frame.0 {
            continue;
        }
        match &mutation.mutation {
            MutationKind::SetBreakerState(scenario_state) => {
                let target = map_scenario_breaker_state(*scenario_state);
                for mut state in &mut breakers {
                    *state = target;
                }
            }
            MutationKind::SetTimerRemaining(remaining) => {
                if let Some(ref mut timer) = node_timer {
                    timer.remaining = *remaining;
                }
            }
            MutationKind::SpawnExtraEntities(count) => {
                for _ in 0..*count {
                    targets.commands.spawn(Transform::default());
                }
            }
            MutationKind::MoveBolt(x, y) => {
                for mut position in &mut bolts {
                    position.0.x = *x;
                    position.0.y = *y;
                }
            }
            MutationKind::TogglePause => {
                if let Some(ref state) = pause.state
                    && let Some(ref mut next) = pause.next
                {
                    match ***state {
                        PlayingState::Active => next.set(PlayingState::Paused),
                        PlayingState::Paused => next.set(PlayingState::Active),
                    }
                }
            }
            MutationKind::SetRunStat(counter, value) => {
                if let Some(ref mut stats) = targets.run_stats {
                    apply_set_run_stat(stats, *counter, *value);
                }
            }
            MutationKind::DecrementRunStat(counter) => {
                if let Some(ref mut stats) = targets.run_stats {
                    apply_decrement_run_stat(stats, *counter);
                }
            }
            MutationKind::InjectOverStackedChip {
                chip_name,
                stacks,
                max_stacks,
            } => {
                if let Some(ref mut inventory) = targets.chip_inventory {
                    inventory.force_insert_entry(chip_name, *stacks, *max_stacks);
                }
            }
            MutationKind::InjectDuplicateOffers { chip_name } => {
                apply_inject_duplicate_offers(
                    chip_name,
                    &mut targets.chip_offers,
                    &mut targets.commands,
                );
            }
            MutationKind::InjectMaxedChipOffer { chip_name } => {
                apply_inject_maxed_chip_offer(
                    chip_name,
                    &mut targets.chip_inventory,
                    &mut targets.chip_offers,
                    &mut targets.commands,
                );
            }
        }
    }
}

/// Sets the named [`RunStats`] counter to `value`.
fn apply_set_run_stat(stats: &mut RunStats, counter: RunStatCounter, value: u32) {
    match counter {
        RunStatCounter::NodesCleared => stats.nodes_cleared = value,
        RunStatCounter::CellsDestroyed => stats.cells_destroyed = value,
        RunStatCounter::BumpsPerformed => stats.bumps_performed = value,
        RunStatCounter::PerfectBumps => stats.perfect_bumps = value,
        RunStatCounter::BoltsLost => stats.bolts_lost = value,
    }
}

/// Decrements the named [`RunStats`] counter by 1 (saturating at 0).
fn apply_decrement_run_stat(stats: &mut RunStats, counter: RunStatCounter) {
    match counter {
        RunStatCounter::NodesCleared => {
            stats.nodes_cleared = stats.nodes_cleared.saturating_sub(1);
        }
        RunStatCounter::CellsDestroyed => {
            stats.cells_destroyed = stats.cells_destroyed.saturating_sub(1);
        }
        RunStatCounter::BumpsPerformed => {
            stats.bumps_performed = stats.bumps_performed.saturating_sub(1);
        }
        RunStatCounter::PerfectBumps => {
            stats.perfect_bumps = stats.perfect_bumps.saturating_sub(1);
        }
        RunStatCounter::BoltsLost => {
            stats.bolts_lost = stats.bolts_lost.saturating_sub(1);
        }
    }
}

/// Injects a [`ChipOffers`] resource containing two identical chips (triggers
/// [`InvariantKind::OfferingNoDuplicates`]).
fn apply_inject_duplicate_offers(
    chip_name: &str,
    chip_offers: &mut Option<ResMut<ChipOffers>>,
    commands: &mut Commands,
) {
    use breaker::chips::definition::{ChipDefinition, Rarity, TriggerChain};
    let def = ChipDefinition {
        name: chip_name.to_owned(),
        description: String::new(),
        rarity: Rarity::Common,
        max_stacks: 3,
        effects: vec![TriggerChain::Piercing(1)],
        ingredients: None,
    };
    let offers = ChipOffers(vec![
        ChipOffering::Normal(def.clone()),
        ChipOffering::Normal(def),
    ]);
    if let Some(existing) = chip_offers {
        **existing = offers;
    } else {
        commands.insert_resource(offers);
    }
}

/// Injects a [`ChipOffers`] resource containing a chip already maxed in
/// [`ChipInventory`] (triggers [`InvariantKind::MaxedChipNeverOffered`]).
fn apply_inject_maxed_chip_offer(
    chip_name: &str,
    chip_inventory: &mut Option<ResMut<ChipInventory>>,
    chip_offers: &mut Option<ResMut<ChipOffers>>,
    commands: &mut Commands,
) {
    use breaker::chips::definition::{ChipDefinition, Rarity, TriggerChain};
    let def = ChipDefinition {
        name: chip_name.to_owned(),
        description: String::new(),
        rarity: Rarity::Common,
        max_stacks: 1,
        effects: vec![TriggerChain::Piercing(1)],
        ingredients: None,
    };
    if let Some(inventory) = chip_inventory {
        inventory.force_insert_entry(chip_name, 1, 1);
    }
    let offers = ChipOffers(vec![ChipOffering::Normal(def)]);
    if let Some(existing) = chip_offers {
        **existing = offers;
    } else {
        commands.insert_resource(offers);
    }
}

/// Sets [`ScenarioStats::entered_playing`] to `true` when [`SpawnNodeComplete`]
/// fires, indicating all game entities are spawned and ready.
///
/// Frame counting and invariant checking are gated on `entered_playing`, so
/// no scenario frames advance until the node is fully loaded and spawned.
pub fn mark_entered_playing_on_spawn_complete(
    mut spawn_reader: MessageReader<SpawnNodeComplete>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if spawn_reader.read().next().is_some()
        && let Some(ref mut s) = stats
    {
        s.entered_playing = true;
    }
}

/// Run condition: returns `true` when [`ScenarioStats::entered_playing`] is `true`.
///
/// Used as a `run_if` guard to prevent frame counting and frame-limit
/// checking from running before the game has entered `Playing`.
#[must_use]
pub fn entered_playing(stats: Option<Res<ScenarioStats>>) -> bool {
    stats.is_some_and(|s| s.entered_playing)
}
