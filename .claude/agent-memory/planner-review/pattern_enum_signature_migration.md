---
name: Enum variant signature changes break existing tests
description: When a spec changes a public enum variant's field signature (e.g., adding a parameter to TriggerChain::OnImpact), ALL existing construction sites in tests break — spec must enumerate them as prerequisites
type: feedback
---

When a spec proposes changing the signature of an enum variant (adding fields, changing field types, renaming), every existing test that constructs or pattern-matches on that variant will fail to compile.

**Why:** Enum variant signature changes are not additive — they're breaking changes at every call site. Specs that focus on NEW behaviors often miss the existing code that must be updated first.

**How to apply:** When reviewing a spec that modifies an enum variant signature:
1. Grep for all construction sites of the variant (`TriggerChain::OnImpact(` etc.)
2. Grep for all pattern match sites
3. Count them — if >5, the prerequisite work may need its own commit
4. Verify the spec either (a) lists them all as prerequisite updates or (b) explicitly says main agent handles them
5. Check RON files and scenario RON files that serialize the variant
