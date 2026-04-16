//! Fires salvos when `SalvoFireTimer.0 <= 0.0`. Spawns salvo entities per
//! `AttackPattern`, resets the timer, and sets `SurvivalTimer.started = true`.
//! Skips turrets in `PhantomPhase::Ghost`.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::ApplyVelocity;

use crate::{
    cells::{
        behaviors::{
            phantom::components::PhantomPhase,
            survival::{
                components::{SurvivalPattern, SurvivalTimer, SurvivalTurret},
                salvo::components::{
                    SALVO_DAMAGE, SALVO_FIRE_INTERVAL, SALVO_HALF_EXTENT, SALVO_SPEED, Salvo,
                    SalvoDamage, SalvoFireTimer, SalvoSource,
                },
            },
        },
        definition::AttackPattern,
    },
    prelude::*,
};

/// Half-angle for spread pattern fan (30 degrees).
const SALVO_SPREAD_HALF_ANGLE: f32 = std::f32::consts::PI / 6.0;

type TurretFireQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut SalvoFireTimer,
        Option<&'static mut SurvivalTimer>,
        &'static SurvivalPattern,
        &'static Position2D,
        Option<&'static PhantomPhase>,
    ),
    (With<SurvivalTurret>, Without<Dead>),
>;

/// Fires salvos from turrets whose fire timer has expired.
pub(crate) fn fire_survival_turret(mut turret_query: TurretFireQuery, mut commands: Commands) {
    for (entity, mut fire_timer, survival_timer, pattern, pos, phantom) in &mut turret_query {
        if fire_timer.0 > 0.0 {
            continue;
        }

        // Skip Ghost-phase turrets
        if phantom.is_some_and(|p| *p == PhantomPhase::Ghost) {
            continue;
        }

        // Spawn salvos according to attack pattern
        match pattern.0 {
            AttackPattern::StraightDown => {
                spawn_salvo(&mut commands, entity, pos.0, Vec2::new(0.0, -SALVO_SPEED));
            }
            AttackPattern::Spread(n) => {
                if n == 0 {
                    // No salvos for Spread(0)
                } else if n == 1 {
                    // Single salvo straight down
                    spawn_salvo(&mut commands, entity, pos.0, Vec2::new(0.0, -SALVO_SPEED));
                } else {
                    for i in 0..n {
                        let angle = (i as f32 / (n - 1) as f32)
                            .mul_add(2.0 * SALVO_SPREAD_HALF_ANGLE, -SALVO_SPREAD_HALF_ANGLE);
                        let vel = Vec2::new(SALVO_SPEED * angle.sin(), -SALVO_SPEED * angle.cos());
                        spawn_salvo(&mut commands, entity, pos.0, vel);
                    }
                }
            }
        }

        // Reset fire timer
        fire_timer.0 = SALVO_FIRE_INTERVAL;

        // Set started = true on survival timer (if present)
        if let Some(mut timer) = survival_timer {
            timer.started = true;
        }
    }
}

fn spawn_salvo(commands: &mut Commands, source: Entity, position: Vec2, velocity: Vec2) {
    commands.spawn((
        Salvo,
        SalvoDamage(SALVO_DAMAGE),
        SalvoSource(source),
        Position2D(position),
        Velocity2D(velocity),
        Scale2D {
            x: SALVO_HALF_EXTENT * 2.0,
            y: SALVO_HALF_EXTENT * 2.0,
        },
        Aabb2D::new(Vec2::ZERO, Vec2::splat(SALVO_HALF_EXTENT)),
        CollisionLayers::new(
            SALVO_LAYER,
            CELL_LAYER | BOLT_LAYER | BREAKER_LAYER | WALL_LAYER,
        ),
        ApplyVelocity,
        Hp::new(1.0),
        KilledBy::default(),
        CleanupOnExit::<NodeState>::default(),
    ));
}
