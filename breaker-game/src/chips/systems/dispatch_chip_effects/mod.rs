//! Thin dispatcher: reads [`ChipSelected`] messages, looks up the chip in the
//! [`ChipRegistry`], and dispatches effects via `RootEffect::On` target routing.

mod system;

pub(crate) use system::dispatch_chip_effects;

#[cfg(test)]
mod tests;
