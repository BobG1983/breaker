pub(crate) mod bridge;
#[cfg(test)]
mod tests;

pub(crate) use bridge::{
    bridge_breaker_impacted, bridge_cell_impacted, bridge_wall_impacted, register,
};
