//! Tether beam systems — tick damage, cleanup dead targets.

use bevy::{ecs::entity::Entities, prelude::*};
use rantzsoft_spatial2d::components::Position2D;

use super::components::*;
use crate::{
    cells::components::Cell,
    effect_v3::components::EffectSourceChip,
    shared::death_pipeline::{DamageDealt, Dead},
};

/// Alive cell lookup — entity + position, excludes dead cells.
type AliveCellQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Position2D), (With<Cell>, Without<Dead>)>;

/// Applies damage to cells intersecting the beam line between the two bolts.
pub(crate) fn tick_tether_beam(
    beam_query: Query<(
        Entity,
        &TetherBeamSource,
        &TetherBeamDamage,
        &TetherBeamWidth,
        Option<&EffectSourceChip>,
    )>,
    position_query: Query<&Position2D>,
    cell_query: AliveCellQuery,
    mut damage_writer: MessageWriter<DamageDealt<Cell>>,
) {
    for (beam_entity, source, beam_damage, &TetherBeamWidth(beam_width), chip) in &beam_query {
        let pos_a = position_query.get(source.bolt_a).map(|p| p.0);
        let pos_b = position_query.get(source.bolt_b).map(|p| p.0);

        let (Ok(a), Ok(b)) = (pos_a, pos_b) else {
            continue;
        };

        let beam_dir = b - a;
        let beam_len = beam_dir.length();
        if beam_len < f32::EPSILON {
            continue;
        }
        let beam_norm = beam_dir / beam_len;
        let perp = Vec2::new(-beam_norm.y, beam_norm.x);

        for (cell_entity, cell_pos) in &cell_query {
            let offset = cell_pos.0 - a;
            let along = offset.dot(beam_norm);
            let across = offset.dot(perp).abs();

            if along >= 0.0 && along <= beam_len && across <= beam_width {
                damage_writer.write(DamageDealt {
                    dealer:      Some(beam_entity),
                    target:      cell_entity,
                    amount:      beam_damage.0,
                    source_chip: chip.and_then(|c| c.0.clone()),
                    _marker:     std::marker::PhantomData,
                });
            }
        }
    }
}

