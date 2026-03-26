//! Seeds `ChipRegistry` from loaded `ChipDefinition` assets.

mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::seed_chip_registry;
