---
name: Chip/Evolution Ecosystem Coherence Audit
description: Full audit of chip overlaps, orphan chips, evolution power curves, and design gaps. 7 priority actions identified.
type: project
---

## Chip/Evolution Ecosystem Audit — 2026-03-30

Full audit written to `.claude/specs/audit-design-coherence.md`.

### Critical Overlaps Found
1. **Surge vs Overclock** — identical trigger/effect, Surge makes Overclock obsolete. Need burst vs sustained differentiation.
2. **Singularity vs Glass Cannon** — Singularity strictly better. Glass Cannon needs redesign around fragility theme.
3. **Desperation vs Last Stand** — same mechanic, different numbers. Desperation needs secondary effect.

### Power Curve Issues
- **Too cheap**: Nova Lance (44%), Arcwelder (43%), Voltchain (50%) — near-accidental qualification
- **Too expensive**: Railgun (100%), Second Wind (100%) — requires maxing both ingredients
- **Damage Boost overloaded** — appears in 4 evolutions, making it a universal evolution key

### 6 Orphan Chips (no evolution path)
Aftershock, Amp, Impact, Pulse, Reflex, Quick Stop — all need evolution paths. Suggested pairings in the audit doc.

### RON File Status (updated 2026-04-02)
Previously flagged 5 missing RON files. Current status:
- `flashstep.evolution.ron` — NOW EXISTS (`assets/chips/evolutions/`)
- `chain_reaction.evolution.ron` — NOW EXISTS (`assets/chips/evolutions/`)
- `feedback_loop.chip.ron` — NOW EXISTS (`assets/chips/standard/`)
- `powder_keg.chip.ron` — NOW EXISTS (`assets/chips/standard/`)
- `chain_hit` chip — STILL MISSING (verified 2026-04-06; no RON file under assets/chips/)
Evolution RON files moved to `assets/chips/evolutions/` (plural, not `evolution/`).

### Design Gap: Trigger Diversity
PerfectBumped dominates the evolution trigger portfolio. Wall-bank, defense, and positioning playstyles are underserved by the evolution system.

**Why:** Comprehensive design coherence checkpoint. Reference before any chip/evolution content work.
**How to apply:** Consult priority action list when designing new chips or evolutions. Check overlap pairs before approving similar new chips.
