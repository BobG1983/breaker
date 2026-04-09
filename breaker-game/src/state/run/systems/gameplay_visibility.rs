//! Systems to hide/show breaker and bolt entities during state transitions.

use bevy::prelude::*;

use crate::{bolt::components::Bolt, breaker::components::Breaker};

/// Hides all breaker and bolt entities by setting [`Visibility::Hidden`].
///
/// Runs on `OnExit(RunState::Node)` so gameplay entities are invisible during
/// `ChipSelect` and `RunEnd` screens.
pub(crate) fn hide_gameplay_entities(
    mut breakers: Query<&mut Visibility, (With<Breaker>, Without<Bolt>)>,
    mut bolts: Query<&mut Visibility, (With<Bolt>, Without<Breaker>)>,
) {
    for mut vis in &mut breakers {
        *vis = Visibility::Hidden;
    }
    for mut vis in &mut bolts {
        *vis = Visibility::Hidden;
    }
}

/// Shows all breaker and bolt entities by setting [`Visibility::Inherited`].
///
/// Runs on `OnEnter(RunState::Node)` so gameplay entities reappear when
/// returning from `ChipSelect` to the next node.
pub(crate) fn show_gameplay_entities(
    mut breakers: Query<&mut Visibility, (With<Breaker>, Without<Bolt>)>,
    mut bolts: Query<&mut Visibility, (With<Bolt>, Without<Breaker>)>,
) {
    for mut vis in &mut breakers {
        *vis = Visibility::Inherited;
    }
    for mut vis in &mut bolts {
        *vis = Visibility::Inherited;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::test_utils::TestAppBuilder;

    fn hide_app() -> App {
        TestAppBuilder::new()
            .with_system(Update, hide_gameplay_entities)
            .build()
    }

    fn show_app() -> App {
        TestAppBuilder::new()
            .with_system(Update, show_gameplay_entities)
            .build()
    }

    // ── hide_gameplay_entities ────────────────────────────────────────

    #[test]
    fn hide_sets_hidden_on_breaker() {
        let mut app = hide_app();
        app.world_mut().spawn((Breaker, Visibility::Inherited));
        app.update();

        let vis = app
            .world_mut()
            .query_filtered::<&Visibility, With<Breaker>>()
            .single(app.world())
            .unwrap();
        assert_eq!(*vis, Visibility::Hidden);
    }

    #[test]
    fn hide_sets_hidden_on_bolt() {
        let mut app = hide_app();
        app.world_mut().spawn((Bolt, Visibility::Inherited));
        app.update();

        let vis = app
            .world_mut()
            .query_filtered::<&Visibility, With<Bolt>>()
            .single(app.world())
            .unwrap();
        assert_eq!(*vis, Visibility::Hidden);
    }

    #[test]
    fn hide_handles_multiple_entities() {
        let mut app = hide_app();
        app.world_mut().spawn((Breaker, Visibility::Inherited));
        app.world_mut().spawn((Breaker, Visibility::Inherited));
        app.world_mut().spawn((Bolt, Visibility::Inherited));
        app.world_mut().spawn((Bolt, Visibility::Inherited));
        app.world_mut().spawn((Bolt, Visibility::Inherited));
        app.update();

        for vis in app.world_mut().query::<&Visibility>().iter(app.world()) {
            assert_eq!(*vis, Visibility::Hidden);
        }
    }

    #[test]
    fn hide_no_panic_with_empty_world() {
        let mut app = hide_app();
        app.update();
    }

    // ── show_gameplay_entities ────────────────────────────────────────

    #[test]
    fn show_sets_inherited_on_breaker() {
        let mut app = show_app();
        app.world_mut().spawn((Breaker, Visibility::Hidden));
        app.update();

        let vis = app
            .world_mut()
            .query_filtered::<&Visibility, With<Breaker>>()
            .single(app.world())
            .unwrap();
        assert_eq!(*vis, Visibility::Inherited);
    }

    #[test]
    fn show_sets_inherited_on_bolt() {
        let mut app = show_app();
        app.world_mut().spawn((Bolt, Visibility::Hidden));
        app.update();

        let vis = app
            .world_mut()
            .query_filtered::<&Visibility, With<Bolt>>()
            .single(app.world())
            .unwrap();
        assert_eq!(*vis, Visibility::Inherited);
    }

    #[test]
    fn show_handles_multiple_entities() {
        let mut app = show_app();
        app.world_mut().spawn((Breaker, Visibility::Hidden));
        app.world_mut().spawn((Breaker, Visibility::Hidden));
        app.world_mut().spawn((Bolt, Visibility::Hidden));
        app.world_mut().spawn((Bolt, Visibility::Hidden));
        app.world_mut().spawn((Bolt, Visibility::Hidden));
        app.update();

        for vis in app.world_mut().query::<&Visibility>().iter(app.world()) {
            assert_eq!(*vis, Visibility::Inherited);
        }
    }

    #[test]
    fn show_no_panic_with_empty_world() {
        let mut app = show_app();
        app.update();
    }

    // ── unrelated entities unaffected ────────────────────────────────

    #[test]
    fn hide_does_not_affect_unmarked_entities() {
        let mut app = hide_app();
        app.world_mut().spawn(Visibility::Inherited);
        app.update();

        let vis = app
            .world_mut()
            .query::<&Visibility>()
            .single(app.world())
            .unwrap();
        assert_eq!(*vis, Visibility::Inherited);
    }
}
