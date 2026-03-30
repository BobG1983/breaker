pub(crate) mod checker;

#[cfg(test)]
mod tests;

pub use checker::check_aabb_matches_entity_dimensions;
