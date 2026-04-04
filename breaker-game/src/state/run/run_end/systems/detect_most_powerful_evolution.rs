//! System to detect `MostPowerfulEvolution` highlight at run end.

use bevy::prelude::*;

use crate::state::run::resources::*;

/// Examines [`HighlightTracker::evolution_damage`] and records a
/// `MostPowerfulEvolution` highlight in [`RunStats`] for the chip with the
/// highest total damage.
///
/// Does NOT emit [`HighlightTriggered`] — this highlight is run-end only.
pub(crate) fn detect_most_powerful_evolution(
    tracker: Res<HighlightTracker>,
    mut stats: ResMut<RunStats>,
) {
    if tracker.evolution_damage.is_empty() {
        return;
    }

    let Some((max_name, max_damage)) = tracker
        .evolution_damage
        .iter()
        .max_by(|a, b| a.1.total_cmp(b.1))
    else {
        return;
    };

    stats.highlights.push(RunHighlight {
        kind: HighlightKind::MostPowerfulEvolution,
        node_index: 0,
        value: *max_damage,
        detail: Some(max_name.clone()),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::run::{
        messages::HighlightTriggered,
        resources::{HighlightKind, NodeOutcome, RunHighlight},
    };

    #[derive(Resource, Default)]
    struct CapturedHighlightTriggered(Vec<HighlightTriggered>);

    fn collect_highlight_triggered(
        mut reader: MessageReader<HighlightTriggered>,
        mut captured: ResMut<CapturedHighlightTriggered>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<HighlightTriggered>()
            .init_resource::<RunStats>()
            .init_resource::<NodeOutcome>()
            .init_resource::<HighlightTracker>()
            .init_resource::<CapturedHighlightTriggered>()
            .add_systems(
                Update,
                (detect_most_powerful_evolution, collect_highlight_triggered).chain(),
            );
        app
    }

    // --- Behavior 6: Detects single evolution with correct fields ---

    #[test]
    fn detects_single_evolution_with_correct_fields() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .evolution_damage
            .insert("Piercing Barrage".to_owned(), 250.0);

        app.update();

        let stats = app.world().resource::<RunStats>();
        let highlight = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::MostPowerfulEvolution);
        assert!(
            highlight.is_some(),
            "should detect MostPowerfulEvolution highlight when evolution_damage has entry"
        );
        let highlight = highlight.unwrap();
        assert_eq!(
            highlight.node_index, 0,
            "node_index should be 0 for run-end detection"
        );
        assert!(
            (highlight.value - 250.0).abs() < f32::EPSILON,
            "value should be 250.0 (total damage), got {}",
            highlight.value
        );

        assert_eq!(
            highlight.detail,
            Some("Piercing Barrage".to_owned()),
            "detail should be set to 'Piercing Barrage'"
        );
    }

    // --- Behavior 7: Picks highest damage when multiple exist ---

    #[test]
    fn picks_highest_damage_when_multiple_exist() {
        let mut app = test_app();
        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            tracker
                .evolution_damage
                .insert("Piercing Barrage".to_owned(), 250.0);
            tracker
                .evolution_damage
                .insert("Chain Lightning".to_owned(), 400.0);
            tracker
                .evolution_damage
                .insert("Fire Storm".to_owned(), 150.0);
        }

        app.update();

        let stats = app.world().resource::<RunStats>();
        let highlights: Vec<&RunHighlight> = stats
            .highlights
            .iter()
            .filter(|h| h.kind == HighlightKind::MostPowerfulEvolution)
            .collect();
        assert_eq!(
            highlights.len(),
            1,
            "should produce exactly 1 MostPowerfulEvolution highlight, got {}",
            highlights.len()
        );
        assert!(
            (highlights[0].value - 400.0).abs() < f32::EPSILON,
            "value should be 400.0 (highest damage), got {}",
            highlights[0].value
        );
        assert_eq!(
            highlights[0].detail,
            Some("Chain Lightning".to_owned()),
            "detail should be 'Chain Lightning' (highest damage)"
        );
    }

    // --- Behavior 8: Tie-breaking picks deterministically ---

    #[test]
    fn tie_breaking_picks_deterministically() {
        let mut app = test_app();
        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            tracker.evolution_damage.insert("Alpha".to_owned(), 300.0);
            tracker.evolution_damage.insert("Beta".to_owned(), 300.0);
        }

        app.update();

        let stats = app.world().resource::<RunStats>();
        let highlights: Vec<&RunHighlight> = stats
            .highlights
            .iter()
            .filter(|h| h.kind == HighlightKind::MostPowerfulEvolution)
            .collect();
        assert_eq!(
            highlights.len(),
            1,
            "should produce exactly 1 MostPowerfulEvolution highlight even with tie, got {}",
            highlights.len()
        );
        assert!(
            (highlights[0].value - 300.0).abs() < f32::EPSILON,
            "value should be 300.0 (tied damage), got {}",
            highlights[0].value
        );
        assert!(
            highlights[0].detail.is_some(),
            "detail should be Some (either 'Alpha' or 'Beta')"
        );
        let name = highlights[0].detail.as_ref().unwrap();
        assert!(
            name == "Alpha" || name == "Beta",
            "detail should be 'Alpha' or 'Beta', got '{name}'"
        );
    }

    // --- Behavior 9: Does nothing when evolution_damage is empty ---

    #[test]
    fn does_nothing_when_evolution_damage_is_empty() {
        let mut app = test_app();
        // evolution_damage defaults to empty HashMap

        app.update();

        let stats = app.world().resource::<RunStats>();
        assert!(
            stats.highlights.is_empty(),
            "highlights should remain empty when evolution_damage is empty, got {:?}",
            stats.highlights.len()
        );
        assert!(
            !stats
                .highlights
                .iter()
                .any(|h| h.kind == HighlightKind::MostPowerfulEvolution),
            "should not have any MostPowerfulEvolution highlight"
        );
    }

    // --- Behavior 10: Does NOT emit HighlightTriggered message ---

    #[test]
    fn does_not_emit_highlight_triggered_message() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .evolution_damage
            .insert("Piercing Barrage".to_owned(), 250.0);

        app.update();

        let captured = app.world().resource::<CapturedHighlightTriggered>();
        assert!(
            captured.0.is_empty(),
            "should NOT emit any HighlightTriggered messages for MostPowerfulEvolution, got {}",
            captured.0.len()
        );
    }

    // --- Behavior 11: Adds to existing highlights without replacing ---

    #[test]
    fn adds_to_existing_highlights_without_replacing() {
        let mut app = test_app();
        // Pre-fill with 3 existing highlights
        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            for i in 0..3 {
                stats.highlights.push(RunHighlight {
                    kind: HighlightKind::MassDestruction,
                    node_index: i,
                    value: 10.0,
                    detail: None,
                });
            }
        }
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .evolution_damage
            .insert("Piercing Barrage".to_owned(), 250.0);

        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.highlights.len(),
            4,
            "should have 4 total highlights (3 original + 1 MostPowerfulEvolution), got {}",
            stats.highlights.len()
        );
        let mass_count = stats
            .highlights
            .iter()
            .filter(|h| h.kind == HighlightKind::MassDestruction)
            .count();
        assert_eq!(
            mass_count, 3,
            "original 3 MassDestruction highlights should be preserved"
        );
        let evo_count = stats
            .highlights
            .iter()
            .filter(|h| h.kind == HighlightKind::MostPowerfulEvolution)
            .count();
        assert_eq!(
            evo_count, 1,
            "should have exactly 1 MostPowerfulEvolution highlight"
        );
    }
}
