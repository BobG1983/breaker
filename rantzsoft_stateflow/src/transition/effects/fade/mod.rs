pub(crate) mod effect;

#[cfg(test)]
mod tests;

pub use effect::{FadeIn, FadeInConfig, FadeOut, FadeOutConfig};
pub(crate) use effect::{
    fade_in_end, fade_in_run, fade_in_start, fade_out_end, fade_out_run, fade_out_start,
};
