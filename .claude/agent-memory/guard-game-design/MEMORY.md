# Game Design Guard Memory

## Decisions
- [Multiplicative Stacking Approved](decision-multiplicative-stacking.md) — Phase 3 additive-to-multiplicative change evaluated and approved against all pillars
- [Phase 4+5 Effect Roster Evaluation](evaluation-phase4-5-effects.md) — 14/15 effects approved; Shield redesigned to charges; BASE_BOLT_DAMAGE gap RESOLVED (BoltDefinition.base_damage)
- [Runtime Effects Round 2](evaluation-runtime-effects-round2.md) — source_chip threading, Shield charges, Chain Lightning arcs all approved; tuning notes
- [Full Verification 2026-03-30](evaluation-full-verification-2026-03-30.md) — Both blockers RESOLVED; 3 open concerns: breaker archetype differentiation, Surge permanent stacking, Attraction(Breaker) gate

- [Wave 3 TetherBeam Chain + Inherit Fix](evaluation-wave3-tetherbeam-inherit.md) — Chain mode approved with spawn governor; inherit fix approved as correctness; bolt-proliferation archetype validated
- [Bolt Builder Migration](evaluation-bolt-builder-migration.md) — Directional steering, BreakerReflectionSpread rename, PrimaryBolt marker all approved
- [Breaker Builder Pattern](evaluation-breaker-builder-pattern.md) — Typestate builder, LivesSetting, ClampRange, 35+ field definition, Visual dimension all approved
- [Wall Builder Pattern](evaluation-wall-builder-pattern.md) — Typestate builder, WallDefinition RON, WallRegistry, per-side definitions, Visible dimension all approved

- [Shield Wall Refactor](evaluation-shield-wall-refactor.md) — Charges to timed visible wall APPROVED; tension tuning needed (5s too generous); cell shielding removal correct
- Node Sequencing & Mod System — Pass 6: 10 Terminal candidates proposed (designer picks 5); Volatile Revenge kept; 4 originals rejected — see `docs/todos/detail/node-sequencing-refactor/` and `docs/todos/detail/mod-system-design.md` for authoritative state
- [Bolt Birthing Animation](evaluation-bolt-birthing-animation.md) — Concept approved; 0.3s too slow (recommend 0.12-0.15s); linear lerp weak (ease-out); AnimateIn gate questionable; tether beam interaction approved; quit chain approved

- [Protocol Brainstorm Evaluation](evaluation-protocol-brainstorm.md) — 15 protocol designs; 2 legendaries promoted; archetypes and hazard counter-play mapped

## Audits
- [Chip/Evolution Coherence Audit](audit-chip-evolution-coherence.md) — 3 critical overlaps, 6 orphan chips, power curve issues; 4 of 5 missing RON files now exist (flashstep, chain_reaction, feedback_loop, powder_keg)

## Open Concerns
- [Open Design Concerns](open-design-concerns.md) — Breaker archetype differentiation, Surge permanent stacking, Attraction(Breaker) gate

## Session History
See [ephemeral/](ephemeral/) — not committed.
