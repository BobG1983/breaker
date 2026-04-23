//! Internal systems for the damage pipeline.

mod apply_damage;
mod apply_damage_boosts;
mod apply_heal;
mod apply_vulnerable;
mod despawn;
mod detect_deaths;
mod handle_kill;
mod invulnerable_filter;

pub(crate) use apply_damage::apply_damage;
pub(crate) use apply_damage_boosts::apply_damage_boosts;
pub(crate) use apply_heal::apply_heal;
pub(crate) use apply_vulnerable::apply_vulnerable;
pub(crate) use despawn::process_despawn_requests;
pub(crate) use detect_deaths::detect_deaths;
pub(crate) use handle_kill::handle_kill;
pub(crate) use invulnerable_filter::invulnerable_filter;
