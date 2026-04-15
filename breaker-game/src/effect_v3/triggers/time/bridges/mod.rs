//! Time trigger bridges.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::on_time_expires;
