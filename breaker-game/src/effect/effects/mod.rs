//! Effect modules — one per effect, each with `fire()`, `reverse()`, `register()`.

pub(crate) mod fire_helpers;

pub(crate) use fire_helpers::{effective_range, entity_position, spawn_extra_bolt};

/// Steer toward nearest entity of a type.
pub mod attraction;
/// Flat bump force increase.
pub mod bump_force;
/// Spawn two bolts chained together.
pub mod chain_bolt;
/// Arc damage jumping between cells.
pub mod chain_lightning;
/// Multiplicative damage bonus.
pub mod damage_boost;
/// Escalating chaos — fires multiple random effects per cell destroyed.
pub mod entropy_engine;
/// Instant area damage burst.
pub mod explode;
/// Gravity well that attracts bolts within radius.
pub mod gravity_well;
/// Decrement lives.
pub mod life_lost;
/// Pass through cells instead of bouncing.
pub mod piercing;
/// Beam through cells in velocity direction.
pub mod piercing_beam;
/// Shockwave at every active bolt position.
pub mod pulse;
/// Breaker deceleration multiplier.
pub mod quick_stop;
/// Stacking damage bonus on consecutive cell hits.
pub mod ramping_damage;
/// Weighted random selection from a pool.
pub mod random_effect;
/// Invisible bottom wall that bounces bolt once.
pub mod second_wind;
/// Temporary breaker protection.
pub mod shield;
/// Expanding ring of area damage.
pub mod shockwave;
/// Size increase (bolt radius or breaker width).
pub mod size_boost;
/// Spawn additional bolts.
pub mod spawn_bolts;
/// Temporary phantom bolt with infinite piercing.
pub mod spawn_phantom;
/// Multiplicative speed scaling.
pub mod speed_boost;
/// Two bolts connected by a damaging beam.
pub mod tether_beam;
/// Subtract time from node timer.
pub mod time_penalty;

/// Register all effect runtime systems.
pub(crate) fn register(app: &mut bevy::prelude::App) {
    speed_boost::register(app);
    damage_boost::register(app);
    piercing::register(app);
    size_boost::register(app);
    bump_force::register(app);
    shockwave::register(app);
    chain_lightning::register(app);
    piercing_beam::register(app);
    pulse::register(app);
    shield::register(app);
    gravity_well::register(app);
    spawn_phantom::register(app);
    entropy_engine::register(app);
    ramping_damage::register(app);
    explode::register(app);
    spawn_bolts::register(app);
    chain_bolt::register(app);
    attraction::register(app);
    quick_stop::register(app);
    tether_beam::register(app);
    life_lost::register(app);
    time_penalty::register(app);
    second_wind::register(app);
    random_effect::register(app);
}
