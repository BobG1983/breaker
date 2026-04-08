//! Terminal methods: `build_core()`, `spawn_inner()`, and terminal impls.

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, Scale2D};

use super::types::*;
use crate::{
    cells::{components::*, definition::CellBehavior},
    effect::{EffectCommandsExt, EffectNode, RootEffect},
    shared::{BOLT_LAYER, CELL_LAYER, GameDrawLayer},
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

fn resolve_effects(optional: &OptionalCellData) -> Option<Vec<RootEffect>> {
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

fn dispatch_effects(commands: &mut Commands, entity: Entity, effects: Option<Vec<RootEffect>>) {
    if let Some(effects) = effects {
        let entries: Vec<(String, EffectNode)> = effects
            .into_iter()
            .flat_map(|root| {
                let RootEffect::On { then, .. } = root;
                then.into_iter().map(|node| (String::new(), node))
            })
            .collect();
        commands.push_bound_effects(entity, entries);
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
        CellHealth::new(hp),
        CellWidth::new(params.width),
        CellHeight::new(params.height),
    );

    (identity, physics, health_components)
}

// ── spawn_inner() ─────────────────────────────────────────────────────────

fn spawn_inner(commands: &mut Commands, core: impl Bundle, optional: OptionalCellData) -> Entity {
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

    if let Some(entities) = optional.locked_entities.clone() {
        entity.insert((Locked, LockAdjacents(entities)));
    }

    let behaviors = resolve_behaviors(&optional);
    for behavior in behaviors {
        match behavior {
            CellBehavior::Regen { rate } => {
                entity.insert(CellRegen { rate });
            }
        }
    }

    let id = entity.id();
    dispatch_effects(commands, id, resolve_effects(&optional));
    id
}

// ── Headless terminal impls (test-only — production uses rendered) ────────

#[cfg(test)]
impl CellBuilder<HasPosition, HasDimensions, HasHealth, Headless> {
    /// Spawns a headless cell entity with all components.
    pub(crate) fn spawn(self, commands: &mut Commands) -> Entity {
        let params = CoreParams {
            pos: self.position.pos,
            width: self.dimensions.width,
            height: self.dimensions.height,
            hp: self.health.hp,
        };
        let core = build_core(&params, &self.optional);
        spawn_inner(commands, core, self.optional)
    }
}

// ── Rendered terminal impls ───────────────────────────────────────────────

impl CellBuilder<HasPosition, HasDimensions, HasHealth, Rendered> {
    /// Spawns a rendered cell entity with all components.
    pub(crate) fn spawn(self, commands: &mut Commands) -> Entity {
        let mesh = self.visual.mesh.clone();
        let material = self.visual.material.clone();
        let params = CoreParams {
            pos: self.position.pos,
            width: self.dimensions.width,
            height: self.dimensions.height,
            hp: self.health.hp,
        };
        let core = build_core(&params, &self.optional);
        let entity = spawn_inner(commands, core, self.optional);
        commands.entity(entity).insert((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            GameDrawLayer::Cell,
        ));
        entity
    }
}
