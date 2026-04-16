//! Salvo entity components and constants.

use bevy::prelude::*;

/// Marker identifying a salvo projectile entity.
#[derive(Component, Debug)]
pub(crate) struct Salvo;

/// Damage dealt by a salvo to cells on contact.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct SalvoDamage(pub(crate) f32);

/// The turret entity that spawned this salvo.
#[derive(Component, Debug)]
pub(crate) struct SalvoSource(pub(crate) Entity);

/// Fire-rate countdown timer on turret entities. Decremented each tick.
/// When <= 0, the turret fires and the timer resets to `SALVO_FIRE_INTERVAL`.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct SalvoFireTimer(pub(crate) f32);

/// Default seconds between turret shots.
pub(crate) const SALVO_FIRE_INTERVAL: f32 = 2.0;

/// Default salvo downward velocity magnitude.
pub(crate) const SALVO_SPEED: f32 = 300.0;

/// Default damage dealt by a salvo to a cell.
pub(crate) const SALVO_DAMAGE: f32 = 10.0;

/// Half-extent of salvo AABB for collision (square).
pub(crate) const SALVO_HALF_EXTENT: f32 = 3.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn salvo_marker_debug_contains_name() {
        let marker = Salvo;
        let debug_str = format!("{marker:?}");
        assert!(
            debug_str.contains("Salvo"),
            "debug output should contain 'Salvo', got: {debug_str}"
        );
    }

    #[test]
    fn salvo_damage_wraps_f32() {
        let dmg = SalvoDamage(5.0);
        assert!(
            (dmg.0 - 5.0).abs() < f32::EPSILON,
            "SalvoDamage.0 should be 5.0, got {}",
            dmg.0
        );
    }

    #[test]
    fn salvo_damage_zero_is_valid() {
        let dmg = SalvoDamage(0.0);
        assert!(
            dmg.0.abs() < f32::EPSILON,
            "SalvoDamage(0.0).0 should be 0.0"
        );
    }

    #[test]
    fn salvo_damage_max_is_valid() {
        let dmg = SalvoDamage(f32::MAX);
        assert_eq!(
            dmg.0.to_bits(),
            f32::MAX.to_bits(),
            "SalvoDamage(f32::MAX).0 should be f32::MAX"
        );
    }

    #[test]
    fn salvo_source_wraps_entity() {
        let entity = Entity::PLACEHOLDER;
        let source = SalvoSource(entity);
        assert_eq!(
            source.0, entity,
            "SalvoSource.0 should equal the provided entity"
        );
    }

    #[test]
    fn salvo_fire_timer_wraps_f32() {
        let timer = SalvoFireTimer(2.0);
        assert!(
            (timer.0 - 2.0).abs() < f32::EPSILON,
            "SalvoFireTimer.0 should be 2.0, got {}",
            timer.0
        );
    }

    #[test]
    fn salvo_fire_timer_zero_is_valid() {
        let timer = SalvoFireTimer(0.0);
        assert!(
            timer.0.abs() < f32::EPSILON,
            "SalvoFireTimer(0.0).0 should be 0.0"
        );
    }
}
