use bevy::prelude::*;

/// Configuration parameters for firing the `CircuitBreaker` effect.
///
/// Bundles the multiple config fields into a single struct to keep `fire()`
/// under the argument count limit.
pub(crate) struct CircuitBreakerConfig {
    /// Total bumps required per cycle before reward fires.
    pub bumps_required: u32,
    /// Number of extra bolts to spawn on reward.
    pub spawn_count: u32,
    /// Whether spawned bolts inherit `BoundEffects` from the source entity.
    pub inherit: bool,
    /// Shockwave maximum radius.
    pub shockwave_range: f32,
    /// Shockwave expansion speed.
    pub shockwave_speed: f32,
}

/// Tracks the countdown state for the `CircuitBreaker` effect.
///
/// Inserted by `fire()` on first call. Decremented on each subsequent call.
/// When `remaining` reaches 0, rewards fire and the counter resets.
#[derive(Component, Debug, Clone)]
pub(crate) struct CircuitBreakerCounter {
    /// Bumps remaining before reward fires.
    pub remaining: u32,
    /// Total bumps required per cycle (used for reset).
    pub bumps_required: u32,
    /// Number of extra bolts to spawn on reward.
    pub spawn_count: u32,
    /// Whether spawned bolts inherit `BoundEffects` from the source entity.
    pub inherit: bool,
    /// Shockwave maximum radius.
    pub shockwave_range: f32,
    /// Shockwave expansion speed.
    pub shockwave_speed: f32,
}

/// Fires the `CircuitBreaker` effect on an entity.
///
/// Each call counts as one bump toward the countdown. On first call, inserts a
/// `CircuitBreakerCounter` with `remaining = bumps_required - 1`. Subsequent
/// calls decrement `remaining` by 1. When `remaining` reaches 0, spawns reward
/// bolts and a shockwave, then resets the counter to `bumps_required`.
pub(crate) fn fire(
    entity: Entity,
    config: &CircuitBreakerConfig,
    source_chip: &str,
    world: &mut World,
) {
    // Guard: bail if entity has been despawned.
    if world.get_entity(entity).is_err() {
        return;
    }

    if let Some(mut counter) = world.get_mut::<CircuitBreakerCounter>(entity) {
        // Subsequent call: decrement remaining.
        counter.remaining -= 1;
        if counter.remaining == 0 {
            // Extract config values before dropping the mutable borrow.
            let sc = counter.spawn_count;
            let inh = counter.inherit;
            let sr = counter.shockwave_range;
            let ss = counter.shockwave_speed;
            let br = counter.bumps_required;
            counter.remaining = br;
            crate::effect::effects::spawn_bolts::fire(entity, sc, None, inh, source_chip, world);
            crate::effect::effects::shockwave::fire(entity, sr, 0.0, 1, ss, source_chip, world);
        }
    } else {
        // First call: insert counter with remaining = bumps_required - 1.
        let remaining = config.bumps_required.saturating_sub(1);
        world.entity_mut(entity).insert(CircuitBreakerCounter {
            remaining,
            bumps_required: config.bumps_required,
            spawn_count: config.spawn_count,
            inherit: config.inherit,
            shockwave_range: config.shockwave_range,
            shockwave_speed: config.shockwave_speed,
        });
        if remaining == 0 {
            // bumps_required was 1 -- trigger reward immediately and reset.
            if let Some(mut counter) = world.get_mut::<CircuitBreakerCounter>(entity) {
                counter.remaining = config.bumps_required;
            }
            crate::effect::effects::spawn_bolts::fire(
                entity,
                config.spawn_count,
                None,
                config.inherit,
                source_chip,
                world,
            );
            crate::effect::effects::shockwave::fire(
                entity,
                config.shockwave_range,
                0.0,
                1,
                config.shockwave_speed,
                source_chip,
                world,
            );
        }
    }
}

/// Reverses the `CircuitBreaker` effect on an entity.
///
/// Removes `CircuitBreakerCounter` from the entity if present.
pub(crate) fn reverse(entity: Entity, _source_chip: &str, world: &mut World) {
    world.entity_mut(entity).remove::<CircuitBreakerCounter>();
}

/// Registers runtime systems for the `CircuitBreaker` effect.
///
/// No-op -- all logic lives in `fire()`/`reverse()`.
pub(crate) const fn register(_app: &mut App) {}
