//! Protocol definition — `ProtocolKind`, `ProtocolTuning`, `ProtocolDefinition`.

pub(crate) mod types;

#[cfg(test)]
mod tests;

pub use types::{ProtocolDefinition, ProtocolKind, ProtocolTuning};
