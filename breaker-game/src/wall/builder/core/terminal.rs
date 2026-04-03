//! Terminal build/spawn impls for `WallBuilder`.

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Scale2D, Spatial};

use super::types::*;
use crate::{
    effect::{EffectCommandsExt, EffectNode, RootEffect},
    shared::{BOLT_LAYER, GameDrawLayer, WALL_LAYER},
    wall::components::Wall,
};

// ── Resolution helpers ────────────────────────────────────────────────────

/// Resolves the final half-thickness: override > definition > default (90.0).
fn resolve_half_thickness(optional: &OptionalWallData) -> f32 {
    optional
        .override_half_thickness
        .or(optional.definition_half_thickness)
        .unwrap_or(DEFAULT_HALF_THICKNESS)
}

/// Resolves the final effects: override > definition > empty.
fn resolve_effects(optional: &OptionalWallData) -> Option<Vec<RootEffect>> {
    optional
        .override_effects
        .clone()
        .or_else(|| optional.definition_effects.clone())
        .filter(|e| !e.is_empty())
}

/// Dispatches resolved effects to a wall entity via `push_bound_effects`.
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

/// Builds the core component bundle shared by all wall builds (no visual).
///
/// Uses `Spatial::builder()` for position components, ensuring all spatial
/// components (`Position2D`, `PreviousPosition`, Spatial marker) are consistently
/// initialized. `GlobalPosition2D` is auto-inserted via `Spatial2D`'s `#[require]`
/// and propagated by the spatial sync system.
fn build_core(position: Vec2, half_extents: Vec2) -> impl Bundle + use<> {
    let spatial = Spatial::builder().at_position(position).build();
    let identity = (
        Wall,
        spatial,
        Scale2D {
            x: half_extents.x,
            y: half_extents.y,
        },
    );
    let physics = (
        Aabb2D::new(Vec2::ZERO, half_extents),
        CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
        GameDrawLayer::Wall,
    );
    (identity, physics)
}

// ── Invisible build/spawn ─────────────────────────────────────────────────

impl<S: SideData> WallBuilder<S, Invisible> {
    /// Builds the wall component bundle (no visual components).
    #[must_use]
    pub(crate) fn build(self) -> impl Bundle + use<S> {
        let ht = resolve_half_thickness(&self.optional);
        let position = self.side.compute_position(ht);
        let half_extents = self.side.compute_half_extents(ht);
        build_core(position, half_extents)
    }

    /// Spawns an invisible wall entity and dispatches initial effects if present.
    pub(crate) fn spawn(self, commands: &mut Commands) -> Entity {
        let effects = resolve_effects(&self.optional);
        let entity = commands.spawn(self.build()).id();
        dispatch_effects(commands, entity, effects);
        entity
    }
}

// ── Visible build/spawn ───────────────────────────────────────────────────

#[allow(dead_code, reason = "test-only until system-param callers exist")]
impl<S: SideData> WallBuilder<S, Visible> {
    /// Builds the wall component bundle with visual components (`Mesh2d` + `MeshMaterial2d`).
    #[must_use]
    pub(crate) fn build(self) -> impl Bundle + use<S> {
        let ht = resolve_half_thickness(&self.optional);
        let position = self.side.compute_position(ht);
        let half_extents = self.side.compute_half_extents(ht);
        (
            build_core(position, half_extents),
            Mesh2d(self.visual.mesh),
            MeshMaterial2d(self.visual.material),
        )
    }

    /// Spawns a visible wall entity and dispatches initial effects if present.
    pub(crate) fn spawn(self, commands: &mut Commands) -> Entity {
        let effects = resolve_effects(&self.optional);
        let entity = commands.spawn(self.build()).id();
        dispatch_effects(commands, entity, effects);
        entity
    }
}
