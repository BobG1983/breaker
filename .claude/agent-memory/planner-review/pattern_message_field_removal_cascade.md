---
name: Message field removal cascade
description: Removing a field from a Message struct breaks ALL construction sites; specs must enumerate every file with constructions (not just "update ALL")
type: feedback
---

When a spec removes a field from a Message struct (e.g., `multiplier` from `BumpPerformed`), every test that constructs the struct will fail to compile. Specs that say "Update ALL BumpPerformed constructions" without listing the files lead to missed sites.

**Why:** Writer-code agents work file-by-file and may not grep the entire codebase. A vague "update ALL" instruction doesn't tell them HOW MANY sites there are or WHERE they are.

**How to apply:** For message field removals, grep for `StructName {` across the entire `src/` tree and list every file + approximate count of constructions. Include both production code (system fn bodies) and test code (#[cfg(test)] modules). Also check for `.field_name` reads — even if the struct compiles without the field, existing code reading the removed field will also break.

Related pattern: pattern_message_field_additions.md (the inverse — adding fields)
