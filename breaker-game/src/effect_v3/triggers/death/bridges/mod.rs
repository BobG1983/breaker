//! Death trigger bridges.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{
    on_bolt_destroyed, on_breaker_destroyed, on_cell_destroyed, on_salvo_destroyed,
    on_wall_destroyed,
};
