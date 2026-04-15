//! Chain lightning systems — tick chain arcs.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::tick_chain_lightning;
