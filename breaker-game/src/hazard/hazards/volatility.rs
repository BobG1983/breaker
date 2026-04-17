//! Volatility hazard — interval-based cell recovery.
//!
//! Cells that are left untouched regrow HP in discrete interval ticks. Each
//! interval emits a `HealDealt<Cell>` (capped via `HealCap::Max` against a
//! lifted `Hp.max = 2× starting`) so growth rides the unified heal pipeline.
//! Any `DamageDealt<Cell>` resets the cell's timer to zero — a cell "touched"
//! within the interval stays at its current HP. Effective interval shrinks
//! with hazard stacks (floored at 1.0s). Authoritative design doc:
//! `docs/todos/detail/mod-system-design/hazards/volatility.md`.

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    cells::components::Cell,
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
    shared::death_pipeline::{
        DamageDealt, HealCap, Hp, heal_dealt::HealDealt, sets::DeathPipelineSystems,
    },
};

/// Tracks time since this cell was last damaged. Advanced by
/// `volatility_grow_cells`; reset to 0.0 by `reset_volatility_on_damage` on
/// any incoming `DamageDealt<Cell>`.
#[derive(Component, Debug)]
pub(crate) struct VolatilityTimer {
    /// Seconds elapsed since the last damage reset.
    pub(crate) elapsed: f32,
}

/// Per-run tuning extracted from [`HazardTuning::Volatility`] at activation.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct VolatilityConfig {
    /// HP gained per growth interval.
    pub(crate) hp_per_interval: f32,
    /// Base interval between growth ticks at stack 1 (seconds).
    pub(crate) interval_secs:   f32,
    /// Multiplier on `Hp.starting` that caps growth and defines the lifted
    /// `Hp.max` written by `attach_volatility_timers`.
    pub(crate) max_multiplier:  f32,
}

impl VolatilityConfig {
    /// Effective interval between growth ticks for the given stack count.
    /// Formula: `interval_secs / (1.0 + 0.25 * (stacks - 1))`, floored at 1.0s.
    /// Returns `interval_secs` at `stacks == 1`; returns `f32::INFINITY` when
    /// `stacks == 0` (hazard inactive — gated by `run_if` before reaching here).
    #[must_use]
    pub(crate) fn effective_interval(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return f32::INFINITY;
        }
        let extra = stacks.saturating_sub(1) as f32;
        (self.interval_secs / 0.25f32.mul_add(extra, 1.0)).max(1.0)
    }
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Volatility {
        hp_per_interval,
        interval_secs,
        max_multiplier,
    } = *tuning
    else {
        warn!("volatility::activate called with non-Volatility tuning");
        return;
    };
    commands.insert_resource(VolatilityConfig {
        hp_per_interval,
        interval_secs,
        max_multiplier,
    });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            attach_volatility_timers,
            reset_volatility_on_damage.after(DeathPipelineSystems::ApplyDamage),
            volatility_grow_cells.before(DeathPipelineSystems::ApplyHeal),
        )
            .chain()
            .run_if(hazard_active(HazardKind::Volatility))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// Two responsibilities per tick:
/// (1) Lift each cell's `Hp.max` to `max(existing_max, hp.starting * max_multiplier)`
///     so the heal pipeline's `HealCap::Max` clamp permits growth up to 2× starting.
/// (2) Insert `VolatilityTimer { elapsed: 0.0 }` on any cell missing one.
/// The `Hp.max` lift runs for every cell in the query — it is not gated on
/// `VolatilityTimer` presence — but writes are guarded so already-lifted cells
/// don't trigger `Changed<Hp>`.
fn attach_volatility_timers(
    mut commands: Commands,
    config: Option<Res<VolatilityConfig>>,
    mut cells: Query<(Entity, &mut Hp, Option<&VolatilityTimer>), With<Cell>>,
) {
    let Some(config) = config else { return };
    for (entity, mut hp, timer) in &mut cells {
        let target = hp.starting * config.max_multiplier;
        let new_max = Some(hp.max.map_or(target, |m| m.max(target)));
        if hp.max != new_max {
            hp.max = new_max;
        }

        if timer.is_none() {
            commands
                .entity(entity)
                .insert(VolatilityTimer { elapsed: 0.0 });
        }
    }
}

/// Zeros `VolatilityTimer.elapsed` on every cell that receives a
/// `DamageDealt<Cell>` message. The damage amount is irrelevant — any hit
/// (including 0.0 and NaN) resets the timer because the cell was touched.
/// Silently skips targets that lack a `VolatilityTimer` or have been despawned.
fn reset_volatility_on_damage(
    mut reader: MessageReader<DamageDealt<Cell>>,
    mut cells: Query<&mut VolatilityTimer, With<Cell>>,
) {
    for msg in reader.read() {
        if let Ok(mut timer) = cells.get_mut(msg.target) {
            timer.elapsed = 0.0;
        }
    }
}

