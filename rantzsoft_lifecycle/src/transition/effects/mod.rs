//! Built-in transition effects.
//!
//! Each module defines a pair of transition effects (In/Out or `OneShot`)
//! with their config resources and start/run/end systems.

/// Dissolve transition effect.
pub mod dissolve;
/// Fade transition effect.
pub mod fade;
/// Iris transition effect.
pub mod iris;
/// Pixelate transition effect.
pub mod pixelate;
/// Shared transition types and components.
pub mod shared;
/// Slide transition effect.
pub mod slide;
/// Wipe transition effect.
pub mod wipe;

pub(crate) mod registration;

// Public re-exports — effect structs
pub use dissolve::{DissolveIn, DissolveInConfig, DissolveOut, DissolveOutConfig};
pub use fade::{FadeIn, FadeInConfig, FadeOut, FadeOutConfig};
pub use iris::{IrisIn, IrisInConfig, IrisOut, IrisOutConfig};
pub use pixelate::{PixelateIn, PixelateInConfig, PixelateOut, PixelateOutConfig};
pub(crate) use registration::register_builtin_transitions;
pub use shared::{ScreenSize, TransitionOverlay, TransitionProgress, WipeDirection};
pub use slide::{Slide, SlideConfig, SlideDirection};
pub use wipe::{WipeIn, WipeInConfig, WipeOut, WipeOutConfig};

#[cfg(test)]
mod tests;
