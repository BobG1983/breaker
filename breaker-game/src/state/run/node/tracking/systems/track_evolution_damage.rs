//! System to accumulate per-evolution-chip damage for `MostPowerfulEvolution` highlight detection.

use bevy::prelude::*;

use crate::{prelude::*, state::run::resources::HighlightTracker};

/// Reads [`DamageDealt<Cell>`] messages and accumulates damage per evolution chip name
/// in [`HighlightTracker::evolution_damage`].
///
/// Messages with `source: None` are ignored.
pub(crate) fn track_evolution_damage(
    mut reader: MessageReader<DamageDealt<Cell>>,
    mut tracker: ResMut<HighlightTracker>,
) {
    for msg in reader.read() {
        if let Some(name) = &msg.source {
            *tracker
                .evolution_damage
                .entry(name.0.as_ref().to_owned())
                .or_insert(0.0) += msg.amount;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::*;
    use crate::{
        chips::definition::Rarity, state::run::node::lifecycle::systems::reset_highlight_tracker,
    };

    /// Builder-format `SourceId` for the canonical "Piercing Barrage" evolution
    /// chip used across these tests. The production tracker keys on the FULL
    /// source string (e.g. `chip:Piercing Barrage:Evolution`), so the test
    /// assertions look up that exact key.
    fn piercing_barrage_source() -> SourceId {
        SourceId::chip("Piercing Barrage")
            .rarity(Rarity::Evolution)
            .build()
    }

    /// Builder-format `SourceId` for an alternate evolution chip used by the
    /// multi-chip independence test.
    fn chain_lightning_source() -> SourceId {
        SourceId::chip("Chain Lightning")
            .rarity(Rarity::Evolution)
            .build()
    }

    /// Returns the canonical map key used by `track_evolution_damage` —
    /// the full source string content of the builder-built id.
    fn key_of(source: &SourceId) -> String {
        source.0.as_ref().to_owned()
    }

    #[derive(Resource)]
    struct TestMessages(Vec<DamageDealt<Cell>>);

    fn enqueue_messages(msg_res: Res<TestMessages>, mut writer: MessageWriter<DamageDealt<Cell>>) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    fn test_app() -> App {
        TestAppBuilder::new()
            .with_message::<DamageDealt<Cell>>()
            .with_resource::<HighlightTracker>()
            .with_system(
                FixedUpdate,
                (enqueue_messages, track_evolution_damage).chain(),
            )
            .build()
    }

    fn make_damage(amount: f32, source: Option<SourceId>) -> DamageDealt<Cell> {
        DamageDealt::<Cell> {
            dealer: None,
            attributed_to: None,
            target: Entity::PLACEHOLDER,
            amount,
            source,
            _marker: PhantomData,
        }
    }

    // --- Behavior 14: Accumulates damage for a single evolution chip ---

    #[test]
    fn accumulates_damage_for_single_evolution_chip() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![make_damage(
            25.0,
            Some(piercing_barrage_source()),
        )]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        let damage = tracker
            .evolution_damage
            .get(&key_of(&piercing_barrage_source()));
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

    // --- Behavior 14: Accumulates across multiple messages for same evolution ---

    #[test]
    fn accumulates_across_multiple_messages_for_same_evolution() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![
            make_damage(10.0, Some(piercing_barrage_source())),
            make_damage(15.0, Some(piercing_barrage_source())),
            make_damage(5.0, Some(piercing_barrage_source())),
        ]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        let damage = tracker
            .evolution_damage
            .get(&key_of(&piercing_barrage_source()));
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

    // --- Behavior 14 edge case: Pre-existing damage accumulates with new ---

    #[test]
    fn pre_existing_damage_accumulates_with_new_messages() {
        let mut app = test_app();
        // Pre-seed the tracker with existing damage
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .evolution_damage
            .insert(key_of(&piercing_barrage_source()), 20.0);

        app.insert_resource(TestMessages(vec![make_damage(
            10.0,
            Some(piercing_barrage_source()),
        )]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        let damage = tracker
            .evolution_damage
            .get(&key_of(&piercing_barrage_source()));
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

    // --- Behavior 14: Tracks multiple chips independently ---

    #[test]
    fn tracks_multiple_chips_independently() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![
            make_damage(25.0, Some(piercing_barrage_source())),
            make_damage(40.0, Some(chain_lightning_source())),
        ]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        let piercing = tracker
            .evolution_damage
            .get(&key_of(&piercing_barrage_source()));
        assert!(
            piercing.is_some(),
            "should track 'Piercing Barrage' independently"
        );
        assert!(
            (piercing.unwrap() - 25.0).abs() < f32::EPSILON,
            "evolution_damage['Piercing Barrage'] should be 25.0, got {}",
            piercing.unwrap()
        );

        let chain = tracker
            .evolution_damage
            .get(&key_of(&chain_lightning_source()));
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

    // --- Behavior 14: Ignores source: None ---

    #[test]
    fn ignores_damage_cell_with_no_source_chip() {
        let mut app = test_app();
        app.insert_resource(TestMessages(vec![make_damage(50.0, None)]));
        tick(&mut app);

        let tracker = app.world().resource::<HighlightTracker>();
        assert!(
            tracker.evolution_damage.is_empty(),
            "evolution_damage should remain empty when source_chip is None, got {:?}",
            tracker.evolution_damage
        );
    }

    // --- Behavior 14: evolution_damage persists across reset_highlight_tracker ---

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
            .insert(key_of(&piercing_barrage_source()), 100.0);

        app.update();

        let tracker = app.world().resource::<HighlightTracker>();
        let damage = tracker
            .evolution_damage
            .get(&key_of(&piercing_barrage_source()));
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
