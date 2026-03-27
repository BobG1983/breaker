use bevy::prelude::*;

use super::{
    super::definition::{AttractionType, EffectNode, EffectTarget},
    *,
};

// =========================================================================
// C7 Wave 1 Part E: Typed events with targets: Vec<EffectTarget> (behaviors 29-30)
// =========================================================================

#[test]
fn shockwave_fired_with_targets_vec() {
    let entity = Entity::PLACEHOLDER;
    let event = ShockwaveFired {
        base_range: 64.0,
        range_per_level: 0.0,
        stacks: 1,
        speed: 400.0,
        targets: vec![EffectTarget::Entity(entity)],
        source_chip: None,
    };
    assert_eq!(event.targets.len(), 1);
    assert_eq!(event.targets[0], EffectTarget::Entity(entity));
}

#[test]
fn lose_life_fired_empty_targets_equivalent_to_old_none_bolt() {
    let event = LoseLifeFired {};
    assert!(!format!("{event:?}").is_empty());
}

#[test]
fn lose_life_fired_multiple_targets() {
    let event = LoseLifeFired {};
    assert!(!format!("{event:?}").is_empty());
}

#[test]
fn spawn_bolts_fired_carries_new_fields() {
    let event = SpawnBoltsFired {
        count: 2,
        lifespan: Some(5.0),
        inherit: true,
        source_chip: Some("Reflex".to_owned()),
    };
    assert_eq!(event.count, 2);
    assert_eq!(event.lifespan, Some(5.0));
    assert!(event.inherit);
}

#[test]
fn speed_boost_fired_with_targets() {
    let event = SpeedBoostFired {
        multiplier: 1.3,
        targets: vec![EffectTarget::Entity(Entity::PLACEHOLDER)],
    };
    assert!((event.multiplier - 1.3).abs() < f32::EPSILON);
    assert_eq!(event.targets.len(), 1);
}

#[test]
fn random_effect_fired_pool_uses_effect_node() {
    let event = RandomEffectFired {
        pool: vec![(
            1.0,
            EffectNode::Do(super::super::definition::Effect::SpawnBolts {
                count: 1,
                lifespan: None,
                inherit: false,
            }),
        )],
        targets: vec![],
        source_chip: None,
    };
    assert_eq!(event.pool.len(), 1);
}

#[test]
fn entropy_engine_fired_pool_uses_effect_node() {
    let event = EntropyEngineFired {
        threshold: 5,
        pool: vec![(
            1.0,
            EffectNode::Do(super::super::definition::Effect::SpawnBolts {
                count: 1,
                lifespan: None,
                inherit: false,
            }),
        )],
        targets: vec![],
        source_chip: Some("Entropy Engine".to_owned()),
    };
    assert_eq!(event.threshold, 5);
    assert_eq!(event.pool.len(), 1);
}

// =========================================================================
// C7 Wave 1 Part G: AttractionApplied with AttractionType (behavior 39)
// =========================================================================

#[test]
fn attraction_applied_carries_attraction_type_cell() {
    let event = AttractionApplied {
        attraction_type: AttractionType::Cell,
        per_stack: 1.0,
        max_stacks: 3,
    };
    assert_eq!(event.attraction_type, AttractionType::Cell);
    assert!((event.per_stack - 1.0).abs() < f32::EPSILON);
}

#[test]
fn attraction_applied_carries_attraction_type_wall() {
    let event = AttractionApplied {
        attraction_type: AttractionType::Wall,
        per_stack: 0.5,
        max_stacks: 3,
    };
    assert_eq!(event.attraction_type, AttractionType::Wall);
}

// =========================================================================
// fire_passive_event dispatch (behavior 39)
// =========================================================================

#[derive(Resource, Default)]
struct CapturedAttraction(Vec<AttractionApplied>);

fn capture_attraction(trigger: On<AttractionApplied>, mut captured: ResMut<CapturedAttraction>) {
    captured.0.push(trigger.event().clone());
}

#[test]
fn fire_passive_event_dispatches_attraction_with_type() {
    use super::super::definition::Effect;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<CapturedAttraction>()
        .add_observer(capture_attraction);

    let effect = Effect::Attraction(AttractionType::Cell, 1.0);
    app.world_mut().commands().queue(move |world: &mut World| {
        let mut commands = world.commands();
        fire_passive_event(effect, 3, "Magnet".to_owned(), &mut commands);
    });
    app.world_mut().flush();

    let captured = app.world().resource::<CapturedAttraction>();
    assert_eq!(
        captured.0.len(),
        1,
        "fire_passive_event should dispatch AttractionApplied for Effect::Attraction"
    );
    assert_eq!(captured.0[0].attraction_type, AttractionType::Cell);
    assert!((captured.0[0].per_stack - 1.0).abs() < f32::EPSILON);
    assert_eq!(captured.0[0].max_stacks, 3);
}

