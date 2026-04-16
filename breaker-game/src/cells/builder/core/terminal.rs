//! Terminal methods: `build_core()`, `spawn_inner()`, and terminal impls.

use bevy::prelude::*;
use rantzsoft_spatial2d::propagation::PositionPropagation;

use super::types::*;
use crate::{
    cells::{
        behaviors::{
            guarded::components::ring_slot_offset,
            volatile::stamp::{STAMP_SOURCE, volatile_tree},
        },
        components::*,
        definition::CellBehavior,
    },
    effect_v3::{
        commands::EffectCommandsExt,
        types::{RootNode, Tree},
    },
    prelude::*,
    shared::GameDrawLayer,
};

// ── Resolution helpers ────────────────────────────────────────────────────

fn resolve_alias(optional: &OptionalCellData) -> Option<String> {
    optional
        .alias
        .clone()
        .or_else(|| optional.definition_params.as_ref().map(|d| d.alias.clone()))
}

fn resolve_required_to_clear(optional: &OptionalCellData) -> bool {
    optional
        .required_to_clear
        .or_else(|| {
            optional
                .definition_params
                .as_ref()
                .map(|d| d.required_to_clear)
        })
        .unwrap_or(false)
}

fn resolve_damage_visuals(optional: &OptionalCellData) -> Option<CellDamageVisuals> {
    optional.damage_visuals.clone().or_else(|| {
        optional
            .definition_params
            .as_ref()
            .map(|d| d.damage_visuals.clone())
    })
}

fn resolve_effects(optional: &OptionalCellData) -> Option<Vec<RootNode>> {
    optional
        .effects
        .clone()
        .or_else(|| {
            optional
                .definition_params
                .as_ref()
                .and_then(|d| d.effects.clone())
        })
        .filter(|e| !e.is_empty())
}

fn resolve_behaviors(optional: &OptionalCellData) -> Vec<CellBehavior> {
    let mut behaviors = optional
        .definition_params
        .as_ref()
        .map(|d| d.behaviors.clone())
        .unwrap_or_default();
    behaviors.extend(optional.behaviors.clone());
    behaviors
}

fn resolve_hp(hp: f32, optional: &OptionalCellData) -> f32 {
    optional.override_hp.unwrap_or(hp)
}

fn dispatch_effects(commands: &mut Commands, entity: Entity, effects: Option<Vec<RootNode>>) {
    if let Some(effects) = effects {
        for root in effects {
            match root {
                RootNode::Stamp(_target, tree) => {
                    commands.stamp_effect(entity, String::new(), tree);
                }
                RootNode::Spawn(_kind, _tree) => {
                    // Spawn-type effects are handled by the SpawnStampRegistry,
                    // not by direct entity stamping. Deferred.
                }
            }
        }
    }
}

// ── build_core() ──────────────────────────────────────────────────────────

fn build_core(params: &CoreParams, optional: &OptionalCellData) -> impl Bundle + use<> {
    let hp = resolve_hp(params.hp, optional);

    let identity = (
        Cell,
        Position2D(params.pos),
        Scale2D {
            x: params.width,
            y: params.height,
        },
    );

    let physics = (
        Aabb2D::new(
            Vec2::ZERO,
            Vec2::new(params.width / 2.0, params.height / 2.0),
        ),
        CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
    );

    let health_components = (
        Hp::new(hp),
        KilledBy::default(),
        CellWidth::new(params.width),
        CellHeight::new(params.height),
    );

    (identity, physics, health_components)
}

// ── spawn_inner() ─────────────────────────────────────────────────────────

fn spawn_inner(
    commands: &mut Commands,
    core: impl Bundle,
    mut optional: OptionalCellData,
) -> Entity {
    let mut entity = commands.spawn(core);

    if let Some(alias) = resolve_alias(&optional) {
        entity.insert(CellTypeAlias(alias));
    }

    if resolve_required_to_clear(&optional) {
        entity.insert(RequiredToClear);
    }

    if let Some(visuals) = resolve_damage_visuals(&optional) {
        entity.insert(visuals);
    }

    if let Some(entities) = optional.locked_entities.take() {
        entity.insert((LockCell, Locked, Locks(entities)));
    }

    let behaviors = resolve_behaviors(&optional);
    let entity_id = entity.id();
    let mut volatile_stamps: Vec<(Entity, Tree)> = Vec::new();

    for behavior in behaviors {
        match behavior {
            CellBehavior::Regen { rate } => {
                entity.insert((RegenCell, Regen, RegenRate(rate)));
            }
            CellBehavior::Guarded(_) => {
                entity.insert(GuardedCell);
            }
            CellBehavior::Volatile { damage, radius } => {
                entity.insert(VolatileCell);
                volatile_stamps.push((entity_id, volatile_tree(damage, radius)));
            }
            CellBehavior::Sequence { group, position } => {
                entity.insert((
                    SequenceCell,
                    SequenceGroup(group),
                    SequencePosition(position),
                ));
            }
            CellBehavior::Armored { value, facing } => {
                entity.insert((ArmoredCell, ArmorValue(value), ArmorFacing(facing)));
            }
            CellBehavior::Phantom {
                cycle_secs,
                telegraph_secs,
                starting_phase,
            } => {
                let config = PhantomConfig {
                    cycle_secs,
                    telegraph_secs,
                };
                let timer = PhantomTimer(config.duration_for(starting_phase));
                entity.insert((PhantomCell, starting_phase, timer, config));
                if starting_phase == PhantomPhase::Ghost {
                    entity.insert(CollisionLayers::new(0, 0));
                }
            }
            CellBehavior::Magnetic { radius, strength } => {
                entity.insert((MagneticCell, MagneticField { radius, strength }));
            }
        }
    }

    // Deferred: `entity` borrows `commands` for the duration of the loop.
    for (id, tree) in volatile_stamps {
        commands.stamp_effect(id, STAMP_SOURCE.to_owned(), tree);
    }

    dispatch_effects(commands, entity_id, resolve_effects(&optional));
    entity_id
}

