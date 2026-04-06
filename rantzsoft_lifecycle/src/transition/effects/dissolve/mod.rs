pub(crate) mod effect;

#[cfg(test)]
mod tests;

pub use effect::{DissolveIn, DissolveInConfig, DissolveOut, DissolveOutConfig};
pub(crate) use effect::{
    dissolve_in_end, dissolve_in_run, dissolve_in_start, dissolve_out_end, dissolve_out_run,
    dissolve_out_start,
};
