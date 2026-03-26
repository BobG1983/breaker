//! Spawns in-game text popups when highlight moments are detected.

mod system;

pub(crate) use system::spawn_highlight_text;

#[cfg(test)]
mod tests;
