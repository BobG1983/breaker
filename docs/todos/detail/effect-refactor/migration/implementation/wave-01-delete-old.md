# Wave 1: Delete Old Effect Domain

## Goal
Remove the entire old `src/effect/` directory. Clean slate for the new implementation.

## Steps

1. Delete `src/effect/` entirely.
2. Remove `mod effect;` from `src/lib.rs` (or wherever it's declared).
3. Comment out or remove all `use crate::effect::*` imports across the codebase — these will be compile errors. Do NOT fix them yet (wave 2 handles that).
4. Remove `EffectPlugin` registration from the app builder (game.rs or equivalent).
5. The codebase will NOT compile after this wave. That is expected.

## Verification
None — this is destructive prep. The build is intentionally broken.

## Notes
- Do NOT delete `docs/todos/detail/effect-refactor/` — that's the spec, not the code.
- Do NOT delete shared types that other domains use (anything in `src/shared/`).
- Do NOT delete RON asset files in `assets/` — those are migrated in wave 3.
- Use `#[expect(unused_imports)]` on files you partially clean up if needed.

## Docs to read
None — this wave is mechanical deletion.
