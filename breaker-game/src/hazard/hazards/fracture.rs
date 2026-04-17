//! Fracture hazard — destroyed cells spawn low-HP debris cells in
//! adjacent positions. Stacking increases the debris count per death
//! (count only; debris HP stays at 1). Positions are world-space offsets
//! from the victim; the cells domain resolves any overlap naturally.

use bevy::prelude::*;

use crate::{
    cells::components::{Cell, CellHeight, CellWidth},
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
    shared::{
        collision_layers::{BOLT_LAYER, CELL_LAYER},
        death_pipeline::{Destroyed, Hp, KilledBy},
    },
};

/// Default debris cell dimensions. Matches the pristine cell footprint
/// used by `bolt_cell_collision` tests; keeps debris playable without
/// needing to consult per-node grid metrics here.
const DEBRIS_WIDTH: f32 = 70.0;
const DEBRIS_HEIGHT: f32 = 24.0;
/// Fixed 1 HP per debris cell regardless of stack — stacking increases
/// count only, per the design.
const DEBRIS_HP: f32 = 1.0;

/// World-space offsets for the four orthogonal debris positions, in the
/// order debris are placed (right, left, up, down). Over-stacks that
/// exceed four simply run out of offsets — the effective cap at stack 3+
/// is four debris per death.
const DEBRIS_OFFSETS: [Vec2; 4] = [
    Vec2::new(DEBRIS_WIDTH, 0.0),
    Vec2::new(-DEBRIS_WIDTH, 0.0),
    Vec2::new(0.0, DEBRIS_HEIGHT),
    Vec2::new(0.0, -DEBRIS_HEIGHT),
];

/// Per-run tuning extracted from [`HazardTuning::Fracture`].
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct FractureConfig {
    /// Debris spawned per death at stack 1.
    pub(crate) base_splits:      u32,
    /// Additional debris spawned per stack beyond the first.
    pub(crate) per_level_splits: u32,
}

impl FractureConfig {
    /// Debris count for the given stack count. 0 at stack 0; capped at
    /// [`DEBRIS_OFFSETS.len()`] so we never index out of bounds.
    #[must_use]
    pub(crate) const fn splits_for(self, stacks: u32) -> u32 {
        if stacks == 0 {
            return 0;
        }
        let extra = stacks.saturating_sub(1);
        let raw = self
            .base_splits
            .saturating_add(self.per_level_splits.saturating_mul(extra));
        if raw > DEBRIS_OFFSETS.len() as u32 {
            DEBRIS_OFFSETS.len() as u32
        } else {
            raw
        }
    }
}

