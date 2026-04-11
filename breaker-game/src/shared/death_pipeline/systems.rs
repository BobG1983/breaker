//! Death pipeline systems — damage application, death detection, and despawn processing.
//!
//! All system bodies are `todo!()` stubs. Implementation deferred to Phase 2.

use bevy::prelude::*;

use super::game_entity::GameEntity;

/// Processes `DamageDealt<T>` messages, decrements `Hp`, and sets `KilledBy` on
/// the killing blow. Uses `Without<Dead>` to skip entities already confirmed dead.
///
/// Generic over the entity marker type — monomorphized for Cell, Bolt, Wall, Breaker.
pub fn apply_damage<T: GameEntity>() {
    todo!()
}

/// Detects cells with Hp <= 0 and sends `KillYourself<Cell>`.
pub fn detect_cell_deaths() {
    todo!()
}

/// Detects bolts with Hp <= 0 and sends `KillYourself<Bolt>`.
pub fn detect_bolt_deaths() {
    todo!()
}

/// Detects walls with Hp <= 0 and sends `KillYourself<Wall>`.
pub fn detect_wall_deaths() {
    todo!()
}

/// Detects breakers with Hp <= 0 and sends `KillYourself<Breaker>`.
pub fn detect_breaker_deaths() {
    todo!()
}

/// Processes `DespawnEntity` messages — despawns entities via `try_despawn`.
///
/// This is the ONLY system that despawns entities in the death pipeline.
/// Runs in `PostFixedUpdate`.
pub fn process_despawn_requests() {
    todo!()
}
