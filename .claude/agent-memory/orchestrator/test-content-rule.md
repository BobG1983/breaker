---
name: Tests should not depend on specific content RON files
description: Tests create their own data; only a single "all RONs parse" test validates content files
type: feedback
---

Tests should NOT read specific content RON files (like fortress.node.ron) because content files aren't permanent. Tests should create layouts/definitions programmatically for testing purposes.

**Why:** Content files are designed to change — they're game data, not contracts. Tests that depend on specific content values are brittle and couple tests to content authoring.

**How to apply:** Create test data inline in test helpers. A single `all_*_rons_parse` test per asset folder is fine for release validation (iterates all RON files, checks they deserialize and pass validation), but no test should reference a specific content file by name.
