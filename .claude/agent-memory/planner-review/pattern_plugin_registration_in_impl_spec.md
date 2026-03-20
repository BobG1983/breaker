---
name: plugin_registration_in_impl_spec
description: New systems added by impl specs must include plugin.rs registration — schedule, ordering, run_if, and set membership
type: feedback
---

When an implementation spec adds a new system function (e.g., `bridge_bump_whiff`), it must also specify:
1. Registration in the domain's `plugin.rs`
2. Schedule placement (`FixedUpdate`, `OnEnter(...)`, etc.)
3. Ordering constraints (`.after(...)`, `.before(...)`)
4. `run_if` conditions (state-gated, resource-gated)
5. Set membership (`.in_set(...)`) if the domain exports sets for cross-domain ordering

**Why:** Without plugin registration, the system function exists in the codebase but never runs. This is a silent failure — tests pass (they call the function directly) but the feature doesn't work in-game.

**How to apply:** For every new system function in an impl spec, verify the spec includes a `plugin.rs` registration line with all five elements above. Compare against existing systems in the same plugin for the pattern.
