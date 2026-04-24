use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    cells::components::Cell,
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
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
            reset_volatility_on_damage.after(DmgSystems::ApplyDamage),
            volatility_grow_cells.before(DmgSystems::ApplyHeal),
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
pub(crate) fn attach_volatility_timers(
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
pub(crate) fn reset_volatility_on_damage(
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
pub(crate) fn volatility_grow_cells(
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
                    healer:        None,
                    attributed_to: None,
                    target:        entity,
                    amount:        config.hp_per_interval,
                    cap:           HealCap::Max,
                    source:        Some(SourceId::from("hazard:volatility")),
                    _marker:       PhantomData,
                });
            }
            timer.elapsed -= effective_interval;
        }
    }
}