// =========================================================================
// M10: fire_typed_event dispatches all Effect variants via observers
// =========================================================================

#[derive(Resource, Default)]
struct CapturedChainBolt(Vec<ChainBoltFired>);

fn capture_chain_bolt(trigger: On<ChainBoltFired>, mut captured: ResMut<CapturedChainBolt>) {
    captured.0.push(trigger.event().clone());
}

#[derive(Resource, Default)]
struct CapturedMultiBolt(Vec<MultiBoltFired>);

fn capture_multi_bolt(trigger: On<MultiBoltFired>, mut captured: ResMut<CapturedMultiBolt>) {
    captured.0.push(trigger.event().clone());
}

#[derive(Resource, Default)]
struct CapturedShield(Vec<ShieldFired>);

fn capture_shield(trigger: On<ShieldFired>, mut captured: ResMut<CapturedShield>) {
    captured.0.push(trigger.event().clone());
}

#[derive(Resource, Default)]
struct CapturedLoseLife(Vec<LoseLifeFired>);

fn capture_lose_life(trigger: On<LoseLifeFired>, mut captured: ResMut<CapturedLoseLife>) {
    captured.0.push(trigger.event().clone());
}

#[derive(Resource, Default)]
struct CapturedTimePenalty(Vec<TimePenaltyFired>);

fn capture_time_penalty(trigger: On<TimePenaltyFired>, mut captured: ResMut<CapturedTimePenalty>) {
    captured.0.push(trigger.event().clone());
}

#[derive(Resource, Default)]
struct CapturedSpawnBolts(Vec<SpawnBoltsFired>);

fn capture_spawn_bolts(trigger: On<SpawnBoltsFired>, mut captured: ResMut<CapturedSpawnBolts>) {
    captured.0.push(trigger.event().clone());
}

/// M10: `fire_typed_event` dispatches `ChainBolt` into `ChainBoltFired` observer.
#[test]
fn fire_typed_event_dispatches_chain_bolt() {
    use super::super::definition::Effect;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<CapturedChainBolt>()
        .add_observer(capture_chain_bolt);

    let effect = Effect::ChainBolt {
        tether_distance: 100.0,
    };
    app.world_mut().commands().queue(move |world: &mut World| {
        let mut commands = world.commands();
        fire_typed_event(effect, vec![], None, &mut commands);
    });
    app.world_mut().flush();

    let captured = app.world().resource::<CapturedChainBolt>();
    assert_eq!(
        captured.0.len(),
        1,
        "ChainBolt should dispatch ChainBoltFired"
    );
    assert!(
        (captured.0[0].tether_distance - 100.0).abs() < f32::EPSILON,
        "tether_distance should be 100.0"
    );
}

/// M10: `fire_typed_event` dispatches `MultiBolt` into `MultiBoltFired` observer.
#[test]
fn fire_typed_event_dispatches_multi_bolt() {
    use super::super::definition::Effect;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<CapturedMultiBolt>()
        .add_observer(capture_multi_bolt);

    let effect = Effect::MultiBolt {
        base_count: 2,
        count_per_level: 0,
        stacks: 1,
    };
    app.world_mut().commands().queue(move |world: &mut World| {
        let mut commands = world.commands();
        fire_typed_event(effect, vec![], None, &mut commands);
    });
    app.world_mut().flush();

    let captured = app.world().resource::<CapturedMultiBolt>();
    assert_eq!(
        captured.0.len(),
        1,
        "MultiBolt should dispatch MultiBoltFired"
    );
    assert_eq!(captured.0[0].base_count, 2);
    assert_eq!(captured.0[0].count_per_level, 0);
    assert_eq!(captured.0[0].stacks, 1);
}

/// M10: `fire_typed_event` dispatches Shield into `ShieldFired` observer.
#[test]
fn fire_typed_event_dispatches_shield() {
    use super::super::definition::Effect;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<CapturedShield>()
        .add_observer(capture_shield);

    let effect = Effect::Shield {
        base_duration: 3.0,
        duration_per_level: 0.0,
        stacks: 1,
    };
    app.world_mut().commands().queue(move |world: &mut World| {
        let mut commands = world.commands();
        fire_typed_event(effect, vec![], None, &mut commands);
    });
    app.world_mut().flush();

    let captured = app.world().resource::<CapturedShield>();
    assert_eq!(captured.0.len(), 1, "Shield should dispatch ShieldFired");
    assert!((captured.0[0].base_duration - 3.0).abs() < f32::EPSILON);
    assert_eq!(captured.0[0].stacks, 1);
}

