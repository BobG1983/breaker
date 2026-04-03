//! State enum definitions.

mod app_state;
mod chip_select_state;
mod game_phase;
mod game_state;
mod menu_state;
mod node_state;
mod playing_state;
mod run_end_state;
mod run_phase;

// Old state types (used by all existing systems, deleted in Wave 4e)
// New hierarchical state types (registered alongside old ones, used starting in Wave 4b)
pub use app_state::AppState;
pub use chip_select_state::ChipSelectState;
pub use game_phase::GamePhase;
pub use game_state::GameState;
pub use menu_state::MenuState;
pub use node_state::NodeState;
pub use playing_state::PlayingState;
pub use run_end_state::RunEndState;
pub use run_phase::RunPhase;
