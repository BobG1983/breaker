//! System to release locked cells when all adjacent cells are destroyed.

mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::check_lock_release;
