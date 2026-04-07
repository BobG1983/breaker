pub(crate) mod effect;

#[cfg(test)]
mod tests;

pub use effect::{WipeIn, WipeInConfig, WipeOut, WipeOutConfig};
pub(crate) use effect::{
    wipe_in_end, wipe_in_run, wipe_in_start, wipe_out_end, wipe_out_run, wipe_out_start,
};
