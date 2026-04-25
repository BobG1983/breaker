//! W2 Behavior 37 (primary home): Tether redirects Diffusion-ring damage.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::{
    super::system::tether_emit_partner,
    helpers::{
        add_tether_stacks, canonical_tether_config, install_tether_config, spawn_linked_pair,
    },
};
use crate::{
    hazard::{
        definition::HazardKind,
        hazards::diffusion::system::{
            DiffusionConfig, DiffusionInstances, PendingDiffusionEmissions, diffusion_emit_rings,
            diffusion_reduce_primary,
        },
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

// ── W2 Behavior 37: Tether does NOT skip diffusion-sourced messages ──

#[test]
fn tether_redirects_diffusion_ring_damage() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<ActiveHazards>()
        .with_resource::<PendingDiffusionEmissions>()
        .with_resource::<DiffusionInstances>()
        .build();

    // Diffusion + Tether both active.
    app.world_mut().insert_resource(DiffusionConfig {
        base_share_percent:      50.0,
        share_per_level_percent: 0.0,
    });
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Diffusion);
    install_tether_config(&mut app, canonical_tether_config());
    add_tether_stacks(&mut app, 1);

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
            tether_emit_partner
                .in_set(DmgSystems::PostApplyDamage)
                .run_if(hazard_active(HazardKind::Tether))
                .run_if(in_state(NodeState::Playing)),
        ),
    );

    // Cells: C0, C1 (tether-linked to P), C2.
    let c0 = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::ZERO),
            Hp::new(100.0),
            KilledBy { killer: None },
        ))
        .id();
    let c2 = app
        .world_mut()
        .spawn((
            Cell,
            Position2D(Vec2::new(60.0, 0.0)),
            Hp::new(100.0),
            KilledBy { killer: None },
        ))
        .id();
    let _ = c2;
    let (c1, p) = spawn_linked_pair(
        &mut app,
        Vec2::new(30.0, 0.0),
        Vec2::new(500.0, 500.0), // far partner
    );
    let _ = c1;
    let _ = p;
    let bolt = app.world_mut().spawn_empty().id();

    // Frame N: primary on C0.
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

    // Run several frames for the cascade to play out.
    for _ in 0..4 {
        tick(&mut app);
    }

    // P received some damage (via tether from the ring on C1).
    let p_hp = app.world().get::<Hp>(p).unwrap().current;
    assert!(
        p_hp < 100.0,
        "tether partner P must receive damage via ring-sourced tether redirect"
    );
}
