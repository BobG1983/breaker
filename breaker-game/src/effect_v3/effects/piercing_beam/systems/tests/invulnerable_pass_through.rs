//! W7 Behavior 7 (PiercingBeam): consumer must NOT pre-filter
//! `Invulnerable` cells — the pipeline's `invulnerable_filter::<Cell>` zeroes
//! the amount in `ApplyDamage`. `Dead` cells ARE filtered by the consumer's
//! `Without<Dead>` query.

use bevy::prelude::*;
use rand::SeedableRng;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

use super::helpers::*;
use crate::{effect_v3::traits::Fireable, prelude::*};

#[test]
fn invulnerable_cell_receives_message_pipeline_zeros_damage() {
    let mut app = piercing_pipeline_app();
    let source = spawn_beam_source(&mut app);
    let cell_pos = Vec2::new(0.0, 50.0);
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
        ))
        .id();

    make_config().fire(
        source,
        "",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        1,
        "consumer must NOT pre-filter Invulnerable",
    );
    assert_eq!(collector.0[0].target, cell);
    // amount snapshot is post-MutateDamage by the time collector reads in
    // Last; len/target/Hp triplet pins the invariant ("consumer didn't filter;
    // pipeline zeroed it"). Asserting amount == 10.0 here is unreachable —
    // invulnerable_filter zeroes it in ApplyDamage before Last runs.

    let hp = app.world().get::<Hp>(cell).expect("Hp").current;
    assert!(
        (hp - 100.0).abs() < 1e-4,
        "pipeline's invulnerable_filter zeroes applied damage → Hp 100.0, got {hp}",
    );
}

#[test]
fn dead_cell_excluded_live_cell_receives_message() {
    let mut app = piercing_pipeline_app();
    let source = spawn_beam_source(&mut app);
    // Dead cell in beam — must be filtered by consumer's Without<Dead>.
    let _dead = app
        .world_mut()
        .spawn((Cell, Position2D(Vec2::new(0.0, 50.0)), Dead))
        .id();
    let live = spawn_cell_with_hp(&mut app, Vec2::new(0.0, 150.0), 100.0);

    make_config().fire(
        source,
        "",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        1,
        "Dead cell is filtered by Without<Dead>; only live cell gets a message",
    );
    assert_eq!(collector.0[0].target, live);
}
