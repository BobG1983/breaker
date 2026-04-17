//! Cascade hazard — heals neighbouring cells when one dies. Stacks add
//! linear heal per neighbour. "Adjacent" is approximated as any cell whose
//! centre is within `ADJACENCY_RADIUS` world units of the destroyed cell's
//! position; this avoids depending on grid-coordinate lookup which the
//! cells domain doesn't expose today.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    cells::components::Cell,
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
    shared::death_pipeline::{Destroyed, Hp},
};

/// World-space distance (squared) within which a living cell counts as a
/// neighbour of a destroyed cell. Tuned to ~1.25× a 50-unit cell width so
/// horizontally + vertically adjacent grid neighbours register without
/// pulling in diagonals across a wide gap.
const ADJACENCY_RADIUS_SQ: f32 = 70.0 * 70.0;

/// Per-run tuning extracted from [`HazardTuning::Cascade`] at activation.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct CascadeConfig {
    /// HP healed to each adjacent cell at stack 1.
    pub(crate) base_heal:      f32,
    /// Additional HP healed per stack beyond the first.
    pub(crate) per_level_heal: f32,
}

impl CascadeConfig {
    /// Heal amount per neighbour for the given stack count.
    /// Returns 0.0 when `stacks == 0`.
    #[must_use]
    pub(crate) const fn heal_per_neighbour(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        self.per_level_heal.mul_add(extra, self.base_heal)
    }
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Cascade {
        base_heal,
        per_level_heal,
    } = *tuning
    else {
        warn!("cascade::activate called with non-Cascade tuning");
        return;
    };
    commands.insert_resource(CascadeConfig {
        base_heal,
        per_level_heal,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        cascade_heal_on_death
            .run_if(hazard_active(HazardKind::Cascade))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// For every `Destroyed<Cell>` message, heal each living cell within
/// [`ADJACENCY_RADIUS_SQ`] of the victim by `heal_per_neighbour(stacks)`.
/// Heal is clamped against `Hp.max` (or `Hp.starting` if max is unset).
fn cascade_heal_on_death(
    mut reader: MessageReader<Destroyed<Cell>>,
    active: Res<ActiveHazards>,
    config: Option<Res<CascadeConfig>>,
    mut cells: Query<(Entity, &Position2D, &mut Hp), With<Cell>>,
) {
    let Some(config) = config else { return };
    let stacks = active.stacks(HazardKind::Cascade);
    let heal = config.heal_per_neighbour(stacks);
    if heal <= 0.0 {
        return;
    }
    let deaths: Vec<(Entity, Vec2)> = reader.read().map(|m| (m.victim, m.victim_pos)).collect();
    if deaths.is_empty() {
        return;
    }
    for (entity, position, mut hp) in &mut cells {
        if hp.current <= 0.0 {
            continue;
        }
        let neighbour_count = deaths
            .iter()
            .filter(|(victim, victim_pos)| {
                *victim != entity && position.0.distance_squared(*victim_pos) <= ADJACENCY_RADIUS_SQ
            })
            .count();
        if neighbour_count == 0 {
            continue;
        }
        let ceiling = hp.max.unwrap_or(hp.starting);
        let total = (neighbour_count as f32) * heal;
        hp.current = (hp.current + total).min(ceiling);
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use bevy::ecs::message::Messages;

    use super::*;
    use crate::prelude::TestAppBuilder;

    fn test_app_playing() -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_resource::<ActiveHazards>()
            .with_message::<Destroyed<Cell>>()
            .build()
    }

    fn spawn_cell_at(app: &mut App, pos: Vec2, current: f32, starting: f32) -> Entity {
        app.world_mut()
            .spawn((
                Cell,
                Position2D(pos),
                Hp {
                    current,
                    starting,
                    max: None,
                },
            ))
            .id()
    }

    fn send_cell_destroyed(app: &mut App, victim: Entity, victim_pos: Vec2) {
        app.world_mut()
            .resource_mut::<Messages<Destroyed<Cell>>>()
            .write(Destroyed::<Cell> {
                victim,
                killer: None,
                victim_pos,
                killer_pos: None,
                _marker: PhantomData,
            });
    }

    fn run_fixed_update(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── heal_per_neighbour formula ────────────────────────────────────────

    #[test]
    fn heal_zero_stacks_is_zero() {
        let cfg = CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        };
        assert!((cfg.heal_per_neighbour(0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn heal_stack_one_equals_base() {
        let cfg = CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        };
        assert!((cfg.heal_per_neighbour(1) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn heal_stack_three_adds_two_levels() {
        let cfg = CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        };
        // base 1.0 + 0.5 * 2 = 2.0.
        assert!((cfg.heal_per_neighbour(3) - 2.0).abs() < f32::EPSILON);
    }

    // ── cascade_heal_on_death — adjacency + heal ──────────────────────────

    #[test]
    fn neighbour_within_radius_gets_healed() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, cascade_heal_on_death);
        app.world_mut().insert_resource(CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Cascade);

        let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
        let dead = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
        send_cell_destroyed(&mut app, dead, Vec2::new(0.0, 0.0));

        run_fixed_update(&mut app);

        let hp = app.world().get::<Hp>(neighbour).unwrap();
        assert!(
            (hp.current - 6.0).abs() < f32::EPSILON,
            "neighbour at distance 50 should heal by 1.0 from 5.0 → 6.0, got {}",
            hp.current
        );
    }

    #[test]
    fn cell_outside_radius_is_not_healed() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, cascade_heal_on_death);
        app.world_mut().insert_resource(CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Cascade);

        let far = spawn_cell_at(&mut app, Vec2::new(200.0, 0.0), 5.0, 10.0);
        let dead = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
        send_cell_destroyed(&mut app, dead, Vec2::new(0.0, 0.0));

        run_fixed_update(&mut app);

        let hp = app.world().get::<Hp>(far).unwrap();
        assert!(
            (hp.current - 5.0).abs() < f32::EPSILON,
            "cell at distance 200 (outside ADJACENCY_RADIUS) must not heal, got {}",
            hp.current
        );
    }

    #[test]
    fn dead_cell_is_not_self_healed() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, cascade_heal_on_death);
        app.world_mut().insert_resource(CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Cascade);

        let dead = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
        send_cell_destroyed(&mut app, dead, Vec2::new(0.0, 0.0));

        run_fixed_update(&mut app);

        let hp = app.world().get::<Hp>(dead).unwrap();
        assert!(
            hp.current.abs() < f32::EPSILON,
            "dead cell must not heal itself, got {}",
            hp.current
        );
    }

    #[test]
    fn multiple_deaths_compound_heal_to_shared_neighbour() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, cascade_heal_on_death);
        app.world_mut().insert_resource(CascadeConfig {
            base_heal:      1.0,
            per_level_heal: 0.5,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Cascade);

        let neighbour = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 5.0, 20.0);
        let dead_a = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 0.0, 10.0);
        let dead_b = spawn_cell_at(&mut app, Vec2::new(-50.0, 0.0), 0.0, 10.0);
        send_cell_destroyed(&mut app, dead_a, Vec2::new(50.0, 0.0));
        send_cell_destroyed(&mut app, dead_b, Vec2::new(-50.0, 0.0));

