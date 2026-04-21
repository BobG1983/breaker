//! System to spawn the chip selection screen UI.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::spawn_chip_select;
