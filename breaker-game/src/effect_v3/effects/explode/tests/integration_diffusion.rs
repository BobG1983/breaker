//! W7 Behavior 9 (Explode + Diffusion integration): the W7 payoff pin —
//! same-tick mutator (Diffusion `MutateDamage`) sees and reduces the
//! Explode-emitted damage. Pre-W7, `fire()` wrote `DamageDealt<Cell>`
//! synchronously at command-flush time, bypassing the `EmitDamage` set, so
//! the Diffusion reducer never observed it. Post-W7, the consumer system
//! emits in `EmitDamage` and `MutateDamage` runs immediately after — so the
//! reduction lands on the same tick.
//!
//! Cell layout:
//! - `c0` at `(0, 0)` — the explode primary target.
//! - `c1` at `(50, 0)` — within `ADJACENCY_RADIUS = 70.0` of `c0` so
//!   diffusion picks it up as a neighbor.
//!   Range = 20 ⟹ only `c0` is hit by the explode (50 > 20 excludes `c1`).
//!   Diffusion `base_share_percent = 50.0` ⟹ half the primary damage shifts
//!   from `c0` to `c1`.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

use crate::{
    chips::definition::Rarity,
    effect_v3::{effects::explode::config::ExplodeConfig, traits::Fireable},
    mutators::hazards::{
        definition::HazardKind,
        diffusion::system::{
            DiffusionConfig, DiffusionInstances, PendingDiffusionEmissions, diffusion_emit_rings,
            diffusion_reduce_primary,
        },
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

fn build_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_physics()
        .with_effects_pipeline()
        .with_resource::<ActiveHazards>()
        .with_resource::<PendingDiffusionEmissions>()
        .with_resource::<DiffusionInstances>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut().insert_resource(DiffusionConfig {
        base_share_percent:      50.0,
        share_per_level_percent: 0.0,
    });
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Diffusion);
    app.add_systems(
        FixedUpdate,
        (
            diffusion_reduce_primary
                .in_set(DmgSystems::MutateDamage)
                .run_if(hazard_active(HazardKind::Diffusion)),
            diffusion_emit_rings
                .in_set(DmgSystems::PostApplyDamage)
                .run_if(hazard_active(HazardKind::Diffusion)),
        ),
    );
    app
}

fn spawn_cell(app: &mut App, pos: Vec2, hp: f32) -> Entity {
    let e = app
        .world_mut()
        .spawn((
            Cell,
            Hp::new(hp),
            KilledBy { killer: None },
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            Aabb2D::new(Vec2::ZERO, Vec2::splat(5.0)),
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
        ))
        .id();
    app.world_mut().run_schedule(FixedUpdate);
    e
}

#[test]
fn diffusion_reduces_explode_primary_in_same_tick_then_ring_lands_on_neighbor() {
    let mut app = build_app();
    let c0 = spawn_cell(&mut app, Vec2::ZERO, 100.0);
    let c1 = spawn_cell(&mut app, Vec2::new(50.0, 0.0), 100.0);
    let bolt = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let config = ExplodeConfig {
        range:  OrderedFloat(20.0),
        damage: OrderedFloat(100.0),
    };
    config.fire(bolt, "explode-chip", app.world_mut());

    // Tick 1: explode consumer emits in EmitDamage; diffusion_reduce_primary
    // halves c0's primary in MutateDamage; ApplyDamage lands 50.0 on c0;
    // diffusion_emit_rings queues / emits the 50.0 ring for c1 in
    // PostApplyDamage.
    tick(&mut app);
    // Tick 2: the diffusion ring DamageDealt<Cell> for c1 flows through
    // EmitDamage → MutateDamage → ApplyDamage; c1 lands 50.0.
    tick(&mut app);

    let hp_c0 = app.world().get::<Hp>(c0).expect("c0 Hp").current;
    let hp_c1 = app.world().get::<Hp>(c1).expect("c1 Hp").current;
    assert!(
        (hp_c0 - 50.0).abs() < 1e-4,
        "c0 took primary 100.0 reduced 50% to 50.0 → expected Hp 50.0, got {hp_c0} \
         (Hp 0.0 indicates pre-W7 timing where Diffusion missed the primary)",
    );
    assert!(
        (hp_c1 - 50.0).abs() < 1e-4,
        "c1 took diffusion ring 50.0 → expected Hp 50.0, got {hp_c1} \
         (Hp 100.0 indicates the ring never emitted)",
    );
    let total_absorbed = (100.0 - hp_c0) + (100.0 - hp_c1);
    assert!(
        (total_absorbed - 100.0).abs() < 1e-4,
        "HP conservation: total absorbed across both cells must equal the \
         100.0 input damage; got {total_absorbed}",
    );
}

#[test]
fn two_explode_sources_same_tick_diffusion_runs_per_primary() {
    let mut app = build_app();
    let c0 = spawn_cell(&mut app, Vec2::ZERO, 100.0);
    let c1 = spawn_cell(&mut app, Vec2::new(50.0, 0.0), 100.0);
    let bolt_a = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();
    let bolt_b = app.world_mut().spawn(Position2D(Vec2::ZERO)).id();

    let config = ExplodeConfig {
        range:  OrderedFloat(20.0),
        damage: OrderedFloat(50.0),
    };

    let source_a = SourceId::chip("Explode-A").rarity(Rarity::Rare).build();
    let source_b = SourceId::chip("Explode-B").rarity(Rarity::Rare).build();
    config.fire(bolt_a, &source_a.0.clone(), app.world_mut());
    config.fire(bolt_b, &source_b.0.clone(), app.world_mut());

    tick(&mut app);
    tick(&mut app);

    let hp_c0 = app.world().get::<Hp>(c0).expect("c0 Hp").current;
    let hp_c1 = app.world().get::<Hp>(c1).expect("c1 Hp").current;
    // Each primary 50.0 splits 25/25: c0 absorbs 25 from each = 50.0, c1
    // absorbs 25 from each = 50.0. HP conservation across both cells: 100.0.
    assert!(
        (hp_c0 - 50.0).abs() < 1e-4,
        "c0 took 25 + 25 = 50.0 damage → expected Hp 50.0, got {hp_c0}",
    );
    assert!(
        (hp_c1 - 50.0).abs() < 1e-4,
        "c1 took 25 + 25 = 50.0 damage → expected Hp 50.0, got {hp_c1}",
    );
    let total_absorbed = (100.0 - hp_c0) + (100.0 - hp_c1);
    assert!(
        (total_absorbed - 100.0).abs() < 1e-4,
        "HP conservation: 50 + 50 = 100 total damage absorbed across both cells, got {total_absorbed}",
    );
}
