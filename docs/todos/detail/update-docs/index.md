# Update the docs — design-doc alignment sweep

Pure documentation-alignment work. No code changes, no RON changes, no test changes — every entry updates a design doc, architecture doc, or cross-cutting doc to match shipped behavior, pins design decisions that were left implicit, or **promotes the per-mechanic design docs out of `mod-system-design/` into `docs/design/{protocols,hazards}/` as canonical** (cleansed to describe the post-TODO target).

## Why this is TODO #0

Unblocks every subsequent TODO that references "update design doc X" as a side deliverable. Instead of each per-mechanic TODO dragging its design-doc edits along (and creating merge noise with this sweep), the sweep lands FIRST, canonicalizes the doc set, and subsequent TODOs touch only the pieces of the docs that genuinely change under their code edits.

**Critically, this sweep eliminates the multiple-sources-of-truth problem for protocols and hazards.** Today, `docs/todos/detail/mod-system-design/` holds a frozen pre-TODO baseline that contradicts the fix-broken deltas. Subagents implementing TODO #9 would hit conflicts. Promoting the per-mechanic docs into `docs/design/` (cleansed to post-TODO state) and deleting `mod-system-design/` gives TODO #9 a single canonical design source.

Doesn't block anything technically — these are documentation touches, doc-only failures don't cascade. But landing early is cheapest: spec writers, reviewers, and future-me read these docs constantly; stale docs cost context re-verification on every read.

## Structure

Each file in this folder is a single targeted doc update. Files are self-contained — they specify which file(s) to edit, which paragraphs/sections, and what the replacement text is.

Files:

### Per-mechanic design-doc alignment

| File | Target |
|------|--------|
| `iron-curtain-design-doc-abs-symmetric.md` | `docs/todos/detail/mod-system-design/protocols/iron_curtain.md` — document the `|x|`-symmetric falloff formula |
| `reckless-dash-design-doc-bump-performed.md` | `docs/todos/detail/mod-system-design/protocols/reckless_dash.md` — fix the trigger-message reference |
| `renewal-design-doc-alignment.md` | `docs/todos/detail/mod-system-design/hazards/renewal.md` — align design/impl divergence |
| `sympathy-design-doc-alignment.md` | `docs/todos/detail/mod-system-design/hazards/sympathy.md` — document Sympathy's post-apply chain position |
| `gravity-surge-design-doc-multiplicative-formula.md` | `docs/todos/detail/mod-system-design/hazards/gravity_surge.md` — formula correction |
| `erosion-design-doc-effect-stack-pattern.md` | `docs/todos/detail/mod-system-design/hazards/erosion.md` — name EffectStack as canonical |
| `fracture-design-doc-pins.md` | `docs/todos/detail/mod-system-design/hazards/fracture.md` — pin worked examples |
| `resonance-design-doc-alignment.md` | `docs/todos/detail/mod-system-design/hazards/resonance.md` — fix design/impl divergence |
| `tier-regression-design-doc-once-per-run.md` | `docs/todos/detail/mod-system-design/protocols/tier_regression.md` — correct the "whether taken or not" language |
| `diffusion-flat-share-documentation.md` | `docs/todos/detail/mod-system-design/hazards/diffusion.md` + `docs/architecture/effects.md` — document flat-share-per-ring as deliberate simplification |
| `haste-effect-stack-canonical.md` | `docs/todos/detail/mod-system-design/hazards/haste.md` — commit to EffectStack pattern (drop `ApplyBoltSpeedMultiplier` hedge) |
| `overcharge-design-doc-alignment.md` | `docs/todos/detail/mod-system-design/hazards/overcharge.md` — §Config Resource field rename; §Systems delete `attach_overcharge_tracker`; §Expected Behaviors worked examples |

### Architecture-doc additions

| File | Target |
|------|--------|
| `architecture-doc-accepted-patterns.md` | `docs/architecture/plugins.md` — new `§ Accepted Cross-Domain Mechanisms` section |

### Wholesale promotion

| File | Target |
|------|--------|
| `promote-mod-system-design.md` | Move + cleanse 31 per-mechanic design docs from `docs/todos/detail/mod-system-design/{protocols,hazards}/` to `docs/design/{protocols,hazards}/`. Delete `mod-system-design/`. Update `fix-broken-protocols-and-hazards/` cross-refs. |

Removed (originally proposed but invalidated):
- `attach-every-tick-is-canonical.md` — pinned the WRONG pattern as canonical. The real canonical is `Added<T>`; code migration lives in `fix-broken-protocols-and-hazards/attach-system-migration.md`.
- `volatility-design-doc-attach-every-tick.md` — aligned the Volatility design doc to the wrong impl. The design doc already says "runs on cell spawn" (correct). The attach-system-migration fixes the impl to match.

## Ordering within the sweep

Land in any order — every file is independent. Parallel waves are possible (one writer per file, batch-reviewed). A single commit per file keeps diff-review focused.

## Scope boundary

In scope:
- Text changes in `docs/design/`, `docs/architecture/`, `docs/todos/detail/mod-system-design/`.

Out of scope:
- Any code change.
- Any RON change (covered by `audit/remediations/ron-tuning-values.md`).
- Any test change.
- CLAUDE.md / rules files (`.claude/rules/*`) — those are orchestrator rules, not design docs.

## Reference

Dependencies: none. Can land standalone at any time.

Subsumes: all files previously named `doc-fix-*` in `audit/remediations/`. Those files moved here verbatim (prefix stripped).
