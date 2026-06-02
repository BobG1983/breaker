//! W7 Behavior 10 (Explode): empty cell set / dead-only set produces zero
//! `DamageDealt<Cell>` messages and does not panic. The
//! `ExplodeEmissionRequested` is still written (consumer is responsible for
//! handling empty cell sets gracefully — `fire()` does NOT pre-check).

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rand::SeedableRng;

use super::{
    super::super::{config::ExplodeConfig, messages::ExplodeEmissionRequested},
    helpers::*,
};
use crate::{effect_v3::traits::Fireable, prelude::*};

#[test]
fn empty_quadtree_no_panic_zero_damage_one_request() {
    let mut app = explode_pipeline_app();
    let source = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();
    // No cells spawned — but still init the quadtree for an empty pass.
    app.world_mut().run_schedule(FixedUpdate);

    let config = ExplodeConfig {
        range:  OrderedFloat(50.0),
        damage: OrderedFloat(10.0),
    };

    // Snapshot request count BEFORE the tick — fire() must write one even
    // with zero cells.
    config.fire(
        source,
        "",
        app.world_mut(),
        &mut rand_chacha::ChaCha8Rng::seed_from_u64(0),
    );
    let req_buf = app.world().resource::<Messages<ExplodeEmissionRequested>>();
    let request_count = req_buf.iter_current_update_messages().count();
    assert_eq!(
        request_count, 1,
        "fire() always writes one request, even with no cells",
    );

    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert_eq!(
        collector.0.len(),
        0,
        "no cells means zero DamageDealt<Cell>",
    );
}

#[test]
fn dead_only_cells_yield_zero_damage_no_panic() {
    let mut app = explode_pipeline_app();
    let source = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();
    let _dead_a = spawn_dead_cell_at(&mut app, Vec2::new(20.0, 0.0));
    let _dead_b = spawn_dead_cell_at(&mut app, Vec2::new(30.0, 0.0));

    let config = ExplodeConfig {
        range:  OrderedFloat(50.0),
        damage: OrderedFloat(10.0),
    };
    config.fire(
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
        0,
        "all-dead cell set produces zero DamageDealt<Cell> (Without<Dead> filter)",
    );
}
