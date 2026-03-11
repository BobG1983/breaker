//! Debug domain systems.

#[cfg(feature = "dev")]
mod debug_ui;

#[cfg(feature = "dev")]
pub use debug_ui::debug_ui_system;
