use super::helpers::*;

// -------------------------------------------------------------------------
// inject_scenario_input — writes Bump for scripted frame
// -------------------------------------------------------------------------

/// `inject_scenario_input` must translate `crate::types::GameAction::Bump` from
/// the scripted driver to `breaker::input::resources::GameAction::Bump` and write
/// it into [`InputActions`] when the current frame matches.
///
/// Given: [`ScenarioInputDriver`] with `Scripted` input that has `Bump` at frame 10,
/// [`ScenarioFrame`] = 10, and empty [`InputActions`].
///
/// After the system runs, `InputActions` must contain `GameAction::Bump`.
#[test]
fn inject_scenario_input_writes_bump_for_scripted_frame() {
    use breaker::input::resources::{GameAction as BreakerGameAction, InputActions};

    use crate::{
        input::{InputDriver, ScriptedInput},
        types::{GameAction as ScenarioGameAction, ScriptedFrame, ScriptedParams},
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let scripted = ScriptedInput::new(&ScriptedParams {
        actions: vec![ScriptedFrame {
            frame: 10,
            actions: vec![ScenarioGameAction::Bump],
        }],
    });
    app.insert_resource(ScenarioInputDriver(InputDriver::Scripted(scripted)))
        .insert_resource(ScenarioFrame(10))
        .insert_resource(InputActions::default())
        .add_systems(Update, inject_scenario_input);

    app.update();

    let actions = app.world().resource::<InputActions>();
    assert!(
        actions.active(BreakerGameAction::Bump),
        "expected InputActions to contain Bump after inject_scenario_input at frame 10"
    );
}

// -------------------------------------------------------------------------
// inject_scenario_input — empty for unmatched frame
// -------------------------------------------------------------------------

/// `inject_scenario_input` must leave [`InputActions`] empty when the current
/// frame does not match any scripted entry.
///
/// Given: Scripted driver with `Bump` at frame 10, [`ScenarioFrame`] = 5.
///
/// After the system runs, `InputActions` must remain empty.
#[test]
fn inject_scenario_input_empty_for_unmatched_frame() {
    use breaker::input::resources::{GameAction as BreakerGameAction, InputActions};

    use crate::{
        input::{InputDriver, ScriptedInput},
        types::{GameAction as ScenarioGameAction, ScriptedFrame, ScriptedParams},
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let scripted = ScriptedInput::new(&ScriptedParams {
        actions: vec![ScriptedFrame {
            frame: 10,
            actions: vec![ScenarioGameAction::Bump],
        }],
    });
    app.insert_resource(ScenarioInputDriver(InputDriver::Scripted(scripted)))
        .insert_resource(ScenarioFrame(5))
        .insert_resource(InputActions::default())
        .add_systems(Update, inject_scenario_input);

    app.update();

    let actions = app.world().resource::<InputActions>();
    assert!(
        !actions.active(BreakerGameAction::Bump),
        "expected InputActions to NOT contain Bump at frame 5 (no scripted entry)"
    );
    assert!(
        actions.0.is_empty(),
        "expected InputActions to be empty at unmatched frame 5, got {:?}",
        actions.0
    );
}

// -------------------------------------------------------------------------
// init_scenario_input — creates driver resource
// -------------------------------------------------------------------------

/// `init_scenario_input` must read [`ScenarioConfig`] and insert a
/// [`ScenarioInputDriver`] resource into the world.
///
/// Given: A Bevy app with [`ScenarioConfig`] containing `Chaos` input strategy.
/// After the system runs, the world must contain [`ScenarioInputDriver`].
#[test]
fn init_scenario_input_creates_driver_resource() {
    use crate::types::{ChaosParams, InputStrategy};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.insert_resource(ScenarioConfig {
        definition: ScenarioDefinition {
            breaker: "aegis".to_owned(),
            layout: "corridor".to_owned(),
            input: InputStrategy::Chaos(ChaosParams { action_prob: 0.3 }),
            max_frames: 1000,
            disallowed_failures: vec![],
            ..Default::default()
        },
    });
    app.add_systems(Update, init_scenario_input);

    app.update();

    assert!(
        app.world().get_resource::<ScenarioInputDriver>().is_some(),
        "expected ScenarioInputDriver resource to exist after init_scenario_input ran"
    );
}

// -------------------------------------------------------------------------
// ScenarioStats — actions_injected incremented by inject_scenario_input
// -------------------------------------------------------------------------

/// When `inject_scenario_input` writes an action, `ScenarioStats::actions_injected`
/// must be incremented.
///
/// Given: Scripted driver with `Bump` at frame 5, [`ScenarioFrame`] = 5,
/// and [`ScenarioStats`] with `actions_injected = 0`.
/// After the system runs, `stats.actions_injected == 1`.
#[test]
fn scenario_stats_actions_injected_incremented_by_inject_scenario_input() {
    use breaker::input::resources::InputActions;

    use crate::{
        input::{InputDriver, ScriptedInput},
        invariants::ScenarioStats,
        types::{GameAction as ScenarioGameAction, ScriptedFrame, ScriptedParams},
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let scripted = ScriptedInput::new(&ScriptedParams {
        actions: vec![ScriptedFrame {
            frame: 5,
            actions: vec![ScenarioGameAction::Bump],
        }],
    });
    app.insert_resource(ScenarioInputDriver(InputDriver::Scripted(scripted)))
        .insert_resource(ScenarioFrame(5))
        .insert_resource(InputActions::default())
        .init_resource::<ScenarioStats>()
        .add_systems(Update, inject_scenario_input);

    app.update();

    let stats = app.world().resource::<ScenarioStats>();
    assert_eq!(
        stats.actions_injected, 1,
        "expected actions_injected == 1 after one action was injected, got {}",
        stats.actions_injected
    );
}
