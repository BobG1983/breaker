# Revisit piercing model end-to-end

## Status

**NEEDS DETAIL** — this is a collection of open questions surfaced during W6/W7 of the `rantzsoft_dmg` port. A real design pass should happen before implementation.

## Open questions

### 1. One-shot boosts and pierce decisions

`resolve_bolt_cell_hit` uses `DamageBoostStack::aggregate_persistent` — one-shot boosts do NOT count toward `would_destroy`. But one-shots DO apply to delivered damage via the pipeline.

Consequence: a one-shot `DamageBoost` that makes a hit lethal still kills the cell, but the bolt reflects instead of piercing (because the pierce check used persistent-only boost and said "not lethal").

Is this intentional? The current code comment doesn't claim it is — it's an artifact of "we didn't want to speculatively consume one-shots during lookahead".

Options:
- (a) Accept it; document it as designed behavior.
- (b) Let one-shots affect pierce decisions; accept that `would_destroy`'s prediction is only reliable when the bolt actually gets to consume them (which it will, since `DamageDealt<Cell>` writes happen from this same system).
- (c) Require "pierce gets preview, kill gets actual"; reconcile with a reactive pierce (see §2).

### 2. Speculative vs reactive pierce

Current: pierce decision is made SYNCHRONOUSLY inside the CCD sweep, before damage is actually applied. The system predicts.

Alternative: pierce decision is reactive — emit a tentative hit, let the pipeline apply damage, check `Destroyed<Cell>` next tick and then decide whether the bolt keeps going.

Trade-offs:
- Speculative (current): pierce feels instantaneous; divergence risk (see §1).
- Reactive: pierce has a 1-frame "stickiness" before the bolt continues; cleaner data flow; no duplicated math.

This is a design call, not just an engineering one.

### 3. Chain pierce overkill math

If a bolt with one-shot boosts enters a CCD sweep that would hit 3 cells in a row:
- Cell A: pierce decision says "yes, would kill" (using persistent boost). One-shot available.
- Cell B: same pierce decision runs; one-shot is still in the stack.
- Cell C: same.

But the PIPELINE applies damage sequentially: cell A consumes the one-shot, cell B receives only the persistent-boosted amount, cell C same.

The pierce decision for cells B and C was made assuming the full persistent-only value, which is actually what they receive. So chain pierce is consistent with the persistent-only pierce model — but only by coincidence. If we ever let one-shots affect pierce (§1 option b), we'd need to model consumption during lookahead.

### 4. Armor interaction

`check_armor_direction` consumes `PiercingRemaining` when armor blocks a hit. It runs `.after(BoltSystems::CellCollision).before(DmgSystems::ApplyDamage)`. The bolt's pierce decision was already made in `CellCollision`, using a `PiercingRemaining` that the armor check hasn't yet decremented.

Is there a scenario where the bolt's decision says "pierce, I have 3 charges" but armor later consumes 3 charges, leaving the bolt with 0 charges at the next cell, yet the CCD sweep has already committed to piercing through multiple cells?

Needs a trace.

### 5. Design doc

There isn't a current piercing design doc. Writing one — even short — would likely surface which of the above are bugs vs. features.

## Dependencies

- ~~TODO #26 (pierce-decision damage duplication) — introduces `preview_damage` so pierce and pipeline use the same function.~~ **LANDED** — `rantzsoft_dmg::preview_damage` exists and is wired into `bolt_cell_collision`. This prerequisite is resolved.
- TODO #9 (broken protocols/hazards) and the `Fireable::fire` refactor (W9 of port-to-rantzsoft-dmg.md) touch similar emission patterns; piercing-beam is the obvious sibling.

## Non-goals

- ~~Don't start this work until #26 lands~~ — `preview_damage` has landed; the gate is open.
- Don't assume current behavior is correct. Investigate first, decide second.

## Deliverable

A design doc at `docs/design/bolt/piercing.md` answering questions 1–4 with explicit decisions, followed by whatever code changes follow from those decisions.
