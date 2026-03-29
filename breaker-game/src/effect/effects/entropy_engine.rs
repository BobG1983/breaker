//! Escalating chaos — fires multiple random effects on the primary bolt per cell destroyed.

use bevy::prelude::*;
use rand::distr::{Distribution, weighted::WeightedIndex};

use crate::{
    effect::{EffectNode, StagedEffects},
    shared::{playing_state::PlayingState, rng::GameRng},
};

/// Tracks the kill count within the current node for entropy scaling.
#[derive(Component, Debug, Clone)]
pub struct EntropyEngineState {
    /// Cells destroyed this node (resets between nodes).
    pub kill_count: u32,
}

/// Fires multiple random effects from the weighted pool.
///
/// Number of effects scales with kill count up to `max_effects`.
/// Resets between nodes.
pub(crate) fn fire(
    entity: Entity,
    max_effects: u32,
    pool: &[(f32, EffectNode)],
    world: &mut World,
) {
    // Step 1: Insert EntropyEngineState if absent
    if world.get::<EntropyEngineState>(entity).is_none() {
        world
            .entity_mut(entity)
            .insert(EntropyEngineState { kill_count: 0 });
    }

    // Step 2: Increment kill_count and compute effects to fire
    let effects_to_fire = {
        let mut state = world.get_mut::<EntropyEngineState>(entity).unwrap();
        state.kill_count = state.kill_count.saturating_add(1);
        state.kill_count.min(max_effects)
    };

    // Step 3: Empty pool guard AFTER kill_count increment
    if pool.is_empty() {
        warn!("entropy_engine: empty pool for entity {:?}", entity);
        return;
    }

    if effects_to_fire == 0 {
        return;
    }

    // Step 4: Pre-sample all indices from GameRng before dispatching
    let selected_indices: Vec<usize> = {
        let mut rng = world.resource_mut::<GameRng>();
        let weights: Vec<f32> = pool.iter().map(|(w, _)| *w).collect();
        let Ok(dist) = WeightedIndex::new(&weights) else {
            warn!("entropy_engine: all-zero weights for entity {:?}", entity);
            return;
        };
        (0..effects_to_fire)
            .map(|_| dist.sample(&mut rng.0))
            .collect()
    };

    // Step 5: Dispatch effects
    for idx in selected_indices {
        let node = pool[idx].1.clone();
        match node {
            EffectNode::Do(effect) => effect.fire(entity, world),
            other => {
                if let Some(mut staged) = world.get_mut::<StagedEffects>(entity) {
                    staged.0.push((String::new(), other));
                } else {
                    world
                        .entity_mut(entity)
                        .insert(StagedEffects(vec![(String::new(), other)]));
                }
            }
        }
    }
}

/// No-op — inner effects handle their own reversal.
pub(crate) fn reverse(_entity: Entity, _world: &mut World) {}

/// Registers systems for `EntropyEngine` effect.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        OnEnter(PlayingState::Active),
        reset_entropy_engine_on_node_start,
    );
}

