//! Hazard select state — sub-state of [`RunState::HazardSelect`].

use bevy::prelude::*;

use super::RunState;

/// Hazard selection lifecycle state.
///
/// Sub-state of [`RunState::HazardSelect`]. Controls the hazard selection
/// screen that appears between tiers on tier-9+ boss clears.
#[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[source(RunState = RunState::HazardSelect)]
pub enum HazardSelectState {
    /// Hazard select loading (pass-through).
    #[default]
    Loading,
    /// Animate hazard select entrance (pass-through until transitions are wired).
    AnimateIn,
    /// Player is selecting a hazard.
    Selecting,
    /// Animate hazard select exit (pass-through until transitions are wired).
    AnimateOut,
    /// Hazard select teardown — parent `RunState` watches for this.
    Teardown,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn default_hazard_select_state_is_loading() {
        assert_eq!(HazardSelectState::default(), HazardSelectState::Loading);
    }

    /// G.3 main case: `SubState` defaults to `Loading` when the parent first
    /// enters `RunState::HazardSelect`. Exercises only the transition up to
    /// `HazardSelect`, not through to `Selecting` (covered separately by G.4).
    #[test]
    fn substate_defaults_to_loading_when_parent_enters_hazard_select() {
        use crate::{prelude::*, shared::test_utils::TestAppBuilder};

        let mut app = TestAppBuilder::new().with_state_hierarchy().build();

        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunState>>()
            .set(RunState::HazardSelect);
        app.update();

        let state = app
            .world()
            .get_resource::<State<HazardSelectState>>()
            .expect("HazardSelectState should be active under RunState::HazardSelect");
        assert_eq!(
            state.get(),
            &HazardSelectState::Loading,
            "default substate on parent entry should be Loading"
        );
    }

    #[test]
    fn all_five_variants_exist_and_are_mutually_distinct() {
        let variants = [
            HazardSelectState::Loading,
            HazardSelectState::AnimateIn,
            HazardSelectState::Selecting,
            HazardSelectState::AnimateOut,
            HazardSelectState::Teardown,
        ];
        let debug_strings: HashSet<String> = variants.iter().map(|v| format!("{v:?}")).collect();
        assert_eq!(
            debug_strings.len(),
            5,
            "expected 5 unique debug strings for 5 variants, got {debug_strings:?}"
        );
        assert!(debug_strings.contains("Loading"));
        assert!(debug_strings.contains("AnimateIn"));
        assert!(debug_strings.contains("Selecting"));
        assert!(debug_strings.contains("AnimateOut"));
        assert!(debug_strings.contains("Teardown"));
    }
}
