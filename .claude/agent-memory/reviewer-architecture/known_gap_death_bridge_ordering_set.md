---
name: Death bridges cross-domain ordering anchor (RESOLVED)
description: EffectV3Systems::Death set added as the cross-domain tag surface for death-trigger bridges; ordering.md updated; gap closed
type: project
---

**Status: RESOLVED as of Follow-up 6 (quadtree migration branch).**

The four death-trigger bridges are now tagged with `EffectV3Systems::Death`,
which is configured as `.after(DeathPipelineSystems::HandleKill)` inside the
death triggers' own register function:

- `on_cell_destroyed`
- `on_bolt_destroyed`
- `on_wall_destroyed`
- `on_breaker_destroyed`

Registration site: `breaker-game/src/effect_v3/triggers/death/register.rs:17-30`
Set definition: `breaker-game/src/effect_v3/sets.rs:24` (`EffectV3Systems::Death`)

**Why the set is legitimate** (per `ordering.md:19` phase-set exception):
- Consistent with existing `EffectV3Systems::Bridge` / `Tick` / `Conditions`,
  which also host multiple systems from a single plugin.
- Cross-plugin consumer already in use: `cells/behaviors/volatile/tests/group_f.rs:24`
  uses `before(EffectV3Systems::Death)` to inject test `Destroyed<Cell>` messages.
- The set exists specifically to provide a single tag surface so consumers never
  have to reference individual bridge function names across domain boundaries
  (matches `ordering.md:16` "never reference bare system function names across
  domain boundaries").

**Why the tick-delay side effect still stands:** Moving bridges to `.after(HandleKill)`
was a latent bug fix so `Trigger::Died` reads same-tick `Destroyed<T>`. Effects
fired by death bridges (e.g., volatile cell explosions) still apply damage on
the *next* FixedUpdate tick because they are queued after `ApplyDamage` ran.
Tests/scenarios asserting chain timing must expect the 1-tick delay. See
`docs/architecture/effects/death_pipeline.md` if/when step 7 documents this.

**Doc status:** `docs/architecture/ordering.md:40` notes "death bridges are
tagged `EffectV3Systems::Death`, not `Bridge`"; row at line 43-44 documents the
`Death` set; FixedUpdate chain at line 183-184 shows the `.after(HandleKill)`
ordering. Ordering.md is current as of this PR.

**Split configure_sets caveat:** `EffectV3Plugin::build` configures
`Bridge → Tick → Conditions` in `plugin.rs:28-35`. `Death` is configured
separately inside `triggers::death::register::register`, which is invoked from
`plugin.rs:62` during plugin build. The owning plugin is still responsible for
configure_sets (it happens transitively via the helper). Not a violation — the
constraint is co-located with the systems that own it.
