//! Shockwave systems — tick expansion, damage application, despawn.

use bevy::prelude::*;
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

/// Shockwave tick + damage query.
type ShockwaveQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Position2D,
        &'static ShockwaveRadius,
        &'static ShockwaveBaseDamage,
        &'static ShockwaveDamageMultiplier,
        &'static mut ShockwaveDamaged,
        Option<&'static EffectSourceChip>,
    ),
>;

/// Expands shockwave radius each frame based on speed.
pub fn tick_shockwave(mut query: Query<(&mut ShockwaveRadius, &ShockwaveSpeed)>, time: Res<Time>) {
    let dt = time.delta_secs();
    for (mut radius, speed) in &mut query {
        radius.0 = speed.0.mul_add(dt, radius.0);
    }
}

/// Applies damage to cells within the expanding shockwave radius.
pub(crate) fn apply_shockwave_damage(
    mut shockwave_query: ShockwaveQuery,
    cell_query: AliveCellQuery,
    mut damage_writer: MessageWriter<DamageDealt<Cell>>,
) {
    for (sw_entity, sw_pos, sw_radius, base_dmg, dmg_mult, mut damaged, chip) in
        &mut shockwave_query
    {
        for (cell_entity, cell_pos) in &cell_query {
            if damaged.0.contains(&cell_entity) {
                continue;
            }
            let distance = sw_pos.0.distance(cell_pos.0);
            if distance <= sw_radius.0 {
                damaged.0.insert(cell_entity);
                let damage = base_dmg.0 * dmg_mult.0;
                damage_writer.write(DamageDealt {
                    dealer:      Some(sw_entity),
                    target:      cell_entity,
                    amount:      damage,
                    source_chip: chip.and_then(|c| c.0.clone()),
                    _marker:     std::marker::PhantomData,
                });
            }
        }
    }
}

