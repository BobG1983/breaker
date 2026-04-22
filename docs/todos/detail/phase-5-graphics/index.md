# Phase 5: Graphics foundation

Squashed from the former Phase 5c–5w per-sub-phase breakdown. Approach re-considered from scratch — start with cataloging and art direction, THEN implement systems, THEN replace placeholders.

## Steps

1. **Catalog all required graphics.** Enumerate every visual element the game needs: breakers, bolts, cells (per kind, per HP tier), walls, backgrounds, VFX for each protocol/hazard/chip effect, bump grade feedback, HUD, chip cards, screens, transitions, highlight moments. Output: a single document listing every item with a short description. No art yet.
2. **Decide what the look is.** Art direction pass. Reference boards, palette decisions, motion principles (kinetic/readable/punchy), and the role of each visual element in reinforcing the design pillars. Output: a style guide the remaining steps execute against.
3. **Create particle and VFX systems or crates.** Build the technical foundation needed by the style guide. Likely: particle plugin (possibly a new `rantzsoft_*` crate), post-processing pipeline, tint/flicker primitives, easing utilities. Scope driven by what step 2 actually demands — no speculative systems.
4. **Add non-placeholder graphics.** Replace every placeholder (tints, flickers, simple shapes) with the real visuals from the style guide. One mechanic at a time, prioritized by player-visibility.

## Status

Planning. No implementation has started. Prior per-sub-phase detail files (5c–5w) were deleted when this TODO was squashed — the new approach starts from step 1.

## Sub-files

- [`placeholder-vfx.md`](placeholder-vfx.md) — placeholder-VFX sweep for deferred hazard/protocol visuals (drift arrow, echo ghost tint, resonance/overcharge/renewal tints, etc.). Rides with step 4; interim step-3-adjacent placeholders can land sooner if the mechanic is otherwise invisible without them.
