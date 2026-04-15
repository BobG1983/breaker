//! Locked behavior systems.

mod check_lock_release;
pub(crate) mod sync_lock_invulnerable;

pub(crate) use check_lock_release::check_lock_release;