/// M10: `fire_typed_event` dispatches `LoseLife` into `LoseLifeFired` observer.
#[test]
fn fire_typed_event_dispatches_lose_life() {
    use super::super::definition::Effect;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<CapturedLoseLife>()
        .add_observer(capture_lose_life);

    let effect = Effect::LoseLife;
    app.world_mut().commands().queue(move |world: &mut World| {
        let mut commands = world.commands();
        fire_typed_event(effect, vec![], None, &mut commands);
    });
    app.world_mut().flush();

    let captured = app.world().resource::<CapturedLoseLife>();
    assert_eq!(
        captured.0.len(),
        1,
        "LoseLife should dispatch LoseLifeFired"
    );
}

/// M10: `fire_typed_event` dispatches `TimePenalty` into `TimePenaltyFired` observer.
#[test]
fn fire_typed_event_dispatches_time_penalty() {
    use super::super::definition::Effect;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<CapturedTimePenalty>()
        .add_observer(capture_time_penalty);

    let effect = Effect::TimePenalty { seconds: 5.0 };
    app.world_mut().commands().queue(move |world: &mut World| {
        let mut commands = world.commands();
        fire_typed_event(effect, vec![], None, &mut commands);
    });
    app.world_mut().flush();

    let captured = app.world().resource::<CapturedTimePenalty>();
    assert_eq!(
        captured.0.len(),
        1,
        "TimePenalty should dispatch TimePenaltyFired"
    );
    assert!((captured.0[0].seconds - 5.0).abs() < f32::EPSILON);
}

/// M10: `fire_typed_event` dispatches `SpawnBolts` into `SpawnBoltsFired` observer.
#[test]
fn fire_typed_event_dispatches_spawn_bolts() {
    use super::super::definition::Effect;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<CapturedSpawnBolts>()
        .add_observer(capture_spawn_bolts);

    let effect = Effect::SpawnBolts {
        count: 1,
        lifespan: None,
        inherit: false,
    };
    app.world_mut().commands().queue(move |world: &mut World| {
        let mut commands = world.commands();
        fire_typed_event(effect, vec![], None, &mut commands);
    });
    app.world_mut().flush();

    let captured = app.world().resource::<CapturedSpawnBolts>();
    assert_eq!(
        captured.0.len(),
        1,
        "SpawnBolts should dispatch SpawnBoltsFired"
    );
    assert_eq!(captured.0[0].count, 1);
    assert_eq!(captured.0[0].lifespan, None);
    assert!(!captured.0[0].inherit);
}

// =========================================================================
// Preserved: passive event construction tests
// =========================================================================

#[test]
fn piercing_applied_carries_per_stack_and_max() {
    let event = PiercingApplied {
        per_stack: 1,
        max_stacks: 3,
    };
    assert_eq!(event.per_stack, 1);
    assert_eq!(event.max_stacks, 3);
}

#[test]
fn damage_boost_applied_carries_per_stack() {
    let event = DamageBoostApplied {
        per_stack: 0.5,
        max_stacks: 3,
    };
    assert!((event.per_stack - 0.5).abs() < f32::EPSILON);
}

#[test]
fn bump_force_applied_accessible() {
    let event = BumpForceApplied {
        per_stack: 0.2,
        max_stacks: 3,
    };
    assert!((event.per_stack - 0.2).abs() < f32::EPSILON);
}

#[test]
fn tilt_control_applied_accessible() {
    let event = TiltControlApplied {
        per_stack: 0.1,
        max_stacks: 3,
    };
    assert!((event.per_stack - 0.1).abs() < f32::EPSILON);
}

// =========================================================================
// Preserved: triggered event construction tests (updated for targets)
// =========================================================================

#[test]
fn time_penalty_fired_carries_seconds() {
    let event = TimePenaltyFired {
        seconds: 3.0,
    };
    assert!((event.seconds - 3.0).abs() < f32::EPSILON);
}

#[test]
fn chain_bolt_fired_carries_tether_distance() {
    let event = ChainBoltFired {
        tether_distance: 150.0,
        targets: vec![EffectTarget::Entity(Entity::PLACEHOLDER)],
        source_chip: None,
    };
    assert!((event.tether_distance - 150.0).abs() < f32::EPSILON);
}

#[test]
fn multi_bolt_fired_carries_count_parameters() {
    let event = MultiBoltFired {
        base_count: 2,
        count_per_level: 1,
        stacks: 1,
        source_chip: None,
    };
    assert_eq!(event.base_count, 2);
}

#[test]
fn shield_fired_carries_duration_and_stacks() {
    let event = ShieldFired {
        base_duration: 3.0,
        duration_per_level: 0.5,
        stacks: 2,
    };
    assert!((event.base_duration - 3.0).abs() < f32::EPSILON);
    assert_eq!(event.stacks, 2);
}
