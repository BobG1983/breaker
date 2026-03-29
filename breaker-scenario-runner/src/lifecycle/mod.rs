//! Scenario lifecycle — state navigation, frame counter, and exit logic.
//!
//! [`ScenarioLifecycle`] is a Bevy plugin that:
//! - Bypasses menus: `OnEnter(GameState::MainMenu)` → immediately enters `Playing`
//! - Auto-skips chip selection: `PostUpdate` when `ChipOffers` exists → `TransitionIn`
//! - Counts fixed-update frames via [`ScenarioFrame`]
//! - Exits when `max_frames` is reached; optionally exits when the run ends
//!   naturally (controlled by [`ScenarioDefinition::allow_early_end`])

pub use systems::*;

mod systems;

#[cfg(test)]
mod tests;
