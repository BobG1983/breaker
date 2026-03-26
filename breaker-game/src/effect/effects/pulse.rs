//! Pulse effect handler — shockwave at every bolt position.
//!
//! Observes [`PulseFired`] and spawns a shockwave entity at each bolt's
//! position. Functionally equivalent to firing a shockwave at every bolt
//! simultaneously.

use bevy::{prelude::*, sprite_render::AlphaMode2d};
use rantzsoft_spatial2d::components::{Position2D, Scale2D, Spatial2D};

use crate::{
    bolt::components::Bolt,
    chips::components::DamageBoost,
    effect::{
        definition::EffectTarget,
        effects::shockwave::{ShockwaveAlreadyHit, ShockwaveDamage, ShockwaveRadius, ShockwaveSpeed},
    },
    shared::{BASE_BOLT_DAMAGE, CleanupOnNodeExit, GameDrawLayer},
};

// ---------------------------------------------------------------------------
// Typed event
// ---------------------------------------------------------------------------

/// Fired when a pulse effect resolves (shockwave at every bolt).
#[derive(Event, Clone, Debug)]
pub(crate) struct PulseFired {
    /// Base radius of each shockwave.
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
// Observer — spawns shockwave entity at each bolt position
// ---------------------------------------------------------------------------

/// Observer: spawns a shockwave entity at every active bolt position.
pub(crate) fn handle_pulse(
    trigger: On<PulseFired>,
    mut commands: Commands,
    bolt_query: Query<(Entity, &Position2D, Option<&DamageBoost>), With<Bolt>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let event = trigger.event();

    if event.speed <= 0.0 {
        return;
    }

    let extra_stacks =
        f32::from(u16::try_from(event.stacks.saturating_sub(1)).unwrap_or(u16::MAX));
    let max = event.base_range + extra_stacks * event.range_per_level;

    for (bolt_entity, bolt_pos, damage_boost) in &bolt_query {
        let damage = BASE_BOLT_DAMAGE * (1.0 + damage_boost.map_or(0.0, |b| b.0));

        commands.spawn((
            Position2D(bolt_pos.0),
            ShockwaveRadius { current: 0.0, max },
            ShockwaveSpeed(event.speed),
            ShockwaveDamage {
                damage,
                source_chip: event.source_chip.clone(),
                source_bolt: Some(bolt_entity),
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
}

/// Registers observers for the pulse effect.
pub(crate) fn register(app: &mut App) {
    app.add_observer(handle_pulse);
}

#[cfg(test)]
mod tests {
    use rantzsoft_spatial2d::components::Position2D;

    use super::*;
    use crate::{
        bolt::components::Bolt,
        chips::components::DamageBoost,
        effect::effects::shockwave::{ShockwaveDamage, ShockwaveRadius, ShockwaveSpeed},
        shared::BASE_BOLT_DAMAGE,
    };

    // --- Test infrastructure ---

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .add_observer(handle_pulse);
        app
    }

    fn spawn_bolt(app: &mut App, x: f32, y: f32) -> Entity {
        app.world_mut()
            .spawn((Bolt, Position2D(Vec2::new(x, y))))
            .id()
    }

    fn spawn_bolt_with_damage_boost(app: &mut App, x: f32, y: f32, boost: f32) -> Entity {
        app.world_mut()
            .spawn((Bolt, Position2D(Vec2::new(x, y)), DamageBoost(boost)))
            .id()
    }

    fn trigger_pulse(app: &mut App, base_range: f32, range_per_level: f32, stacks: u32, speed: f32) {
        app.world_mut().commands().trigger(PulseFired {
            base_range,
            range_per_level,
            stacks,
            speed,
            targets: vec![],
            source_chip: None,
        });
        app.world_mut().flush();
    }

    fn shockwave_entity_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<ShockwaveRadius>>()
            .iter(app.world())
            .count()
    }

    // =========================================================================
    // Tests
    // =========================================================================

    /// Behavior 1: Pulse spawns a shockwave entity at each active bolt position.
    /// Given 2 bolts at (10.0, 20.0) and (50.0, 60.0), pulse should produce
    /// 2 shockwave entities with correct `ShockwaveRadius` and `ShockwaveSpeed`.
    #[test]
    fn pulse_spawns_shockwave_at_each_bolt_position() {
        let mut app = test_app();
        let _bolt_a = spawn_bolt(&mut app, 10.0, 20.0);
        let _bolt_b = spawn_bolt(&mut app, 50.0, 60.0);

        trigger_pulse(&mut app, 32.0, 0.0, 1, 400.0);

        assert_eq!(
            shockwave_entity_count(&mut app),
            2,
            "pulse should spawn one shockwave per bolt (2 bolts -> 2 shockwaves)"
        );

        // Verify each shockwave has the correct radius and speed
        let mut radii: Vec<(f32, f32)> = app
            .world_mut()
            .query::<(&ShockwaveRadius, &ShockwaveSpeed)>()
            .iter(app.world())
            .map(|(r, s)| (r.max, s.0))
            .collect();
        radii.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        for (max_radius, speed) in &radii {
            assert!(
                (*max_radius - 32.0).abs() < f32::EPSILON,
                "each shockwave max radius should be 32.0, got {max_radius}"
            );
            assert!(
                (*speed - 400.0).abs() < f32::EPSILON,
                "each shockwave speed should be 400.0, got {speed}"
            );
        }
    }

    /// Behavior 2: Pulse respects `DamageBoost` on the source bolt.
    /// A bolt with `DamageBoost(0.5)` should produce a shockwave with
    /// damage = `BASE_BOLT_DAMAGE` * (1.0 + 0.5) = 15.0.
    #[test]
    fn pulse_respects_damage_boost() {
        let mut app = test_app();
        let _bolt = spawn_bolt_with_damage_boost(&mut app, 0.0, 0.0, 0.5);

        trigger_pulse(&mut app, 32.0, 0.0, 1, 400.0);

        assert_eq!(
            shockwave_entity_count(&mut app),
            1,
            "pulse should spawn one shockwave for the single bolt"
        );

        let dmg = app
            .world_mut()
            .query::<&ShockwaveDamage>()
            .iter(app.world())
            .next()
            .expect("shockwave entity should have ShockwaveDamage");
        let expected_damage = BASE_BOLT_DAMAGE * (1.0 + 0.5);
        assert!(
            (dmg.damage - expected_damage).abs() < f32::EPSILON,
            "with DamageBoost(0.5), shockwave damage should be {expected_damage}, got {}",
            dmg.damage
        );
    }

    /// Behavior 3: Pulse range scales with stacks.
    /// `stacks=3`, `base_range=32`, `range_per_level=16` -> max = 32 + (3-1)*16 = 64.0.
    #[test]
    fn pulse_range_scales_with_stacks() {
        let mut app = test_app();
        let _bolt = spawn_bolt(&mut app, 0.0, 0.0);

        trigger_pulse(&mut app, 32.0, 16.0, 3, 400.0);

        assert_eq!(
            shockwave_entity_count(&mut app),
            1,
            "pulse should spawn one shockwave for the single bolt"
        );

        let radius = app
            .world_mut()
            .query::<&ShockwaveRadius>()
            .iter(app.world())
            .next()
            .expect("shockwave entity should have ShockwaveRadius");
        // max = base_range + (stacks - 1) * range_per_level = 32 + 2*16 = 64
        assert!(
            (radius.max - 64.0).abs() < f32::EPSILON,
            "max radius should be 64.0 (32 + 2*16), got {}",
            radius.max
        );
    }

    /// Behavior 4: Zero speed is a no-op -- no shockwave entities spawned.
    #[test]
    fn pulse_speed_zero_is_noop() {
        let mut app = test_app();
        let _bolt = spawn_bolt(&mut app, 0.0, 0.0);

        trigger_pulse(&mut app, 32.0, 0.0, 1, 0.0);

        assert_eq!(
            shockwave_entity_count(&mut app),
            0,
            "zero speed pulse should not spawn any shockwave entities"
        );
    }

    /// Behavior 5: No bolts in play -> no shockwave entities, no panic.
    #[test]
    fn pulse_no_bolts_is_noop() {
        let mut app = test_app();

        // No bolts spawned
        trigger_pulse(&mut app, 32.0, 0.0, 1, 400.0);

        assert_eq!(
            shockwave_entity_count(&mut app),
            0,
            "pulse with no bolts should not spawn any shockwave entities"
        );
    }
}
