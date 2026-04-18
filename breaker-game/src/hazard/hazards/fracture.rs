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
    /// Debris count for the given stack count. Returns `0` when
    /// `stacks == 0` (hazard inactive). For `stacks >= 1`, returns
    /// `min(base_splits + per_level_splits * (stacks - 1),
    /// DEBRIS_OFFSETS.len() as u32)` — linear scaling in stacks, saturated
    /// at the number of orthogonal offsets (4) so indexing into
    /// `DEBRIS_OFFSETS` is always in-bounds. `saturating_sub`,
    /// `saturating_mul`, and `saturating_add` prevent overflow on
    /// pathological `u32::MAX` inputs. `const fn` so config consumers can
    /// evaluate at compile time.
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

/// Inserts `FractureConfig` from `HazardTuning::Fracture { base_splits,
/// per_level_splits }`. Called each time the player picks Fracture from
/// `hazards::activate`; last write wins (overwrites any prior
/// `FractureConfig`; stack count is owned by `ActiveHazards`, not the
/// config; debris already spawned retain their existing `Hp` — the
/// config only governs future spawn count). Does not mutate
/// `ActiveHazards`. Warns and no-ops on a non-Fracture tuning variant,
/// leaving any existing `FractureConfig` intact.
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

/// Registers the Fracture `FixedUpdate` system `fracture_on_death`,
/// gated on `hazard_active(HazardKind::Fracture)` AND
/// `in_state(NodeState::Playing)`. `fracture_on_death` reads
/// `Destroyed<Cell>` and spawns `count` debris cells per destroyed
/// cell at orthogonal offsets, where `count = FractureConfig::splits_for(stacks)`.
/// No `DeathPipelineSystems` ordering — debris are spawned directly via
/// `commands.spawn(...)`, not via a `SpawnDebrisCell` message
/// (pending the message pipeline in Commit 5 / Wave 7).
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
/// the same collision layers as regular cells. Early-returns (draining
/// the message reader via `reader.clear()`) when `FractureConfig` is
/// absent, or when `splits_for(stacks) == 0` (zero-stack inactive
/// path) — the drain prevents stale `Destroyed<Cell>` messages from
/// spawning debris on a later tick after the hazard activates.
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
    use std::{collections::HashSet, marker::PhantomData, time::Duration};

    use bevy::{ecs::world::CommandQueue, prelude::Messages};
    use rantzsoft_stateflow::CleanupOnExit;

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

    fn test_app_not_playing() -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .with_resource::<ActiveHazards>()
            .with_message::<Destroyed<Cell>>()
            .build()
    }

    const fn canonical_config() -> FractureConfig {
        FractureConfig {
            base_splits:      2,
            per_level_splits: 1,
        }
    }

    fn install_fracture_config(app: &mut App, cfg: FractureConfig) {
        app.world_mut().insert_resource(cfg);
    }

    fn add_fracture_stacks(app: &mut App, count: u32) {
        let mut active = app.world_mut().resource_mut::<ActiveHazards>();
        for _ in 0..count {
            active.add_stack(HazardKind::Fracture);
        }
    }

    fn activate_now(app: &mut App, tuning: &HazardTuning) {
        let mut queue = CommandQueue::default();
        {
            let mut commands = Commands::new(&mut queue, app.world());
            activate(tuning, &mut commands);
        }
        queue.apply(app.world_mut());
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

    fn approx_eq_vec2(a: Vec2, b: Vec2, tol: f32) -> bool {
        (a - b).length() < tol
    }

    // ── splits_for formula ────────────────────────────────────────────────

    // Behavior 1 — stack 0 returns 0 (identity).
    #[test]
    fn splits_zero_stacks_is_zero() {
        let cfg = FractureConfig {
            base_splits:      2,
            per_level_splits: 1,
        };
        assert_eq!(cfg.splits_for(0), 0);
    }

    #[test]
    fn splits_zero_stacks_is_zero_even_with_large_base() {
        let cfg = FractureConfig {
            base_splits:      100,
            per_level_splits: 50,
        };
        assert_eq!(cfg.splits_for(0), 0);
    }

    // Behavior 2 — stack 1 returns base_splits.
    #[test]
    fn splits_stack_one_is_base() {
        let cfg = FractureConfig {
            base_splits:      2,
            per_level_splits: 1,
        };
        assert_eq!(cfg.splits_for(1), 2);
    }

    #[test]
    fn splits_stack_one_ignores_per_level_multiplier() {
        let cfg = FractureConfig {
            base_splits:      2,
            per_level_splits: 999,
        };
        assert_eq!(cfg.splits_for(1), 2);
    }

    // Behavior 3 — stack 2 returns base + per_level.
    #[test]
    fn splits_stack_two_is_base_plus_per_level() {
        let cfg = canonical_config();
        assert_eq!(cfg.splits_for(2), 3);
    }

    #[test]
    fn splits_stack_two_with_zero_base_uses_per_level() {
        let cfg = FractureConfig {
            base_splits:      0,
            per_level_splits: 3,
        };
        assert_eq!(cfg.splits_for(2), 3);
    }

    // Behavior 4 — stack 3 returns base + 2*per_level.
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
    fn splits_stack_three_with_zero_base_still_grows() {
        let cfg = FractureConfig {
            base_splits:      0,
            per_level_splits: 2,
        };
        // 0 + 2 * 2 = 4
        assert_eq!(cfg.splits_for(3), 4);
    }

    // Behavior 5 — stack 10 capped at 4.
    #[test]
    fn splits_are_capped_at_offset_count() {
        let cfg = FractureConfig {
            base_splits:      2,
            per_level_splits: 1,
        };
        // Stack 10 would be 11 but we have only 4 offsets.
        assert_eq!(cfg.splits_for(10), 4);
    }

    #[test]
    fn splits_constant_when_per_level_is_zero() {
        let cfg = FractureConfig {
            base_splits:      2,
            per_level_splits: 0,
        };
        // per_level=0 → constant at base_splits across all stacks ≥ 1; cap not engaged.
        assert_eq!(cfg.splits_for(10), 2);
    }

    // Behavior 6 — base=0, per_level=0 → 0 at all stacks.
    #[test]
    fn splits_zero_config_is_zero_at_all_stacks() {
        let cfg = FractureConfig {
            base_splits:      0,
            per_level_splits: 0,
        };
        for k in [0u32, 1, 2, 3, 5, 10] {
            assert_eq!(cfg.splits_for(k), 0, "k={k}");
        }
    }

    #[test]
    fn splits_zero_base_with_large_per_level_stack_one_is_zero_stack_two_caps() {
        let cfg = FractureConfig {
            base_splits:      0,
            per_level_splits: 100,
        };
        // Stack 1: 0 + 100 * 0 = 0 (extra = 0 short-circuits).
        assert_eq!(cfg.splits_for(1), 0);
        // Stack 2: 0 + 100 * 1 = 100, cap at 4.
        assert_eq!(cfg.splits_for(2), 4);
    }

    // Behavior 7 — base=4 stack=1 at cap boundary (not over).
    #[test]
    fn splits_base_four_stack_one_is_exactly_four() {
        let cfg = FractureConfig {
            base_splits:      4,
            per_level_splits: 0,
        };
        // raw == 4: does NOT trigger `raw > 4` (strict inequality).
        assert_eq!(cfg.splits_for(1), 4);
    }

    #[test]
    fn splits_base_five_stack_one_is_clamped_to_four() {
        let cfg = FractureConfig {
            base_splits:      5,
            per_level_splits: 0,
        };
        // raw == 5: triggers `raw > 4` cap.
        assert_eq!(cfg.splits_for(1), 4);
    }

    // Behavior 8 — u32::MAX overflow safety.
    #[test]
    fn splits_overflow_saturates_and_caps_at_four() {
        let cfg = FractureConfig {
            base_splits:      u32::MAX,
            per_level_splits: u32::MAX,
        };
        assert_eq!(cfg.splits_for(u32::MAX), 4);
    }

    #[test]
    fn splits_saturating_add_never_wraps_under_cap() {
        let cfg = FractureConfig {
            base_splits:      u32::MAX,
            per_level_splits: 1,
        };
        // Stack 2: extra = 1; mul = 1; add saturates to u32::MAX; cap to 4.
        assert_eq!(cfg.splits_for(2), 4);
    }

    // Behavior 9 — stack 4 raw=5 clamped to 4.
    #[test]
    fn splits_stack_four_canonical_clamps_to_four() {
        let cfg = canonical_config();
        // 2 + 1 * 3 = 5, clamped to 4.
        assert_eq!(cfg.splits_for(4), 4);
    }

    #[test]
    fn splits_base_three_per_level_one_stack_two_is_exactly_four() {
        let cfg = FractureConfig {
            base_splits:      3,
            per_level_splits: 1,
        };
        // 3 + 1 * 1 = 4 (NOT clamped — strict `>` means raw=4 passes).
        assert_eq!(cfg.splits_for(2), 4);
    }

    // ── fracture_on_death ────────────────────────────────────────────────

    // Behavior 10 — stack 1 spawns 2 debris at ±(70,0).
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
    fn stack_one_spawns_two_debris_at_negative_fractional_victim() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 1);

        let victim = Vec2::new(-42.5, 17.25);
        write_cell_destroyed(&mut app, victim);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
        let offsets: Vec<Vec2> = query
            .iter(app.world())
            .map(|(_, pos)| pos.0 - victim)
            .collect();
        assert_eq!(offsets.len(), 2);
        for offset in &offsets {
            assert!(
                approx_eq_vec2(*offset, Vec2::new(70.0, 0.0), 1e-4)
                    || approx_eq_vec2(*offset, Vec2::new(-70.0, 0.0), 1e-4),
                "unexpected offset {offset:?}"
            );
        }
    }

    // Behavior 11 — stack 2 spawns 3 debris (right, left, up).
    #[test]
    fn stack_two_spawns_three_debris_right_left_up() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 2);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
        let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
        assert_eq!(positions.len(), 3);

        let expected = [
            Vec2::new(70.0, 0.0),
            Vec2::new(-70.0, 0.0),
            Vec2::new(0.0, 24.0),
        ];
        for exp in &expected {
            assert!(
                positions.iter().any(|p| approx_eq_vec2(*p, *exp, 1e-4)),
                "expected offset {exp:?} missing"
            );
        }
        // DOWN offset must NOT be present at stack 2.
        let down = Vec2::new(0.0, -24.0);
        assert!(
            !positions.iter().any(|p| approx_eq_vec2(*p, down, 1e-4)),
            "down offset should not be present at stack 2"
        );
    }

    #[test]
    fn three_base_splits_at_stack_one_also_fills_first_three_offsets() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(
            &mut app,
            FractureConfig {
                base_splits:      3,
                per_level_splits: 0,
            },
        );
        add_fracture_stacks(&mut app, 1);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
        let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
        assert_eq!(positions.len(), 3);

        let expected = [
            Vec2::new(70.0, 0.0),
            Vec2::new(-70.0, 0.0),
            Vec2::new(0.0, 24.0),
        ];
        for exp in &expected {
            assert!(
                positions.iter().any(|p| approx_eq_vec2(*p, *exp, 1e-4)),
                "expected offset {exp:?} missing"
            );
        }
    }

    // Behavior 12 — stack 3 spawns 4 debris at all orthogonal offsets.
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
    fn stack_three_fills_all_four_orthogonal_offsets() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 3);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
        let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
        assert_eq!(positions.len(), 4);

        let expected = [
            Vec2::new(70.0, 0.0),
            Vec2::new(-70.0, 0.0),
            Vec2::new(0.0, 24.0),
            Vec2::new(0.0, -24.0),
        ];
        for exp in &expected {
            assert!(
                positions.iter().any(|p| approx_eq_vec2(*p, *exp, 1e-4)),
                "expected offset {exp:?} missing"
            );
        }
    }

    // Behavior 13 — stack 10 also spawns 4 (cap enforcement at spawn time).
    #[test]
    fn stack_ten_spawns_four_debris_all_offsets() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 10);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
        let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
        assert_eq!(positions.len(), 4);

        let expected = [
            Vec2::new(70.0, 0.0),
            Vec2::new(-70.0, 0.0),
            Vec2::new(0.0, 24.0),
            Vec2::new(0.0, -24.0),
        ];
        for exp in &expected {
            assert!(
                positions.iter().any(|p| approx_eq_vec2(*p, *exp, 1e-4)),
                "expected offset {exp:?} missing"
            );
        }
    }

    #[test]
    fn stack_one_hundred_still_caps_at_four_debris() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 100);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 4);
    }

    // Behavior 14 — debris carries full collision suite.
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
    fn debris_collision_suite_includes_hp_and_killed_by() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 1);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(
            &Cell,
            &CellWidth,
            &CellHeight,
            &Aabb2D,
            &CollisionLayers,
            &Hp,
            &KilledBy,
        )>();
        assert_eq!(query.iter(app.world()).count(), 2);
    }

    // Behavior 14b — debris CellWidth/CellHeight pinned to concrete values.
    #[test]
    fn debris_cell_width_and_height_are_seventy_and_twenty_four() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 1);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app
            .world_mut()
            .query::<(&FractureDebris, &CellWidth, &CellHeight)>();
        let dims: Vec<(f32, f32)> = query
            .iter(app.world())
            .map(|(_, cw, ch)| (cw.value, ch.value))
            .collect();
        assert_eq!(dims.len(), 2);
        for (w, h) in &dims {
            assert!((w - 70.0).abs() < f32::EPSILON, "width = {w}");
            assert!((h - 24.0).abs() < f32::EPSILON, "height = {h}");
        }
    }

    #[test]
    fn debris_scale_matches_pristine_cell_footprint() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 1);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(&FractureDebris, &Scale2D)>();
        let scales: Vec<Scale2D> = query.iter(app.world()).map(|(_, s)| *s).collect();
        assert_eq!(scales.len(), 2);
        for scale in &scales {
            assert!(
                (scale.x - 70.0).abs() < f32::EPSILON,
                "scale.x = {}",
                scale.x
            );
            assert!(
                (scale.y - 24.0).abs() < f32::EPSILON,
                "scale.y = {}",
                scale.y
            );
        }
    }

    // Behavior 15 — FractureDebris marker is present on debris and only on debris.
    #[test]
    fn debris_carries_fracture_debris_marker_and_cell() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 1);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(&Cell, &FractureDebris)>();
        assert_eq!(query.iter(app.world()).count(), 2);
    }

    #[test]
    fn fracture_debris_marker_does_not_attach_to_plain_cells() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 1);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        // Initial debris count is 2.
        {
            let mut query = app.world_mut().query::<&FractureDebris>();
            assert_eq!(query.iter(app.world()).count(), 2);
        }

        // Spawn a plain Cell with no FractureDebris marker.
        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(500.0, 500.0))));
        app.update();

        // Still only 2 FractureDebris entities.
        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 2);
    }

    // Behavior 16 — debris HP = 1 regardless of stack count.
    #[test]
    fn debris_hp_is_one_at_stack_three() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 3);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(&FractureDebris, &Hp)>();
        let hps: Vec<f32> = query.iter(app.world()).map(|(_, hp)| hp.current).collect();
        assert_eq!(hps.len(), 4);
        for hp in &hps {
            assert!((hp - 1.0).abs() < f32::EPSILON, "hp = {hp}");
        }
    }

    #[test]
    fn debris_hp_is_one_at_stack_ten_cap() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 10);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(&FractureDebris, &Hp)>();
        let hps: Vec<f32> = query.iter(app.world()).map(|(_, hp)| hp.current).collect();
        assert_eq!(hps.len(), 4);
        for hp in &hps {
            assert!((hp - 1.0).abs() < f32::EPSILON, "hp = {hp}");
        }
    }

    // Behavior 17 — CleanupOnExit<NodeState> forward-compatibility regression pin.
    #[test]
    fn debris_carries_cleanup_on_exit_node_state_at_stack_one() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 1);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app
            .world_mut()
            .query::<(&FractureDebris, &CleanupOnExit<NodeState>)>();
        assert_eq!(query.iter(app.world()).count(), 2);
    }

    #[test]
    fn debris_carries_cleanup_on_exit_node_state_at_stack_three() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 3);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app
            .world_mut()
            .query::<(&FractureDebris, &CleanupOnExit<NodeState>)>();
        assert_eq!(query.iter(app.world()).count(), 4);
    }

    // Behavior 18 — boundary-magnitude victim positions stay finite.
    #[test]
    fn boundary_magnitude_victim_position_produces_finite_debris() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 3);

        let victim = Vec2::new(1e6, -1e6);
        write_cell_destroyed(&mut app, victim);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
        let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
        assert_eq!(positions.len(), 4);
        for pos in &positions {
            assert!(pos.x.is_finite(), "pos.x not finite: {pos:?}");
            assert!(pos.y.is_finite(), "pos.y not finite: {pos:?}");
        }

        let relative: Vec<Vec2> = positions.iter().map(|p| *p - victim).collect();
        let expected = [
            Vec2::new(70.0, 0.0),
            Vec2::new(-70.0, 0.0),
            Vec2::new(0.0, 24.0),
            Vec2::new(0.0, -24.0),
        ];
        for exp in &expected {
            assert!(
                relative.iter().any(|r| approx_eq_vec2(*r, *exp, 1e-1)),
                "expected relative offset {exp:?} missing"
            );
        }
    }

    #[test]
    fn boundary_magnitude_victim_position_sign_symmetric() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 3);

        let victim = Vec2::new(-1e6, 1e6);
        write_cell_destroyed(&mut app, victim);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
        let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
        assert_eq!(positions.len(), 4);
        for pos in &positions {
            assert!(pos.x.is_finite());
            assert!(pos.y.is_finite());
        }

        let relative: Vec<Vec2> = positions.iter().map(|p| *p - victim).collect();
        let expected = [
            Vec2::new(70.0, 0.0),
            Vec2::new(-70.0, 0.0),
            Vec2::new(0.0, 24.0),
            Vec2::new(0.0, -24.0),
        ];
        for exp in &expected {
            assert!(
                relative.iter().any(|r| approx_eq_vec2(*r, *exp, 1e-1)),
                "expected relative offset {exp:?} missing"
            );
        }
    }

    // Behavior 19 — no config → no debris; reader cleared.
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
    fn no_config_reader_drains_and_second_tick_with_config_does_not_retroactively_spawn() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        add_fracture_stacks(&mut app, 1);

        // First tick: no config → reader.clear() drains the message.
        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        // Install config and tick again WITHOUT writing a new message.
        install_fracture_config(&mut app, canonical_config());
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    // Behavior 20 — zero stacks → no debris; reader cleared.
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

    #[test]
    fn zero_stacks_reader_drains_and_stack_added_mid_way_does_not_retroactively_spawn() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());

        // First tick: zero stacks → reader.clear() drains the message.
        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        // Add a stack; tick again without a new message.
        add_fracture_stacks(&mut app, 1);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    // Behavior 21 — multiple deaths each spawn independently.
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
    fn three_deaths_at_stack_two_spawn_nine_debris_with_per_victim_offsets() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 2);

        let victims = [
            Vec2::new(10.0, 20.0),
            Vec2::new(100.0, 200.0),
            Vec2::new(-50.0, 0.0),
        ];
        for v in &victims {
            write_cell_destroyed(&mut app, *v);
        }
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
        let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
        assert_eq!(positions.len(), 9);

        let expected_offsets = [
            Vec2::new(70.0, 0.0),
            Vec2::new(-70.0, 0.0),
            Vec2::new(0.0, 24.0),
        ];
        for victim in &victims {
            for offset in &expected_offsets {
                let expected_pos = *victim + *offset;
                assert!(
                    positions
                        .iter()
                        .any(|p| approx_eq_vec2(*p, expected_pos, 1e-4)),
                    "missing debris at {expected_pos:?} (victim {victim:?} + offset {offset:?})"
                );
            }
        }
    }

    // Behavior 22 — recursive fracture pin: destroying a FractureDebris cell also spawns.
    #[test]
    fn destroying_a_fracture_debris_cell_also_spawns_new_debris() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 1);

        // Seed an existing debris-marked entity as if it were already in the world.
        let victim = Vec2::new(200.0, 200.0);
        app.world_mut()
            .spawn((Cell, FractureDebris, Position2D(victim)));

        write_cell_destroyed(&mut app, victim);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
        let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
        assert_eq!(positions.len(), 3);

        // Two NEW debris at the death position ± (70, 0).
        let expected_right = victim + Vec2::new(70.0, 0.0);
        let expected_left = victim + Vec2::new(-70.0, 0.0);
        assert!(
            positions
                .iter()
                .any(|p| approx_eq_vec2(*p, expected_right, 1e-4)),
            "missing right-offset debris"
        );
        assert!(
            positions
                .iter()
                .any(|p| approx_eq_vec2(*p, expected_left, 1e-4)),
            "missing left-offset debris"
        );
        // Seeded debris still present at victim position.
        assert!(
            positions.iter().any(|p| approx_eq_vec2(*p, victim, 1e-4)),
            "original debris should still be present"
        );
    }

    #[test]
    fn recursive_fracture_scales_with_stack_count() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 3);

        let victim = Vec2::new(200.0, 200.0);
        app.world_mut()
            .spawn((Cell, FractureDebris, Position2D(victim)));

        write_cell_destroyed(&mut app, victim);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 5);
    }

    // Behavior 23 — duplicate victim positions produce independent debris entities.
    #[test]
    fn duplicate_victim_positions_produce_independent_debris_entities() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 1);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        write_cell_destroyed(&mut app, Vec2::ZERO);
        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 6);
    }

    #[test]
    fn duplicate_victim_positions_produce_six_distinct_entities() {
        let mut app = test_app_playing();
        app.add_systems(FixedUpdate, fracture_on_death);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 1);

        write_cell_destroyed(&mut app, Vec2::ZERO);
        write_cell_destroyed(&mut app, Vec2::ZERO);
        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<FractureDebris>>();
        let entities: HashSet<Entity> = query.iter(app.world()).collect();
        assert_eq!(entities.len(), 6);
    }

    // ── register integration ────────────────────────────────────────────

    // Behavior 24 — full register path spawns debris under active gate.
    #[test]
    fn register_with_config_and_stack_spawns_debris_on_death() {
        let mut app = test_app_playing();
        register(&mut app);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 1);

        write_cell_destroyed(&mut app, Vec2::new(50.0, 50.0));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
        let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
        assert_eq!(positions.len(), 2);

        let expected = [Vec2::new(120.0, 50.0), Vec2::new(-20.0, 50.0)];
        for exp in &expected {
            assert!(
                positions.iter().any(|p| approx_eq_vec2(*p, *exp, 1e-4)),
                "missing expected position {exp:?}"
            );
        }
    }

    #[test]
    fn register_second_tick_without_message_spawns_no_new_debris() {
        let mut app = test_app_playing();
        register(&mut app);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 1);

        write_cell_destroyed(&mut app, Vec2::new(50.0, 50.0));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
        // Second tick, no new message.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 2);
    }

    // Behavior 25 — gate off (stacks = 0) → no debris; gate reopens cleanly.
    #[test]
    fn register_gate_off_stacks_zero_does_not_spawn() {
        let mut app = test_app_playing();
        register(&mut app);
        install_fracture_config(&mut app, canonical_config());
        // NO stacks added — gate off.

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    #[test]
    fn register_gate_reopens_when_stack_added_spawns_post_open_messages() {
        let mut app = test_app_playing();
        register(&mut app);
        install_fracture_config(&mut app, canonical_config());

        // First tick: gate off, no message written.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        // Open the gate, THEN write the message.
        add_fracture_stacks(&mut app, 1);
        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 2);
    }

    #[test]
    fn register_pregate_messages_accumulate_until_gate_opens() {
        // Pin Bevy MessageReader semantics under a run_if gate: when the
        // gate is closed, the system does not consume messages; the
        // reader's cursor stays put. Opening the gate on a later tick lets
        // the system read ALL unread messages — including the pre-gate
        // ones. Shared pattern with overcharge / drift / gravity_surge.
        //
        // Setup: gate starts CLOSED (no Fracture stack). Write one
        // Destroyed<Cell>. Tick (gate off → no-op). Toggle gate ON
        // without writing a new message. Tick again. The pre-gate
        // message must now be consumed → 2 debris spawn.
        let mut app = test_app_playing();
        register(&mut app);
        install_fracture_config(&mut app, canonical_config());

        // Tick 1 — gate off, pre-gate death written.
        write_cell_destroyed(&mut app, Vec2::ZERO);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
        let mut q = app.world_mut().query::<&FractureDebris>();
        assert_eq!(
            q.iter(app.world()).count(),
            0,
            "gate closed → no debris this tick"
        );

        // Tick 2 — open gate, write no new message. Pre-gate message
        // retained by Bevy's double-buffered Messages<T> is consumed now.
        add_fracture_stacks(&mut app, 1);
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(
            query.iter(app.world()).count(),
            2,
            "gate open → retained pre-gate message consumed → 2 debris"
        );
    }

    // Behavior 26 — gate off (NodeState != Playing) → no debris.
    #[test]
    fn register_gate_off_state_not_playing_does_not_spawn() {
        let mut app = test_app_not_playing();
        register(&mut app);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 1);
        // Do NOT write message — gate is off, reader won't drain.

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    #[test]
    fn register_positive_control_state_playing_does_spawn() {
        // Paired positive control: same setup as the gate-off test except for
        // test_app_playing() vs test_app_not_playing(). Divergence in this
        // pair proves the NodeState gate is the discriminator.
        let mut app = test_app_playing();
        register(&mut app);
        install_fracture_config(&mut app, canonical_config());
        add_fracture_stacks(&mut app, 1);
        write_cell_destroyed(&mut app, Vec2::ZERO);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 2);
    }

    // Behavior 27 — gate toggles open mid-run.
    #[test]
    fn register_gate_toggles_open_mid_run_processes_fresh_stacks() {
        let mut app = test_app_playing();
        register(&mut app);
        install_fracture_config(&mut app, canonical_config());
        // Gate off initially (no stacks).

        // First tick: gate off, no processing.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        // Open the gate with stack=2, then write the message.
        add_fracture_stacks(&mut app, 2);
        write_cell_destroyed(&mut app, Vec2::new(10.0, 20.0));

        // Second tick: gate open, stack=2 → count=3.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
        let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
        assert_eq!(positions.len(), 3);

        let victim = Vec2::new(10.0, 20.0);
        let expected = [
            victim + Vec2::new(70.0, 0.0),
            victim + Vec2::new(-70.0, 0.0),
            victim + Vec2::new(0.0, 24.0),
        ];
        for exp in &expected {
            assert!(
                positions.iter().any(|p| approx_eq_vec2(*p, *exp, 1e-4)),
                "missing expected position {exp:?}"
            );
        }
    }

    #[test]
    fn register_gate_open_third_tick_without_message_spawns_nothing_new() {
        let mut app = test_app_playing();
        register(&mut app);
        install_fracture_config(&mut app, canonical_config());

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        add_fracture_stacks(&mut app, 2);
        write_cell_destroyed(&mut app, Vec2::new(10.0, 20.0));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        // Third tick, no new message.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 3);
    }

    // ── activate — preserved + new ────────────────────────────────────────

    // Behavior 28 — matching tuning inserts config.
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
    fn activate_now_with_matching_tuning_pins_both_fields() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Fracture {
                base_splits:      2,
                per_level_splits: 1,
            },
        );

        let cfg = app.world().resource::<FractureConfig>();
        assert_eq!(cfg.base_splits, 2);
        assert_eq!(cfg.per_level_splits, 1);
    }

    // Behavior 29 — mismatched Decay tuning inserts nothing.
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

    #[test]
    fn activate_now_with_mismatched_decay_nan_does_not_panic() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Decay {
                base_percent:      f32::NAN,
                per_level_percent: f32::NAN,
            },
        );
        assert!(app.world().get_resource::<FractureConfig>().is_none());
    }

    // Behavior 30 — mismatched Haste tuning inserts nothing.
    #[test]
    fn activate_now_with_mismatched_haste_does_nothing() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Haste {
                base_percent:      0.1,
                per_level_percent: 0.05,
            },
        );
        assert!(app.world().get_resource::<FractureConfig>().is_none());
    }

    #[test]
    fn activate_now_with_mismatched_drift_does_nothing() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Drift {
                force:           100.0,
                period_secs:     8.0,
                per_level_force: 33.3,
            },
        );
        assert!(app.world().get_resource::<FractureConfig>().is_none());
    }

    // Behavior 31 — mismatched Overcharge tuning inserts nothing.
    #[test]
    fn activate_now_with_mismatched_overcharge_does_nothing() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Overcharge {
                base_frac:      0.05,
                per_level_frac: 0.03,
            },
        );
        assert!(app.world().get_resource::<FractureConfig>().is_none());
    }

    #[test]
    fn activate_now_with_mismatched_cascade_does_nothing() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Cascade {
                base_heal:      1.0,
                per_level_heal: 0.5,
            },
        );
        assert!(app.world().get_resource::<FractureConfig>().is_none());
    }

    // Behavior 32 — second activate overwrites (last-write-wins).
    #[test]
    fn second_activate_overwrites_fracture_config() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Fracture {
                base_splits:      1,
                per_level_splits: 0,
            },
        );
        activate_now(
            &mut app,
            &HazardTuning::Fracture {
                base_splits:      4,
                per_level_splits: 2,
            },
        );

        let cfg = app.world().resource::<FractureConfig>();
        assert_eq!(cfg.base_splits, 4);
        assert_eq!(cfg.per_level_splits, 2);
    }

    #[test]
    fn third_activate_overwrites_to_zero_values() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Fracture {
                base_splits:      1,
                per_level_splits: 0,
            },
        );
        activate_now(
            &mut app,
            &HazardTuning::Fracture {
                base_splits:      4,
                per_level_splits: 2,
            },
        );
        activate_now(
            &mut app,
            &HazardTuning::Fracture {
                base_splits:      0,
                per_level_splits: 0,
            },
        );

        let cfg = app.world().resource::<FractureConfig>();
        assert_eq!(cfg.base_splits, 0);
        assert_eq!(cfg.per_level_splits, 0);
    }

    // Behavior 33 — activate on empty world does not panic.
    #[test]
    fn activate_now_on_empty_world_does_not_panic() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Fracture {
                base_splits:      2,
                per_level_splits: 1,
            },
        );

        let cfg = app.world().resource::<FractureConfig>();
        assert_eq!(cfg.base_splits, 2);
        assert_eq!(cfg.per_level_splits, 1);
    }

    #[test]
    fn activate_now_mismatch_after_match_preserves_existing_config() {
        let mut app = TestAppBuilder::new().build();
        activate_now(
            &mut app,
            &HazardTuning::Fracture {
                base_splits:      2,
                per_level_splits: 1,
            },
        );
        activate_now(
            &mut app,
            &HazardTuning::Decay {
                base_percent:      0.05,
                per_level_percent: 0.03,
            },
        );

        let cfg = app.world().resource::<FractureConfig>();
        assert_eq!(cfg.base_splits, 2);
        assert_eq!(cfg.per_level_splits, 1);
    }
}
