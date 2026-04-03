---
name: ShieldActive cross-domain write exception — ELIMINATED
description: ShieldActive no longer exists as of Shield refactor (2026-04-02); shield is now a timed floor wall (ShieldWall) with collision handled by bolt_wall_collision
type: project
---

**ELIMINATED as of Shield refactor (2026-04-02, commit e887570).**

`ShieldActive` component NO LONGER EXISTS. The cross-domain write exception in `docs/architecture/plugins.md` for this component was removed with the refactor.

Shield is now implemented as a timed visible floor wall (`ShieldWall` + `ShieldWallTimer`) that uses the normal `bolt_wall_collision` path. No cross-domain writes needed.

- `bolt_lost` — no longer reads `ShieldActive`
- `handle_cell_hit` — no longer reads `ShieldActive`
- Cell shielding via Shield effect: REMOVED

**Do NOT re-flag any absence of ShieldActive cross-domain write exceptions — the entire mechanism was redesigned.**

The old plugins.md exception section has been deleted.
