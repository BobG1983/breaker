//! Generic 2D damage primitives for Bevy 0.18 games.
//!
//! This crate will provide reusable damage building blocks — HP tracking,
//! dead-state marking, invulnerability windows, and multi-stack damage
//! messages — intended for any 2D Bevy game. It obeys the zero-game-knowledge
//! contract documented in `.claude/rules/rantzsoft-crates.md`: no game-specific
//! vocabulary, entities, or assumptions leak into this crate.
//!
//! This crate root currently ships no public API; traits, components,
//! messages, systems, and the plugin arrive in subsequent phases as a
//! `Dmgable`-driven pipeline.

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles() {}
}