// ── Headless terminal impls (test-only — production uses rendered) ────────

#[cfg(test)]
impl CellBuilder<HasPosition, HasDimensions, HasHealth, Headless> {
    /// Spawns a headless cell entity with all components.
    pub(crate) fn spawn(mut self, commands: &mut Commands) -> Entity {
        let guarded_data = self.optional.guarded_data.take();
        let params = CoreParams {
            pos:    self.position.pos,
            width:  self.dimensions.width,
            height: self.dimensions.height,
            hp:     self.health.hp,
        };
        let core = build_core(&params, &self.optional);
        let entity = spawn_inner(commands, core, self.optional);
        if let Some(guarded_data) = guarded_data {
            commands.entity(entity).insert(GuardedCell);
            spawn_guardian_children(commands, entity, &params, &guarded_data);
        }
        entity
    }
}

// ── Rendered terminal impls ───────────────────────────────────────────────

impl CellBuilder<HasPosition, HasDimensions, HasHealth, Rendered> {
    /// Spawns a rendered cell entity with all components.
    pub(crate) fn spawn(mut self, commands: &mut Commands) -> Entity {
        let mesh = self.visual.mesh.clone();
        let material = self.visual.material.clone();
        let guarded_data = self.optional.guarded_data.take();
        let params = CoreParams {
            pos:    self.position.pos,
            width:  self.dimensions.width,
            height: self.dimensions.height,
            hp:     self.health.hp,
        };
        let core = build_core(&params, &self.optional);
        let entity = spawn_inner(commands, core, self.optional);
        commands.entity(entity).insert((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            GameDrawLayer::Cell,
        ));
        if let Some(guarded_data) = guarded_data {
            commands.entity(entity).insert(GuardedCell);
            spawn_guardian_children(commands, entity, &params, &guarded_data);
        }
        entity
    }
}

/// Spawns guardian children around a guarded parent cell.
///
/// Each guardian gets full cell components (Cell, health, position, collision)
/// plus guardian-specific components (`GuardianCell`, `GuardianSlot`, `SlideTarget`, etc.).
/// Guardian dimensions are square (`cell_height` x `cell_height`).
/// If `guardian_visuals` is `Some`, each guardian also receives visual components.
fn spawn_guardian_children(
    commands: &mut Commands,
    parent_entity: Entity,
    parent_params: &CoreParams,
    guarded_data: &GuardedSpawnData,
) {
    let config = &guarded_data.guardian_config;
    let guardian_dim = config.cell_height;
    let guardian_half = guardian_dim / 2.0;

    for &slot in &guarded_data.slots {
        let (ox, oy) = ring_slot_offset(slot);
        let world_pos = Vec2::new(
            ox.mul_add(config.step_x, parent_params.pos.x),
            oy.mul_add(config.step_y, parent_params.pos.y),
        );
        let initial_target = (slot + 1) % 8;

        let mut entity = commands.spawn((
            (
                Cell,
                GuardianCell,
                GuardianSlot(slot),
                SlideTarget(initial_target),
                GuardianSlideSpeed(config.slide_speed),
                GuardianGridStep {
                    step_x: config.step_x,
                    step_y: config.step_y,
                },
                Hp::new(config.hp),
                KilledBy::default(),
            ),
            (
                CellWidth::new(guardian_dim),
                CellHeight::new(guardian_dim),
                Position2D(world_pos),
                PositionPropagation::Absolute,
                Scale2D {
                    x: guardian_dim,
                    y: guardian_dim,
                },
                Aabb2D::new(Vec2::ZERO, Vec2::new(guardian_half, guardian_half)),
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                ChildOf(parent_entity),
            ),
        ));

        if let Some((ref mesh, ref material)) = guarded_data.guardian_visuals {
            entity.insert((
                Mesh2d(mesh.clone()),
                MeshMaterial2d(material.clone()),
                GameDrawLayer::Cell,
            ));
        }
    }
}
