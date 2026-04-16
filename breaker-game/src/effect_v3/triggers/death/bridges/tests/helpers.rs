use std::marker::PhantomData;

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::super::system::{
    on_bolt_destroyed, on_breaker_destroyed, on_cell_destroyed, on_wall_destroyed,
};
use crate::{
    effect_v3::{
        effects::SpeedBoostConfig,
        types::{EffectType, Tree, Trigger},
    },
    prelude::*,
};

// -- Test message resources -----------------------------------------------

#[derive(Resource, Default)]
pub(super) struct TestCellDestroyedMessages(pub(super) Vec<Destroyed<Cell>>);

#[derive(Resource, Default)]
pub(super) struct TestBoltDestroyedMessages(pub(super) Vec<Destroyed<Bolt>>);

#[derive(Resource, Default)]
pub(super) struct TestBreakerDestroyedMessages(pub(super) Vec<Destroyed<Breaker>>);

#[derive(Resource, Default)]
pub(super) struct TestWallDestroyedMessages(pub(super) Vec<Destroyed<Wall>>);

fn inject_cell_destroyed(
    messages: Res<TestCellDestroyedMessages>,
    mut writer: MessageWriter<Destroyed<Cell>>,
) {
    for msg in &messages.0 {
        writer.write(msg.clone());
    }
}

fn inject_bolt_destroyed(
    messages: Res<TestBoltDestroyedMessages>,
    mut writer: MessageWriter<Destroyed<Bolt>>,
) {
    for msg in &messages.0 {
        writer.write(msg.clone());
    }
}

fn inject_breaker_destroyed(
    messages: Res<TestBreakerDestroyedMessages>,
    mut writer: MessageWriter<Destroyed<Breaker>>,
) {
    for msg in &messages.0 {
        writer.write(msg.clone());
    }
}

fn inject_wall_destroyed(
    messages: Res<TestWallDestroyedMessages>,
    mut writer: MessageWriter<Destroyed<Wall>>,
) {
    for msg in &messages.0 {
        writer.write(msg.clone());
    }
}

pub(super) fn cell_death_test_app() -> App {
    TestAppBuilder::new()
        .with_message::<Destroyed<Cell>>()
        .with_resource::<TestCellDestroyedMessages>()
        .with_system(
            FixedUpdate,
            (
                inject_cell_destroyed.before(on_cell_destroyed),
                on_cell_destroyed,
            ),
        )
        .build()
}

pub(super) fn bolt_death_test_app() -> App {
    TestAppBuilder::new()
        .with_message::<Destroyed<Bolt>>()
        .with_resource::<TestBoltDestroyedMessages>()
        .with_system(
            FixedUpdate,
            (
                inject_bolt_destroyed.before(on_bolt_destroyed),
                on_bolt_destroyed,
            ),
        )
        .build()
}

pub(super) fn breaker_death_test_app() -> App {
    TestAppBuilder::new()
        .with_message::<Destroyed<Breaker>>()
        .with_resource::<TestBreakerDestroyedMessages>()
        .with_system(
            FixedUpdate,
            (
                inject_breaker_destroyed.before(on_breaker_destroyed),
                on_breaker_destroyed,
            ),
        )
        .build()
}

pub(super) fn wall_death_test_app() -> App {
    TestAppBuilder::new()
        .with_message::<Destroyed<Wall>>()
        .with_resource::<TestWallDestroyedMessages>()
        .with_system(
            FixedUpdate,
            (
                inject_wall_destroyed.before(on_wall_destroyed),
                on_wall_destroyed,
            ),
        )
        .build()
}

/// Helper to build a When(trigger, Fire(SpeedBoost)) tree.
pub(super) fn death_speed_tree(name: &str, trigger: Trigger, multiplier: f32) -> (String, Tree) {
    (
        name.to_string(),
        Tree::When(
            trigger,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(multiplier),
            }))),
        ),
    )
}

pub(super) fn destroyed_cell(victim: Entity, killer: Option<Entity>) -> Destroyed<Cell> {
    Destroyed {
        victim,
        killer,
        victim_pos: Vec2::ZERO,
        killer_pos: killer.map(|_| Vec2::ZERO),
        _marker: PhantomData,
    }
}

pub(super) fn destroyed_bolt(victim: Entity, killer: Option<Entity>) -> Destroyed<Bolt> {
    Destroyed {
        victim,
        killer,
        victim_pos: Vec2::ZERO,
        killer_pos: killer.map(|_| Vec2::ZERO),
        _marker: PhantomData,
    }
}

pub(super) fn destroyed_breaker(victim: Entity, killer: Option<Entity>) -> Destroyed<Breaker> {
    Destroyed {
        victim,
        killer,
        victim_pos: Vec2::ZERO,
        killer_pos: killer.map(|_| Vec2::ZERO),
        _marker: PhantomData,
    }
}

pub(super) fn destroyed_wall(victim: Entity, killer: Option<Entity>) -> Destroyed<Wall> {
    Destroyed {
        victim,
        killer,
        victim_pos: Vec2::ZERO,
        killer_pos: killer.map(|_| Vec2::ZERO),
        _marker: PhantomData,
    }
}
