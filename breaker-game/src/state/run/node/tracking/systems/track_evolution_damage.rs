//! System to accumulate per-evolution-chip damage for `MostPowerfulEvolution` highlight detection.

use bevy::prelude::*;

use crate::{cells::messages::DamageCell, state::run::resources::HighlightTracker};

/// Reads [`DamageCell`] messages and accumulates damage per evolution chip name
/// in [`HighlightTracker::evolution_damage`].
///
/// Messages with `source_chip: None` are ignored.
pub(crate) fn track_evolution_damage(
    mut reader: MessageReader<DamageCell>,
    mut tracker: ResMut<HighlightTracker>,
) {
    for msg in reader.read() {
        if let Some(name) = &msg.source_chip {
            *tracker.evolution_damage.entry(name.clone()).or_insert(0.0) += msg.damage;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        shared::test_utils::TestAppBuilder,
        state::run::{
            node::lifecycle::systems::reset_highlight_tracker, resources::HighlightTracker,
        },
    };

    #[derive(Resource)]
    struct TestMessages(Vec<DamageCell>);

    fn enqueue_messages(msg_res: Res<TestMessages>, mut writer: MessageWriter<DamageCell>) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    fn test_app() -> App {
        TestAppBuilder::new()
            .with_message::<DamageCell>()
            .with_resource::<HighlightTracker>()
            .with_system(
                FixedUpdate,
                (enqueue_messages, track_evolution_damage).chain(),
            )
            .build()
    }

    use crate::shared::test_utils::tick;

    // --- Behavior 1: Accumulates damage for a single evolution chip ---

    #[test]
    fn accumulates_damage_for_single_evolution_chip() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![DamageCell {
            cell:        Entity::PLACEHOLDER,
            damage:      25.0,
            source_chip: Some("Piercing Barrage".to_owned()),
        }]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        let damage = tracker.evolution_damage.get("Piercing Barrage");
        assert!(
            damage.is_some(),
            "evolution_damage should have entry for 'Piercing Barrage'"
        );
        assert!(
            (damage.unwrap() - 25.0).abs() < f32::EPSILON,
            "evolution_damage['Piercing Barrage'] should be 25.0, got {}",
            damage.unwrap()
        );
    }

    // --- Behavior 2: Accumulates across multiple messages for same evolution ---

    #[test]
    fn accumulates_across_multiple_messages_for_same_evolution() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![
            DamageCell {
                cell:        Entity::PLACEHOLDER,
                damage:      10.0,
                source_chip: Some("Piercing Barrage".to_owned()),
            },
            DamageCell {
                cell:        Entity::PLACEHOLDER,
                damage:      15.0,
                source_chip: Some("Piercing Barrage".to_owned()),
            },
            DamageCell {
                cell:        Entity::PLACEHOLDER,
                damage:      5.0,
                source_chip: Some("Piercing Barrage".to_owned()),
            },
        ]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        let damage = tracker.evolution_damage.get("Piercing Barrage");
        assert!(
            damage.is_some(),
            "evolution_damage should have entry for 'Piercing Barrage'"
        );
        assert!(
            (damage.unwrap() - 30.0).abs() < f32::EPSILON,
            "evolution_damage['Piercing Barrage'] should be 30.0 (10+15+5), got {}",
            damage.unwrap()
        );
    }

    // --- Behavior 2 edge case: Pre-existing damage accumulates with new ---

    #[test]
    fn pre_existing_damage_accumulates_with_new_messages() {
        let mut app = test_app();
        // Pre-seed the tracker with existing damage
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .evolution_damage
            .insert("Piercing Barrage".to_owned(), 20.0);

        app.insert_resource(TestMessages(vec![DamageCell {
            cell:        Entity::PLACEHOLDER,
            damage:      10.0,
            source_chip: Some("Piercing Barrage".to_owned()),
        }]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        let damage = tracker.evolution_damage.get("Piercing Barrage");
        assert!(
            damage.is_some(),
            "evolution_damage should have entry for 'Piercing Barrage'"
        );
        assert!(
            (damage.unwrap() - 30.0).abs() < f32::EPSILON,
            "evolution_damage['Piercing Barrage'] should be 30.0 (20 pre-existing + 10 new), got {}",
            damage.unwrap()
        );
    }

    // --- Behavior 3: Tracks multiple chips independently ---

    #[test]
    fn tracks_multiple_chips_independently() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![
            DamageCell {
                cell:        Entity::PLACEHOLDER,
                damage:      25.0,
                source_chip: Some("Piercing Barrage".to_owned()),
            },
            DamageCell {
                cell:        Entity::PLACEHOLDER,
                damage:      40.0,
                source_chip: Some("Chain Lightning".to_owned()),
            },
        ]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        let piercing = tracker.evolution_damage.get("Piercing Barrage");
        assert!(
            piercing.is_some(),
            "should track 'Piercing Barrage' independently"
        );
        assert!(
            (piercing.unwrap() - 25.0).abs() < f32::EPSILON,
            "evolution_damage['Piercing Barrage'] should be 25.0, got {}",
            piercing.unwrap()
        );

        let chain = tracker.evolution_damage.get("Chain Lightning");
        assert!(
            chain.is_some(),
            "should track 'Chain Lightning' independently"
        );
        assert!(
            (chain.unwrap() - 40.0).abs() < f32::EPSILON,
            "evolution_damage['Chain Lightning'] should be 40.0, got {}",
            chain.unwrap()
        );
    }

    // --- Behavior 4: Ignores source_chip: None ---

    #[test]
    fn ignores_damage_cell_with_no_source_chip() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![DamageCell {
            cell:        Entity::PLACEHOLDER,
            damage:      50.0,
            source_chip: None,
        }]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        assert!(
            tracker.evolution_damage.is_empty(),
            "evolution_damage should remain empty when source_chip is None, got {:?}",
            tracker.evolution_damage
        );
    }

    // --- Behavior 5: evolution_damage persists across reset_highlight_tracker ---

    #[test]
    fn evolution_damage_persists_across_reset_highlight_tracker() {
        let mut app = TestAppBuilder::new()
            .with_resource::<HighlightTracker>()
            .with_system(Update, reset_highlight_tracker)
            .build();

        // Pre-seed evolution damage
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .evolution_damage
            .insert("Piercing Barrage".to_owned(), 100.0);

        app.update();

        let tracker = app.world().resource::<HighlightTracker>();
        let damage = tracker.evolution_damage.get("Piercing Barrage");
        assert!(
            damage.is_some(),
            "evolution_damage should persist across reset_highlight_tracker"
        );
        assert!(
            (damage.unwrap() - 100.0).abs() < f32::EPSILON,
            "evolution_damage['Piercing Barrage'] should still be 100.0 after reset, got {}",
            damage.unwrap()
        );
    }
}
