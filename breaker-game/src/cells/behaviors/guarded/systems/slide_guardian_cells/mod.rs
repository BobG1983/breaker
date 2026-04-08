//! System to slide guardian cells around their parent's ring.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::slide_guardian_cells;
