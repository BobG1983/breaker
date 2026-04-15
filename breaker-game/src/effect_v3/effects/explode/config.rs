//! `ExplodeConfig` — fire-and-forget area explosion.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::{effect_v3::traits::Fireable, prelude::*};

/// Area explosion dealing flat damage to all cells within range.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExplodeConfig {
    /// Radius of the explosion in world units.
    pub range:  OrderedFloat<f32>,
    /// Flat damage dealt to every cell within range.
    pub damage: OrderedFloat<f32>,
}

impl Fireable for ExplodeConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        // Snapshot position from the source entity.
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);

        let source_chip = if source.is_empty() {
            None
        } else {
            Some(source.to_owned())
        };

        // Collect cells within range (flat damage, no boost multiplier).
        let targets: Vec<Entity> = world
            .query_filtered::<(Entity, &Position2D), (With<Cell>, Without<Dead>)>()
            .iter(world)
            .filter(|(_, cell_pos)| pos.distance(cell_pos.0) <= self.range.0)
            .map(|(e, _)| e)
            .collect();

        // Send damage messages.
        for target in targets {
            world.write_message(DamageDealt {
                dealer: Some(entity),
                target,
                amount: self.damage.0,
                source_chip: source_chip.clone(),
                _marker: std::marker::PhantomData::<Cell>,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::ExplodeConfig;
    use crate::{effect_v3::traits::Fireable, prelude::*};

    // ── Helpers ────────────────────────────────────────────────────────────

    fn explode_test_app() -> App {
        TestAppBuilder::new()
            .with_message_capture::<DamageDealt<Cell>>()
            .build()
    }

    fn spawn_cell_at(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut().spawn((Cell, Position2D(pos))).id()
    }

    fn spawn_dead_cell_at(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut().spawn((Cell, Position2D(pos), Dead)).id()
    }

    fn spawn_source_at(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut().spawn(Position2D(pos)).id()
    }

    // ── Tests ─────────────────────────────────────────────────────────────

    #[test]
    fn fire_damages_all_cells_within_range_including_boundary() {
        let mut app = explode_test_app();
        let source = spawn_source_at(&mut app, Vec2::ZERO);
        let cell_a = spawn_cell_at(&mut app, Vec2::new(20.0, 0.0));
        let cell_b = spawn_cell_at(&mut app, Vec2::new(30.0, 0.0));
        let cell_c = spawn_cell_at(&mut app, Vec2::new(50.0, 0.0)); // exactly on boundary

        let config = ExplodeConfig {
            range:  OrderedFloat(50.0),
            damage: OrderedFloat(10.0),
        };
        config.fire(source, "boom_chip", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 3, "expected 3 DamageDealt<Cell> messages");

        for msg in &msgs.0 {
            assert!(
                (msg.amount - 10.0).abs() < f32::EPSILON,
                "damage amount should be 10.0, got {}",
                msg.amount,
            );
            assert_eq!(msg.dealer, Some(source));
            assert_eq!(msg.source_chip, Some("boom_chip".to_string()));
        }

        let targets: HashSet<Entity> = msgs.0.iter().map(|m| m.target).collect();
        assert_eq!(targets, HashSet::from([cell_a, cell_b, cell_c]));
    }

    #[test]
    fn cells_outside_range_are_unaffected() {
        let mut app = explode_test_app();
        let source = spawn_source_at(&mut app, Vec2::ZERO);
        let _cell = spawn_cell_at(&mut app, Vec2::new(100.0, 0.0));

        let config = ExplodeConfig {
            range:  OrderedFloat(50.0),
            damage: OrderedFloat(10.0),
        };
        config.fire(source, "boom_chip", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 0, "no cells should be damaged outside range");
    }

    #[test]
    fn cell_barely_outside_range_is_not_damaged() {
        let mut app = explode_test_app();
        let source = spawn_source_at(&mut app, Vec2::ZERO);
        let _cell = spawn_cell_at(&mut app, Vec2::new(50.001, 0.0));

        let config = ExplodeConfig {
            range:  OrderedFloat(50.0),
            damage: OrderedFloat(10.0),
        };
        config.fire(source, "boom_chip", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            0,
            "cell at 50.001 should not be damaged with range 50.0"
        );
    }

    #[test]
    fn dead_cells_are_filtered_out() {
        let mut app = explode_test_app();
        let source = spawn_source_at(&mut app, Vec2::ZERO);
        let _dead_cell = spawn_dead_cell_at(&mut app, Vec2::new(20.0, 0.0));

        let config = ExplodeConfig {
            range:  OrderedFloat(50.0),
            damage: OrderedFloat(10.0),
        };
        config.fire(source, "boom_chip", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            0,
            "Dead cells should be filtered by Without<Dead>"
        );
    }

    #[test]
    fn fire_with_no_cells_emits_no_damage_and_does_not_panic() {
        let mut app = explode_test_app();
        let source = spawn_source_at(&mut app, Vec2::ZERO);

        let config = ExplodeConfig {
            range:  OrderedFloat(50.0),
            damage: OrderedFloat(10.0),
        };
        config.fire(source, "boom_chip", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 0, "no cells means no damage messages");
    }

    #[test]
    fn source_entity_without_position_defaults_to_zero() {
        let mut app = explode_test_app();
        let source = app.world_mut().spawn_empty().id(); // no Position2D
        let _cell = spawn_cell_at(&mut app, Vec2::new(20.0, 0.0));

        let config = ExplodeConfig {
            range:  OrderedFloat(50.0),
            damage: OrderedFloat(10.0),
        };
        config.fire(source, "boom_chip", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            1,
            "cell at (20, 0) is within range 50 from fallback (0, 0)"
        );
    }

    #[test]
    fn fire_with_empty_source_string_passes_none_as_source_chip() {
        let mut app = explode_test_app();
        let source = spawn_source_at(&mut app, Vec2::ZERO);
        let _cell = spawn_cell_at(&mut app, Vec2::new(10.0, 0.0));

        let config = ExplodeConfig {
            range:  OrderedFloat(50.0),
            damage: OrderedFloat(10.0),
        };
        config.fire(source, "", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1);
        assert_eq!(
            msgs.0[0].source_chip, None,
            "empty source string should produce None"
        );
    }

    #[test]
    fn fire_with_nonempty_source_string_passes_some_source_chip() {
        let mut app = explode_test_app();
        let source = spawn_source_at(&mut app, Vec2::ZERO);
        let _cell = spawn_cell_at(&mut app, Vec2::new(10.0, 0.0));

        let config = ExplodeConfig {
            range:  OrderedFloat(50.0),
            damage: OrderedFloat(10.0),
        };
        config.fire(source, "bomb_chip", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1);
        assert_eq!(
            msgs.0[0].source_chip,
            Some("bomb_chip".to_string()),
            "non-empty source should produce Some"
        );
    }
}