fn reset_entropy_engine_on_node_start(mut query: Query<&mut EntropyEngineState>) {
    for mut state in &mut query {
        state.kill_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::{
        effect::{
            core::{EffectKind, StagedEffects, Trigger},
            effects::{damage_boost::ActiveDamageBoosts, speed_boost::ActiveSpeedBoosts},
        },
        shared::{game_state::GameState, playing_state::PlayingState, rng::GameRng},
    };

    // ── Behavior 9: fire() with no prior state inserts state and fires 1 effect ──

    #[test]
    fn fire_inserts_state_and_fires_one_effect_when_no_prior_state() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world
            .spawn((StagedEffects::default(), ActiveDamageBoosts(vec![])))
            .id();
        let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

        fire(entity, 3, &pool, &mut world);

        let state = world.get::<EntropyEngineState>(entity).unwrap();
        assert_eq!(
            state.kill_count, 1,
            "kill_count should be 1 (inserted at 0, then saturating_add(1))"
        );
        let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            active.0,
            vec![2.0],
            "1 effect should be fired (kill_count=1, min(1, 3)=1)"
        );
    }

    #[test]
    fn fire_inserts_state_fresh_when_entity_has_none() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world
            .spawn((StagedEffects::default(), ActiveDamageBoosts(vec![])))
            .id();

        // Verify no EntropyEngineState before fire
        assert!(
            world.get::<EntropyEngineState>(entity).is_none(),
            "entity should start without EntropyEngineState"
        );

        let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];
        fire(entity, 3, &pool, &mut world);

        assert!(
            world.get::<EntropyEngineState>(entity).is_some(),
            "EntropyEngineState should be inserted by fire()"
        );
    }

    // ── Behavior 10: fire() increments kill_count and fires N=min(kill_count, max_effects) ──

    #[test]
    fn fire_increments_kill_count_and_fires_n_effects() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world
            .spawn((
                EntropyEngineState { kill_count: 2 },
                StagedEffects::default(),
                ActiveDamageBoosts(vec![]),
            ))
            .id();
        let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

        fire(entity, 5, &pool, &mut world);

        let state = world.get::<EntropyEngineState>(entity).unwrap();
        assert_eq!(state.kill_count, 3, "kill_count should be 3 (2 + 1)");
        let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            active.0.len(),
            3,
            "3 effects should fire (min(3, 5)=3), got {:?}",
            active.0
        );
        // All entries should be 2.0 since there's only one pool entry
        for val in &active.0 {
            assert!(
                (*val - 2.0).abs() < f32::EPSILON,
                "each entry should be 2.0, got {val}"
            );
        }
    }

    #[test]
    fn fire_with_kill_count_4_max_5_fires_5_effects() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world
            .spawn((
                EntropyEngineState { kill_count: 4 },
                StagedEffects::default(),
                ActiveDamageBoosts(vec![]),
            ))
            .id();
        let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

        fire(entity, 5, &pool, &mut world);

        let state = world.get::<EntropyEngineState>(entity).unwrap();
        assert_eq!(state.kill_count, 5, "kill_count should be 5 (4 + 1)");
        let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(active.0.len(), 5, "5 effects should fire (min(5, 5)=5)");
    }

    // ── Behavior 11: fire() caps at max_effects when kill_count exceeds it ──

    #[test]
    fn fire_caps_at_max_effects() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world
            .spawn((
                EntropyEngineState { kill_count: 10 },
                StagedEffects::default(),
                ActiveDamageBoosts(vec![]),
            ))
            .id();
        let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

        fire(entity, 3, &pool, &mut world);

        let state = world.get::<EntropyEngineState>(entity).unwrap();
        assert_eq!(state.kill_count, 11, "kill_count should be 11 (10 + 1)");
        let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            active.0.len(),
            3,
            "effects capped at max_effects=3 despite kill_count=11"
        );
    }

    // ── Behavior 12: fire() with max_effects=1 always fires exactly 1 ──

    #[test]
    fn fire_with_max_effects_1_fires_exactly_one() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world
            .spawn((
                EntropyEngineState { kill_count: 50 },
                StagedEffects::default(),
                ActiveDamageBoosts(vec![]),
            ))
            .id();
        let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

        fire(entity, 1, &pool, &mut world);

        let state = world.get::<EntropyEngineState>(entity).unwrap();
        assert_eq!(state.kill_count, 51, "kill_count should be 51 (50 + 1)");
        let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert_eq!(
            active.0.len(),
            1,
            "exactly 1 effect should fire with max_effects=1"
        );
    }

    // ── Behavior 13: fire() with empty pool still increments kill_count ──

    #[test]
    fn fire_with_empty_pool_increments_kill_count_but_fires_nothing() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world
            .spawn((
                EntropyEngineState { kill_count: 5 },
                StagedEffects::default(),
            ))
            .id();
        let pool: Vec<(f32, EffectNode)> = vec![];

        fire(entity, 3, &pool, &mut world);

        let state = world.get::<EntropyEngineState>(entity).unwrap();
        assert_eq!(
            state.kill_count, 6,
            "kill_count should increment to 6 even with empty pool"
        );
    }

    // ── Behavior 14: fire() selects N effects independently — same effect can repeat ──

    #[test]
    fn fire_selects_effects_independently_total_equals_n() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world
            .spawn((
                EntropyEngineState { kill_count: 2 },
                StagedEffects::default(),
                ActiveDamageBoosts(vec![]),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();
        let pool = vec![
            (1.0, EffectNode::Do(EffectKind::DamageBoost(2.0))),
            (
                1.0,
                EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
            ),
        ];

        fire(entity, 5, &pool, &mut world);

        let state = world.get::<EntropyEngineState>(entity).unwrap();
        assert_eq!(state.kill_count, 3, "kill_count should be 3 (2 + 1)");

        let damage = world.get::<ActiveDamageBoosts>(entity).unwrap();
        let speed = world.get::<ActiveSpeedBoosts>(entity).unwrap();
        let total = damage.0.len() + speed.0.len();
        assert_eq!(
            total, 3,
            "exactly 3 effects should fire (min(3, 5)=3), got {} (damage: {:?}, speed: {:?})",
            total, damage.0, speed.0
        );
    }

    // ── Behavior 15: fire() with non-Do pool entry pushes to StagedEffects ──

    #[test]
    fn fire_with_non_do_pushes_to_staged_effects() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let non_do_node = EffectNode::When {
            trigger: Trigger::CellDestroyed,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
        };
        let entity = world
            .spawn((
                EntropyEngineState { kill_count: 0 },
                StagedEffects::default(),
            ))
            .id();
        let pool = vec![(1.0, non_do_node.clone())];

        fire(entity, 3, &pool, &mut world);

        let state = world.get::<EntropyEngineState>(entity).unwrap();
        assert_eq!(state.kill_count, 1, "kill_count should be 1");

        let staged = world.get::<StagedEffects>(entity).unwrap();
        assert_eq!(
            staged.0.len(),
            1,
            "1 non-Do effect should be pushed to StagedEffects (min(1, 3)=1)"
        );
        assert_eq!(staged.0[0].0, "", "chip name should be empty string");
        assert_eq!(
            staged.0[0].1, non_do_node,
            "the When node should be pushed to StagedEffects"
        );
    }

    // ── Behavior 16: fire() is deterministic for the same seed ──

    #[test]
    fn fire_deterministic_for_same_seed() {
        let pool = vec![
            (1.0, EffectNode::Do(EffectKind::DamageBoost(2.0))),
            (
                2.0,
                EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
            ),
        ];

        // World 1
        let mut world1 = World::new();
        world1.insert_resource(GameRng::from_seed(77));
        let entity1 = world1
            .spawn((
                EntropyEngineState { kill_count: 4 },
                StagedEffects::default(),
                ActiveDamageBoosts(vec![]),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();
        fire(entity1, 5, &pool, &mut world1);

        // World 2
        let mut world2 = World::new();
        world2.insert_resource(GameRng::from_seed(77));
        let entity2 = world2
            .spawn((
                EntropyEngineState { kill_count: 4 },
                StagedEffects::default(),
                ActiveDamageBoosts(vec![]),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();
        fire(entity2, 5, &pool, &mut world2);

        let damage1 = world1.get::<ActiveDamageBoosts>(entity1).unwrap();
        let damage2 = world2.get::<ActiveDamageBoosts>(entity2).unwrap();
        let speed1 = world1.get::<ActiveSpeedBoosts>(entity1).unwrap();
        let speed2 = world2.get::<ActiveSpeedBoosts>(entity2).unwrap();
        let state1 = world1.get::<EntropyEngineState>(entity1).unwrap();
        let state2 = world2.get::<EntropyEngineState>(entity2).unwrap();

        assert_eq!(state1.kill_count, 5, "world1 kill_count should be 5");
        assert_eq!(state2.kill_count, 5, "world2 kill_count should be 5");
        assert_eq!(
            damage1.0, damage2.0,
            "ActiveDamageBoosts must match across worlds with same seed"
        );
        assert_eq!(
            speed1.0, speed2.0,
            "ActiveSpeedBoosts must match across worlds with same seed"
        );
    }

    // ── Behavior 17: fire() with max_effects=0 fires zero effects ──

    #[test]
    fn fire_with_max_effects_zero_fires_nothing() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world
            .spawn((
                EntropyEngineState { kill_count: 3 },
                StagedEffects::default(),
                ActiveDamageBoosts(vec![]),
            ))
            .id();
        let pool = vec![(1.0, EffectNode::Do(EffectKind::DamageBoost(2.0)))];

        fire(entity, 0, &pool, &mut world);

        let state = world.get::<EntropyEngineState>(entity).unwrap();
        assert_eq!(
            state.kill_count, 4,
            "kill_count should increment to 4 even with max_effects=0"
        );
        let active = world.get::<ActiveDamageBoosts>(entity).unwrap();
        assert!(
            active.0.is_empty(),
            "no effects should fire with max_effects=0"
        );
    }

    // ── Behavior 18: fire() with all-zero weights increments kill_count but fires nothing ──

    #[test]
    fn fire_with_all_zero_weights_increments_kill_count_but_no_effects() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let entity = world
            .spawn((
                EntropyEngineState { kill_count: 2 },
                StagedEffects::default(),
                ActiveDamageBoosts(vec![]),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();
        let pool = vec![
            (0.0, EffectNode::Do(EffectKind::DamageBoost(2.0))),
            (
                0.0,
                EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 }),
            ),
        ];

        fire(entity, 5, &pool, &mut world);

        let state = world.get::<EntropyEngineState>(entity).unwrap();
        assert_eq!(
            state.kill_count, 3,
            "kill_count should increment to 3 even with all-zero weights"
        );
        let damage = world.get::<ActiveDamageBoosts>(entity).unwrap();
        let speed = world.get::<ActiveSpeedBoosts>(entity).unwrap();
        assert!(damage.0.is_empty(), "no DamageBoost with all-zero weights");
        assert!(speed.0.is_empty(), "no SpeedBoost with all-zero weights");
    }

    // ── Behavior 19: reverse() is a no-op (entity with state) ──

    #[test]
    fn reverse_with_state_is_noop() {
        let mut world = World::new();
        let entity = world.spawn(EntropyEngineState { kill_count: 5 }).id();

        reverse(entity, &mut world);

        let state = world.get::<EntropyEngineState>(entity).unwrap();
        assert_eq!(state.kill_count, 5, "reverse should not modify kill_count");
    }

    // ── Behavior 20: reverse() on entity without state is a no-op ──

    #[test]
    fn reverse_without_state_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Should not panic
        reverse(entity, &mut world);

        assert!(
            world.get::<EntropyEngineState>(entity).is_none(),
            "no EntropyEngineState should be inserted by reverse"
        );
    }

    // ── Behavior 21: register() wires reset system for OnEnter(PlayingState::Active) ──

    fn test_app_with_reset() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.init_state::<GameState>();
        app.add_sub_state::<PlayingState>();
        register(&mut app);
        app
    }

    fn enter_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();
    }

    #[test]
    fn reset_system_clears_kill_count_on_node_start() {
        let mut app = test_app_with_reset();
        enter_playing(&mut app);

        let entity = app
            .world_mut()
            .spawn(EntropyEngineState { kill_count: 7 })
            .id();

        // Transition out and back in to trigger OnEnter(PlayingState::Active) again
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Loading);
        app.update();

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        let state = app.world().get::<EntropyEngineState>(entity).unwrap();
        assert_eq!(
            state.kill_count, 0,
            "kill_count should be reset to 0 on node start"
        );
    }

    #[test]
    fn reset_system_clears_multiple_entities() {
        let mut app = test_app_with_reset();
        enter_playing(&mut app);

        let entity1 = app
            .world_mut()
            .spawn(EntropyEngineState { kill_count: 7 })
            .id();
        let entity2 = app
            .world_mut()
            .spawn(EntropyEngineState { kill_count: 15 })
            .id();

        // Transition out and back to trigger reset
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Loading);
        app.update();

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        let state1 = app.world().get::<EntropyEngineState>(entity1).unwrap();
        let state2 = app.world().get::<EntropyEngineState>(entity2).unwrap();
        assert_eq!(state1.kill_count, 0, "entity1 kill_count should be reset");
        assert_eq!(state2.kill_count, 0, "entity2 kill_count should be reset");
    }

    // ── Behavior 22: reset system does not remove the component ──

    #[test]
    fn reset_system_does_not_remove_component() {
        let mut app = test_app_with_reset();
        enter_playing(&mut app);

        let entity = app
            .world_mut()
            .spawn(EntropyEngineState { kill_count: 15 })
            .id();

        // Trigger reset
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Loading);
        app.update();

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        assert!(
            app.world().get::<EntropyEngineState>(entity).is_some(),
            "EntropyEngineState component should still exist after reset (not removed)"
        );
        let state = app.world().get::<EntropyEngineState>(entity).unwrap();
        assert_eq!(state.kill_count, 0, "kill_count should be 0 after reset");
    }

    // ── Behavior 23: fire() works with mixed Do and non-Do pool entries ──

    #[test]
    fn fire_with_mixed_do_and_non_do_dispatches_correctly() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let non_do_node = EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        };
        let entity = world
            .spawn((
                EntropyEngineState { kill_count: 1 },
                StagedEffects::default(),
                ActiveDamageBoosts(vec![]),
                ActiveSpeedBoosts(vec![]),
            ))
            .id();
        let pool = vec![
            (1.0, EffectNode::Do(EffectKind::DamageBoost(2.0))),
            (1.0, non_do_node),
        ];

        fire(entity, 5, &pool, &mut world);

        let state = world.get::<EntropyEngineState>(entity).unwrap();
        assert_eq!(state.kill_count, 2, "kill_count should be 2 (1 + 1)");

        let damage = world.get::<ActiveDamageBoosts>(entity).unwrap();
        let staged = world.get::<StagedEffects>(entity).unwrap();
        let total = damage.0.len() + staged.0.len();
        assert_eq!(
            total, 2,
            "exactly 2 dispatches (min(2, 5)=2): damage entries + staged entries = {}, got (damage: {:?}, staged: {:?})",
            total, damage.0, staged.0
        );
    }

    // ── Behavior 24: fire() inserts StagedEffects fresh when absent and non-Do selected ──

    #[test]
    fn fire_inserts_staged_effects_when_absent_and_non_do_selected() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let non_do_node = EffectNode::When {
            trigger: Trigger::CellDestroyed,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
        };
        // Entity has EntropyEngineState but NO StagedEffects
        let entity = world.spawn(EntropyEngineState { kill_count: 0 }).id();
        let pool = vec![(1.0, non_do_node.clone())];

        fire(entity, 3, &pool, &mut world);

        let state = world.get::<EntropyEngineState>(entity).unwrap();
        assert_eq!(state.kill_count, 1, "kill_count should be 1");

        // Per the spec, StagedEffects should be freshly inserted
        let staged = world.get::<StagedEffects>(entity);
        assert!(
            staged.is_some(),
            "StagedEffects should be inserted when absent and non-Do node is selected"
        );
        let staged = staged.unwrap();
        assert_eq!(
            staged.0.len(),
            1,
            "1 non-Do entry should be in StagedEffects"
        );
        assert_eq!(staged.0[0].0, "", "chip name should be empty string");
        assert_eq!(staged.0[0].1, non_do_node, "non-Do node should be pushed");
    }

    #[test]
    fn fire_inserts_both_state_and_staged_effects_when_both_absent() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let non_do_node = EffectNode::When {
            trigger: Trigger::CellDestroyed,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
        };
        // Entity has NEITHER EntropyEngineState NOR StagedEffects
        let entity = world.spawn_empty().id();
        let pool = vec![(1.0, non_do_node)];

        fire(entity, 3, &pool, &mut world);

        assert!(
            world.get::<EntropyEngineState>(entity).is_some(),
            "EntropyEngineState should be inserted"
        );
        assert!(
            world.get::<StagedEffects>(entity).is_some(),
            "StagedEffects should be inserted"
        );
        let state = world.get::<EntropyEngineState>(entity).unwrap();
        assert_eq!(state.kill_count, 1);
    }
}
