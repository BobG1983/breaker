//! Protocol resources — registries, active sets, offers, and run-condition helpers.

pub(crate) mod types;

#[cfg(test)]
mod tests;

pub use types::{ActiveProtocols, ProtocolRegistry};
pub(crate) use types::{ProtocolOffer, UnlockedProtocols, protocol_active};
