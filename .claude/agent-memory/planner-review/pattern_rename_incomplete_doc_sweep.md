---
name: Rename with incomplete doc sweep
description: Field rename specs list some doc files but miss others; always verify all occurrences before approving
type: feedback
---

When a spec includes a field rename (e.g., `bonus_per_hit` → `damage_per_trigger`), the Implementation Spec section lists affected files. This list is often incomplete for docs because the spec author searched by memory rather than by grep.

Common misses:
- The `docs/design/chip-catalog.md` table entries that show RON syntax inline (not code fences)
- The `docs/architecture/content.md` EffectKind enum listing
- The effect's own `docs/design/effects/<name>.md` parameter table
- The `EffectKind` enum doc comment in the actual source file (not just the field)

**Why:** Spec authors tend to list files they remember touching, but inline RON syntax in tables (like chip-catalog.md) doesn't look like "code" so it gets missed.

**How to apply:** When reviewing a rename spec, mentally enumerate: source Rust file, EffectKind enum, all RON files, all doc files that mention the variant in tables or prose. Cross-check the spec's "Update docs" list against this enumeration. Flag any gap as IMPORTANT.
