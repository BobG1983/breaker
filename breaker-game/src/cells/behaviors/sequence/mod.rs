//! Sequence cell behavior — cells belong to a numbered sequence group with a
//! position index. Only the currently-active position in each group takes
//! damage normally; damage to any other member is reverted before death
//! detection. When the active member dies, `position + 1` becomes active.

pub(crate) mod components;
pub(crate) mod systems;

#[cfg(test)]
mod tests;