/// Marks a cell as debris spawned by the Fracture hazard. Lets other
/// systems identify and (optionally) treat debris differently from
/// regular cells. Currently inert — reserved for future
/// "don't-recurse-fracture-on-debris" logic.
#[derive(Component, Debug, Default, Clone, Copy)]
pub(crate) struct FractureDebris;

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Fracture {
        base_splits,
        per_level_splits,
    } = *tuning
    else {
        warn!("fracture::activate called with non-Fracture tuning");
        return;
    };
    commands.insert_resource(FractureConfig {
        base_splits,
        per_level_splits,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        fracture_on_death
            .run_if(hazard_active(HazardKind::Fracture))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Spawns `splits_for(stacks)` debris cells in orthogonal positions
/// around each destroyed cell. Excludes the victim itself — debris
/// cells are `FractureDebris`-marked with `CellWidth`/`CellHeight` and
/// the same collision layers as regular cells.
fn fracture_on_death(
    mut reader: MessageReader<Destroyed<Cell>>,
    active: Res<ActiveHazards>,
    config: Option<Res<FractureConfig>>,
    mut commands: Commands,
) {
    let Some(config) = config else {
        reader.clear();
        return;
    };
    let stacks = active.stacks(HazardKind::Fracture);
    let count = config.splits_for(stacks) as usize;
    if count == 0 {
        reader.clear();
        return;
    }
    for destroyed in reader.read() {
        for offset in &DEBRIS_OFFSETS[..count] {
            let pos = destroyed.victim_pos + *offset;
            commands.spawn((
                Cell,
                FractureDebris,
                Position2D(pos),
                Scale2D {
                    x: DEBRIS_WIDTH,
                    y: DEBRIS_HEIGHT,
                },
                Aabb2D::new(
                    Vec2::ZERO,
                    Vec2::new(DEBRIS_WIDTH / 2.0, DEBRIS_HEIGHT / 2.0),
                ),
                CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                CellWidth::new(DEBRIS_WIDTH),
                CellHeight::new(DEBRIS_HEIGHT),
                Hp::new(DEBRIS_HP),
                KilledBy::default(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{marker::PhantomData, time::Duration};

    use bevy::prelude::Messages;

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

    fn write_cell_destroyed(app: &mut App, pos: Vec2) {
        app.world_mut()
            .resource_mut::<Messages<Destroyed<Cell>>>()
            .write(Destroyed::<Cell> {
                victim:     Entity::PLACEHOLDER,
                killer:     None,
                victim_pos: pos,
                killer_pos: None,
                _marker:    PhantomData,
            });
    }

    fn tick_with_dt(app: &mut App, dt: Duration) {
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .set_timestep(dt);
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(dt);
        app.update();
    }

    // ── splits_for formula ────────────────────────────────────────────────

    #[test]
    fn splits_zero_stacks_is_zero() {
        let cfg = FractureConfig {
            base_splits:      2,
            per_level_splits: 1,
        };
        assert_eq!(cfg.splits_for(0), 0);
    }

    #[test]
    fn splits_stack_one_is_base() {
        let cfg = FractureConfig {
            base_splits:      2,
            per_level_splits: 1,
        };
        assert_eq!(cfg.splits_for(1), 2);
    }

    #[test]
    fn splits_stack_three_adds_two_levels() {
        let cfg = FractureConfig {
            base_splits:      2,
            per_level_splits: 1,
        };
        // 2 + 1 * 2 = 4
        assert_eq!(cfg.splits_for(3), 4);
    }

    #[test]
    fn splits_are_capped_at_offset_count() {
        let cfg = FractureConfig {
            base_splits:      2,
            per_level_splits: 1,
        };
        // Stack 10 would be 11 but we have only 4 offsets.
        assert_eq!(cfg.splits_for(10), 4);
    }

    // ── fracture_on_death ────────────────────────────────────────────────

    #[test]
    fn stack_one_spawns_two_debris() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        app.world_mut().insert_resource(FractureConfig {
            base_splits:      2,
            per_level_splits: 1,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Fracture);

        write_cell_destroyed(&mut app, Vec2::new(100.0, 200.0));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app
            .world_mut()
            .query::<(&FractureDebris, &Position2D, &Hp)>();
        let debris: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(debris.len(), 2);
        for (_, pos, hp) in &debris {
            // Each debris sits at the victim position plus one of the
            // first two offsets (right, left).
            let offset = pos.0 - Vec2::new(100.0, 200.0);
            assert!(
                (offset - Vec2::new(70.0, 0.0)).length() < 1e-4
                    || (offset - Vec2::new(-70.0, 0.0)).length() < 1e-4,
                "debris at unexpected offset: {offset:?}"
            );
            assert!((hp.current - 1.0).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn stack_three_spawns_four_debris() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        app.world_mut().insert_resource(FractureConfig {
            base_splits:      2,
            per_level_splits: 1,
        });
        for _ in 0..3 {
            app.world_mut()
                .resource_mut::<ActiveHazards>()
                .add_stack(HazardKind::Fracture);
        }

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 4);
    }

    #[test]
    fn debris_has_collision_components() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        app.world_mut().insert_resource(FractureConfig {
            base_splits:      2,
            per_level_splits: 1,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Fracture);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app
            .world_mut()
            .query::<(&Cell, &CellWidth, &CellHeight, &Aabb2D, &CollisionLayers)>();
        let count = query.iter(app.world()).count();
        assert_eq!(count, 2, "debris should have full collision suite");
    }

    #[test]
    fn multiple_deaths_each_spawn_debris_independently() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        app.world_mut().insert_resource(FractureConfig {
            base_splits:      2,
            per_level_splits: 1,
        });
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Fracture);

        write_cell_destroyed(&mut app, Vec2::new(100.0, 200.0));
        write_cell_destroyed(&mut app, Vec2::new(500.0, 300.0));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 4);
    }

    #[test]
    fn no_debris_when_config_absent() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Fracture);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    #[test]
    fn no_debris_at_zero_stacks() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        app.world_mut().insert_resource(FractureConfig {
            base_splits:      2,
            per_level_splits: 1,
        });

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    // ── activate — preserved scaffold tests ────────────────────────────────

    #[test]
    fn activate_with_matching_tuning_inserts_config() {
        let mut app = TestAppBuilder::new().build();
        app.add_systems(Update, |mut commands: Commands| {
            activate(
                &HazardTuning::Fracture {
                    base_splits:      2,
                    per_level_splits: 1,
                },
                &mut commands,
            );
        });
        app.update();

        let cfg = app.world().resource::<FractureConfig>();
        assert_eq!(cfg.base_splits, 2);
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
        assert!(app.world().get_resource::<FractureConfig>().is_none());
    }
}
