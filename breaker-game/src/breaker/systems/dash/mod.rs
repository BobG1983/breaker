//! Dash, brake, and settle state machine systems.

mod system;

#[cfg(test)]
mod tests;

pub(super) use system::eased_decel;
pub(crate) use system::update_breaker_state;
