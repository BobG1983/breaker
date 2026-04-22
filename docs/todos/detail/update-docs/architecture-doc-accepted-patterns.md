# Document accepted cross-cutting architectural patterns

## Problems addressed

- `audit/architecture/violations.md` Issue 22 — Effect-stack reconciliation pattern (used by Haste, Erosion, Overcharge, Resonance) not documented in `docs/architecture/plugins.md` as an accepted cross-domain mechanism.
- `audit/architecture/violations.md` Issue 31 — Protocol modules own bolt-scoped components (DebtCollector's `DebtStack` / `DebtCashOut`, EchoStrike's `EchoNetwork` / `EchoPrimed`, RecklessDash's pre-remediation `RiskyDamageBoost`) — convention not documented in architecture.
- `audit/architecture/violations.md` Issue 33 — Damage-modifying hazards (Diffusion, Tether) live in the cells domain; damage-reactive hazards (Sympathy, Cascade, Volatility) live in the hazard domain. Split is architectural but undocumented.
- `audit/architecture/violations.md` Issue 23 — Fractional↔percent authoring convention (`base_xxx_percent: base_xxx_frac * 100.0`) duplicated across hazards. Decision pinned below: keep as-is.
- `audit/architecture/violations.md` Issue 27 — Approaching-threshold files (momentum 365, reckless_dash 344, echo_strike 337). Decision pinned below: watch-list, no remediation until they cross 400.
- `audit/architecture/violations.md` Issue 32 — Two-level anti-feedback guard pattern (Reckless Dash). Obsolete: retired by `breaker-bolt-lost-effect-component.md`. Close as not-a-concern.

## Remediation

### A. Update `docs/architecture/plugins.md` — accepted cross-domain mechanisms

Add a new section `§ Accepted Cross-Domain Mechanisms` BEFORE the `§ Cross-Domain Write Exceptions` section. Move the existing "exception registry" entries into a subsection. The new parent section enumerates mechanisms the architecture sanctions — effect-stack reconciliation, damage-message composition, protocol-scoped components on foreign entities — each with a definition, allowed callers, and the invariants it preserves.

Subsection to add under that parent:

> **Effect-stack reconciliation**
>
> A hazard or protocol domain may own an `EffectStack<T>` component that accumulates source-tagged entries on a foreign-domain entity. Entries are reconciled via `retain_by_source(source)` followed by `push(source, value)` every tick (the "attach every tick" pattern) OR via `fire_effect` / `reverse_effect` / `consume_on_use` upsert semantics.
>
> **Allowed domains**: hazard, protocol, effect_v3.
>
> **Allowed target entities**: Breaker (SizeBoost from Erosion, SpeedBoost from Resonance, DamageBoost from any Pattern A/B amplifier) and Bolt (SpeedBoost from Haste/Overcharge, DamageBoost from Burnout / Debt Collector / Echo Strike / Iron Curtain / Reckless Dash).
>
> **Invariant**: source-tagged entries ensure the target-domain aggregator (the system that sums entries into a single effective value) stays stateless with respect to which domain emitted each entry. Domains cannot collide on the same source string.
>
> **Why it's not a boundary violation**: `EffectStack<T>` is a foreign-domain-INSERTED but target-domain-CONSUMED component. Only the target domain reads the aggregated value. The foreign domain is the producer of source-tagged contributions, not the consumer of target state.

### B. Update `docs/architecture/plugins.md` — protocol-scoped components on foreign entities

Add another subsection under `§ Accepted Cross-Domain Mechanisms`:

> **Protocol-scoped components on bolts**
>
> A protocol module may define and attach its own marker/data components on bolt entities, provided:
> 1. The component is owned by the protocol module (defined in `protocol/protocols/<name>/components.rs`).
> 2. The component is only read by systems in the same protocol module.
> 3. The component does not duplicate data the bolt domain already owns.
>
> Examples: `DebtStack` + `DebtCashOut` (Debt Collector), `EchoNetwork` + `EchoPrimed` (Echo Strike). These are protocol state stored alongside the bolt rather than in a run-scoped `Resource` because the state is per-bolt.
>
> **Cleanup**: components follow the bolt's lifecycle via `CleanupOnExit<RunState>` (primary) or `CleanupOnExit<NodeState>` (extra) — no separate cleanup needed.
>
> **Why it's not a boundary violation**: inserting a component on another domain's entity is the accepted ECS extension mechanism. The protocol domain isn't modifying bolt-owned components; it's attaching its own.

### C. Update `docs/architecture/plugins.md` — damage-mutator vs damage-reactor split

Add a subsection under `§ Accepted Cross-Domain Mechanisms`:

> **Damage-mutator vs damage-reactor hazards**
>
> Hazards that COMPOSE damage — Diffusion (splits a cell hit into attenuated ring emissions), Tether (redirects a fraction to a linked partner) — currently live inside `cells::systems::apply_damage_to_cells` as inline branches. This is a documented architectural exception because these hazards must observe the post-prior-mutator damage amount and emit further damage before the final HP apply.
>
> Hazards that REACT to damage — Sympathy (heals neighbors on cell damage), Cascade (damages neighbors on cell destruction), Volatility (resets on damage) — live in the hazard domain and consume `DamageDealt<Cell>` or `Destroyed<Cell>` messages via `MessageReader` after the damage has applied.
>
> **Target shape**: when `damage-message-mutator-chain.md` lands, damage-mutator hazards move to the hazard domain as `MessageMutator<DamageDealt<Cell>>` systems chained before `apply_damage_to_cells`. The cells-domain exception for Diffusion+Tether math is deleted at that point.

### D. No action on Issue 23 (fractional↔percent convention)

The activation-time `base_xxx_percent: base_xxx_frac * 100.0` translation is simple and readable at each call site. Centralizing via `#[serde(from = ...)]` adapters would save ~3 lines per hazard at the cost of a deserialization-time surprise. Keep the current explicit pattern; no `serde` magic. The duplication is acceptable given the low line count and high legibility.

Close Issue 23 as a pinned decision.

### E. No action on Issue 27 (approaching-threshold files)

The file-split rule triggers at 400 lines. Files at 337, 344, and 365 lines are under the threshold. Watch list only; no remediation until a file crosses 400.

Close Issue 27 as a pinned decision.

### F. Close Issue 32 (two-level anti-feedback guard)

Reckless Dash's anti-feedback guard was retired by `breaker-bolt-lost-effect-component.md` — the new `BreakerBoltLostEffect { life_loss, time_loss }` component replaces the guard with in-place mutation of the breaker-owned effect. No pattern to document because the pattern is gone.

Close Issue 32 as obsolete.

## File change summary

- `docs/architecture/plugins.md` — new `§ Accepted Cross-Domain Mechanisms` section with three subsections (effect-stack reconciliation, protocol-scoped components on bolts, damage-mutator vs damage-reactor split). Existing `§ Cross-Domain Write Exceptions` becomes a child of the new parent section.
- No code changes.