        run_fixed_update(&mut app);

        let hp = app.world().get::<Hp>(neighbour).unwrap();
        assert!(
            (hp.current - 7.0).abs() < f32::EPSILON,
            "two adjacent deaths each heal by 1.0 → 5.0 + 2.0 = 7.0, got {}",
            hp.current
        );
    }

    #[test]
    fn heal_clamps_to_starting_hp() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, cascade_heal_on_death);
        app.world_mut().insert_resource(CascadeConfig {
            base_heal:      10.0,
            per_level_heal: 0.0,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Cascade);

        let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 8.0, 10.0);
        let dead = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
        send_cell_destroyed(&mut app, dead, Vec2::new(0.0, 0.0));

        run_fixed_update(&mut app);

        let hp = app.world().get::<Hp>(neighbour).unwrap();
        assert!(
            (hp.current - 10.0).abs() < f32::EPSILON,
            "heal must clamp to starting HP (10.0), got {}",
            hp.current
        );
    }

    #[test]
    fn no_heal_when_config_absent() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, cascade_heal_on_death);
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Cascade);

        let neighbour = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0), 5.0, 10.0);
        let dead = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0), 0.0, 10.0);
        send_cell_destroyed(&mut app, dead, Vec2::new(0.0, 0.0));

        run_fixed_update(&mut app);

        let hp = app.world().get::<Hp>(neighbour).unwrap();
        assert!((hp.current - 5.0).abs() < f32::EPSILON);
    }

    // ── activate — preserved scaffold tests ────────────────────────────────

    #[test]
    fn activate_with_matching_tuning_inserts_config() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &HazardTuning::Cascade {
                    base_heal:      1.0,
                    per_level_heal: 0.5,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<CascadeConfig>();
        assert!((cfg.base_heal - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn activate_with_mismatched_tuning_does_nothing() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &HazardTuning::Decay {
                    base_percent:      0.05,
                    per_level_percent: 0.03,
                },
                &mut commands,
            );
        });
        app.update();
        assert!(app.world().get_resource::<CascadeConfig>().is_none());
    }
}