/// Despawns shockwaves that have reached their maximum radius.
pub fn despawn_finished_shockwave(
    query: Query<(Entity, &ShockwaveRadius, &ShockwaveMaxRadius)>,
    mut commands: Commands,
) {
    for (entity, radius, max_radius) in &query {
        if radius.0 >= max_radius.0 {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, time::Duration};

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

    // ── Helpers ────────────────────────────────────────────────────────────

    fn damage_test_app() -> App {
        TestAppBuilder::new()
            .with_message_capture::<DamageDealt<Cell>>()
            .with_system(FixedUpdate, apply_shockwave_damage)
            .build()
    }

    /// `tick_shockwave` uses `Res<Time>`, which only resolves to `Time<Fixed>`
    /// when the system is scheduled inside `FixedUpdate`. Registering it in
    /// `Update` would silently use a zero-delta virtual clock.
    fn tick_test_app() -> App {
        TestAppBuilder::new()
            .with_system(FixedUpdate, tick_shockwave)
            .build()
    }

    fn despawn_test_app() -> App {
        TestAppBuilder::new()
            .with_system(FixedUpdate, despawn_finished_shockwave)
            .build()
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

    fn spawn_cell(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut().spawn((Cell, Position2D(pos))).id()
    }

    fn spawn_dead_cell(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut().spawn((Cell, Position2D(pos), Dead)).id()
    }

    fn spawn_shockwave_no_chip(
        app: &mut App,
        pos: Vec2,
        radius: f32,
        base_dmg: f32,
        dmg_mult: f32,
    ) -> Entity {
        app.world_mut()
            .spawn((
                Position2D(pos),
                ShockwaveRadius(radius),
                ShockwaveBaseDamage(base_dmg),
                ShockwaveDamageMultiplier(dmg_mult),
                ShockwaveDamaged(HashSet::new()),
            ))
            .id()
    }

    // ── A. apply_shockwave_damage — damage emission ────────────────────────

    // #1
    #[test]
    fn shockwave_damages_cell_strictly_inside_radius() {
        let mut app = damage_test_app();

        let cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
        let sw = spawn_shockwave_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1, "expected exactly 1 DamageDealt<Cell>");
        assert_eq!(msgs.0[0].target, cell);
        assert_eq!(msgs.0[0].dealer, Some(sw));
        assert!(
            (msgs.0[0].amount - 10.0).abs() < f32::EPSILON,
            "expected amount == 10.0, got {}",
            msgs.0[0].amount,
        );
        assert_eq!(msgs.0[0].source_chip, None);

        let damaged = app.world().get::<ShockwaveDamaged>(sw).unwrap();
        assert!(
            damaged.0.contains(&cell),
            "ShockwaveDamaged should track the bumped cell",
        );
    }

    // #2
    #[test]
    fn shockwave_does_not_damage_cell_outside_radius() {
        let mut app = damage_test_app();

        let _cell = spawn_cell(&mut app, Vec2::new(100.0, 0.0));
        let sw = spawn_shockwave_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            0,
            "expected 0 DamageDealt<Cell> for out-of-range cell",
        );

        let damaged = app.world().get::<ShockwaveDamaged>(sw).unwrap();
        assert_eq!(
            damaged.0.len(),
            0,
            "ShockwaveDamaged must stay empty when no cells are in range",
        );
    }

    // #3
    #[test]
    fn shockwave_includes_cell_exactly_on_boundary() {
        let mut app = damage_test_app();

        // 3-4-5 right triangle: sqrt(30^2 + 40^2) == 50.0 exactly.
        let cell = spawn_cell(&mut app, Vec2::new(30.0, 40.0));
        let _sw = spawn_shockwave_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            1,
            "boundary distance (== radius) must damage the cell",
        );
        assert_eq!(msgs.0[0].target, cell);
    }

    // #4
    #[test]
    fn shockwave_does_not_redamage_previously_damaged_cell() {
        let mut app = damage_test_app();

        let cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
        let sw = spawn_shockwave_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

        tick(&mut app);
        {
            let msgs = app
                .world()
                .resource::<MessageCollector<DamageDealt<Cell>>>();
            assert_eq!(
                msgs.0.len(),
                1,
                "first tick should emit one DamageDealt<Cell>",
            );
        }

        // The `First`-schedule auto-clear empties the collector at the start
        // of the second update, so dedup produces zero messages, not stale ones.
        tick(&mut app);
        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            0,
            "second tick must not re-damage a cell already in ShockwaveDamaged",
        );

        let damaged = app.world().get::<ShockwaveDamaged>(sw).unwrap();
        assert!(
            damaged.0.contains(&cell),
            "ShockwaveDamaged must still contain the cell after the second tick",
        );
    }

    // #5
    #[test]
    fn shockwave_does_not_damage_dead_cell() {
        let mut app = damage_test_app();

        let _dead_cell = spawn_dead_cell(&mut app, Vec2::new(20.0, 0.0));
        let sw = spawn_shockwave_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            0,
            "Dead cells must be filtered out by AliveCellQuery's Without<Dead>",
        );

        let damaged = app.world().get::<ShockwaveDamaged>(sw).unwrap();
        assert_eq!(
            damaged.0.len(),
            0,
            "ShockwaveDamaged must remain empty — dead cells must not be inserted",
        );
    }

    // #6
    #[test]
    fn shockwave_damage_multiplies_base_damage_by_multiplier() {
        let mut app = damage_test_app();

        let _cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
        let _sw = spawn_shockwave_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 2.5);

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1);
        assert!(
            (msgs.0[0].amount - 25.0).abs() < f32::EPSILON,
            "expected amount == 10.0 * 2.5 == 25.0, got {}",
            msgs.0[0].amount,
        );
    }

    // #7
    #[test]
    fn shockwave_with_zero_cells_emits_no_damage_and_does_not_panic() {
        let mut app = damage_test_app();

        // NO cell spawns.
        let sw = spawn_shockwave_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 0, "empty-world shockwave must emit no damage");

        let damaged = app.world().get::<ShockwaveDamaged>(sw).unwrap();
        assert_eq!(damaged.0.len(), 0);
    }

    // #8
    #[test]
    fn shockwave_damages_all_cells_within_radius_in_one_tick() {
        let mut app = damage_test_app();

        let cell_a = spawn_cell(&mut app, Vec2::new(10.0, 0.0));
        let cell_b = spawn_cell(&mut app, Vec2::new(50.0, 0.0));
        let cell_c = spawn_cell(&mut app, Vec2::new(90.0, 0.0));
        let sw = spawn_shockwave_no_chip(&mut app, Vec2::ZERO, 100.0, 5.0, 1.0);

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 3, "expected three damage messages");

        for msg in &msgs.0 {
            assert!(
                (msg.amount - 5.0).abs() < f32::EPSILON,
                "every message should carry amount == 5.0, got {}",
                msg.amount,
            );
        }

        let targets: HashSet<Entity> = msgs.0.iter().map(|m| m.target).collect();
        assert_eq!(targets, HashSet::from([cell_a, cell_b, cell_c]));

        let damaged = app.world().get::<ShockwaveDamaged>(sw).unwrap();
        assert!(damaged.0.contains(&cell_a));
        assert!(damaged.0.contains(&cell_b));
        assert!(damaged.0.contains(&cell_c));
    }

    // #9
    #[test]
    fn two_independent_shockwaves_have_independent_damaged_sets() {
        let mut app = damage_test_app();

        let cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
        let sw_a = spawn_shockwave_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);
        let sw_b = spawn_shockwave_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            2,
            "each independent shockwave must emit its own DamageDealt<Cell>",
        );

        for msg in &msgs.0 {
            assert_eq!(msg.target, cell);
        }

        let dealers: HashSet<Option<Entity>> = msgs.0.iter().map(|m| m.dealer).collect();
        assert_eq!(dealers, HashSet::from([Some(sw_a), Some(sw_b)]));

        assert!(
            app.world()
                .get::<ShockwaveDamaged>(sw_a)
                .unwrap()
                .0
                .contains(&cell),
        );
        assert!(
            app.world()
                .get::<ShockwaveDamaged>(sw_b)
                .unwrap()
                .0
                .contains(&cell),
        );
    }

    // ── B. apply_shockwave_damage — source_chip propagation ────────────────

    #[test]
    fn shockwave_propagates_some_source_chip_in_damage_dealt() {
        let mut app = damage_test_app();

        let _cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
        app.world_mut().spawn((
            Position2D(Vec2::ZERO),
            ShockwaveRadius(50.0),
            ShockwaveBaseDamage(10.0),
            ShockwaveDamageMultiplier(1.0),
            ShockwaveDamaged(HashSet::new()),
            EffectSourceChip(Some("storm_chip".to_string())),
        ));

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1, "expected 1 DamageDealt<Cell> message");
        assert_eq!(
            msgs.0[0].source_chip,
            Some("storm_chip".to_string()),
            "DamageDealt should carry source_chip from EffectSourceChip, got {:?}",
            msgs.0[0].source_chip,
        );
    }

    // #11
    #[test]
    fn shockwave_propagates_none_source_chip() {
        let mut app = damage_test_app();

        let _cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
        app.world_mut().spawn((
            Position2D(Vec2::ZERO),
            ShockwaveRadius(50.0),
            ShockwaveBaseDamage(10.0),
            ShockwaveDamageMultiplier(1.0),
            ShockwaveDamaged(HashSet::new()),
            EffectSourceChip(None),
        ));

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1, "expected 1 DamageDealt<Cell> message");
        assert_eq!(
            msgs.0[0].source_chip, None,
            "EffectSourceChip(None) must survive unchanged, got {:?}",
            msgs.0[0].source_chip,
        );
    }

    // #12
    #[test]
    fn shockwave_without_effect_source_chip_component_writes_none() {
        let mut app = damage_test_app();

        let _cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
        // Deliberately NO EffectSourceChip component.
        let _sw = spawn_shockwave_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            1,
            "missing-component shockwave must still match the query (Option<&EffectSourceChip>)",
        );
        assert_eq!(msgs.0[0].source_chip, None);
    }

    // ── C. tick_shockwave — radius expansion ───────────────────────────────

    // #13
    #[test]
    fn tick_shockwave_expands_radius_by_speed_times_dt() {
        let mut app = tick_test_app();

        let sw = app
            .world_mut()
            .spawn((ShockwaveRadius(0.0), ShockwaveSpeed(200.0)))
            .id();

        tick_with_dt(&mut app, Duration::from_millis(250));

        let radius = app.world().get::<ShockwaveRadius>(sw).unwrap().0;
        assert!(
            (radius - 50.0).abs() < f32::EPSILON,
            "expected radius == 200.0 * 0.25 == 50.0, got {radius}",
        );
    }

    // #14
    #[test]
    fn tick_shockwave_accumulates_across_two_ticks() {
        let mut app = tick_test_app();

        let sw = app
            .world_mut()
            .spawn((ShockwaveRadius(10.0), ShockwaveSpeed(200.0)))
            .id();

        tick_with_dt(&mut app, Duration::from_millis(250));
        tick_with_dt(&mut app, Duration::from_millis(250));

        let radius = app.world().get::<ShockwaveRadius>(sw).unwrap().0;
        // 10.0 + 2 * (200.0 * 0.25) == 10.0 + 100.0 == 110.0
        assert!(
            (radius - 110.0).abs() < f32::EPSILON,
            "expected radius == 110.0 (10.0 + 2 * 50.0), got {radius}",
        );
    }

    // #15
    #[test]
    fn tick_shockwave_with_zero_speed_leaves_radius_unchanged() {
        let mut app = tick_test_app();

        let sw = app
            .world_mut()
            .spawn((ShockwaveRadius(42.0), ShockwaveSpeed(0.0)))
            .id();

        tick_with_dt(&mut app, Duration::from_millis(250));

        let radius = app.world().get::<ShockwaveRadius>(sw).unwrap().0;
        assert!(
            (radius - 42.0).abs() < f32::EPSILON,
            "zero-speed shockwave must not expand, got {radius}",
        );
    }

    // ── D. despawn_finished_shockwave — termination ────────────────────────

    // #16
    #[test]
    fn despawn_finished_shockwave_despawns_when_radius_equals_max() {
        let mut app = despawn_test_app();

        app.world_mut()
            .spawn((ShockwaveRadius(100.0), ShockwaveMaxRadius(100.0)));

        tick(&mut app);

        let count = app
            .world_mut()
            .query::<&ShockwaveRadius>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 0,
            "radius == max_radius must despawn (boundary of >=)",
        );
    }

    // #17
    #[test]
    fn despawn_finished_shockwave_despawns_when_radius_greater_than_max() {
        let mut app = despawn_test_app();

        app.world_mut()
            .spawn((ShockwaveRadius(150.0), ShockwaveMaxRadius(100.0)));

        tick(&mut app);

        let count = app
            .world_mut()
            .query::<&ShockwaveRadius>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "radius > max_radius must despawn");
    }

    // #18
    #[test]
    fn despawn_finished_shockwave_does_not_despawn_when_radius_less_than_max() {
        let mut app = despawn_test_app();

        app.world_mut()
            .spawn((ShockwaveRadius(50.0), ShockwaveMaxRadius(100.0)));

        tick(&mut app);

        let surviving: Vec<(f32, f32)> = app
            .world_mut()
            .query::<(&ShockwaveRadius, &ShockwaveMaxRadius)>()
            .iter(app.world())
            .map(|(r, m)| (r.0, m.0))
            .collect();
        assert_eq!(
            surviving.len(),
            1,
            "expanding shockwave (radius < max) must not despawn",
        );
        assert!((surviving[0].0 - 50.0).abs() < f32::EPSILON);
        assert!((surviving[0].1 - 100.0).abs() < f32::EPSILON);
    }
}
