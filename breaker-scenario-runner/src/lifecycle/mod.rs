//! Scenario lifecycle — state navigation, frame counter, and exit logic.
//!
//! [`ScenarioLifecycle`] is a Bevy plugin that:
//! - Bypasses menus: `OnEnter(MenuState::Main)` → navigates through state hierarchy
//! - Auto-skips chip selection: `PostUpdate` when `ChipOffers` exists → `ChipSelectState::Teardown`
//! - Counts fixed-update frames via [`ScenarioFrame`]
//! - Exits when `max_frames` is reached; optionally exits when the run ends
//!   naturally (controlled by [`ScenarioDefinition::allow_early_end`])

pub use systems::*;

mod systems;

#[cfg(test)]
mod tests;
