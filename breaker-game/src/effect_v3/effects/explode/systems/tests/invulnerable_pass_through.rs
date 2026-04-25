//! W7 Behavior 7 (Explode): the consumer must NOT pre-filter `Invulnerable`
//! cells — that responsibility lives with the pipeline's
//! `invulnerable_filter::<Cell>` in `DmgSystems::ApplyDamage`. The consumer
//! emits one `DamageDealt<Cell>` per in-range live cell (including
//! invulnerable ones); the pipeline zeroes the amount before applying.
//!
//! `Dead` cells, by contrast, ARE filtered out by the consumer's
//! `Without<Dead>` query filter (matches the existing pre-W7 `fire()` narrow
//! phase).

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

use super::{super::super::config::ExplodeConfig, helpers::*};
use crate::{effect_v3::traits::Fireable, prelude::*};

#[test]
fn invulnerable_cell_receives_message_but_pipeline_zeros_damage() {
    let mut app = explode_pipeline_app();
    let source = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();
    let cell_pos = Vec2::new(20.0, 0.0);
    let cell = app
        .world_mut()
        .spawn((
            Cell,
            Hp::new(100.0),
            KilledBy { killer: None },
            Invulnerable,
            Position2D(cell_pos),
            GlobalPosition2D(cell_pos),
            Spatial2D,
            Aabb2D::new(Vec2::ZERO, Vec2::splat(5.0)),
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
        ))
        .id();
    app.world_mut().run_schedule(FixedUpdate);

    let config = ExplodeConfig {
        range:  OrderedFloat(50.0),
        damage: OrderedFloat(10.0),
    };
    config.fire(source, "", app.world_mut());
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        1,
        "consumer must NOT pre-filter Invulnerable — pipeline owns that",
    );
    assert_eq!(collector.0[0].target, cell);
    // amount snapshot is post-MutateDamage by the time collector reads in
    // Last; len/target/Hp triplet pins the invariant ("consumer didn't filter;
    // pipeline zeroed it"). Asserting amount == 10.0 here is unreachable —
    // invulnerable_filter zeroes it in ApplyDamage before Last runs.

    let hp = app.world().get::<Hp>(cell).expect("Hp present").current;
    assert!(
        (hp - 100.0).abs() < 1e-4,
        "pipeline's invulnerable_filter must zero the applied amount → HP \
         remains 100.0, got {hp}",
    );
}

#[test]
fn dead_cell_excluded_live_cell_receives_message() {
    let mut app = explode_pipeline_app();
    let source = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();
    let _dead = spawn_dead_cell_at(&mut app, Vec2::new(20.0, 0.0));
    let live = spawn_cell_with_hp(&mut app, Vec2::new(30.0, 0.0), 100.0);

    let config = ExplodeConfig {
        range:  OrderedFloat(50.0),
        damage: OrderedFloat(10.0),
    };
    config.fire(source, "", app.world_mut());
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        1,
        "Dead cell is filtered by Without<Dead>; only live cell receives a message",
    );
    assert_eq!(collector.0[0].target, live);
}
