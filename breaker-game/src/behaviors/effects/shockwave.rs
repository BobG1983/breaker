//! Shockwave effect handler — expanding wavefront area damage.
//!
//! Observes [`EffectFired`], pattern-matches on [`TriggerChain::Shockwave`],
//! and spawns a [`ShockwaveRadius`] entity that expands over time. Collision
//! with cells is handled by [`shockwave_collision`], which writes [`DamageCell`]
//! messages. The visual is driven by [`animate_shockwave`] which scales the
//! entity based on the current radius.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{collision_layers::CollisionLayers, resources::CollisionQuadtree};
use rantzsoft_spatial2d::components::{Position2D, Scale2D, Spatial2D};

use crate::{
    behaviors::events::EffectFired,
    cells::messages::DamageCell,
    chips::{components::DamageBoost, definition::TriggerChain},
    shared::{BASE_BOLT_DAMAGE, CELL_LAYER, CleanupOnNodeExit, GameDrawLayer},
};

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
    /// The bolt entity that caused this shockwave (for VFX / `DamageCell`).
    pub source_bolt: Entity,
}

/// Tracks which cell entities have already been hit by this shockwave.
#[derive(Component, Debug, Clone, Default)]
pub(crate) struct ShockwaveAlreadyHit(pub HashSet<Entity>);

// ---------------------------------------------------------------------------
// Observer — spawns shockwave entity
// ---------------------------------------------------------------------------

/// Observer: spawns a shockwave wavefront entity when an [`EffectFired`] event
/// carries a [`TriggerChain::Shockwave`] leaf.
///
/// Does NOT write [`DamageCell`] — that is handled by [`shockwave_collision`].
pub(crate) fn handle_shockwave(
    trigger: On<EffectFired>,
    mut commands: Commands,
    bolt_query: Query<(&Position2D, Option<&DamageBoost>)>,
) {
    let TriggerChain::Shockwave {
        base_range,
        range_per_level,
        stacks,
        speed,
    } = &trigger.event().effect
    else {
        return;
    };

    if *speed <= 0.0 {
        return;
    }

    let Some(bolt_entity) = trigger.event().bolt else {
        return;
    };

    let Ok((bolt_pos, damage_boost)) = bolt_query.get(bolt_entity) else {
        return;
    };

    let max = base_range + (stacks.saturating_sub(1) as f32) * range_per_level;
    let damage = BASE_BOLT_DAMAGE * (1.0 + damage_boost.map_or(0.0, |b| b.0));

    commands.spawn((
        Position2D(bolt_pos.0),
        ShockwaveRadius {
            current: 0.0,
            max,
        },
        ShockwaveSpeed(*speed),
        ShockwaveDamage {
            damage,
            source_bolt: bolt_entity,
        },
        ShockwaveAlreadyHit::default(),
        GameDrawLayer::Fx,
        Scale2D::default(),
        CleanupOnNodeExit,
        Spatial2D,
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
        radius.current += speed.0 * time.delta_secs();
        if radius.current >= radius.max {
            commands.entity(entity).despawn();
        }
    }
}

/// Damages cells within the shockwave's current radius via quadtree query.
pub(crate) fn shockwave_collision(
    quadtree: Res<CollisionQuadtree>,
    mut shockwave_query: Query<(&Position2D, &ShockwaveRadius, &ShockwaveDamage, &mut ShockwaveAlreadyHit)>,
    cell_query: Query<Has<crate::cells::components::Locked>>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    for (pos, radius, dmg, mut already_hit) in &mut shockwave_query {
        let candidates = quadtree
            .quadtree
            .query_circle_filtered(pos.0, radius.current, CollisionLayers::new(0, CELL_LAYER));

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
                source_bolt: dmg.source_bolt,
            });
            already_hit.0.insert(candidate);
        }
    }
}

