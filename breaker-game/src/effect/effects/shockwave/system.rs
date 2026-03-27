//! Shockwave effect handler — expanding wavefront area damage.
//!
//! Observes [`ShockwaveFired`] and spawns a [`ShockwaveRadius`] entity that
//! expands over time. Collision with cells is handled by [`shockwave_collision`],
//! which writes [`DamageCell`] messages. The visual is driven by
//! [`animate_shockwave`] which scales the entity based on the current radius.

use std::collections::HashSet;

use bevy::{prelude::*, sprite_render::AlphaMode2d};
use rantzsoft_physics2d::{collision_layers::CollisionLayers, resources::CollisionQuadtree};
use rantzsoft_spatial2d::components::{Position2D, Scale2D, Spatial2D};

use crate::{
    bolt::BASE_BOLT_DAMAGE,
    cells::{components::Locked, messages::DamageCell},
    chips::components::DamageBoost,
    effect::definition::EffectTarget,
    shared::{CELL_LAYER, CleanupOnNodeExit, GameDrawLayer},
};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a shockwave effect resolves.
#[derive(Event, Clone, Debug)]
pub(crate) struct ShockwaveFired {
    /// Base radius of the shockwave effect.
    pub base_range: f32,
    /// Additional radius per stack beyond the first.
    pub range_per_level: f32,
    /// Current stack count.
    pub stacks: u32,
    /// Expansion speed in world units per second.
    pub speed: f32,
    /// The effect targets for this event.
    pub targets: Vec<EffectTarget>,
    /// The chip name that originated this chain, or `None` for breaker chains.
    pub source_chip: Option<String>,
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Current and maximum expansion radius for the shockwave wavefront.
#[derive(Component, Debug, Clone)]
pub(crate) struct ShockwaveRadius {
    /// Current expansion distance from the origin.
    pub current: f32,
    /// Maximum expansion distance — despawn when `current >= max`.
    pub max: f32,
}

/// Expansion speed of the shockwave in world units per second.
#[derive(Component, Debug, Clone)]
pub(crate) struct ShockwaveSpeed(pub f32);

/// Damage payload and source bolt for the shockwave.
#[derive(Component, Debug, Clone)]
pub(crate) struct ShockwaveDamage {
    /// Pre-calculated damage amount.
    pub damage: f32,
    /// The chip name that originated this shockwave, for damage attribution.
    pub source_chip: Option<String>,
    // FUTURE: may be used for upcoming phases
    // /// The bolt entity that caused this shockwave (for VFX / `DamageCell`), if any.
    // pub source_bolt: Option<Entity>,
}

/// Tracks which cell entities have already been hit by this shockwave.
#[derive(Component, Debug, Clone, Default)]
pub(crate) struct ShockwaveAlreadyHit(pub HashSet<Entity>);

// ---------------------------------------------------------------------------
// Observer — spawns shockwave entity
// ---------------------------------------------------------------------------

/// Observer: spawns a shockwave wavefront entity when a [`ShockwaveFired`] event
/// fires.
///
/// Does NOT write [`DamageCell`] — that is handled by [`shockwave_collision`].
pub(crate) fn handle_shockwave(
    trigger: On<ShockwaveFired>,
    mut commands: Commands,
    bolt_query: Query<(&Position2D, Option<&DamageBoost>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let event = trigger.event();

    if event.speed <= 0.0 {
        return;
    }

    let Some(bolt_entity) = event.targets.iter().find_map(|t| match t {
        crate::effect::definition::EffectTarget::Entity(e) => Some(*e),
        crate::effect::definition::EffectTarget::Location(_) => None,
    }) else {
        return;
    };

    let Ok((bolt_pos, damage_boost)) = bolt_query.get(bolt_entity) else {
        return;
    };

    let extra_stacks = f32::from(u16::try_from(event.stacks.saturating_sub(1)).unwrap_or(u16::MAX));
    let max = extra_stacks.mul_add(event.range_per_level, event.base_range);
    let damage = BASE_BOLT_DAMAGE * (1.0 + damage_boost.map_or(0.0, |b| b.0));

    commands.spawn((
        Position2D(bolt_pos.0),
        ShockwaveRadius { current: 0.0, max },
        ShockwaveSpeed(event.speed),
        ShockwaveDamage {
            damage,
            source_chip: event.source_chip.clone(),
        },
        ShockwaveAlreadyHit::default(),
        GameDrawLayer::Fx,
        Scale2D::default(),
        CleanupOnNodeExit,
        Spatial2D,
        Mesh2d(meshes.add(Annulus::new(0.85, 1.0))),
        MeshMaterial2d(materials.add(ColorMaterial {
            color: Color::linear_rgba(0.0, 4.0, 4.0, 0.9),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        })),
    ));
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Expands the shockwave radius each tick and despawns when fully expanded.
pub(crate) fn tick_shockwave(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ShockwaveRadius, &ShockwaveSpeed)>,
) {
    for (entity, mut radius, speed) in &mut query {
        radius.current = speed.0.mul_add(time.delta_secs(), radius.current);
        if radius.current >= radius.max {
            commands.entity(entity).despawn();
        }
    }
}

/// Damages cells within the shockwave's current radius via quadtree query.
pub(crate) fn shockwave_collision(
    quadtree: Res<CollisionQuadtree>,
    mut shockwave_query: Query<(
        &Position2D,
        &ShockwaveRadius,
        &ShockwaveDamage,
        &mut ShockwaveAlreadyHit,
    )>,
    cell_query: Query<Has<Locked>>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    for (pos, radius, dmg, mut already_hit) in &mut shockwave_query {
        let candidates = quadtree.quadtree.query_circle_filtered(
            pos.0,
            radius.current,
            CollisionLayers::new(0, CELL_LAYER),
        );

        for candidate in candidates {
            if already_hit.0.contains(&candidate) {
                continue;
            }

            let Ok(is_locked) = cell_query.get(candidate) else {
                continue;
            };

            if is_locked {
                continue;
            }

            damage_writer.write(DamageCell {
                cell: candidate,
                damage: dmg.damage,
                source_chip: dmg.source_chip.clone(),
            });
            already_hit.0.insert(candidate);
        }
    }
}

/// Scales the shockwave visual based on the current radius and fades alpha.
pub(crate) fn animate_shockwave(
    mut query: Query<(
        &ShockwaveRadius,
        &mut Scale2D,
        Option<&MeshMaterial2d<ColorMaterial>>,
    )>,
    mut materials: Option<ResMut<Assets<ColorMaterial>>>,
) {
    for (radius, mut scale, mat_handle) in &mut query {
        let diameter = radius.current * 2.0;
        scale.x = diameter;
        scale.y = diameter;

        if radius.max <= 0.0 {
            continue;
        }

        if let Some(mat_handle) = mat_handle
            && let Some(ref mut materials) = materials
        {
            let progress = (radius.current / radius.max).clamp(0.0, 1.0);
            let alpha = 0.9 * (1.0 - progress);
            if let Some(material) = materials.get_mut(mat_handle.id()) {
                material.color = material.color.with_alpha(alpha);
            }
        }
    }
}

/// Registers all observers and systems for the shockwave effect.
pub(crate) fn register(app: &mut App) {
    use crate::shared::PlayingState;

    app.add_observer(handle_shockwave);

    // Shockwave expansion + collision
    app.add_systems(
        FixedUpdate,
        (tick_shockwave, shockwave_collision.after(tick_shockwave))
            .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree)
            .run_if(in_state(PlayingState::Active)),
    );

    // Shockwave visual update
    app.add_systems(
        Update,
        animate_shockwave
            .run_if(any_with_component::<ShockwaveRadius>)
            .run_if(in_state(PlayingState::Active)),
    );
}
