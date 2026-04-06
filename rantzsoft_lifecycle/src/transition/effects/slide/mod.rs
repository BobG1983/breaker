pub(crate) mod effect;

#[cfg(test)]
mod tests;

pub use effect::{Slide, SlideConfig, SlideDirection};
pub(crate) use effect::{slide_end, slide_run, slide_start};
