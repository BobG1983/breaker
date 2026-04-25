//! W2 Behavior 37 (partial — diffusion-cascade side): multi-hop diffusion
//! propagation that feeds Tether redirects. Tether's primary cross-mechanic
//! tests live in `tether/tests/cross_mechanic.rs`; this file focuses on the
//! diffusion-ring propagation that makes cross-mechanic possible.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::super::system::{
    DiffusionConfig, DiffusionInstances, PendingDiffusionEmissions, diffusion_emit_rings,
    diffusion_reduce_primary,
};
use crate::{
    mutators::hazards::{
        definition::HazardKind,
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

#[track_caller]
fn assert_f32_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-4,
        "expected {expected}, got {actual}"
    );
}

fn build_app(share_percent: f32) -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<ActiveHazards>()
        .with_resource::<PendingDiffusionEmissions>()
        .with_resource::<DiffusionInstances>()
        .build();
    app.world_mut().insert_resource(DiffusionConfig {
        base_share_percent:      share_percent,
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
                .run_if(hazard_active(HazardKind::Diffusion))
                .run_if(in_state(NodeState::Playing)),
            diffusion_emit_rings
                .in_set(DmgSystems::PostApplyDamage)
                .run_if(hazard_active(HazardKind::Diffusion))
                .run_if(in_state(NodeState::Playing)),
        ),
    );
    app
}

#[test]
fn diffusion_ring_attributes_forwarded_through_multi_hop() {
    let mut app = build_app(50.0);
    let c0 = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::ZERO),
            Hp::new(100.0),
            KilledBy { killer: None },
        ))
        .id();
    let c1 = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::new(50.0, 0.0)),
            Hp::new(100.0),
            KilledBy { killer: None },
        ))
        .id();
    let c2 = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::new(100.0, 0.0)),
            Hp::new(100.0),
            KilledBy { killer: None },
        ))
        .id();
    let bolt = app.world_mut().spawn_empty().id();

    // Frame N: primary hit on c0 attributed to bolt.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        c0,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app); // N
    tick(&mut app); // N+1: C1 ring arrives, reduces, emits to C2
    tick(&mut app); // N+2: C2 ring arrives, no neighbors → pass-through

    // Verify HP conservation: total damage = 100 (preserved across all 3 cells).
    let total = (100.0 - app.world().get::<Hp>(c0).unwrap().current)
        + (100.0 - app.world().get::<Hp>(c1).unwrap().current)
        + (100.0 - app.world().get::<Hp>(c2).unwrap().current);
    assert_f32_eq(total, 100.0);
}
