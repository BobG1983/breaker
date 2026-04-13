//! Pulse systems — emitter tick, ring expansion, damage application, despawn.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;
use rantzsoft_stateflow::CleanupOnExit;

use super::components::{
    PulseEmitter, PulseRing, PulseRingBaseDamage, PulseRingDamageMultiplier, PulseRingDamaged,
    PulseRingMaxRadius, PulseRingRadius, PulseRingSpeed,
};
use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
    cells::components::Cell,
    effect_v3::{components::EffectSourceChip, effects::DamageBoostConfig, stacking::EffectStack},
    shared::death_pipeline::{DamageDealt, Dead},
    state::types::NodeState,
};

/// Alive cell lookup — entity + position, excludes dead cells.
type AliveCellQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Position2D), (With<Cell>, Without<Dead>)>;

/// Pulse ring tick + damage query.
type PulseRingQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Position2D,
        &'static PulseRingRadius,
        &'static PulseRingBaseDamage,
        &'static PulseRingDamageMultiplier,
        &'static mut PulseRingDamaged,
        Option<&'static EffectSourceChip>,
    ),
>;

/// Pulse emitter tick query — reads `BoltBaseDamage` and
/// `EffectStack<DamageBoostConfig>` from the emitter entity for per-ring
/// snapshot at spawn time.
type PulseEmitterQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut PulseEmitter,
        &'static Position2D,
        Option<&'static BoltBaseDamage>,
        Option<&'static EffectStack<DamageBoostConfig>>,
    ),
>;

/// Decrements pulse emitter timers each frame and spawns pulse rings when the
/// timer reaches zero. Each spawned ring snapshots `BoltBaseDamage` and
/// `EffectStack<DamageBoostConfig>` from the emitter entity at spawn time, and
/// inherits the emitter's `source_chip` string via `EffectSourceChip`.
pub fn tick_pulse(mut query: PulseEmitterQuery, time: Res<Time>, mut commands: Commands) {
    let dt = time.delta_secs();

    for (mut emitter, pos, bolt_base_damage_opt, damage_stack_opt) in &mut query {
        emitter.timer -= dt;
        if emitter.timer <= 0.0 {
            emitter.timer += emitter.interval;

            let stacks_f32 = emitter.stacks.saturating_sub(1) as f32;
            let max_radius = emitter
                .range_per_level
                .mul_add(stacks_f32, emitter.base_range);
            let base_damage = bolt_base_damage_opt.map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);
            let damage_mult = damage_stack_opt.map_or(1.0, EffectStack::aggregate);

            commands.spawn((
                PulseRing,
                PulseRingRadius(0.0),
                PulseRingMaxRadius(max_radius),
                PulseRingSpeed(emitter.speed),
                PulseRingDamaged(HashSet::new()),
                PulseRingBaseDamage(base_damage),
                PulseRingDamageMultiplier(damage_mult),
                Position2D(pos.0),
                emitter.source_chip.clone(),
                CleanupOnExit::<NodeState>::default(),
            ));
        }
    }
}

/// Expands pulse ring radius each frame based on speed.
pub fn tick_pulse_ring(mut query: Query<(&mut PulseRingRadius, &PulseRingSpeed)>, time: Res<Time>) {
    let dt = time.delta_secs();
    for (mut radius, speed) in &mut query {
        radius.0 = speed.0.mul_add(dt, radius.0);
    }
}

/// Applies damage to cells within the expanding pulse ring radius.
pub(crate) fn apply_pulse_damage(
    mut pulse_query: PulseRingQuery,
    cell_query: AliveCellQuery,
    mut damage_writer: MessageWriter<DamageDealt<Cell>>,
) {
    for (ring_entity, ring_pos, ring_radius, base_dmg, dmg_mult, mut damaged, chip) in
        &mut pulse_query
    {
        for (cell_entity, cell_pos) in &cell_query {
            if damaged.0.contains(&cell_entity) {
                continue;
            }
            let distance = ring_pos.0.distance(cell_pos.0);
            if distance <= ring_radius.0 {
                damaged.0.insert(cell_entity);
                let damage = base_dmg.0 * dmg_mult.0;
                damage_writer.write(DamageDealt {
                    dealer:      Some(ring_entity),
                    target:      cell_entity,
                    amount:      damage,
                    source_chip: chip.and_then(|c| c.0.clone()),
                    _marker:     std::marker::PhantomData,
                });
            }
        }
    }
}

