use bevy::{platform::collections::HashMap, prelude::*};
use breaker::{breaker::components::BreakerState, shared::GameState};

use crate::{invariants::*, types::InvariantKind};

/// Checks that [`BreakerState`] transitions on the tagged breaker follow the legal path.
///
/// Legal transitions: `Idle → Dashing`, `Settling → Dashing` (re-dash),
/// `Dashing → Braking`, `Dashing → Settling` (dash cancel),
/// `Braking → Settling`, `Settling → Idle`. Any other change fires a [`ViolationEntry`] with
/// [`InvariantKind::ValidBreakerState`].
///
/// Clears tracking on [`GameState`] transitions (e.g., entering `Playing` after a
/// node change) so that forced `reset_breaker` resets to `Idle` are not flagged.
///
/// Skips the first frame per entity (no previous state stored yet for that entity).
pub fn check_valid_breaker_state(
    breakers: Query<(Entity, &BreakerState), With<ScenarioTagBreaker>>,
    mut previous: Local<HashMap<Entity, BreakerState>>,
    game_state: Res<State<GameState>>,
    mut prev_game_state: Local<Option<GameState>>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let current_game = **game_state;
    // On game-state transition (e.g., entering Playing after a node change),
    // clear tracking — `reset_breaker` may have forcibly set any breaker to
    // `Idle`, which is not a state-machine violation.
    if let Some(prev_gs) = *prev_game_state
        && prev_gs != current_game
    {
        previous.clear();
    }
    *prev_game_state = Some(current_game);

    for (entity, &current) in &breakers {
        if let Some(&prev) = previous.get(&entity)
            && prev != current
        {
            let legal = matches!(
                (prev, current),
                (
                    BreakerState::Idle | BreakerState::Settling,
                    BreakerState::Dashing
                ) | (
                    BreakerState::Dashing,
                    BreakerState::Braking | BreakerState::Settling
                ) | (BreakerState::Braking, BreakerState::Settling)
                    | (BreakerState::Settling, BreakerState::Idle)
            );
            if !legal {
                log.0.push(ViolationEntry {
                    frame: frame.0,
                    invariant: InvariantKind::ValidBreakerState,
                    entity: None,
                    message: format!(
                        "ValidBreakerState FAIL frame={} {prev:?} → {current:?}",
                        frame.0,
                    ),
                });
            }
        }
        previous.insert(entity, current);
    }
    previous.retain(|e, _| breakers.contains(*e));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn test_app_valid_breaker_state() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .add_systems(FixedUpdate, check_valid_breaker_state);
        app
    }

    /// `Idle → Braking` is illegal (must go through `Dashing`). The system must
    /// append a [`ViolationEntry`] with [`InvariantKind::ValidBreakerState`].
    ///
    /// Tick 1 seeds `Local` with `Idle`. Tick 2 sees `Braking` → violation.
    #[test]
    fn valid_breaker_state_fires_on_idle_to_braking() {
        let mut app = test_app_valid_breaker_state();

        let entity = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle))
            .id();

        // Tick 1: system stores Idle in Local, no previous to compare → no violation
        tick(&mut app);

        let log_after_tick1 = app.world().resource::<ViolationLog>();
        assert!(
            log_after_tick1.0.is_empty(),
            "no violation expected on first tick (no previous state)"
        );

        // Mutate to Braking (illegal: Idle → Braking)
        *app.world_mut()
            .entity_mut(entity)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Braking;

        // Tick 2: system compares Braking vs previous Idle → should fire
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one ValidBreakerState violation, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::ValidBreakerState);
    }

    /// `Idle → Dashing` is a legal transition. No violation should be recorded.
    #[test]
    fn valid_breaker_state_does_not_fire_on_idle_to_dashing() {
        let mut app = test_app_valid_breaker_state();

        let entity = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle))
            .id();

        // Tick 1: seeds Local with Idle
        tick(&mut app);

        // Change to Dashing (legal: Idle → Dashing)
        *app.world_mut()
            .entity_mut(entity)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Dashing;

        // Tick 2: should NOT fire
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation for Idle→Dashing (legal), got: {:?}",
            log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }

    /// `Settling → Dashing` is legal (breaker can re-dash from settling).
    /// No violation should be recorded.
    #[test]
    fn valid_breaker_state_does_not_fire_on_settling_to_dashing() {
        let mut app = test_app_valid_breaker_state();

        let entity = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Settling))
            .id();

        // Tick 1: seeds Local with Settling
        tick(&mut app);

        // Transition to Dashing (legal: Settling → Dashing)
        *app.world_mut()
            .entity_mut(entity)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Dashing;

        // Tick 2: Settling → Dashing is legal → no violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation for Settling→Dashing (legal), got: {:?}",
            log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }

    /// When the state does not change (`Idle → Idle`), no violation should fire.
    #[test]
    fn valid_breaker_state_does_not_fire_on_no_state_change() {
        let mut app = test_app_valid_breaker_state();

        app.world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle));

        // Tick 1: seeds Local
        tick(&mut app);
        // Tick 2: same state
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when state does not change"
        );
    }

    /// On the very first tick (no previous state stored in `Local`), the system must
    /// not fire even for `Dashing` — there is no prior state to compare.
    #[test]
    fn valid_breaker_state_skips_first_frame_with_no_previous() {
        let mut app = test_app_valid_breaker_state();

        // Start directly in Dashing (would be illegal from Idle, but first frame only)
        app.world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Dashing));

        // Only one tick — Local starts empty, no comparison possible
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation on first frame (Local has no previous)"
        );
    }

    /// Two [`ScenarioTagBreaker`] entities are tracked independently. When entity A
    /// makes a legal transition (`Idle → Dashing`) and entity B makes an illegal
    /// transition (`Idle → Braking`), exactly one violation fires — for entity B.
    #[test]
    fn valid_breaker_state_tracks_two_breakers_independently_one_illegal() {
        let mut app = test_app_valid_breaker_state();

        // Spawn entity A and entity B, both starting Idle
        let entity_a = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle))
            .id();
        let entity_b = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle))
            .id();

        // Tick 1: seeds Local for both A (Idle) and B (Idle)
        tick(&mut app);

        assert!(
            app.world().resource::<ViolationLog>().0.is_empty(),
            "no violation expected after seeding tick (no previous state to compare)"
        );

        // Entity A: Idle → Dashing (legal)
        *app.world_mut()
            .entity_mut(entity_a)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Dashing;

        // Entity B: Idle → Braking (illegal — skips Dashing)
        *app.world_mut()
            .entity_mut(entity_b)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Braking;

        // Tick 2: A is legal, B is illegal → exactly 1 violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly 1 ValidBreakerState violation (entity B's Idle→Braking is illegal), got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::ValidBreakerState);
    }

    /// When both [`ScenarioTagBreaker`] entities make legal transitions
    /// (`Idle → Dashing`), no [`ViolationEntry`] should be recorded.
    #[test]
    fn valid_breaker_state_produces_no_violation_when_both_breakers_transition_legally() {
        let mut app = test_app_valid_breaker_state();

        // Spawn two breakers, both Idle
        let entity_a = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle))
            .id();
        let entity_b = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle))
            .id();

        // Tick 1: seeds Local for A=Idle, B=Idle
        tick(&mut app);

        assert!(
            app.world().resource::<ViolationLog>().0.is_empty(),
            "no violation expected on seeding tick"
        );

        // Both transition Idle → Dashing (legal)
        *app.world_mut()
            .entity_mut(entity_a)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Dashing;
        *app.world_mut()
            .entity_mut(entity_b)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Dashing;

        // Tick 2: both legal → no violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no ValidBreakerState violation when both breakers transition Idle→Dashing (legal), got: {:?}",
            log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }

    /// Spawn a breaker with `BreakerState::Braking`, tick once to seed the Local
    /// `HashMap`, despawn the entity, then spawn a new breaker with
    /// `BreakerState::Idle`. If the `HashMap` is not cleaned up on despawn, the
    /// new entity (which may recycle the ID) will be compared against the stale
    /// `Braking` entry and fire a false `ValidBreakerState` violation.
    ///
    /// After the despawn+respawn cycle, no violation must fire.
    #[test]
    fn valid_breaker_state_no_violation_after_despawn_and_respawn() {
        let mut app = test_app_valid_breaker_state();

        // Spawn first breaker in Braking state
        let entity = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Braking))
            .id();

        // Tick 1: system inserts entity → BreakerState::Braking into Local HashMap
        tick(&mut app);

        assert!(
            app.world().resource::<ViolationLog>().0.is_empty(),
            "no violation expected on first tick (no previous state to compare)"
        );

        // Despawn the breaker — system must remove it from Local HashMap
        app.world_mut().entity_mut(entity).despawn();

        // Tick 2: entity is gone; system should prune stale HashMap entries
        tick(&mut app);

        assert!(
            app.world().resource::<ViolationLog>().0.is_empty(),
            "no violation expected when tagged entity is despawned"
        );

        // Spawn a new breaker with Idle state — may receive a recycled entity ID
        app.world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Idle));

        // Tick 3: new entity appears for first time — no previous state in HashMap → no violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            !log.0
                .iter()
                .any(|v| v.invariant == InvariantKind::ValidBreakerState),
            "expected no ValidBreakerState violation after despawn+respawn cycle, \
            got: {:?}",
            log.0
                .iter()
                .filter(|v| v.invariant == InvariantKind::ValidBreakerState)
                .map(|e| &e.message)
                .collect::<Vec<_>>()
        );
    }

    /// `Dashing → Settling` is the dash-cancel transition triggered by a perfect
    /// bump. It should be legal and produce no violation.
    #[test]
    fn valid_breaker_state_does_not_fire_on_dashing_to_settling_dash_cancel() {
        let mut app = test_app_valid_breaker_state();

        let entity = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Dashing))
            .id();

        // Tick 1: seeds Local with Dashing
        tick(&mut app);

        // Transition to Settling (dash cancel — legal)
        *app.world_mut()
            .entity_mut(entity)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Settling;

        // Tick 2: Dashing → Settling should be legal
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation for Dashing→Settling (dash cancel is legal), got: {:?}",
            log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }

    /// When `GameState` transitions (e.g., re-entering `Playing` for a new node),
    /// the breaker state tracker clears. A breaker that was `Braking` before the
    /// transition and is now `Idle` (from `reset_breaker`) should not fire.
    #[test]
    fn valid_breaker_state_clears_tracking_on_game_state_transition() {
        let mut app = test_app_valid_breaker_state();

        let entity = app
            .world_mut()
            .spawn((ScenarioTagBreaker, BreakerState::Braking))
            .id();

        // Tick 1: seeds tracking with Braking (GameState starts at Loading)
        tick(&mut app);

        assert!(app.world().resource::<ViolationLog>().0.is_empty());

        // Simulate node transition: change GameState so the tracker clears
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::MainMenu);
        app.update(); // process state transition

        // Change breaker to Idle (what reset_breaker does)
        *app.world_mut()
            .entity_mut(entity)
            .get_mut::<BreakerState>()
            .unwrap() = BreakerState::Idle;

        // Tick 2: GameState changed Loading→MainMenu → tracking was cleared
        // → Idle is treated as first frame, no comparison → no violation
        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation after GameState transition clears tracking, got: {:?}",
            log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }
}
