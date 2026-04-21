//! Shared test helpers for the Diffusion redistribution branch tests.
//!
//! Test helpers (`PendingCellDamage`, `enqueue_cell_damage`) are copied locally
//! from `shared/death_pipeline/systems/tests/helpers.rs` (Option A per the
//! spec): the upstream items are `pub(super)` and not importable from the
//! cells-domain test module. Low-duplication cost for the decoupling benefit.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::super::super::system::apply_damage_to_cells;
pub(super) use crate::shared::test_utils::tick;
use crate::{
    hazard::{
        definition::HazardKind, hazards::diffusion::DiffusionConfig, resources::ActiveHazards,
    },
    prelude::*,
    shared::death_pipeline::sets::DeathPipelineSystems,
};

// ── Test helpers (cells-domain copy of PendingCellDamage/enqueue) ─────────

/// Pending `DamageDealt<Cell>` messages enqueued before
/// `apply_damage_to_cells` each tick. Local copy of the shared
/// death-pipeline test helper — keeps cells-domain tests independent.
#[derive(Resource, Default)]
pub(super) struct PendingCellDamage(pub(super) Vec<DamageDealt<Cell>>);

/// System that writes `DamageDealt<Cell>` from `PendingCellDamage` each tick.
pub(super) fn enqueue_cell_damage(
    pending: Res<PendingCellDamage>,
    mut writer: MessageWriter<DamageDealt<Cell>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

pub(super) fn damage_msg(target: Entity, amount: f32, dealer: Option<Entity>) -> DamageDealt<Cell> {
    DamageDealt {
        dealer,
        target,
        amount,
        source_chip: None,
        _marker: PhantomData,
    }
}

/// Canonical builder: state hierarchy at `NodeState::Playing`,
/// `ActiveHazards`, `DamageDealt<Cell>` with capture, `PendingCellDamage` +
/// `enqueue_cell_damage` ordered before `DeathPipelineSystems::ApplyDamage`,
/// and `apply_damage_to_cells` in that set.
pub(super) fn build_apply_damage_to_cells_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message_capture::<DamageDealt<Cell>>()
        .with_resource::<PendingCellDamage>()
        .build();

    // Order `enqueue_cell_damage` before the `ApplyDamage` set, then put
    // `apply_damage_to_cells` INTO the set. We don't add `DeathPipelinePlugin`
    // here because that plugin also registers the generic `apply_damage::<Cell>`
    // which we are replacing.
    app.add_systems(
        FixedUpdate,
        enqueue_cell_damage.before(DeathPipelineSystems::ApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        apply_damage_to_cells.in_set(DeathPipelineSystems::ApplyDamage),
    );
    app
}

/// Variant builder that deliberately OMITS `ActiveHazards`. Used to pin
/// robustness against the resource-registration-order edge case.
pub(super) fn build_apply_damage_to_cells_app_without_active_hazards() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_message_capture::<DamageDealt<Cell>>()
        .with_resource::<PendingCellDamage>()
        .build();

    app.add_systems(
        FixedUpdate,
        enqueue_cell_damage.before(DeathPipelineSystems::ApplyDamage),
    );
    app.add_systems(
        FixedUpdate,
        apply_damage_to_cells.in_set(DeathPipelineSystems::ApplyDamage),
    );
    app
}

pub(super) fn spawn_cell_at(app: &mut App, pos: Vec2, hp: f32) -> Entity {
    app.world_mut()
        .spawn((Cell, Position2D(pos), Hp::new(hp), KilledBy::default()))
        .id()
}

pub(super) fn spawn_cell_at_dead(app: &mut App, pos: Vec2, hp: f32) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp::new(hp),
            KilledBy::default(),
            Dead,
        ))
        .id()
}

pub(super) fn spawn_cell_at_invulnerable(app: &mut App, pos: Vec2, hp: f32) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Position2D(pos),
            Hp::new(hp),
            KilledBy::default(),
            Invulnerable,
        ))
        .id()
}

pub(super) fn install_diffusion_config(app: &mut App, cfg: DiffusionConfig) {
    app.world_mut().insert_resource(cfg);
}

pub(super) fn add_diffusion_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Diffusion);
    }
}

pub(super) const fn canonical_diffusion_config() -> DiffusionConfig {
    DiffusionConfig {
        base_share_percent:      20.0,
        share_per_level_percent: 10.0,
        depth_increase_interval: 5,
    }
}

pub(super) fn hp_of(app: &App, entity: Entity) -> f32 {
    app.world().get::<Hp>(entity).unwrap().current
}

pub(super) fn push_damage(app: &mut App, msg: DamageDealt<Cell>) {
    app.world_mut()
        .resource_mut::<PendingCellDamage>()
        .0
        .push(msg);
}