/// Despawns pulse rings that have reached their maximum radius.
pub fn despawn_finished_pulse_ring(
    query: Query<(Entity, &PulseRingRadius, &PulseRingMaxRadius)>,
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

    use super::{
        PulseEmitter, PulseRing, PulseRingBaseDamage, PulseRingDamageMultiplier, PulseRingDamaged,
        PulseRingMaxRadius, PulseRingRadius, PulseRingSpeed, apply_pulse_damage,
        despawn_finished_pulse_ring, tick_pulse, tick_pulse_ring,
    };
    use crate::{
        bolt::components::BoltBaseDamage,
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
            .with_system(FixedUpdate, apply_pulse_damage)
            .build()
    }

    /// `tick_pulse_ring` uses `Res<Time>`, which only resolves to `Time<Fixed>`
    /// when the system is scheduled inside `FixedUpdate`. Registering it in
    /// `Update` would silently use a zero-delta virtual clock.
    fn tick_test_app() -> App {
        TestAppBuilder::new()
            .with_system(FixedUpdate, tick_pulse_ring)
            .build()
    }

    fn despawn_test_app() -> App {
        TestAppBuilder::new()
            .with_system(FixedUpdate, despawn_finished_pulse_ring)
            .build()
    }

    /// `tick_pulse` test app — exercises the emitter tick (Section F).
    fn emitter_test_app() -> App {
        TestAppBuilder::new()
            .with_system(FixedUpdate, tick_pulse)
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

    fn spawn_pulse_ring_no_chip(
        app: &mut App,
        pos: Vec2,
        radius: f32,
        base_dmg: f32,
        dmg_mult: f32,
    ) -> Entity {
        app.world_mut()
            .spawn((
                Position2D(pos),
                PulseRingRadius(radius),
                PulseRingBaseDamage(base_dmg),
                PulseRingDamageMultiplier(dmg_mult),
                PulseRingDamaged(HashSet::new()),
            ))
            .id()
    }

    // ── A. apply_pulse_damage — damage emission ────────────────────────────

    // #1
    #[test]
    fn pulse_ring_damages_cell_strictly_inside_radius() {
        let mut app = damage_test_app();

        let cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
        let ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1, "expected exactly 1 DamageDealt<Cell>");
        assert_eq!(msgs.0[0].target, cell);
        assert_eq!(msgs.0[0].dealer, Some(ring));
        assert!(
            (msgs.0[0].amount - 10.0).abs() < f32::EPSILON,
            "expected amount == 10.0, got {}",
            msgs.0[0].amount,
        );
        assert_eq!(msgs.0[0].source_chip, None);

        let damaged = app.world().get::<PulseRingDamaged>(ring).unwrap();
        assert!(
            damaged.0.contains(&cell),
            "PulseRingDamaged should track the damaged cell",
        );
    }

    // #2
    #[test]
    fn pulse_ring_does_not_damage_cell_outside_radius() {
        let mut app = damage_test_app();

        let _cell = spawn_cell(&mut app, Vec2::new(100.0, 0.0));
        let ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            0,
            "expected 0 DamageDealt<Cell> for out-of-range cell",
        );

        let damaged = app.world().get::<PulseRingDamaged>(ring).unwrap();
        assert_eq!(
            damaged.0.len(),
            0,
            "PulseRingDamaged must stay empty when no cells are in range",
        );
    }

    // #3
    #[test]
    fn pulse_ring_includes_cell_exactly_on_boundary() {
        let mut app = damage_test_app();

        // 3-4-5 right triangle: sqrt(30^2 + 40^2) == 50.0 exactly.
        let cell = spawn_cell(&mut app, Vec2::new(30.0, 40.0));
        let _ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

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
    fn pulse_ring_does_not_redamage_previously_damaged_cell() {
        let mut app = damage_test_app();

        let cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
        let ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

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
            "second tick must not re-damage a cell already in PulseRingDamaged",
        );

        let damaged = app.world().get::<PulseRingDamaged>(ring).unwrap();
        assert!(
            damaged.0.contains(&cell),
            "PulseRingDamaged must still contain the cell after the second tick",
        );
    }

    // #5
    #[test]
    fn pulse_ring_does_not_damage_dead_cell() {
        let mut app = damage_test_app();

        let _dead_cell = spawn_dead_cell(&mut app, Vec2::new(20.0, 0.0));
        let ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            0,
            "Dead cells must be filtered out by the alive-cell query",
        );

        let damaged = app.world().get::<PulseRingDamaged>(ring).unwrap();
        assert_eq!(
            damaged.0.len(),
            0,
            "PulseRingDamaged must remain empty — dead cells must not be inserted",
        );
    }

    // #6
    #[test]
    fn pulse_ring_damage_multiplies_base_damage_by_multiplier() {
        let mut app = damage_test_app();

        let _cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
        let _ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 2.5);

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
    fn pulse_ring_with_zero_cells_emits_no_damage_and_does_not_panic() {
        let mut app = damage_test_app();

        // NO cell spawns.
        let ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            0,
            "empty-world pulse ring must emit no damage",
        );

        let damaged = app.world().get::<PulseRingDamaged>(ring).unwrap();
        assert_eq!(damaged.0.len(), 0);
    }

    // #8
    #[test]
    fn pulse_ring_damages_all_cells_within_radius_in_one_tick() {
        let mut app = damage_test_app();

        let cell_a = spawn_cell(&mut app, Vec2::new(10.0, 0.0));
        let cell_b = spawn_cell(&mut app, Vec2::new(50.0, 0.0));
        let cell_c = spawn_cell(&mut app, Vec2::new(90.0, 0.0));
        let ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 100.0, 5.0, 1.0);

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

        let damaged = app.world().get::<PulseRingDamaged>(ring).unwrap();
        assert!(damaged.0.contains(&cell_a));
        assert!(damaged.0.contains(&cell_b));
        assert!(damaged.0.contains(&cell_c));
    }

    // #9
    #[test]
    fn two_independent_pulse_rings_have_independent_damaged_sets() {
        let mut app = damage_test_app();

        let cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
        let ring_a = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);
        let ring_b = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            2,
            "each independent pulse ring must emit its own DamageDealt<Cell>",
        );

        for msg in &msgs.0 {
            assert_eq!(msg.target, cell);
        }

        let dealers: HashSet<Option<Entity>> = msgs.0.iter().map(|m| m.dealer).collect();
        assert_eq!(dealers, HashSet::from([Some(ring_a), Some(ring_b)]));

        assert!(
            app.world()
                .get::<PulseRingDamaged>(ring_a)
                .unwrap()
                .0
                .contains(&cell),
        );
        assert!(
            app.world()
                .get::<PulseRingDamaged>(ring_b)
                .unwrap()
                .0
                .contains(&cell),
        );
    }

    // ── B. apply_pulse_damage — source_chip propagation ────────────────────

    // #10
    #[test]
    fn pulse_ring_propagates_some_source_chip_in_damage_dealt() {
        let mut app = damage_test_app();

        let _cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
        app.world_mut().spawn((
            Position2D(Vec2::ZERO),
            PulseRingRadius(50.0),
            PulseRingBaseDamage(10.0),
            PulseRingDamageMultiplier(1.0),
            PulseRingDamaged(HashSet::new()),
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
    fn pulse_ring_propagates_none_source_chip() {
        let mut app = damage_test_app();

        let _cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
        app.world_mut().spawn((
            Position2D(Vec2::ZERO),
            PulseRingRadius(50.0),
            PulseRingBaseDamage(10.0),
            PulseRingDamageMultiplier(1.0),
            PulseRingDamaged(HashSet::new()),
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
    fn pulse_ring_without_effect_source_chip_component_writes_none() {
        let mut app = damage_test_app();

        let _cell = spawn_cell(&mut app, Vec2::new(20.0, 0.0));
        // Deliberately NO EffectSourceChip component.
        let _ring = spawn_pulse_ring_no_chip(&mut app, Vec2::ZERO, 50.0, 10.0, 1.0);

        tick(&mut app);

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            1,
            "missing-component pulse ring must still match the query (Option<&EffectSourceChip>)",
        );
        assert_eq!(msgs.0[0].source_chip, None);
    }

    // ── C. tick_pulse_ring — radius expansion ──────────────────────────────

    // #13
    #[test]
    fn tick_pulse_ring_expands_radius_by_speed_times_dt() {
        let mut app = tick_test_app();

        let ring = app
            .world_mut()
            .spawn((PulseRingRadius(0.0), PulseRingSpeed(200.0)))
            .id();

        tick_with_dt(&mut app, Duration::from_millis(250));

        let radius = app.world().get::<PulseRingRadius>(ring).unwrap().0;
        assert!(
            (radius - 50.0).abs() < f32::EPSILON,
            "expected radius == 200.0 * 0.25 == 50.0, got {radius}",
        );
    }

    // #14
    #[test]
    fn tick_pulse_ring_accumulates_across_two_ticks() {
        let mut app = tick_test_app();

        let ring = app
            .world_mut()
            .spawn((PulseRingRadius(10.0), PulseRingSpeed(200.0)))
            .id();

        tick_with_dt(&mut app, Duration::from_millis(250));
        tick_with_dt(&mut app, Duration::from_millis(250));

        let radius = app.world().get::<PulseRingRadius>(ring).unwrap().0;
        // 10.0 + 2 * (200.0 * 0.25) == 10.0 + 100.0 == 110.0
        assert!(
            (radius - 110.0).abs() < f32::EPSILON,
            "expected radius == 110.0 (10.0 + 2 * 50.0), got {radius}",
        );
    }

    // #15
    #[test]
    fn tick_pulse_ring_with_zero_speed_leaves_radius_unchanged() {
        let mut app = tick_test_app();

        let ring = app
            .world_mut()
            .spawn((PulseRingRadius(42.0), PulseRingSpeed(0.0)))
            .id();

        tick_with_dt(&mut app, Duration::from_millis(250));

        let radius = app.world().get::<PulseRingRadius>(ring).unwrap().0;
        assert!(
            (radius - 42.0).abs() < f32::EPSILON,
            "zero-speed pulse ring must not expand, got {radius}",
        );
    }

    // ── D. despawn_finished_pulse_ring — termination ───────────────────────

    // #16
    #[test]
    fn despawn_finished_pulse_ring_despawns_when_radius_equals_max() {
        let mut app = despawn_test_app();

        app.world_mut()
            .spawn((PulseRingRadius(100.0), PulseRingMaxRadius(100.0)));

        tick(&mut app);

        let count = app
            .world_mut()
            .query::<&PulseRingRadius>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 0,
            "radius == max_radius must despawn (boundary of >=)",
        );
    }

    // #17
    #[test]
    fn despawn_finished_pulse_ring_despawns_when_radius_greater_than_max() {
        let mut app = despawn_test_app();

        app.world_mut()
            .spawn((PulseRingRadius(150.0), PulseRingMaxRadius(100.0)));

        tick(&mut app);

        let count = app
            .world_mut()
            .query::<&PulseRingRadius>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "radius > max_radius must despawn");
    }

    // #18
    #[test]
    fn despawn_finished_pulse_ring_does_not_despawn_when_radius_less_than_max() {
        let mut app = despawn_test_app();

        app.world_mut()
            .spawn((PulseRingRadius(50.0), PulseRingMaxRadius(100.0)));

        tick(&mut app);

        let surviving: Vec<(f32, f32)> = app
            .world_mut()
            .query::<(&PulseRingRadius, &PulseRingMaxRadius)>()
            .iter(app.world())
            .map(|(r, m)| (r.0, m.0))
            .collect();
        assert_eq!(
            surviving.len(),
            1,
            "expanding pulse ring (radius < max) must not despawn",
        );
        assert!((surviving[0].0 - 50.0).abs() < f32::EPSILON);
        assert!((surviving[0].1 - 100.0).abs() < f32::EPSILON);
    }

    // ── F. tick_pulse — timer, interval, and range formula ─────────────────

    // #30
    #[test]
    fn tick_pulse_does_not_fire_when_timer_positive_after_decrement() {
        let mut app = emitter_test_app();

        let emitter = app
            .world_mut()
            .spawn((
                Position2D(Vec2::ZERO),
                PulseEmitter {
                    base_range:      64.0,
                    range_per_level: 16.0,
                    stacks:          1,
                    speed:           200.0,
                    interval:        1.0,
                    timer:           1.0,
                    source_chip:     EffectSourceChip(None),
                },
            ))
            .id();

        tick_with_dt(&mut app, Duration::from_millis(250));

        let count = app
            .world_mut()
            .query::<&PulseRing>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 0,
            "no ring should spawn while timer > 0 after decrement"
        );

        let timer = app.world().get::<PulseEmitter>(emitter).unwrap().timer;
        assert!(
            (timer - 0.75).abs() < f32::EPSILON,
            "timer should equal 1.0 - 0.25 = 0.75, got {timer}",
        );
    }

    // #31
    #[test]
    fn tick_pulse_fires_when_timer_at_or_below_zero_and_refills_by_interval() {
        let mut app = emitter_test_app();

        let emitter = app
            .world_mut()
            .spawn((
                Position2D(Vec2::ZERO),
                PulseEmitter {
                    base_range:      64.0,
                    range_per_level: 16.0,
                    stacks:          1,
                    speed:           200.0,
                    interval:        1.0,
                    timer:           0.25,
                    source_chip:     EffectSourceChip(None),
                },
            ))
            .id();

        tick_with_dt(&mut app, Duration::from_millis(250));

        let count = app
            .world_mut()
            .query::<&PulseRing>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "one ring should spawn when timer hits 0");

        let timer = app.world().get::<PulseEmitter>(emitter).unwrap().timer;
        assert!(
            (timer - 1.0).abs() < f32::EPSILON,
            "timer should refill to 0.25 - 0.25 + 1.0 = 1.0, got {timer}",
        );
    }

    // #32
    #[test]
    fn tick_pulse_refires_on_second_tick_after_another_interval() {
        let mut app = emitter_test_app();

        app.world_mut().spawn((
            Position2D(Vec2::ZERO),
            PulseEmitter {
                base_range:      64.0,
                range_per_level: 16.0,
                stacks:          1,
                speed:           200.0,
                interval:        0.25,
                timer:           0.25,
                source_chip:     EffectSourceChip(None),
            },
        ));

        tick_with_dt(&mut app, Duration::from_millis(250));
        tick_with_dt(&mut app, Duration::from_millis(250));

        let count = app
            .world_mut()
            .query::<&PulseRing>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 2,
            "two ticks at the firing interval should spawn two rings"
        );
    }

    // #33
    #[test]
    fn tick_pulse_max_radius_uses_base_range_for_stacks_one() {
        let mut app = emitter_test_app();

        app.world_mut().spawn((
            Position2D(Vec2::ZERO),
            PulseEmitter {
                base_range:      64.0,
                range_per_level: 16.0,
                stacks:          1,
                speed:           200.0,
                interval:        1.0,
                timer:           0.0,
                source_chip:     EffectSourceChip(None),
            },
        ));

        tick_with_dt(&mut app, Duration::from_millis(250));

        let max_radii: Vec<f32> = app
            .world_mut()
            .query::<&PulseRingMaxRadius>()
            .iter(app.world())
            .map(|r| r.0)
            .collect();
        assert_eq!(max_radii.len(), 1, "expected 1 spawned pulse ring");
        assert!(
            (max_radii[0] - 64.0).abs() < f32::EPSILON,
            "stacks==1 must yield max_radius == base_range (64.0), got {}",
            max_radii[0],
        );
    }

    // #34
    #[test]
    fn tick_pulse_max_radius_adds_range_per_level_for_stacks_three() {
        let mut app = emitter_test_app();

        app.world_mut().spawn((
            Position2D(Vec2::ZERO),
            PulseEmitter {
                base_range:      64.0,
                range_per_level: 16.0,
                stacks:          3,
                speed:           200.0,
                interval:        1.0,
                timer:           0.0,
                source_chip:     EffectSourceChip(None),
            },
        ));

        tick_with_dt(&mut app, Duration::from_millis(250));

        let max_radii: Vec<f32> = app
            .world_mut()
            .query::<&PulseRingMaxRadius>()
            .iter(app.world())
            .map(|r| r.0)
            .collect();
        assert_eq!(max_radii.len(), 1);
        // 64.0 + (3 - 1) * 16.0 == 64.0 + 32.0 == 96.0
        assert!(
            (max_radii[0] - 96.0).abs() < f32::EPSILON,
            "stacks==3 must yield max_radius == 96.0, got {}",
            max_radii[0],
        );
    }

    // #35
    #[test]
    fn tick_pulse_max_radius_uses_saturating_sub_for_stacks_zero() {
        let mut app = emitter_test_app();

        app.world_mut().spawn((
            Position2D(Vec2::ZERO),
            PulseEmitter {
                base_range:      64.0,
                range_per_level: 16.0,
                stacks:          0,
                speed:           200.0,
                interval:        1.0,
                timer:           0.0,
                source_chip:     EffectSourceChip(None),
            },
        ));

        tick_with_dt(&mut app, Duration::from_millis(250));

        let max_radii: Vec<f32> = app
            .world_mut()
            .query::<&PulseRingMaxRadius>()
            .iter(app.world())
            .map(|r| r.0)
            .collect();
        assert_eq!(max_radii.len(), 1);
        // saturating_sub(1) on 0u32 -> 0 -> (stacks_f32 == 0.0) -> max_radius == base_range
        assert!(
            (max_radii[0] - 64.0).abs() < f32::EPSILON,
            "stacks==0 must collapse to base_range via saturating_sub, got {}",
            max_radii[0],
        );
    }

    // #36
    #[test]
    fn tick_pulse_two_independent_emitters_produce_two_independent_rings_in_one_tick() {
        let mut app = emitter_test_app();

        let emitter_a = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::ZERO),
                PulseEmitter {
                    base_range:      64.0,
                    range_per_level: 16.0,
                    stacks:          1,
                    speed:           200.0,
                    interval:        1.0,
                    timer:           0.0,
                    source_chip:     EffectSourceChip(None),
                },
            ))
            .id();
        let emitter_b = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::ZERO),
                PulseEmitter {
                    base_range:      64.0,
                    range_per_level: 16.0,
                    stacks:          1,
                    speed:           200.0,
                    interval:        1.0,
                    timer:           0.0,
                    source_chip:     EffectSourceChip(None),
                },
            ))
            .id();

        tick_with_dt(&mut app, Duration::from_millis(250));

        let ring_count = app
            .world_mut()
            .query::<&PulseRing>()
            .iter(app.world())
            .count();
        assert_eq!(
            ring_count, 2,
            "each of the two emitters should spawn one ring"
        );

        let radii: Vec<f32> = app
            .world_mut()
            .query::<&PulseRingRadius>()
            .iter(app.world())
            .map(|r| r.0)
            .collect();
        assert_eq!(radii.len(), 2);
        for r in &radii {
            assert!(
                r.abs() < f32::EPSILON,
                "newly spawned ring should start at radius 0, got {r}",
            );
        }

        let max_radii: Vec<f32> = app
            .world_mut()
            .query::<&PulseRingMaxRadius>()
            .iter(app.world())
            .map(|r| r.0)
            .collect();
        for m in &max_radii {
            assert!(
                (m - 64.0).abs() < f32::EPSILON,
                "spawned ring max_radius should be 64.0, got {m}",
            );
        }

        let damaged_lens: Vec<usize> = app
            .world_mut()
            .query::<&PulseRingDamaged>()
            .iter(app.world())
            .map(|d| d.0.len())
            .collect();
        assert_eq!(damaged_lens.len(), 2);
        for len in damaged_lens {
            assert_eq!(len, 0, "fresh PulseRingDamaged must be empty");
        }

        // Timer math: starting 0.0, decrement by dt=0.25 → -0.25, fire then
        // `timer += interval` (1.0) → 0.75. Mirrors the `+= interval` production
        // path that #31 also exercises (starting 0.25 → 0.0 → 1.0).
        let timer_a = app.world().get::<PulseEmitter>(emitter_a).unwrap().timer;
        let timer_b = app.world().get::<PulseEmitter>(emitter_b).unwrap().timer;
        assert!((timer_a - 0.75).abs() < f32::EPSILON);
        assert!((timer_b - 0.75).abs() < f32::EPSILON);
    }
}
