//! `AnchorConfig` — anchor bolt in place with enhanced bump.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use super::components::{AnchorActive, AnchorPlanted, AnchorTimer};
use crate::effect_v3::{
    effects::piercing::PiercingConfig,
    stacking::EffectStack,
    traits::{Fireable, Reversible},
};

/// Configuration for the anchor effect on the breaker.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnchorConfig {
    /// How much the bump force is multiplied when planted.
    pub bump_force_multiplier:     OrderedFloat<f32>,
    /// How much wider the perfect timing window becomes when planted.
    pub perfect_window_multiplier: OrderedFloat<f32>,
    /// Seconds the breaker must stand still before planting.
    pub plant_delay:               OrderedFloat<f32>,
}

impl Fireable for AnchorConfig {
    fn fire(&self, entity: Entity, _source: &str, world: &mut World) {
        if world.get_entity(entity).is_err() {
            return;
        }
        world.entity_mut(entity).insert((
            AnchorActive {
                bump_force_multiplier:     self.bump_force_multiplier.0,
                perfect_window_multiplier: self.perfect_window_multiplier.0,
                plant_delay:               self.plant_delay.0,
            },
            AnchorTimer(self.plant_delay.0),
        ));
    }

    fn register(app: &mut App) {
        use super::systems::{detect_breaker_movement, tick_anchor};
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            (detect_breaker_movement, tick_anchor)
                .chain()
                .in_set(EffectV3Systems::Tick),
        );
    }
}

impl Reversible for AnchorConfig {
    fn reverse(&self, entity: Entity, _source: &str, world: &mut World) {
        if world.get_entity(entity).is_err() {
            return;
        }
        world
            .entity_mut(entity)
            .remove::<AnchorActive>()
            .remove::<AnchorTimer>()
            .remove::<AnchorPlanted>();

        if let Some(mut stack) = world.get_mut::<EffectStack<PiercingConfig>>(entity) {
            stack.remove("anchor_piercing", &PiercingConfig { charges: 1 });
        }
    }

    fn reverse_all_by_source(&self, entity: Entity, source: &str, world: &mut World) {
        if world.get_entity(entity).is_err() {
            return;
        }
        world
            .entity_mut(entity)
            .remove::<AnchorActive>()
            .remove::<AnchorTimer>()
            .remove::<AnchorPlanted>();

        if let Some(mut stack) = world.get_mut::<EffectStack<PiercingConfig>>(entity) {
            stack.retain_by_source(source);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::effect_v3::{
        effects::{
            anchor::components::{AnchorActive, AnchorPlanted, AnchorTimer},
            piercing::PiercingConfig,
        },
        stacking::EffectStack,
        traits::{Fireable, Reversible},
    };

    fn make_config() -> AnchorConfig {
        AnchorConfig {
            bump_force_multiplier:     OrderedFloat(2.0),
            perfect_window_multiplier: OrderedFloat(1.5),
            plant_delay:               OrderedFloat(0.5),
        }
    }

    // ── reverse_all_by_source ─────────────────────────────────────────

    #[test]
    fn reverse_all_by_source_removes_anchor_active_timer_and_planted() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        make_config().fire(entity, "anchor_chip", &mut world);
        // AnchorPlanted must be manually inserted because fire() does not insert
        // it — the tick system inserts it when the plant delay expires.
        world.entity_mut(entity).insert(AnchorPlanted);

        make_config().reverse_all_by_source(entity, "anchor_chip", &mut world);

        assert!(
            world.get::<AnchorActive>(entity).is_none(),
            "AnchorActive should be removed"
        );
        assert!(
            world.get::<AnchorTimer>(entity).is_none(),
            "AnchorTimer should be removed"
        );
        assert!(
            world.get::<AnchorPlanted>(entity).is_none(),
            "AnchorPlanted should be removed"
        );
    }

    #[test]
    fn reverse_all_by_source_removes_all_piercing_entries_from_source() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // Fire anchor.
        make_config().fire(entity, "anchor_piercing", &mut world);
        world.entity_mut(entity).insert(AnchorPlanted);

        // Manually set up piercing stack with entries from multiple sources.
        PiercingConfig { charges: 1 }.fire(entity, "anchor_piercing", &mut world);
        PiercingConfig { charges: 3 }.fire(entity, "splinter", &mut world);
        PiercingConfig { charges: 2 }.fire(entity, "anchor_piercing", &mut world);

        make_config().reverse_all_by_source(entity, "anchor_piercing", &mut world);

        let stack = world.get::<EffectStack<PiercingConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(entries[0].0, "splinter");
        assert_eq!(entries[0].1.charges, 3);

        assert!(world.get::<AnchorActive>(entity).is_none());
        assert!(world.get::<AnchorTimer>(entity).is_none());
        assert!(world.get::<AnchorPlanted>(entity).is_none());
    }

    #[test]
    fn reverse_all_by_source_uses_passed_source_not_hardcoded() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        make_config().fire(entity, "my_anchor_chip", &mut world);
        world.entity_mut(entity).insert(AnchorPlanted);

        PiercingConfig { charges: 1 }.fire(entity, "my_anchor_chip", &mut world);
        PiercingConfig { charges: 3 }.fire(entity, "my_anchor_chip", &mut world);
        PiercingConfig { charges: 2 }.fire(entity, "splinter", &mut world);

        make_config().reverse_all_by_source(entity, "my_anchor_chip", &mut world);

        let stack = world.get::<EffectStack<PiercingConfig>>(entity).unwrap();
        assert_eq!(stack.len(), 1);
        let entries: Vec<_> = stack.iter().collect();
        assert_eq!(entries[0].0, "splinter");
        assert_eq!(entries[0].1.charges, 2);

        assert!(world.get::<AnchorActive>(entity).is_none());
        assert!(world.get::<AnchorTimer>(entity).is_none());
        assert!(world.get::<AnchorPlanted>(entity).is_none());
    }

    #[test]
    fn reverse_all_by_source_on_entity_without_anchor_components_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        make_config().reverse_all_by_source(entity, "anchor_piercing", &mut world);
        // No panic.
    }

    #[test]
    fn reverse_all_by_source_removes_markers_even_when_piercing_stack_absent() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        make_config().fire(entity, "anchor_piercing", &mut world);
        world.entity_mut(entity).insert(AnchorPlanted);
        // No EffectStack<PiercingConfig> on entity.

        make_config().reverse_all_by_source(entity, "anchor_piercing", &mut world);

        assert!(world.get::<AnchorActive>(entity).is_none());
        assert!(world.get::<AnchorTimer>(entity).is_none());
        assert!(world.get::<AnchorPlanted>(entity).is_none());
    }
}
