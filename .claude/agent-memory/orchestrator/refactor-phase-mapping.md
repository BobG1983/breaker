---
name: REFACTOR phase is distributed across Phase 2 + Phase 3 + /simplify
description: TDD REFACTOR maps to reviewers (find issues) → Phase 3 routing (fix issues) → /simplify (catch the rest) → wiring — not a single agent or step
type: feedback
---

The TDD REFACTOR phase is not a single step — it's distributed across the post-implementation pipeline:

1. **Reviewers find what to refactor** (Phase 2): reviewer-quality, reviewer-correctness, reviewer-bevy-api, reviewer-performance, reviewer-architecture
2. **Phase 3 routing executes the fixes**: reviewer findings route to writer-code or inline main-agent fixes
3. **`/simplify` catches the rest**: runs on changed code after Phase 3 settles
4. **Wiring is the final cleanup**: main agent integrates modules

REFACTOR is complete when all Phase 2 agents pass and `/simplify` finds nothing to change.

**Why:** The original docs said "RED → GREEN → REFACTOR" but REFACTOR was never cleanly called out as a distinct stage. Readers couldn't find where it lived. This mapping makes it explicit.

**How to apply:** When describing the TDD cycle to specs, agents, or the user, refer to steps 9–12 of the flow as the REFACTOR phase. Don't label only wiring as REFACTOR — the reviewers and fix routing are the substantive part.
