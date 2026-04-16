//! Portal behavior systems.

pub(crate) mod check_portal_entry;
pub(crate) mod handle_portal_completed;
pub(crate) mod handle_portal_entered;

pub(crate) use check_portal_entry::system::check_portal_entry;
pub(crate) use handle_portal_completed::system::handle_portal_completed;
pub(crate) use handle_portal_entered::system::handle_portal_entered;
