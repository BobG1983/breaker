//! `DeathPipelinePlugin` — registers the unified damage -> death -> heal -> despawn pipeline.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::DeathPipelinePlugin;
