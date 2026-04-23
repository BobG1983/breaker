//! `handle_kill::<T>` — see `system.rs` for docs.

mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::handle_kill;
