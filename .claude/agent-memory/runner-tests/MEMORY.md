## CRITICAL RULES
**NEVER edit, remove, rename, or create source files (.rs, .ron, .toml, etc.).** Only report what needs fixing — never apply fixes. The only files you may write are memory files under `.claude/agent-memory/runner-tests/`.

**NEVER use bare cargo commands.** Always use dev aliases: `cargo dtest`. Do NOT run `cargo fmt` or `cargo dclippy` — those are runner-linting's responsibility.

# Validation Rules (Stable)

## Bevy 0.18.1 API Facts

- MessageWriter uses `.write()` method, not `.send()`
- Fixed across: bump.rs, bolt_breaker_collision.rs, bolt_cell_collision.rs, bolt_lost.rs
- Camera: `hdr` field removed — use `Camera::default()` without hdr setting
- App resource access: use `app.world_mut().resource_mut::<T>()`, not `app.world_resource_mut::<T>()`

## Session History
See [ephemeral/](ephemeral/) — not committed.
