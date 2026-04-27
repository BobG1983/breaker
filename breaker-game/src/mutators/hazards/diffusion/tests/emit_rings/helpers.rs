//! Shared helpers for the `diffusion_emit_rings` test suite.

use bevy::prelude::*;

use super::super::super::system::{
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
pub(super) fn assert_f32_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-4,
        "expected {expected}, got {actual}"
    );
}

pub(super) fn build_emit_app(share_percent: f32) -> App {
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

pub(super) fn spawn_cell_at(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp::new(100.0),
            KilledBy { killer: None },
        ))
        .id()
}

pub(super) fn install_cell_hp(app: &mut App, cell: Entity, hp: f32) {
    app.world_mut().entity_mut(cell).insert(Hp::new(hp));
}
