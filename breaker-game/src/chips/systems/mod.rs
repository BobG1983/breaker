//! Systems for the chips domain.

pub(crate) mod build_chip_catalog;
pub(crate) mod dispatch_chip_effects;

pub(crate) use build_chip_catalog::build_chip_catalog;
pub(crate) use dispatch_chip_effects::dispatch_chip_effects;