/// Removes tether beam entities when either endpoint bolt no longer exists.
pub fn cleanup_tether_beams(
    beam_query: Query<(Entity, &TetherBeamSource)>,
    entities: &Entities,
    mut commands: Commands,
) {
    for (beam_entity, source) in &beam_query {
        if !entities.contains(source.bolt_a) || !entities.contains(source.bolt_b) {
            commands.entity(beam_entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use rantzsoft_spatial2d::components::Position2D;

    use super::*;
    use crate::{
        cells::components::Cell,
        effect_v3::components::EffectSourceChip,
        shared::{
            death_pipeline::{DamageDealt, Dead},
            test_utils::{MessageCollector, TestAppBuilder, tick},
        },
    };

    // ── Test app helpers ───────────────────────────────────────────────────

    fn tether_test_app() -> App {
        TestAppBuilder::new()
            .with_message_capture::<DamageDealt<Cell>>()
            .with_system(FixedUpdate, tick_tether_beam)
            .build()
    }

    fn cleanup_test_app() -> App {
        TestAppBuilder::new()
            .with_system(FixedUpdate, cleanup_tether_beams)
            .build()
    }

    /// Spawns a placeholder "bolt endpoint" — the tick system only reads
    /// `Position2D` from the entities stored in `TetherBeamSource`, so no
    /// `Bolt` marker is required.
    fn spawn_endpoint(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut().spawn(Position2D(pos)).id()
    }

    fn spawn_alive_cell(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut().spawn((Cell, Position2D(pos))).id()
    }

    fn spawn_dead_cell(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut().spawn((Cell, Position2D(pos), Dead)).id()
    }

    fn damage_msgs(app: &App) -> Vec<DamageDealt<Cell>> {
        app.world()
            .resource::<MessageCollector<DamageDealt<Cell>>>()
            .0
            .clone()
    }

    // ── Group A — tick_tether_beam geometry ────────────────────────────────

    #[test]
    fn cell_on_beam_line_is_damaged() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let cell_entity = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));

        let beam_entity = app
            .world_mut()
            .spawn((
                TetherBeamSource { bolt_a, bolt_b },
                TetherBeamDamage(12.5),
                TetherBeamWidth(10.0),
            ))
            .id();

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(
            msgs.len(),
            1,
            "expected exactly 1 DamageDealt<Cell> message"
        );
        assert_eq!(msgs[0].target, cell_entity);
        assert!((msgs[0].amount - 12.5).abs() < 1e-6);
        assert_eq!(msgs[0].dealer, Some(beam_entity));
    }

    #[test]
    fn cell_beyond_bolt_b_is_not_damaged() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let _cell = spawn_alive_cell(&mut app, Vec2::new(150.0, 0.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        tick(&mut app);

        assert_eq!(damage_msgs(&app).len(), 0);
    }

    #[test]
    fn cell_behind_bolt_a_is_not_damaged() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let _cell = spawn_alive_cell(&mut app, Vec2::new(-10.0, 0.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        tick(&mut app);

        assert_eq!(damage_msgs(&app).len(), 0);
    }

    #[test]
    fn cell_at_bolt_a_position_is_damaged() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let cell_entity = spawn_alive_cell(&mut app, Vec2::new(0.0, 0.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].target, cell_entity);
    }

    #[test]
    fn cell_at_beam_len_boundary_is_damaged() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let cell_entity = spawn_alive_cell(&mut app, Vec2::new(100.0, 0.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].target, cell_entity);
    }

    #[test]
    fn cell_within_half_width_is_damaged_both_sides() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let positive_cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 9.0));
        let negative_cell = spawn_alive_cell(&mut app, Vec2::new(50.0, -9.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(msgs.len(), 2, "both symmetric cells must be damaged");
        let targets: Vec<Entity> = msgs.iter().map(|m| m.target).collect();
        assert!(targets.contains(&positive_cell));
        assert!(targets.contains(&negative_cell));
    }

    #[test]
    fn cell_at_half_width_boundary_is_damaged_both_sides() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let positive_cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 10.0));
        let negative_cell = spawn_alive_cell(&mut app, Vec2::new(50.0, -10.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(msgs.len(), 2);
        let targets: Vec<Entity> = msgs.iter().map(|m| m.target).collect();
        assert!(targets.contains(&positive_cell));
        assert!(targets.contains(&negative_cell));
    }

    #[test]
    fn cell_outside_half_width_is_not_damaged_both_sides() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let _positive_cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 11.0));
        let _negative_cell = spawn_alive_cell(&mut app, Vec2::new(50.0, -11.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        tick(&mut app);

        assert_eq!(damage_msgs(&app).len(), 0);
    }

    #[test]
    fn dead_cells_are_never_damaged() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let _dead_cell = spawn_dead_cell(&mut app, Vec2::new(50.0, 0.0));
        let alive_cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 5.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].target, alive_cell);
    }

    #[test]
    fn despawned_bolt_a_produces_no_damage_and_no_panic() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let _cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        app.world_mut().despawn(bolt_a);

        tick(&mut app);

        assert_eq!(damage_msgs(&app).len(), 0);
        let beam_count = app
            .world_mut()
            .query::<&TetherBeamSource>()
            .iter(app.world())
            .count();
        assert_eq!(
            beam_count, 1,
            "tick must not despawn the beam when an endpoint is missing"
        );
    }

    #[test]
    fn despawned_bolt_b_produces_no_damage_and_no_panic() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let _cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        app.world_mut().despawn(bolt_b);

        tick(&mut app);

        assert_eq!(damage_msgs(&app).len(), 0);
        let beam_count = app
            .world_mut()
            .query::<&TetherBeamSource>()
            .iter(app.world())
            .count();
        assert_eq!(beam_count, 1);
    }

    #[test]
    fn zero_length_beam_produces_no_damage() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let _cell_at_origin = spawn_alive_cell(&mut app, Vec2::new(0.0, 0.0));
        let _cell_nearby = spawn_alive_cell(&mut app, Vec2::new(5.0, 0.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        tick(&mut app);

        assert_eq!(damage_msgs(&app).len(), 0);
    }

    #[test]
    fn damage_amount_equals_tether_beam_damage_value() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let _cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(msgs.len(), 1);
        assert!((msgs[0].amount - 12.5).abs() < 1e-6);
    }

    #[test]
    fn zero_damage_still_emits_message() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let _cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(0.0),
            TetherBeamWidth(10.0),
        ));

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].amount.abs() < 1e-6);
    }

    #[test]
    fn multiple_beams_each_emit_correctly_attributed_damage() {
        let mut app = tether_test_app();

        // Beam 1 — along y=0
        let beam1_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let beam1_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let _cell1 = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));
        let beam1_entity = app
            .world_mut()
            .spawn((
                TetherBeamSource {
                    bolt_a: beam1_a,
                    bolt_b: beam1_b,
                },
                TetherBeamDamage(10.0),
                TetherBeamWidth(10.0),
                EffectSourceChip(None),
            ))
            .id();

        // Beam 2 — along y=50
        let beam2_a = spawn_endpoint(&mut app, Vec2::new(0.0, 50.0));
        let beam2_b = spawn_endpoint(&mut app, Vec2::new(100.0, 50.0));
        let _cell2 = spawn_alive_cell(&mut app, Vec2::new(50.0, 50.0));
        let beam2_entity = app
            .world_mut()
            .spawn((
                TetherBeamSource {
                    bolt_a: beam2_a,
                    bolt_b: beam2_b,
                },
                TetherBeamDamage(20.0),
                TetherBeamWidth(10.0),
                EffectSourceChip(None),
            ))
            .id();

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(msgs.len(), 2, "expected exactly 2 messages, one per beam");

        let beam1_msg = msgs
            .iter()
            .find(|m| m.dealer == Some(beam1_entity))
            .expect("beam1 damage message missing");
        assert!((beam1_msg.amount - 10.0).abs() < 1e-6);

        let beam2_msg = msgs
            .iter()
            .find(|m| m.dealer == Some(beam2_entity))
            .expect("beam2 damage message missing");
        assert!((beam2_msg.amount - 20.0).abs() < 1e-6);
    }

    // ── Group A (width) — TetherBeamWidth drives per-entity hit-test ───────

    /// Behavior 1: Narrow beam (width 5.0) damages only cells inside or on
    /// the 5.0 half-width boundary.
    #[test]
    fn narrow_beam_width_5_damages_only_cells_within_5_units_perpendicular() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let cell_on_line = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));
        let cell_inside = spawn_alive_cell(&mut app, Vec2::new(50.0, 4.0));
        let cell_boundary = spawn_alive_cell(&mut app, Vec2::new(50.0, 5.0));
        let _cell_outside = spawn_alive_cell(&mut app, Vec2::new(50.0, 6.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(5.0),
        ));

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(
            msgs.len(),
            3,
            "expected exactly 3 DamageDealt<Cell> messages (on_line, inside, boundary)"
        );
        let targets: Vec<Entity> = msgs.iter().map(|m| m.target).collect();
        assert!(
            targets.contains(&cell_on_line),
            "cell on beam line must be damaged"
        );
        assert!(
            targets.contains(&cell_inside),
            "cell at distance 4.0 must be damaged"
        );
        assert!(
            targets.contains(&cell_boundary),
            "cell exactly on half-width 5.0 must be damaged (inclusive boundary)"
        );
    }

    /// Behavior 2: Wide beam (width 20.0) damages cells out to 20.0 perp
    /// distance — locks that width is not stale 10.0 constant.
    #[test]
    fn wide_beam_width_20_damages_cells_out_to_20_units_perpendicular() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let cell_beyond_old_10 = spawn_alive_cell(&mut app, Vec2::new(50.0, 11.0));
        let cell_near_boundary = spawn_alive_cell(&mut app, Vec2::new(50.0, 19.0));
        let cell_boundary = spawn_alive_cell(&mut app, Vec2::new(50.0, 20.0));
        let _cell_outside = spawn_alive_cell(&mut app, Vec2::new(50.0, 21.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(20.0),
        ));

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(
            msgs.len(),
            3,
            "expected 3 damage messages for cells within half-width 20.0"
        );
        let targets: Vec<Entity> = msgs.iter().map(|m| m.target).collect();
        assert!(
            targets.contains(&cell_beyond_old_10),
            "cell at 11.0 must be damaged — proves width is NOT stale 10.0"
        );
        assert!(targets.contains(&cell_near_boundary));
        assert!(
            targets.contains(&cell_boundary),
            "cell exactly on half-width 20.0 must be damaged (inclusive boundary)"
        );
    }

    /// Behavior 3: Two beams in the same world read width per-entity.
    /// Regression lock: width must be stamped on the entity, not hardcoded.
    #[test]
    fn two_coexisting_beams_use_per_entity_widths() {
        let mut app = tether_test_app();

        // Beam 1: narrow (width 5.0) along y=0
        let beam1_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let beam1_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let cell_narrow_inside = spawn_alive_cell(&mut app, Vec2::new(50.0, 3.0));
        let _cell_narrow_outside = spawn_alive_cell(&mut app, Vec2::new(50.0, 8.0));
        let beam1_entity = app
            .world_mut()
            .spawn((
                TetherBeamSource {
                    bolt_a: beam1_a,
                    bolt_b: beam1_b,
                },
                TetherBeamDamage(10.0),
                TetherBeamWidth(5.0),
            ))
            .id();

        // Beam 2: wide (width 20.0) along y=200
        let beam2_a = spawn_endpoint(&mut app, Vec2::new(0.0, 200.0));
        let beam2_b = spawn_endpoint(&mut app, Vec2::new(100.0, 200.0));
        let cell_wide_inside = spawn_alive_cell(&mut app, Vec2::new(50.0, 215.0));
        let _cell_wide_outside = spawn_alive_cell(&mut app, Vec2::new(50.0, 225.0));
        let beam2_entity = app
            .world_mut()
            .spawn((
                TetherBeamSource {
                    bolt_a: beam2_a,
                    bolt_b: beam2_b,
                },
                TetherBeamDamage(20.0),
                TetherBeamWidth(20.0),
            ))
            .id();

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(
            msgs.len(),
            2,
            "expected exactly 2 damage messages (one per beam, inside-cells only)"
        );

        let beam1_msg = msgs
            .iter()
            .find(|m| m.target == cell_narrow_inside)
            .expect("narrow-inside cell must be damaged by beam 1");
        assert_eq!(beam1_msg.dealer, Some(beam1_entity));
        assert!((beam1_msg.amount - 10.0).abs() < 1e-6);

        let beam2_msg = msgs
            .iter()
            .find(|m| m.target == cell_wide_inside)
            .expect("wide-inside cell must be damaged by beam 2");
        assert_eq!(beam2_msg.dealer, Some(beam2_entity));
        assert!((beam2_msg.amount - 20.0).abs() < 1e-6);
    }

    /// Behavior 4: Width 0.0 damages only cells exactly on the beam line.
    #[test]
    fn width_zero_damages_only_cells_exactly_on_beam_line() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let cell_on_line = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));
        let _cell_just_off = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.001));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(0.0),
        ));

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(
            msgs.len(),
            1,
            "with width 0.0, only the cell exactly on the line may be damaged"
        );
        assert_eq!(msgs[0].target, cell_on_line);
    }

    /// Behavior 5: Diagonal beam with width 5.0 — perpendicular distance
    /// from (60, 40) to line y=x is ≈14.14, which is outside half-width 5.0.
    #[test]
    fn diagonal_beam_width_5_does_not_damage_cell_at_14_14_perpendicular() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 100.0));
        let _cell = spawn_alive_cell(&mut app, Vec2::new(60.0, 40.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(5.0),
        ));

        tick(&mut app);

        assert_eq!(
            damage_msgs(&app).len(),
            0,
            "perp distance ≈14.14 > half-width 5.0, must not be damaged"
        );
    }

    /// Behavior 6: Same diagonal geometry, width 20.0 — cell at ~14.14
    /// perpendicular distance IS damaged.
    #[test]
    fn diagonal_beam_width_20_damages_cell_at_14_14_perpendicular() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 100.0));
        let cell = spawn_alive_cell(&mut app, Vec2::new(60.0, 40.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(20.0),
        ));

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(
            msgs.len(),
            1,
            "perp distance ≈14.14 < half-width 20.0, must be damaged exactly once"
        );
        assert_eq!(msgs[0].target, cell);
    }

    /// Behavior 7: Width and damage are orthogonal — width does not clamp
    /// or scale damage.
    #[test]
    fn width_is_independent_of_damage_on_same_beam_entity() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 2.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(99.0),
            TetherBeamWidth(3.0),
        ));

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].target, cell);
        assert!(
            (msgs[0].amount - 99.0).abs() < 1e-6,
            "damage must be 99.0 — width 3.0 must not affect damage amount"
        );
    }

    /// Behavior 7a: Beam entity without `TetherBeamWidth` is silently
    /// skipped by the required-component query — not damaged with a fallback.
    #[test]
    fn beam_without_tether_beam_width_is_silently_skipped_by_query() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let _cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));

        // Intentionally spawn WITHOUT TetherBeamWidth — the tick query must
        // require &TetherBeamWidth and skip this beam entirely.
        let beam_entity = app
            .world_mut()
            .spawn((TetherBeamSource { bolt_a, bolt_b }, TetherBeamDamage(12.5)))
            .id();

        tick(&mut app);

        assert_eq!(
            damage_msgs(&app).len(),
            0,
            "beam without TetherBeamWidth must be skipped by the query — \
             this locks a required-component contract, not an Option fallback"
        );
        // The beam entity itself must NOT be despawned by tick (tick only
        // reads; cleanup is a separate system).
        assert!(
            app.world().get_entity(beam_entity).is_ok(),
            "tick_tether_beam must not despawn the beam"
        );
    }

    // ── Group B — tick_tether_beam source_chip propagation ─────────────────

    #[test]
    fn tether_beam_propagates_source_chip_some_in_damage_dealt() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let _cell_mid = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));
        let _cell_far = spawn_alive_cell(&mut app, Vec2::new(75.0, 0.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
            EffectSourceChip(Some("storm_coil".to_string())),
        ));

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(msgs.len(), 2, "expected 2 DamageDealt<Cell> messages");
        for msg in &msgs {
            assert_eq!(
                msg.source_chip,
                Some("storm_coil".to_string()),
                "all messages must carry Some(\"storm_coil\") source_chip, got {:?}",
                msg.source_chip,
            );
        }
    }

    #[test]
    fn tether_beam_propagates_source_chip_none_in_damage_dealt() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let _cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
            EffectSourceChip(None),
        ));

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].source_chip, None);
    }

    #[test]
    fn tether_beam_missing_source_chip_component_propagates_none() {
        let mut app = tether_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        let _cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));

        // Beam entity is spawned WITHOUT EffectSourceChip at all.
        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        tick(&mut app);

        let msgs = damage_msgs(&app);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].source_chip, None);
    }

    // ── Group C — cleanup_tether_beams despawn semantics ───────────────────

    #[test]
    fn cleanup_despawns_beam_when_bolt_a_is_gone() {
        let mut app = cleanup_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        app.world_mut().despawn(bolt_a);

        tick(&mut app);

        let beam_count = app
            .world_mut()
            .query::<&TetherBeamSource>()
            .iter(app.world())
            .count();
        assert_eq!(beam_count, 0);
    }

    #[test]
    fn cleanup_despawns_beam_when_bolt_b_is_gone() {
        let mut app = cleanup_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        app.world_mut().despawn(bolt_b);

        tick(&mut app);

        let beam_count = app
            .world_mut()
            .query::<&TetherBeamSource>()
            .iter(app.world())
            .count();
        assert_eq!(beam_count, 0);
    }

    #[test]
    fn cleanup_leaves_beam_intact_when_both_endpoints_alive() {
        let mut app = cleanup_test_app();

        // Beam 1
        let beam1_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let beam1_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
        app.world_mut().spawn((
            TetherBeamSource {
                bolt_a: beam1_a,
                bolt_b: beam1_b,
            },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        // Beam 2 — independent pair of alive bolts
        let beam2_a = spawn_endpoint(&mut app, Vec2::new(200.0, 0.0));
        let beam2_b = spawn_endpoint(&mut app, Vec2::new(300.0, 0.0));
        app.world_mut().spawn((
            TetherBeamSource {
                bolt_a: beam2_a,
                bolt_b: beam2_b,
            },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        tick(&mut app);

        let beam_count = app
            .world_mut()
            .query::<&TetherBeamSource>()
            .iter(app.world())
            .count();
        assert_eq!(beam_count, 2);

        // Verify the first beam still points at its original endpoints.
        let beams: Vec<TetherBeamSource> = app
            .world_mut()
            .query::<&TetherBeamSource>()
            .iter(app.world())
            .cloned()
            .collect();
        let has_beam1 = beams
            .iter()
            .any(|b| b.bolt_a == beam1_a && b.bolt_b == beam1_b);
        let has_beam2 = beams
            .iter()
            .any(|b| b.bolt_a == beam2_a && b.bolt_b == beam2_b);
        assert!(has_beam1, "beam 1 endpoint pair must still exist");
        assert!(has_beam2, "beam 2 endpoint pair must still exist");
    }

    #[test]
    fn cleanup_despawns_beam_when_both_endpoints_are_gone() {
        let mut app = cleanup_test_app();
        let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
        let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));

        app.world_mut().spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ));

        app.world_mut().despawn(bolt_a);
        app.world_mut().despawn(bolt_b);

        tick(&mut app);

        let beam_count = app
            .world_mut()
            .query::<&TetherBeamSource>()
            .iter(app.world())
            .count();
        assert_eq!(beam_count, 0);
    }
}
