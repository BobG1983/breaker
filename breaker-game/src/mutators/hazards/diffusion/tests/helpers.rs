use bevy::{ecs::world::CommandQueue, prelude::*};

use super::super::system::*;
use crate::{
    mutators::hazards::{
        definition::{HazardKind, HazardTuning},
        resources::ActiveHazards,
    },
    prelude::*,
};

// ── Helpers (Fracture 5-helper pattern) ──────────────────────────────

pub(super) fn test_app_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<DamageDealt<Cell>>()
        .build()
}

pub(super) const fn canonical_config() -> DiffusionConfig {
    DiffusionConfig {
        base_share_percent:      20.0,
        share_per_level_percent: 10.0,
    }
}

pub(super) fn add_diffusion_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Diffusion);
    }
}

pub(super) fn activate_now(app: &mut App, tuning: &HazardTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}