/// Scales the shockwave visual based on the current radius.
pub(crate) fn animate_shockwave(mut query: Query<(&ShockwaveRadius, &mut Scale2D)>) {
    for (radius, mut scale) in &mut query {
        scale.x = radius.current * 2.0;
        scale.y = radius.current * 2.0;
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_physics2d::{
        aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
    };
    use rantzsoft_spatial2d::components::{Position2D, Scale2D, Spatial2D};

    use super::*;
    use crate::{
        cells::{
            components::{Cell, CellHealth, Locked},
            messages::DamageCell,
        },
        chips::{components::DamageBoost, definition::TriggerChain},
        shared::{BASE_BOLT_DAMAGE, BOLT_LAYER, CELL_LAYER, CleanupOnNodeExit, GameDrawLayer},
    };

    // --- Test infrastructure ---

    /// Captured `DamageCell` messages written by the shockwave collision system.
    #[derive(Resource, Default)]
    struct CapturedDamage(Vec<DamageCell>);

    fn capture_damage(mut reader: MessageReader<DamageCell>, mut captured: ResMut<CapturedDamage>) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(RantzPhysics2dPlugin)
            .add_message::<DamageCell>()
            .init_resource::<CapturedDamage>()
            .add_systems(FixedPostUpdate, capture_damage)
            .add_observer(handle_shockwave);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn spawn_bolt(app: &mut App, x: f32, y: f32) -> Entity {
        app.world_mut().spawn(Position2D(Vec2::new(x, y))).id()
    }

    fn spawn_bolt_with_damage_boost(app: &mut App, x: f32, y: f32, boost: f32) -> Entity {
        app.world_mut()
            .spawn((Position2D(Vec2::new(x, y)), DamageBoost(boost)))
            .id()
    }

    fn spawn_cell(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
        app.world_mut()
            .spawn((
                Cell,
                CellHealth::new(hp),
                Aabb2D::new(Vec2::ZERO, Vec2::new(35.0, 12.0)),
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                Position2D(Vec2::new(x, y)),
                Spatial2D,
                GameDrawLayer::Cell,
            ))
            .id()
    }

    fn spawn_locked_cell(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
        app.world_mut()
            .spawn((
                Cell,
                CellHealth::new(hp),
                Locked,
                Aabb2D::new(Vec2::ZERO, Vec2::new(35.0, 12.0)),
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                Position2D(Vec2::new(x, y)),
                Spatial2D,
                GameDrawLayer::Cell,
            ))
            .id()
    }

    fn trigger_shockwave(app: &mut App, bolt: Entity, range: f32, speed: f32) {
        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::Shockwave {
                base_range: range,
                range_per_level: 0.0,
                stacks: 1,
                speed,
            },
            bolt: Some(bolt),
        });
        app.world_mut().flush();
        tick(app);
    }

    /// Count entities with [`ShockwaveRadius`] component.
    fn shockwave_entity_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<ShockwaveRadius>>()
            .iter(app.world())
            .count()
    }

    /// Get the first shockwave entity (panics if none).
    fn get_shockwave_entity(app: &mut App) -> Entity {
        app.world_mut()
            .query_filtered::<Entity, With<ShockwaveRadius>>()
            .iter(app.world())
            .next()
            .expect("should have at least one ShockwaveRadius entity")
    }

    // =========================================================================
    // Part A: Observer spawns entity
    // =========================================================================

    /// Behavior 1: Observer spawns `ShockwaveRadius` entity at bolt position
    /// with correct components. Max = base_range + (stacks-1) * range_per_level.
    #[test]
    fn observer_spawns_shockwave_entity_at_bolt_position() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 50.0, 100.0);

        // base_range=64, range_per_level=32, stacks=2 -> max = 64 + (2-1)*32 = 96
        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::Shockwave {
                base_range: 64.0,
                range_per_level: 32.0,
                stacks: 2,
                speed: 400.0,
            },
            bolt: Some(bolt),
        });
        app.world_mut().flush();
        tick(&mut app);

        // A ShockwaveRadius entity should exist
        assert_eq!(
            shockwave_entity_count(&mut app),
            1,
            "observer should spawn exactly one ShockwaveRadius entity"
        );

        let sw_entity = get_shockwave_entity(&mut app);
        let world = app.world();

        // Position2D should match the bolt's position
        let pos = world
            .get::<Position2D>(sw_entity)
            .expect("shockwave entity should have Position2D");
        assert!(
            (pos.0.x - 50.0).abs() < f32::EPSILON,
            "shockwave x should be 50.0, got {}",
            pos.0.x
        );
        assert!(
            (pos.0.y - 100.0).abs() < f32::EPSILON,
            "shockwave y should be 100.0, got {}",
            pos.0.y
        );

        // ShockwaveRadius should start at 0.0 with max = 96.0
        let radius = world
            .get::<ShockwaveRadius>(sw_entity)
            .expect("shockwave entity should have ShockwaveRadius");
        assert!(
            radius.current.abs() < f32::EPSILON,
            "initial radius.current should be 0.0, got {}",
            radius.current
        );
        assert!(
            (radius.max - 96.0).abs() < f32::EPSILON,
            "radius.max should be 96.0, got {}",
            radius.max
        );

        // ShockwaveSpeed
        let speed = world
            .get::<ShockwaveSpeed>(sw_entity)
            .expect("shockwave entity should have ShockwaveSpeed");
        assert!(
            (speed.0 - 400.0).abs() < f32::EPSILON,
            "ShockwaveSpeed should be 400.0, got {}",
            speed.0
        );

        // ShockwaveDamage with damage = BASE_BOLT_DAMAGE * 1.0 = 10.0
        let dmg = world
            .get::<ShockwaveDamage>(sw_entity)
            .expect("shockwave entity should have ShockwaveDamage");
        assert!(
            (dmg.damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
            "damage should be {} (no boost), got {}",
            BASE_BOLT_DAMAGE,
            dmg.damage
        );
        assert_eq!(dmg.source_bolt, bolt, "source_bolt should be the triggering bolt");

        // ShockwaveAlreadyHit (empty)
        let already_hit = world
            .get::<ShockwaveAlreadyHit>(sw_entity)
            .expect("shockwave entity should have ShockwaveAlreadyHit");
        assert!(
            already_hit.0.is_empty(),
            "ShockwaveAlreadyHit should start empty"
        );

        // GameDrawLayer::Fx
        let draw_layer = world
            .get::<GameDrawLayer>(sw_entity)
            .expect("shockwave entity should have GameDrawLayer");
        assert!(
            matches!(draw_layer, GameDrawLayer::Fx),
            "draw layer should be Fx"
        );

        // CleanupOnNodeExit
        assert!(
            world.get::<CleanupOnNodeExit>(sw_entity).is_some(),
            "shockwave entity should have CleanupOnNodeExit"
        );

        // Spatial2D (required component bundle)
        assert!(
            world.get::<Spatial2D>(sw_entity).is_some(),
            "shockwave entity should have Spatial2D"
        );

        // Scale2D::default (1.0, 1.0) — animation will set it
        let scale = world
            .get::<Scale2D>(sw_entity)
            .expect("shockwave entity should have Scale2D");
        assert!(
            (scale.x - 1.0).abs() < f32::EPSILON && (scale.y - 1.0).abs() < f32::EPSILON,
            "initial Scale2D should be (1.0, 1.0), got ({}, {})",
            scale.x,
            scale.y
        );

        // NO DamageCell messages should have been written by the observer
        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            0,
            "observer should NOT write DamageCell — collision system handles that; got {}",
            captured.0.len()
        );
    }

    /// Behavior 2: Without `DamageBoost`, damage = `BASE_BOLT_DAMAGE` * 1.0.
    #[test]
    fn observer_calculates_damage_without_boost() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);

        trigger_shockwave(&mut app, bolt, 96.0, 400.0);

        let sw_entity = get_shockwave_entity(&mut app);
        let dmg = app
            .world()
            .get::<ShockwaveDamage>(sw_entity)
            .expect("shockwave entity should have ShockwaveDamage");
        assert!(
            (dmg.damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
            "without DamageBoost, damage should be {} (BASE_BOLT_DAMAGE * 1.0), got {}",
            BASE_BOLT_DAMAGE,
            dmg.damage
        );
    }

    /// Behavior 2 variant: With `DamageBoost(0.5)`, damage = 10.0 * 1.5 = 15.0.
    #[test]
    fn observer_calculates_damage_with_boost() {
        let mut app = test_app();
        let bolt = spawn_bolt_with_damage_boost(&mut app, 0.0, 0.0, 0.5);

        trigger_shockwave(&mut app, bolt, 96.0, 400.0);

        let sw_entity = get_shockwave_entity(&mut app);
        let dmg = app
            .world()
            .get::<ShockwaveDamage>(sw_entity)
            .expect("shockwave entity should have ShockwaveDamage");
        // damage = BASE_BOLT_DAMAGE * (1.0 + 0.5) = 10.0 * 1.5 = 15.0
        assert!(
            (dmg.damage - 15.0).abs() < f32::EPSILON,
            "with DamageBoost(0.5), damage should be 15.0 (10.0 * 1.5), got {}",
            dmg.damage
        );
    }

    /// Behavior 3: Zero speed means do not spawn (return early).
    #[test]
    fn observer_does_not_spawn_when_speed_is_zero() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);

        trigger_shockwave(&mut app, bolt, 96.0, 0.0);

        assert_eq!(
            shockwave_entity_count(&mut app),
            0,
            "zero speed should result in no ShockwaveRadius entity"
        );
    }

    /// Behavior 4: Non-Shockwave effect does not spawn entity.
    #[test]
    fn observer_ignores_non_shockwave_effects() {
        let mut app = test_app();
        let bolt = spawn_bolt(&mut app, 0.0, 0.0);

        app.world_mut().commands().trigger(EffectFired {
            effect: TriggerChain::MultiBolt {
                base_count: 3,
                count_per_level: 0,
                stacks: 1,
            },
            bolt: Some(bolt),
        });
        app.world_mut().flush();
        tick(&mut app);

        assert_eq!(
            shockwave_entity_count(&mut app),
            0,
            "MultiBolt effect should not spawn a ShockwaveRadius entity"
        );
    }

    // =========================================================================
    // Part B: tick_shockwave
    // =========================================================================

    /// Behavior 5: `tick_shockwave` expands radius by speed * dt.
    #[test]
    fn tick_shockwave_expands_radius_by_speed_times_dt() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, tick_shockwave);

        let entity = app
            .world_mut()
            .spawn((
                ShockwaveRadius {
                    current: 0.0,
                    max: 96.0,
                },
                ShockwaveSpeed(400.0),
            ))
            .id();

        tick(&mut app);

        let radius = app
            .world()
            .get::<ShockwaveRadius>(entity)
            .expect("entity should still exist");
        // dt = 1/64 = 0.015625, expansion = 400.0 * 0.015625 = 6.25
        let expected = 400.0 / 64.0;
        assert!(
            (radius.current - expected).abs() < 0.1,
            "after one tick, radius.current should be ~{expected:.2}, got {:.2}",
            radius.current
        );
    }

    /// Behavior 6: `tick_shockwave` despawns when current >= max.
    #[test]
    fn tick_shockwave_despawns_when_fully_expanded() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, tick_shockwave);

        let entity = app
            .world_mut()
            .spawn((
                ShockwaveRadius {
                    current: 95.0,
                    max: 96.0,
                },
                ShockwaveSpeed(400.0),
            ))
            .id();

        // One tick: 95.0 + 6.25 = 101.25 >= 96.0 -> despawn
        tick(&mut app);

        assert!(
            app.world().get_entity(entity).is_err(),
            "entity should be despawned when current >= max"
        );
    }

    // =========================================================================
    // Part C: shockwave_collision
    // =========================================================================

    /// Behavior 7: Damages cells within current radius, adds to
    /// `ShockwaveAlreadyHit`.
    #[test]
    fn shockwave_collision_damages_cells_within_radius() {
        let mut app = test_app();
        app.add_systems(
            FixedUpdate,
            shockwave_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
        );

        // Spawn a cell at (30, 0)
        let cell = spawn_cell(&mut app, 30.0, 0.0, 20.0);

        // Let quadtree update
        tick(&mut app);

        // Spawn shockwave at origin with radius already covering the cell
        let sw_bolt = spawn_bolt(&mut app, 0.0, 0.0);
        app.world_mut().spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            ShockwaveRadius {
                current: 50.0,
                max: 96.0,
            },
            ShockwaveDamage {
                damage: 10.0,
                source_bolt: sw_bolt,
            },
            ShockwaveAlreadyHit::default(),
        ));

        tick(&mut app);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            1,
            "cell at distance 30 should be hit by shockwave with radius 50, got {} hits",
            captured.0.len()
        );
        assert_eq!(captured.0[0].cell, cell);
        assert!(
            (captured.0[0].damage - 10.0).abs() < f32::EPSILON,
            "damage should be 10.0, got {}",
            captured.0[0].damage
        );
    }

    /// Behavior 8: Skips already-hit cells.
    #[test]
    fn shockwave_collision_skips_already_hit_cells() {
        let mut app = test_app();
        app.add_systems(
            FixedUpdate,
            shockwave_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
        );

        let cell = spawn_cell(&mut app, 30.0, 0.0, 20.0);

        // Let quadtree update
        tick(&mut app);

        let sw_bolt = spawn_bolt(&mut app, 0.0, 0.0);
        let mut already_hit = HashSet::new();
        already_hit.insert(cell);
        app.world_mut().spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            ShockwaveRadius {
                current: 50.0,
                max: 96.0,
            },
            ShockwaveDamage {
                damage: 10.0,
                source_bolt: sw_bolt,
            },
            ShockwaveAlreadyHit(already_hit),
        ));

        tick(&mut app);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            0,
            "already-hit cell should not receive DamageCell again, got {} hits",
            captured.0.len()
        );
    }

    /// Behavior 9: Skips locked cells.
    #[test]
    fn shockwave_collision_skips_locked_cells() {
        let mut app = test_app();
        app.add_systems(
            FixedUpdate,
            shockwave_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
        );

        let _locked = spawn_locked_cell(&mut app, 30.0, 0.0, 20.0);

        // Let quadtree update
        tick(&mut app);

        let sw_bolt = spawn_bolt(&mut app, 0.0, 0.0);
        app.world_mut().spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            ShockwaveRadius {
                current: 50.0,
                max: 96.0,
            },
            ShockwaveDamage {
                damage: 10.0,
                source_bolt: sw_bolt,
            },
            ShockwaveAlreadyHit::default(),
        ));

        tick(&mut app);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            0,
            "locked cell should not receive DamageCell, got {} hits",
            captured.0.len()
        );
    }

    /// Behavior 10: Cell without `Aabb2D` is invisible to quadtree query.
    #[test]
    fn shockwave_collision_only_finds_cells_via_quadtree() {
        let mut app = test_app();
        app.add_systems(
            FixedUpdate,
            shockwave_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
        );

        // Bare cell without Aabb2D — not in quadtree
        app.world_mut().spawn((
            Cell,
            CellHealth::new(20.0),
            Position2D(Vec2::new(10.0, 0.0)),
            Spatial2D,
            GameDrawLayer::Cell,
        ));

        // Properly registered cell
        let registered = spawn_cell(&mut app, 20.0, 0.0, 20.0);

        // Let quadtree update
        tick(&mut app);

        let sw_bolt = spawn_bolt(&mut app, 0.0, 0.0);
        app.world_mut().spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            ShockwaveRadius {
                current: 50.0,
                max: 96.0,
            },
            ShockwaveDamage {
                damage: 10.0,
                source_bolt: sw_bolt,
            },
            ShockwaveAlreadyHit::default(),
        ));

        tick(&mut app);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            1,
            "only the registered cell (in quadtree) should be hit — bare cell invisible; got {}",
            captured.0.len()
        );
        assert_eq!(
            captured.0[0].cell, registered,
            "DamageCell should target the registered cell"
        );
    }

    /// Behavior 11: Shockwave uses `CollisionLayers::new(0, CELL_LAYER)`,
    /// so bolts are not hit.
    #[test]
    fn shockwave_collision_does_not_hit_bolt_entities() {
        let mut app = test_app();
        app.add_systems(
            FixedUpdate,
            shockwave_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
        );

        // Spawn a bolt entity with AABB (as if it were in the quadtree on BOLT_LAYER)
        app.world_mut().spawn((
            Position2D(Vec2::new(10.0, 0.0)),
            Aabb2D::new(Vec2::ZERO, Vec2::new(8.0, 8.0)),
            CollisionLayers::new(BOLT_LAYER, CELL_LAYER),
            Spatial2D,
        ));

        // Also spawn a proper cell to confirm collision works
        let cell = spawn_cell(&mut app, 20.0, 0.0, 20.0);

        // Let quadtree update
        tick(&mut app);

        let sw_bolt = spawn_bolt(&mut app, 0.0, 0.0);
        app.world_mut().spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            ShockwaveRadius {
                current: 50.0,
                max: 96.0,
            },
            ShockwaveDamage {
                damage: 10.0,
                source_bolt: sw_bolt,
            },
            ShockwaveAlreadyHit::default(),
        ));

        tick(&mut app);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            1,
            "only the cell should be hit, not the bolt entity; got {} hits",
            captured.0.len()
        );
        assert_eq!(
            captured.0[0].cell, cell,
            "DamageCell should target the cell, not the bolt"
        );
    }

    // =========================================================================
    // Part D: animate_shockwave
    // =========================================================================

    /// Behavior 12: `animate_shockwave` sets `Scale2D.x = current * 2.0` and
    /// `Scale2D.y = current * 2.0`.
    #[test]
    fn animate_shockwave_scales_by_diameter() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, animate_shockwave);

        let entity = app
            .world_mut()
            .spawn((
                ShockwaveRadius {
                    current: 48.0,
                    max: 96.0,
                },
                Scale2D::default(),
            ))
            .id();

        tick(&mut app);

        let scale = app
            .world()
            .get::<Scale2D>(entity)
            .expect("entity should have Scale2D");
        // current * 2.0 = 48.0 * 2.0 = 96.0
        assert!(
            (scale.x - 96.0).abs() < f32::EPSILON,
            "Scale2D.x should be 96.0 (48.0 * 2.0), got {}",
            scale.x
        );
        assert!(
            (scale.y - 96.0).abs() < f32::EPSILON,
            "Scale2D.y should be 96.0 (48.0 * 2.0), got {}",
            scale.y
        );
    }

    /// Behavior 12 edge case: `current = 0.0` should produce `Scale2D` (0.0, 0.0),
    /// NOT panic. This is why we write fields directly instead of using
    /// `Scale2D::new` (which panics on zero).
    #[test]
    fn animate_shockwave_zero_radius_does_not_panic() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, animate_shockwave);

        let entity = app
            .world_mut()
            .spawn((
                ShockwaveRadius {
                    current: 0.0,
                    max: 96.0,
                },
                Scale2D::default(),
            ))
            .id();

        tick(&mut app);

        let scale = app
            .world()
            .get::<Scale2D>(entity)
            .expect("entity should have Scale2D");
        assert!(
            scale.x.abs() < f32::EPSILON,
            "Scale2D.x should be 0.0 at zero radius, got {}",
            scale.x
        );
        assert!(
            scale.y.abs() < f32::EPSILON,
            "Scale2D.y should be 0.0 at zero radius, got {}",
            scale.y
        );
    }

    // =========================================================================
    // Part E: Integration
    // =========================================================================

    /// Behavior 13: Multi-tick wavefront — inner cells hit before outer cells.
    ///
    /// Spawns cells at distance 20 and 80. After a few ticks the wavefront
    /// should have reached the inner cell but not yet the outer one.
    #[test]
    fn multi_tick_wavefront_hits_inner_cells_before_outer() {
        let mut app = test_app();
        app.add_systems(
            FixedUpdate,
            (
                tick_shockwave,
                shockwave_collision
                    .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree)
                    .after(tick_shockwave),
            ),
        );

        let inner_cell = spawn_cell(&mut app, 20.0, 0.0, 20.0);
        let outer_cell = spawn_cell(&mut app, 80.0, 0.0, 20.0);

        // Let quadtree update
        tick(&mut app);

        // Spawn shockwave: speed=400, max=96
        // After 1 tick: radius = 400/64 ~= 6.25 (inner cell NOT hit yet)
        // After 4 ticks: radius ~= 25.0 (inner cell at 20 IS hit, outer at 80 NOT)
        let sw_bolt = spawn_bolt(&mut app, 0.0, 0.0);
        app.world_mut().spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            ShockwaveRadius {
                current: 0.0,
                max: 96.0,
            },
            ShockwaveSpeed(400.0),
            ShockwaveDamage {
                damage: 10.0,
                source_bolt: sw_bolt,
            },
            ShockwaveAlreadyHit::default(),
        ));

        // Tick 4 times: radius ~= 4 * 6.25 = 25.0
        for _ in 0..4 {
            tick(&mut app);
        }

        let captured = app.world().resource::<CapturedDamage>();

        // Inner cell at distance 20 should be hit (radius ~25)
        let hit_inner = captured.0.iter().any(|m| m.cell == inner_cell);
        assert!(
            hit_inner,
            "inner cell at distance 20 should be hit after radius expands to ~25"
        );

        // Outer cell at distance 80 should NOT be hit yet
        let hit_outer = captured.0.iter().any(|m| m.cell == outer_cell);
        assert!(
            !hit_outer,
            "outer cell at distance 80 should NOT be hit yet (radius ~25)"
        );
    }

    /// Behavior 14: Dangling source_bolt is acceptable — bolt may be despawned
    /// while the shockwave entity is still alive.
    #[test]
    fn dangling_source_bolt_does_not_panic() {
        let mut app = test_app();
        app.add_systems(
            FixedUpdate,
            shockwave_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
        );

        let cell = spawn_cell(&mut app, 30.0, 0.0, 20.0);

        // Let quadtree update
        tick(&mut app);

        // Spawn a bolt, then despawn it — creating a stale entity
        let stale_bolt = app.world_mut().spawn_empty().id();
        app.world_mut().despawn(stale_bolt);

        app.world_mut().spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            ShockwaveRadius {
                current: 50.0,
                max: 96.0,
            },
            ShockwaveDamage {
                damage: 10.0,
                source_bolt: stale_bolt,
            },
            ShockwaveAlreadyHit::default(),
        ));

        // Should not panic even though source_bolt is a stale entity
        tick(&mut app);

        let captured = app.world().resource::<CapturedDamage>();
        assert_eq!(
            captured.0.len(),
            1,
            "cell should still be damaged even with a stale source_bolt; got {} hits",
            captured.0.len()
        );
        assert_eq!(captured.0[0].cell, cell);
        assert_eq!(
            captured.0[0].source_bolt, stale_bolt,
            "DamageCell.source_bolt should carry the stale entity (no panic)"
        );
    }
}
