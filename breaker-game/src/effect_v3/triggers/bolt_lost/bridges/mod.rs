//! Bolt-lost trigger bridges.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::on_bolt_lost_occurred;
