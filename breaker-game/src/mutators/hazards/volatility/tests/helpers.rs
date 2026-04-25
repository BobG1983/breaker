use std::{marker::PhantomData, time::Duration};

use bevy::prelude::*;

use super::super::system::*;
use crate::{
    cells::components::Cell,
    mutators::hazards::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
};

// ── Helpers ────────────────────────────────────────────────────────────

/// Default builder for Groups A/B/D/E/F. Registers `ActiveHazards`,
/// `DamageDealt<Cell>` + `HealDealt<Cell>` messages, and the heal
/// capture collector.
pub(super) fn test_app_playing() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message::<DamageDealt<Cell>>()
        .with_message_capture::<HealDealt<Cell>>()
        .build()
}

/// Variant for Group C tests. Adds `PendingCellDamage` + `enqueue_cell_damage`
/// ordered `.before(DmgSystems::ApplyDamage)` and wires
/// `apply_damage::<Cell>` via `register_dmgable::<Cell>` so the
/// reset-ordering behaviours can be exercised end-to-end.
pub(super) fn test_app_playing_with_damage() -> App {
    use rantzsoft_dmg::{RantzDmgAppExt, RantzDmgPlugin};
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .with_message_capture::<HealDealt<Cell>>()
        .with_resource::<PendingCellDamage>()
        .build();
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<Cell>();
    app.add_systems(
        FixedUpdate,
        enqueue_cell_damage.before(DmgSystems::ApplyDamage),
    );
    app
}

/// Spawns a cell in the post-attach state: `Hp.max = Some(starting * 2.0)`
/// with a `VolatilityTimer { elapsed }` already attached. `KilledBy` is
/// included so `apply_damage::<Cell>` can process damage in Group C tests.
pub(super) fn spawn_cell_with_timer(
    app: &mut App,
    current: f32,
    starting: f32,
    elapsed: f32,
) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Hp {
                current,
                starting,
                max: Some(starting * 2.0),
            },
            KilledBy { killer: None },
            VolatilityTimer { elapsed },
        ))
        .id()
}

/// Variant that accepts an explicit `max` — used for cells constructed
/// with a specific cap (e.g., at the 2× cap or slightly under/over it).
pub(super) fn spawn_cell_with_timer_max(
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
            KilledBy { killer: None },
            VolatilityTimer { elapsed },
        ))
        .id()
}

/// Spawns a cell WITHOUT `VolatilityTimer` — used for Group D tests that
/// exercise `attach_volatility_timers` itself.
pub(super) fn spawn_cell_no_timer(
    app: &mut App,
    current: f32,
    starting: f32,
    max: Option<f32>,
) -> Entity {
    app.world_mut()
        .spawn((
            Cell,
            Hp {
                current,
                starting,
                max,
            },
            KilledBy { killer: None },
        ))
        .id()
}

pub(super) fn tick_with_dt(app: &mut App, dt: Duration) {
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
/// `.before(DmgSystems::ApplyDamage)` by `test_app_playing_with_damage`.
pub(super) fn enqueue_cell_damage(
    mut pending: ResMut<PendingCellDamage>,
    mut writer: MessageWriter<DamageDealt<Cell>>,
) {
    for (target, amount) in pending.0.drain(..) {
        writer.write(DamageDealt {
            dealer: None,
            attributed_to: None,
            target,
            amount,
            source: None,
            _marker: PhantomData,
        });
    }
}

pub(super) fn heals_for(app: &App, target: Entity) -> Vec<HealDealt<Cell>> {
    app.world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0
        .iter()
        .filter(|m| m.target == target)
        .cloned()
        .collect()
}

pub(super) fn heal_collector_len(app: &App) -> usize {
    app.world()
        .resource::<MessageCollector<HealDealt<Cell>>>()
        .0
        .len()
}

pub(super) fn add_volatility_stacks(app: &mut App, count: u32) {
    let mut active = app.world_mut().resource_mut::<ActiveHazards>();
    for _ in 0..count {
        active.add_stack(HazardKind::Volatility);
    }
}

pub(super) fn default_config() -> VolatilityConfig {
    VolatilityConfig {
        hp_per_interval: 1.0,
        interval_secs:   5.0,
        max_multiplier:  2.0,
    }
}

pub(super) fn install_default_config(app: &mut App) {
    app.world_mut().insert_resource(default_config());
}

pub(super) fn register_volatility_systems(app: &mut App) {
    super::super::system::wire(app);
}
