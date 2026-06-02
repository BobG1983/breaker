pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::reseed_hazard_rng;
