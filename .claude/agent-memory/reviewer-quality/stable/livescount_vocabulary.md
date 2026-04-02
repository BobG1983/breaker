---
name: LivesCount is established vocabulary
description: LivesCount is the correct project term, documented in architecture and design docs — not a vocabulary violation
type: project
---

`LivesCount` is the correct identifier for the component tracking breaker lives. It is referenced in:
- `docs/architecture/builders/breaker.md`
- `docs/design/effects/lose_life.md`
- `docs/design/graphics/catalog/feedback.md`

**Why:** It predates the reviewer and is established codebase vocabulary. Do not flag as a vocabulary issue.
**How to apply:** When reviewing any file that uses `LivesCount`, treat it as correct.