/// Advances each cell's `VolatilityTimer.elapsed` by `dt`. While the timer
/// has crossed an interval, emits one `HealDealt<Cell>` per crossed interval
/// (subject to the pre-send cap gate `hp.current < hp.starting * max_multiplier`).
/// The timer subtract inside the while-loop is UNCONDITIONAL relative to the
/// cap gate — cells at cap still consume their interval budget.
fn volatility_grow_cells(
    time: Res<Time<Fixed>>,
    config: Option<Res<VolatilityConfig>>,
    active: Res<ActiveHazards>,
    mut writer: MessageWriter<HealDealt<Cell>>,
    mut cells: Query<(Entity, &Hp, &mut VolatilityTimer), With<Cell>>,
) {
    let Some(config) = config else { return };
    let stacks = active.stacks(HazardKind::Volatility);
    let effective_interval = config.effective_interval(stacks);
    if !effective_interval.is_finite() || effective_interval <= 0.0 {
        return;
    }
    let dt = time.delta_secs();
    // Defensive no-op: Bevy's FixedUpdate should always produce positive dt.
    if dt <= 0.0 {
        return;
    }
    let cap = config.max_multiplier;

    for (entity, hp, mut timer) in &mut cells {
        if hp.current <= 0.0 {
            continue;
        }
        timer.elapsed += dt;
        // `hp.current` is an immutable snapshot for this tick — HealDealt is queued
        // for apply_heal but not applied in this system. All while-loop iterations
        // see the same value.
        let cell_cap = hp.starting * cap;
        while timer.elapsed >= effective_interval {
            if hp.current < cell_cap {
                writer.write(HealDealt::<Cell> {
                    healer:  None,
                    target:  entity,
                    amount:  config.hp_per_interval,
                    cap:     HealCap::Max,
                    source:  Some("hazard:volatility".to_string()),
                    _marker: PhantomData,
                });
            }
            timer.elapsed -= effective_interval;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{marker::PhantomData, time::Duration};

    use super::*;
    use crate::{
        cells::behaviors::survival::salvo::components::Salvo,
        hazard::hazards::renewal::RenewalConfig,
        shared::death_pipeline::{HealCap, systems::apply_damage},
    };

    // ── Helpers ────────────────────────────────────────────────────────────

    /// Default builder for Groups A/B/D/E/F. Registers `ActiveHazards`,
    /// `DamageDealt<Cell>` + `HealDealt<Cell>` messages, and the heal
    /// capture collector.
    fn test_app_playing() -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_resource::<ActiveHazards>()
            .with_message::<DamageDealt<Cell>>()
            .with_message_capture::<HealDealt<Cell>>()
            .build()
    }

    /// Variant for Group C tests. Adds `PendingCellDamage` + `enqueue_cell_damage`
    /// ordered `.before(DeathPipelineSystems::ApplyDamage)` and registers
    /// `apply_damage::<Cell>` in `DeathPipelineSystems::ApplyDamage` so the
    /// reset-ordering behaviours can be exercised end-to-end.
    fn test_app_playing_with_damage() -> App {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_resource::<ActiveHazards>()
            .with_message::<DamageDealt<Cell>>()
            .with_message_capture::<HealDealt<Cell>>()
            .with_resource::<PendingCellDamage>()
            .build();
        app.add_systems(
            FixedUpdate,
            enqueue_cell_damage.before(DeathPipelineSystems::ApplyDamage),
        );
        app.add_systems(
            FixedUpdate,
            apply_damage::<Cell>.in_set(DeathPipelineSystems::ApplyDamage),
        );
        app
    }

    /// Spawns a cell in the post-attach state: `Hp.max = Some(starting * 2.0)`
    /// with a `VolatilityTimer { elapsed }` already attached. `KilledBy` is
    /// included so `apply_damage::<Cell>` can process damage in Group C tests.
    fn spawn_cell_with_timer(app: &mut App, current: f32, starting: f32, elapsed: f32) -> Entity {
        app.world_mut()
            .spawn((
                Cell,
                Hp {
                    current,
                    starting,
                    max: Some(starting * 2.0),
                },
                KilledBy::default(),
                VolatilityTimer { elapsed },
            ))
            .id()
    }

    /// Variant that accepts an explicit `max` — used for cells constructed
    /// with a specific cap (e.g., at the 2× cap or slightly under/over it).
    fn spawn_cell_with_timer_max(
        app: &mut App,
        current: f32,
        starting: f32,
        max: Option<f32>,
        elapsed: f32,
    ) -> Entity {
        app.world_mut()
            .spawn((
                Cell,
                Hp {
                    current,
                    starting,
                    max,
                },
                KilledBy::default(),
                VolatilityTimer { elapsed },
            ))
            .id()
    }

    /// Spawns a cell WITHOUT `VolatilityTimer` — used for Group D tests that
    /// exercise `attach_volatility_timers` itself.
    fn spawn_cell_no_timer(app: &mut App, current: f32, starting: f32, max: Option<f32>) -> Entity {
        app.world_mut()
            .spawn((
                Cell,
                Hp {
                    current,
                    starting,
                    max,
                },
                KilledBy::default(),
            ))
            .id()
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

    /// Test resource drained each tick by `enqueue_cell_damage` into
    /// `DamageDealt<Cell>` messages.
    #[derive(Resource, Default)]
    pub(super) struct PendingCellDamage(pub Vec<(Entity, f32)>);

    /// Enqueue system — drains `PendingCellDamage` and writes one
    /// `DamageDealt<Cell>` per entry. Registered
    /// `.before(DeathPipelineSystems::ApplyDamage)` by `test_app_playing_with_damage`.
    pub(super) fn enqueue_cell_damage(
        mut pending: ResMut<PendingCellDamage>,
        mut writer: MessageWriter<DamageDealt<Cell>>,
    ) {
        for (target, amount) in pending.0.drain(..) {
            writer.write(DamageDealt {
                dealer: None,
                target,
                amount,
                source_chip: None,
                _marker: PhantomData,
            });
        }
    }

    fn heals_for(app: &App, target: Entity) -> Vec<HealDealt<Cell>> {
        app.world()
            .resource::<MessageCollector<HealDealt<Cell>>>()
            .0
            .iter()
            .filter(|m| m.target == target)
            .cloned()
            .collect()
    }

    fn heal_collector_len(app: &App) -> usize {
        app.world()
            .resource::<MessageCollector<HealDealt<Cell>>>()
            .0
            .len()
    }

    fn add_volatility_stacks(app: &mut App, count: u32) {
        let mut active = app.world_mut().resource_mut::<ActiveHazards>();
        for _ in 0..count {
            active.add_stack(HazardKind::Volatility);
        }
    }

    fn default_config() -> VolatilityConfig {
        VolatilityConfig {
            hp_per_interval: 1.0,
            interval_secs:   5.0,
            max_multiplier:  2.0,
        }
    }

    fn install_default_config(app: &mut App) {
        app.world_mut().insert_resource(default_config());
    }

    fn register_volatility_systems(app: &mut App) {
        super::register(app);
    }

    // ══════════════════════════════════════════════════════════════════════
    // Group A — volatility_grow_cells interval / heal semantics
    // ══════════════════════════════════════════════════════════════════════

    // Behavior 1
    #[test]
    fn emits_heal_dealt_after_one_interval_stack_1() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 1, "expected exactly one HealDealt<Cell>");
        let msg = &heals[0];
        assert_eq!(msg.target, cell);
        assert!(
            (msg.amount - 1.0).abs() < f32::EPSILON,
            "amount should be 1.0, got {}",
            msg.amount
        );
        assert!(matches!(msg.cap, HealCap::Max));
        assert_eq!(msg.healer, None);
        assert_eq!(msg.source, Some("hazard:volatility".to_string()));

        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        assert!(
            timer.elapsed.abs() < 1e-5,
            "timer.elapsed should roll over to ~0.0, got {}",
            timer.elapsed
        );
    }

    // Behavior 1 edge — 4.999s tick
    #[test]
    fn no_heal_before_interval_crossed() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(4.999));

        assert_eq!(heal_collector_len(&app), 0);
        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        assert!(
            (timer.elapsed - 4.999).abs() < 1e-3,
            "timer.elapsed should be ~4.999, got {}",
            timer.elapsed
        );
    }

    // Behavior 1 edge — five 1.0s ticks accumulate to one heal
    #[test]
    fn five_1s_ticks_accumulate_to_one_heal() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        let mut emitted = 0usize;
        for _ in 0..5 {
            tick_with_dt(&mut app, Duration::from_secs_f32(1.0));
            emitted += heals_for(&app, cell).len();
        }

        assert_eq!(
            emitted, 1,
            "exactly one HealDealt<Cell> should be emitted across five 1s ticks"
        );
        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        assert!(
            timer.elapsed.abs() < 1e-5,
            "timer.elapsed should roll over to ~0.0 after tick 5, got {}",
            timer.elapsed
        );
    }

    // Behavior 2 — HealCap::Max variant
    #[test]
    fn heal_cap_variant_is_max() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 1);
        assert_eq!(heals[0].cap, HealCap::Max);
    }

    // Behavior 2 edge — source string exact match
    #[test]
    fn heal_source_is_hazard_volatility() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 1);
        assert_eq!(heals[0].source, Some("hazard:volatility".to_string()));
    }

    // Behavior 3 — no heal at cap; timer still rolls over
    #[test]
    fn no_heal_emitted_at_max_cap_but_timer_rolls_over() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer_max(&mut app, 20.0, 10.0, Some(20.0), 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

        assert_eq!(heal_collector_len(&app), 0);
        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        assert!(
            timer.elapsed.abs() < 1e-5,
            "timer.elapsed should roll over to ~0.0 even at cap, got {}",
            timer.elapsed
        );
    }

    // Behavior 3 edge — slightly above cap, no message
    #[test]
    fn no_heal_emitted_slightly_above_cap() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        spawn_cell_with_timer_max(&mut app, 20.000_001, 10.0, Some(20.0), 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

        assert_eq!(heal_collector_len(&app), 0);
    }

    // Behavior 3 edge — just under cap, exactly one heal
    #[test]
    fn heal_emitted_just_under_cap() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer_max(&mut app, 19.999, 10.0, Some(20.0), 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 1);
        assert!((heals[0].amount - 1.0).abs() < f32::EPSILON);
        assert_eq!(heals[0].cap, HealCap::Max);
    }

    // Behavior 3 edge — two sequential ticks at cap, no emissions either tick
    #[test]
    fn at_cap_two_ticks_emit_zero_heals() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        spawn_cell_with_timer_max(&mut app, 20.0, 10.0, Some(20.0), 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));
        assert_eq!(heal_collector_len(&app), 0, "after tick 1 at cap");

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));
        assert_eq!(heal_collector_len(&app), 0, "after tick 2 at cap");
    }

    // Behavior 3 edge — ten sequential 5s ticks baseline, ten emissions
    #[test]
    fn ten_sequential_ticks_under_cap_emit_ten_heals() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        let mut total = 0usize;
        for _ in 0..10 {
            tick_with_dt(&mut app, Duration::from_secs_f32(5.0));
            let heals = heals_for(&app, cell);
            for msg in &heals {
                assert!((msg.amount - 1.0).abs() < f32::EPSILON);
                assert_eq!(msg.cap, HealCap::Max);
            }
            total += heals.len();
        }
        assert_eq!(
            total, 10,
            "expected exactly 10 HealDealt<Cell> across 10 ticks"
        );
    }

    // Behavior 4 — two cells, two distinct heals
    #[test]
    fn heal_target_matches_cell_entity_two_cells() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let a = spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.9);
        let b = spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.9);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.2));

        let collector = &app
            .world()
            .resource::<MessageCollector<HealDealt<Cell>>>()
            .0;
        assert_eq!(collector.len(), 2, "expected exactly two HealDealt<Cell>");
        for msg in collector {
            assert!((msg.amount - 1.0).abs() < f32::EPSILON);
            assert_eq!(msg.cap, HealCap::Max);
        }
        let targets: std::collections::HashSet<Entity> =
            collector.iter().map(|m| m.target).collect();
        let expected: std::collections::HashSet<Entity> = [a, b].into_iter().collect();
        assert_eq!(targets, expected);
    }

    // Behavior 4 edge — four cells
    #[test]
    fn heal_target_matches_cell_entity_four_cells() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cells = [
            spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.9),
            spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.9),
            spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.9),
            spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.9),
        ];

        tick_with_dt(&mut app, Duration::from_secs_f32(0.2));

        let collector = &app
            .world()
            .resource::<MessageCollector<HealDealt<Cell>>>()
            .0;
        assert_eq!(collector.len(), 4);
        let targets: std::collections::HashSet<Entity> =
            collector.iter().map(|m| m.target).collect();
        assert_eq!(targets.len(), 4, "each target should be distinct");
        for c in cells {
            assert!(targets.contains(&c));
        }
    }

    // Behavior 5 — dead cell (current == 0)
    #[test]
    fn dead_cell_receives_no_heal() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 0.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

        assert!(heals_for(&app, cell).is_empty());
    }

    // Behavior 5 edge — negative current (post-overkill)
    #[test]
    fn dead_cell_negative_current_receives_no_heal() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, -1.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

        assert!(heals_for(&app, cell).is_empty());
    }

    // Behavior 5 edge — mixed batch (dead + living)
    #[test]
    fn mixed_dead_and_living_only_living_gets_heal() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let dead = spawn_cell_with_timer(&mut app, 0.0, 10.0, 0.0);
        let living = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

        assert!(heals_for(&app, dead).is_empty());
        let living_heals = heals_for(&app, living);
        assert_eq!(living_heals.len(), 1);
        assert_eq!(living_heals[0].target, living);
    }

    // ══════════════════════════════════════════════════════════════════════
    // Group B — stacking and interval floor
    // ══════════════════════════════════════════════════════════════════════

    // Behavior 6 — stack=3, interval ≈ 3.333s
    #[test]
    fn stack_3_emits_heal_after_effective_interval() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 3);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(3.3334));

        let heals = heals_for(&app, cell);
        assert_eq!(
            heals.len(),
            1,
            "expected one HealDealt<Cell> at stack=3 after 3.3334s"
        );
        assert!((heals[0].amount - 1.0).abs() < f32::EPSILON);
        assert_eq!(heals[0].cap, HealCap::Max);
        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        assert!(
            timer.elapsed.abs() < 1e-3,
            "timer.elapsed should roll over to ~0.0, got {}",
            timer.elapsed
        );
    }

    // Behavior 6 edge — just below interval at stack 3
    #[test]
    fn stack_3_just_below_interval_emits_zero() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 3);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(3.333));

        assert_eq!(heal_collector_len(&app), 0);
        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        assert!(
            (timer.elapsed - 3.333).abs() < 1e-3,
            "timer.elapsed should be ~3.333, got {}",
            timer.elapsed
        );
    }

    // Behavior 6 edge — two intervals exactly at stack 3
    #[test]
    fn stack_3_two_intervals_emits_two_heals() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 3);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(6.667));

        let heals = heals_for(&app, cell);
        assert_eq!(
            heals.len(),
            2,
            "two intervals at stack=3 should emit two heals"
        );
        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        assert!(
            timer.elapsed.abs() < 1e-3,
            "timer.elapsed should be ~0.0, got {}",
            timer.elapsed
        );
    }

    // Behavior 7 — stack=2, interval = 4.0s
    #[test]
    fn stack_2_emits_heal_after_4_seconds() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 2);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(4.0));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 1);
        assert!((heals[0].amount - 1.0).abs() < f32::EPSILON);
        assert_eq!(heals[0].cap, HealCap::Max);
    }

    // Behavior 7 edge — just below 4.0s
    #[test]
    fn stack_2_just_below_interval_emits_zero() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 2);
        register_volatility_systems(&mut app);
        spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(3.999));

        assert_eq!(heal_collector_len(&app), 0);
    }

    // Behavior 7 edge — just above 4.0s
    #[test]
    fn stack_2_just_above_interval_emits_one() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 2);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(4.001));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 1);
    }

    // Behavior 8 — stack=100 floored at 1.0s, 0.999s emits zero
    #[test]
    fn stack_100_floor_0_999s_emits_zero() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 100);
        register_volatility_systems(&mut app);
        spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.999));

        assert_eq!(heal_collector_len(&app), 0);
    }

    // Behavior 8 edge — exactly 1.0s emits one heal
    #[test]
    fn stack_100_floor_1_0s_emits_one() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 100);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 1);
        assert!((heals[0].amount - 1.0).abs() < f32::EPSILON);
        assert_eq!(heals[0].cap, HealCap::Max);
    }

    // Behavior 8 edge — 0.5s tick still no heal
    #[test]
    fn stack_100_half_second_no_heal() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 100);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.5));

        assert_eq!(heal_collector_len(&app), 0);
        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        assert!(
            (timer.elapsed - 0.5).abs() < 1e-3,
            "timer.elapsed should be ~0.5, got {}",
            timer.elapsed
        );
    }

    // Behavior 8 edge — stack=1000 also floors at 1.0s
    #[test]
    fn stack_1000_floor_is_exactly_1_0s() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1000);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 1, "stack=1000 should still use the 1.0s floor");
    }

    // Behavior 9 — stack=0 (hazard inactive)
    #[test]
    fn stack_0_inactive_hazard_no_timer_no_heal_no_max_change() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        // ActiveHazards::default() — no Volatility stacks.
        register_volatility_systems(&mut app);
        let cell = spawn_cell_no_timer(&mut app, 5.0, 5.0, None);

        tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

        assert!(
            app.world().get::<VolatilityTimer>(cell).is_none(),
            "no VolatilityTimer should be attached when hazard inactive"
        );
        let hp = app.world().get::<Hp>(cell).unwrap();
        assert!((hp.current - 5.0).abs() < f32::EPSILON);
        assert!(
            hp.max.is_none(),
            "Hp.max should remain None when hazard inactive"
        );
        assert_eq!(heal_collector_len(&app), 0);
    }

    // ══════════════════════════════════════════════════════════════════════
    // Group C — reset_volatility_on_damage
    // ══════════════════════════════════════════════════════════════════════

    // Behavior 10 — damage resets timer to 0.0
    #[test]
    fn damage_resets_timer_to_zero() {
        let mut app = test_app_playing_with_damage();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 8.0, 10.0, 4.5);

        app.world_mut()
            .resource_mut::<PendingCellDamage>()
            .0
            .push((cell, 2.0));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        // After a reset in the same tick, grow advances elapsed by dt.
        // Tolerate up to one fixed-update tick's worth of accumulation.
        assert!(
            timer.elapsed < 0.02,
            "timer.elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
            timer.elapsed
        );
        let hp = app.world().get::<Hp>(cell).unwrap();
        assert!(
            (hp.current - 6.0).abs() < f32::EPSILON,
            "Hp.current should be 6.0 after 2.0 damage on 8.0, got {}",
            hp.current
        );
    }

    // Behavior 10 edge — reset from 0.0 stays 0.0
    #[test]
    fn damage_reset_from_zero_is_idempotent() {
        let mut app = test_app_playing_with_damage();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        app.world_mut()
            .resource_mut::<PendingCellDamage>()
            .0
            .push((cell, 1.0));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        // After a reset in the same tick, grow advances elapsed by dt.
        // Tolerate up to one fixed-update tick's worth of accumulation.
        assert!(
            timer.elapsed < 0.02,
            "timer.elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
            timer.elapsed
        );
    }

    // Behavior 11 — batch reset: damaged cells reset, undamaged cell advances
    #[test]
    fn batch_damage_only_damaged_cells_reset() {
        let mut app = test_app_playing_with_damage();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let a = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);
        let b = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);
        let c = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);

        {
            let mut pending = app.world_mut().resource_mut::<PendingCellDamage>();
            pending.0.push((a, 1.0));
            pending.0.push((c, 1.0));
        }
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let ta = app.world().get::<VolatilityTimer>(a).unwrap();
        let tc = app.world().get::<VolatilityTimer>(c).unwrap();
        // After a reset in the same tick, grow advances elapsed by dt.
        // Tolerate up to one fixed-update tick's worth of accumulation.
        assert!(
            ta.elapsed < 0.02,
            "A.elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
            ta.elapsed
        );
        assert!(
            tc.elapsed < 0.02,
            "C.elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
            tc.elapsed
        );
        let tb = app.world().get::<VolatilityTimer>(b).unwrap();
        assert!(
            tb.elapsed >= 3.0 && tb.elapsed <= 3.016 + 1e-4,
            "B.elapsed should be in [3.0, 3.016+eps], got {}",
            tb.elapsed
        );
    }

    // Behavior 11 edge — all three damaged, all reset
    #[test]
    fn batch_all_three_damaged_all_reset() {
        let mut app = test_app_playing_with_damage();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let a = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);
        let b = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);
        let c = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);

        {
            let mut pending = app.world_mut().resource_mut::<PendingCellDamage>();
            pending.0.push((a, 1.0));
            pending.0.push((b, 1.0));
            pending.0.push((c, 1.0));
        }
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        for entity in [a, b, c] {
            let t = app.world().get::<VolatilityTimer>(entity).unwrap();
            // After a reset in the same tick, grow advances elapsed by dt.
            // Tolerate up to one fixed-update tick's worth of accumulation.
            assert!(
                t.elapsed < 0.02,
                "elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
                t.elapsed
            );
        }
    }

    // Behavior 11 edge — zero damage, all advance
    #[test]
    fn batch_zero_damaged_all_advance() {
        let mut app = test_app_playing_with_damage();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let a = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);
        let b = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);
        let c = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        for entity in [a, b, c] {
            let t = app.world().get::<VolatilityTimer>(entity).unwrap();
            assert!(
                t.elapsed >= 3.0 && t.elapsed <= 3.016 + 1e-4,
                "elapsed should be in [3.0, 3.016+eps], got {}",
                t.elapsed
            );
        }
    }

    // Behavior 12 — ordering: reset runs after apply_damage
    #[test]
    fn reset_runs_after_apply_damage_both_effects_applied() {
        let mut app = test_app_playing_with_damage();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 2.0);

        app.world_mut()
            .resource_mut::<PendingCellDamage>()
            .0
            .push((cell, 3.0));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert!(
            (hp.current - 7.0).abs() < f32::EPSILON,
            "Hp.current should be 7.0 after 3.0 damage, got {}",
            hp.current
        );
        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        // After a reset in the same tick, grow advances elapsed by dt.
        // Tolerate up to one fixed-update tick's worth of accumulation.
        assert!(
            timer.elapsed < 0.02,
            "timer.elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
            timer.elapsed
        );
    }

    // Behavior 13 — zero-damage hit still resets the timer
    #[test]
    fn zero_damage_hit_resets_timer() {
        let mut app = test_app_playing_with_damage();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.8);

        app.world_mut()
            .resource_mut::<PendingCellDamage>()
            .0
            .push((cell, 0.0));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        // After a reset in the same tick, grow advances elapsed by dt.
        // Tolerate up to one fixed-update tick's worth of accumulation.
        assert!(
            timer.elapsed < 0.02,
            "timer.elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
            timer.elapsed
        );
        let hp = app.world().get::<Hp>(cell).unwrap();
        assert!(
            (hp.current - 10.0).abs() < f32::EPSILON,
            "zero damage should leave Hp unchanged at 10.0, got {}",
            hp.current
        );
    }

    // Behavior 13 edge — negative-zero damage resets
    #[test]
    fn negative_zero_damage_resets_timer() {
        let mut app = test_app_playing_with_damage();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.8);

        app.world_mut()
            .resource_mut::<PendingCellDamage>()
            .0
            .push((cell, -0.0));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        // After a reset in the same tick, grow advances elapsed by dt.
        // Tolerate up to one fixed-update tick's worth of accumulation.
        assert!(
            timer.elapsed < 0.02,
            "timer.elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
            timer.elapsed
        );
    }

    // Behavior 13 edge — NaN damage still resets
    #[test]
    fn nan_damage_resets_timer() {
        let mut app = test_app_playing_with_damage();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.8);

        app.world_mut()
            .resource_mut::<PendingCellDamage>()
            .0
            .push((cell, f32::NAN));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        // After a reset in the same tick, grow advances elapsed by dt.
        // Tolerate up to one fixed-update tick's worth of accumulation.
        assert!(
            timer.elapsed < 0.02,
            "timer.elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
            timer.elapsed
        );
    }

    // Behavior 14 — reset silently skips targets lacking VolatilityTimer
    #[test]
    fn reset_silently_skips_cell_without_timer() {
        let mut app = test_app_playing_with_damage();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        // Register only reset (we deliberately skip the attach system so the
        // cell genuinely has no VolatilityTimer when damage arrives).
        app.add_systems(
            FixedUpdate,
            reset_volatility_on_damage
                .after(DeathPipelineSystems::ApplyDamage)
                .run_if(hazard_active(HazardKind::Volatility))
                .run_if(in_state(NodeState::Playing)),
        );
        let cell = app
            .world_mut()
            .spawn((
                Cell,
                Hp {
                    current:  5.0,
                    starting: 10.0,
                    max:      None,
                },
                KilledBy::default(),
            ))
            .id();

        app.world_mut()
            .resource_mut::<PendingCellDamage>()
            .0
            .push((cell, 1.0));
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        assert!(app.world().get::<VolatilityTimer>(cell).is_none());
        let hp = app.world().get::<Hp>(cell).unwrap();
        assert!((hp.current - 4.0).abs() < f32::EPSILON);
    }

    // Behavior 14 edge — target is non-cell (a bare entity, not a Cell)
    #[test]
    fn reset_silently_skips_non_cell_target() {
        let mut app = test_app_playing_with_damage();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        app.add_systems(
            FixedUpdate,
            reset_volatility_on_damage
                .after(DeathPipelineSystems::ApplyDamage)
                .run_if(hazard_active(HazardKind::Volatility))
                .run_if(in_state(NodeState::Playing)),
        );
        let not_a_cell = app.world_mut().spawn_empty().id();

        app.world_mut()
            .resource_mut::<PendingCellDamage>()
            .0
            .push((not_a_cell, 1.0));
        // Should not panic.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    }

    // ══════════════════════════════════════════════════════════════════════
    // Group D — attach_volatility_timers
    // ══════════════════════════════════════════════════════════════════════

    // Behavior 15 — attach inserts VolatilityTimer when hazard active
    #[test]
    fn attach_inserts_timer_on_cells_when_hazard_active() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_no_timer(&mut app, 10.0, 10.0, None);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        // After an attach in the same tick, grow advances elapsed by dt.
        // Tolerate up to one fixed-update tick's worth of accumulation.
        assert!(
            timer.elapsed < 0.02,
            "timer.elapsed should be near 0.0 after attach (≤ one tick's dt), got {}",
            timer.elapsed
        );
        let hp = app.world().get::<Hp>(cell).unwrap();
        assert_eq!(
            hp.max,
            Some(20.0),
            "Hp.max should be lifted to Some(20.0), got {:?}",
            hp.max
        );
    }

    // Behavior 15 edge — idempotent: pre-existing timer not overwritten,
    // but Hp.max is still lifted.
    #[test]
    fn attach_idempotent_preserves_existing_timer_but_lifts_hp_max() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        // Cell with pre-existing timer at elapsed=2.0 and Hp.max=None.
        let cell = app
            .world_mut()
            .spawn((
                Cell,
                Hp {
                    current:  10.0,
                    starting: 10.0,
                    max:      None,
                },
                VolatilityTimer { elapsed: 2.0 },
            ))
            .id();

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        // Timer insert was skipped; elapsed may advance by dt if grow ran,
        // but must start from the pre-existing 2.0.
        assert!(
            timer.elapsed >= 2.0 - 1e-5,
            "existing timer must not be overwritten to 0.0, got {}",
            timer.elapsed
        );
        let hp = app.world().get::<Hp>(cell).unwrap();
        assert_eq!(
            hp.max,
            Some(20.0),
            "Hp.max should still be lifted to Some(20.0)"
        );
    }

    // Behavior 16 — attach inactive hazard does nothing
    #[test]
    fn attach_inactive_hazard_does_not_insert_timer_or_lift_max() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        // No Volatility stacks.
        register_volatility_systems(&mut app);
        let cell = spawn_cell_no_timer(&mut app, 10.0, 10.0, None);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        assert!(app.world().get::<VolatilityTimer>(cell).is_none());
        let hp = app.world().get::<Hp>(cell).unwrap();
        assert!(
            hp.max.is_none(),
            "Hp.max should remain None, got {:?}",
            hp.max
        );
        assert_eq!(heal_collector_len(&app), 0);
    }

    // Behavior 16 edge — hazard becomes active mid-run
    #[test]
    fn attach_reacts_when_hazard_becomes_active_mid_run() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_no_timer(&mut app, 10.0, 10.0, None);

        // Tick 1 — no timer, no lift.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
        assert!(app.world().get::<VolatilityTimer>(cell).is_none());
        assert!(app.world().get::<Hp>(cell).unwrap().max.is_none());

        // Activate mid-run.
        add_volatility_stacks(&mut app, 1);

        // Tick 2.
        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        // After an attach in the same tick, grow advances elapsed by dt.
        // Tolerate up to one fixed-update tick's worth of accumulation.
        assert!(
            timer.elapsed < 0.02,
            "timer.elapsed should be near 0.0 after attach (≤ one tick's dt), got {}",
            timer.elapsed
        );
        assert_eq!(app.world().get::<Hp>(cell).unwrap().max, Some(20.0));
    }

    // Behavior 17 — cells spawned mid-node receive timer on next tick
    #[test]
    fn cells_spawned_mid_node_receive_timer_on_next_tick() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let a = spawn_cell_no_timer(&mut app, 10.0, 10.0, None);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
        assert!(app.world().get::<VolatilityTimer>(a).is_some());
        assert_eq!(app.world().get::<Hp>(a).unwrap().max, Some(20.0));

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let b = spawn_cell_no_timer(&mut app, 5.0, 5.0, None);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let timer_b = app.world().get::<VolatilityTimer>(b).unwrap();
        // After an attach in the same tick, grow advances elapsed by dt.
        // Tolerate up to one fixed-update tick's worth of accumulation.
        assert!(
            timer_b.elapsed < 0.02,
            "timer_b.elapsed should be near 0.0 after attach (≤ one tick's dt), got {}",
            timer_b.elapsed
        );
        assert_eq!(app.world().get::<Hp>(b).unwrap().max, Some(10.0));
        // A retains its timer and max.
        assert!(app.world().get::<VolatilityTimer>(a).is_some());
        assert_eq!(app.world().get::<Hp>(a).unwrap().max, Some(20.0));
    }

    // Behavior 17 edge — cell B spawned with pre-existing timer
    #[test]
    fn mid_node_cell_with_existing_timer_keeps_it_but_gets_max_lifted() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let b = app
            .world_mut()
            .spawn((
                Cell,
                Hp {
                    current:  5.0,
                    starting: 5.0,
                    max:      None,
                },
                VolatilityTimer { elapsed: 2.0 },
            ))
            .id();

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let timer_b = app.world().get::<VolatilityTimer>(b).unwrap();
        assert!(
            timer_b.elapsed >= 2.0 - 1e-5,
            "existing timer must not be overwritten to 0.0, got {}",
            timer_b.elapsed
        );
        assert_eq!(app.world().get::<Hp>(b).unwrap().max, Some(10.0));
    }

    // Behavior 18 — Hp.max lift from None to Some(2 * starting)
    #[test]
    fn attach_lifts_none_max_to_2x_starting() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_no_timer(&mut app, 5.0, 10.0, None);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert_eq!(hp.max, Some(20.0));
        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        // After an attach in the same tick, grow advances elapsed by dt.
        // Tolerate up to one fixed-update tick's worth of accumulation.
        assert!(
            timer.elapsed < 0.02,
            "timer.elapsed should be near 0.0 after attach (≤ one tick's dt), got {}",
            timer.elapsed
        );
    }

    // Behavior 18 edge — starting = 0.0 yields Hp.max = Some(0.0)
    #[test]
    fn attach_zero_starting_yields_zero_max() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_no_timer(&mut app, 0.0, 0.0, None);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert_eq!(hp.max, Some(0.0));
    }

    // Behavior 18 edge — max_multiplier = 1.0 yields Hp.max = Some(starting)
    #[test]
    fn attach_multiplier_one_yields_starting_max() {
        let mut app = test_app_playing();
        app.world_mut().insert_resource(VolatilityConfig {
            hp_per_interval: 1.0,
            interval_secs:   5.0,
            max_multiplier:  1.0,
        });
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_no_timer(&mut app, 5.0, 10.0, None);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert_eq!(hp.max, Some(10.0));
    }

    // Behavior 19 — higher existing max preserved
    #[test]
    fn attach_preserves_higher_existing_max() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_no_timer(&mut app, 5.0, 10.0, Some(30.0));

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert_eq!(
            hp.max,
            Some(30.0),
            "existing higher max must not be lowered"
        );
        assert!(app.world().get::<VolatilityTimer>(cell).is_some());
    }

    // Behavior 19 edge — existing max equals 2x starting
    #[test]
    fn attach_preserves_max_equal_to_2x_starting() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_no_timer(&mut app, 5.0, 10.0, Some(20.0));

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert_eq!(hp.max, Some(20.0));
    }

    // Behavior 19 edge — existing max is infinity
    #[test]
    fn attach_preserves_infinite_existing_max() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_no_timer(&mut app, 5.0, 10.0, Some(f32::INFINITY));

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert_eq!(hp.max, Some(f32::INFINITY));
    }

    // Behavior 20 — lower existing max raised to 2x starting
    #[test]
    fn attach_raises_lower_existing_max_to_2x_starting() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_no_timer(&mut app, 5.0, 10.0, Some(12.0));

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert_eq!(hp.max, Some(20.0));
        assert!(app.world().get::<VolatilityTimer>(cell).is_some());
    }

    // Behavior 20 edge — slightly below 2x starting
    #[test]
    fn attach_raises_19_999_to_20() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_no_timer(&mut app, 5.0, 10.0, Some(19.999));

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert_eq!(hp.max, Some(20.0));
    }

    // Behavior 20 edge — existing max = 0.0 is raised to 2x starting
    #[test]
    fn attach_raises_zero_existing_max_to_2x_starting() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_no_timer(&mut app, 5.0, 10.0, Some(0.0));

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert_eq!(hp.max, Some(20.0));
    }

    // ══════════════════════════════════════════════════════════════════════
    // Group E — multi-interval accumulation (pause/resume)
    // ══════════════════════════════════════════════════════════════════════

    // Behavior 21 — 12.5s tick emits two heals, elapsed ≈ 2.5
    #[test]
    fn tick_12_5s_emits_two_heals_and_remainder() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(12.5));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 2);
        for msg in &heals {
            assert!((msg.amount - 1.0).abs() < f32::EPSILON);
            assert_eq!(msg.cap, HealCap::Max);
        }
        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        assert!(
            (timer.elapsed - 2.5).abs() < 1e-5,
            "timer.elapsed should be ~2.5, got {}",
            timer.elapsed
        );
    }

    // Behavior 21 edge — exactly three intervals
    #[test]
    fn tick_15_0s_emits_three_heals_and_rolls_over() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(15.0));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 3);
        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        assert!(timer.elapsed.abs() < 1e-5);
    }

    // Behavior 21 edge — 5.0 + EPSILON emits exactly one
    #[test]
    fn tick_5_0_plus_epsilon_emits_one() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0 + f32::EPSILON));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 1, "5.0 + EPSILON should emit exactly one heal");
    }

    // Behavior 21 edge — 50.0s emits ten heals
    #[test]
    fn tick_50s_emits_ten_heals() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(50.0));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 10);
        for msg in &heals {
            assert!((msg.amount - 1.0).abs() < f32::EPSILON);
            assert_eq!(msg.cap, HealCap::Max);
        }
    }

    // Behavior 22 — per-iteration cap check from live Hp; two emissions at current=19.0
    #[test]
    fn per_iteration_cap_check_reads_live_hp_current_19() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 19.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(10.0));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 2);
        for msg in &heals {
            assert!((msg.amount - 1.0).abs() < f32::EPSILON);
            assert_eq!(msg.cap, HealCap::Max);
        }
    }

    // Behavior 22 edge — at cap, two intervals emit zero
    #[test]
    fn per_iteration_at_cap_two_intervals_emit_zero() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer_max(&mut app, 20.0, 10.0, Some(20.0), 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(10.0));

        assert_eq!(heal_collector_len(&app), 0);
        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        assert!(timer.elapsed.abs() < 1e-5);
    }

    // Behavior 22 edge — base case at current=19.5
    #[test]
    fn per_iteration_current_19_5_two_emissions() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 19.5, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(10.0));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 2);
    }

    // ══════════════════════════════════════════════════════════════════════
    // Group F — cross-cutting
    // ══════════════════════════════════════════════════════════════════════

    // Behavior 23 — heal amount == hp_per_interval, not scaled by dt or stacks
    #[test]
    fn heal_amount_is_hp_per_interval_0_25() {
        let mut app = test_app_playing();
        app.world_mut().insert_resource(VolatilityConfig {
            hp_per_interval: 0.25,
            interval_secs:   5.0,
            max_multiplier:  2.0,
        });
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 1);
        assert!((heals[0].amount - 0.25).abs() < f32::EPSILON);
        assert_eq!(heals[0].cap, HealCap::Max);
    }

    // Behavior 23 edge — hp_per_interval = 3.0
    #[test]
    fn heal_amount_is_hp_per_interval_3_0() {
        let mut app = test_app_playing();
        app.world_mut().insert_resource(VolatilityConfig {
            hp_per_interval: 3.0,
            interval_secs:   5.0,
            max_multiplier:  2.0,
        });
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 1);
        assert!((heals[0].amount - 3.0).abs() < f32::EPSILON);
        assert_eq!(heals[0].cap, HealCap::Max);
    }

    // Behavior 23 edge — stacks change interval, not amount
    #[test]
    fn stacks_change_interval_not_amount() {
        let mut app = test_app_playing();
        app.world_mut().insert_resource(VolatilityConfig {
            hp_per_interval: 3.0,
            interval_secs:   5.0,
            max_multiplier:  2.0,
        });
        add_volatility_stacks(&mut app, 5);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        // Effective interval at stack=5: 5.0 / (1 + 0.25*4) = 5.0 / 2.0 = 2.5s.
        tick_with_dt(&mut app, Duration::from_secs_f32(2.5));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 1);
        assert!((heals[0].amount - 3.0).abs() < f32::EPSILON);
        assert_eq!(heals[0].cap, HealCap::Max);
    }

    // Behavior 24 — healer == None
    #[test]
    fn heal_healer_is_none() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

        let heals = heals_for(&app, cell);
        assert_eq!(heals.len(), 1);
        assert_eq!(heals[0].healer, None);
    }

    // Behavior 25 — message lands in HealDealt<Cell>, not other T's
    #[test]
    fn heal_lands_in_cell_queue_not_other_monomorphizations() {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .in_state_node_playing()
            .with_resource::<ActiveHazards>()
            .with_message::<DamageDealt<Cell>>()
            .with_message_capture::<HealDealt<Cell>>()
            .with_message_capture::<HealDealt<Bolt>>()
            .with_message_capture::<HealDealt<Breaker>>()
            .with_message_capture::<HealDealt<Wall>>()
            .with_message_capture::<HealDealt<Salvo>>()
            .build();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

        assert_eq!(
            app.world()
                .resource::<MessageCollector<HealDealt<Cell>>>()
                .0
                .len(),
            1
        );
        assert!(
            app.world()
                .resource::<MessageCollector<HealDealt<Bolt>>>()
                .0
                .is_empty()
        );
        assert!(
            app.world()
                .resource::<MessageCollector<HealDealt<Breaker>>>()
                .0
                .is_empty()
        );
        assert!(
            app.world()
                .resource::<MessageCollector<HealDealt<Wall>>>()
                .0
                .is_empty()
        );
        assert!(
            app.world()
                .resource::<MessageCollector<HealDealt<Salvo>>>()
                .0
                .is_empty()
        );
    }

    // Behavior 26 — Volatility + Renewal active; Volatility's source tag is preserved
    #[test]
    fn two_hazards_active_volatility_source_tag_preserved() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        app.world_mut().insert_resource(RenewalConfig {
            base_period_secs:         10.0,
            per_level_reduction_frac: 0.2,
        });
        add_volatility_stacks(&mut app, 1);
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Renewal);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

        let heals: Vec<_> = heals_for(&app, cell)
            .into_iter()
            .filter(|m| m.source == Some("hazard:volatility".to_string()))
            .collect();
        assert_eq!(heals.len(), 1);
        assert_eq!(heals[0].cap, HealCap::Max);
        assert_eq!(heals[0].source, Some("hazard:volatility".to_string()));
    }

    // Behavior 27 — timer advances when at cap, pre-send gate blocks only emit
    #[test]
    fn timer_still_advances_when_at_cap() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer_max(&mut app, 20.0, 10.0, Some(20.0), 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(2.5));

        assert_eq!(heal_collector_len(&app), 0);
        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        assert!(
            (timer.elapsed - 2.5).abs() < 1e-5,
            "timer.elapsed should be ~2.5, got {}",
            timer.elapsed
        );
    }

    // Behavior 27 edge — second 2.5s tick from elapsed≈2.5 rolls over, still 0 messages
    #[test]
    fn at_cap_second_tick_crosses_interval_still_zero_emissions() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer_max(&mut app, 20.0, 10.0, Some(20.0), 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(2.5));
        tick_with_dt(&mut app, Duration::from_secs_f32(2.5));

        assert_eq!(heal_collector_len(&app), 0);
        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        assert!(
            timer.elapsed.abs() < 1e-5,
            "timer.elapsed should roll over to ~0.0, got {}",
            timer.elapsed
        );
    }

    // Behavior 27 edge — at cap, 10.0s (two intervals) still emits zero
    #[test]
    fn at_cap_two_intervals_emit_zero_and_roll_over() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_with_timer_max(&mut app, 20.0, 10.0, Some(20.0), 0.0);

        tick_with_dt(&mut app, Duration::from_secs_f32(10.0));

        assert_eq!(heal_collector_len(&app), 0);
        let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
        assert!(timer.elapsed.abs() < 1e-5);
    }

    // Behavior 28 — Hp.max NOT restored when hazard deactivates
    #[test]
    fn hp_max_not_restored_on_deactivation() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_no_timer(&mut app, 10.0, 10.0, None);

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
        assert_eq!(app.world().get::<Hp>(cell).unwrap().max, Some(20.0));

        // Deactivate via backdoor.
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .force_insert_entry(HazardKind::Volatility, 0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert_eq!(
            hp.max,
            Some(20.0),
            "Hp.max must remain lifted after deactivation (no restore)"
        );
        assert_eq!(heal_collector_len(&app), 0);
    }

    // Behavior 28 edge — pre-existing game max (Some(15.0)) lifted to Some(20.0), not restored
    #[test]
    fn hp_max_not_restored_even_when_original_was_some() {
        let mut app = test_app_playing();
        install_default_config(&mut app);
        add_volatility_stacks(&mut app, 1);
        register_volatility_systems(&mut app);
        let cell = spawn_cell_no_timer(&mut app, 10.0, 10.0, Some(15.0));

        tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
        assert_eq!(
            app.world().get::<Hp>(cell).unwrap().max,
            Some(20.0),
            "Hp.max should be lifted from 15.0 to 20.0 on first tick"
        );

        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .force_insert_entry(HazardKind::Volatility, 0);

        tick_with_dt(&mut app, Duration::from_secs_f32(5.0));

        let hp = app.world().get::<Hp>(cell).unwrap();
        assert_eq!(
            hp.max,
            Some(20.0),
            "Hp.max must NOT be restored to Some(15.0) after deactivation"
        );
    }

    // ══════════════════════════════════════════════════════════════════════
    // activate — preserved scaffold test (mismatched tuning)
    // ══════════════════════════════════════════════════════════════════════

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
        assert!(app.world().get_resource::<VolatilityConfig>().is_none());
    }
}
