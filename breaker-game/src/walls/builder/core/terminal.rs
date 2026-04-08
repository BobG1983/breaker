//! Terminal build/spawn impls for `WallBuilder`.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Spatial;

use super::types::*;
use crate::{
    effect::EffectCommandsExt,
    prelude::*,
    shared::{BOLT_LAYER, GameDrawLayer, WALL_LAYER},
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

// ── Invisible spawn ──────────────────────────────────────────────────────

impl<S: SideData> WallBuilder<S, Invisible> {
    /// Spawns an invisible wall entity and dispatches initial effects if present.
    pub(crate) fn spawn(self, commands: &mut Commands) -> Entity {
        let effects = resolve_effects(&self.optional);
        let ht = resolve_half_thickness(&self.optional);
        let position = self.side.compute_position(ht);
        let half_extents = self.side.compute_half_extents(ht);
        let entity = commands.spawn(build_core(position, half_extents)).id();
        dispatch_effects(commands, entity, effects);
        entity
    }
}

// ── Visible spawn ────────────────────────────────────────────────────────

#[allow(dead_code, reason = "test-only until system-param callers exist")]
impl<S: SideData> WallBuilder<S, Visible> {
    /// Spawns a visible wall entity and dispatches initial effects if present.
    pub(crate) fn spawn(self, commands: &mut Commands) -> Entity {
        let effects = resolve_effects(&self.optional);
        let ht = resolve_half_thickness(&self.optional);
        let position = self.side.compute_position(ht);
        let half_extents = self.side.compute_half_extents(ht);
        let entity = commands
            .spawn((
                build_core(position, half_extents),
                Mesh2d(self.visual.mesh),
                MeshMaterial2d(self.visual.material),
            ))
            .id();
        dispatch_effects(commands, entity, effects);
        entity
    }
}
