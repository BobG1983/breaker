# Implementation Waves

Ordered sequence of 15 implementation waves. Each wave is self-contained — complete one before starting the next. RED/GREEN gates within each TDD wave.

## Clippy Rule

**NEVER** use `#[allow(...)]` to suppress clippy lints. The lint config is intentional.

**CAN** use `#[expect(...)]` for temporary suppression of: `dead_code`, `unused_imports`, `unused_variables`, `unreachable_code`, `clippy::unimplemented`, `clippy::todo`. These are expected during stub waves when code exists but isn't wired up yet.

**Wave 13 step 1** removes ALL `#[expect(...)]` annotations. If removing an expect causes a lint failure, the underlying issue must be fixed — not re-suppressed.

## Waves

### Scaffold
1. [wave-01-delete-old.md](wave-01-delete-old.md) — Delete old effect domain
2. [wave-02-scaffold.md](wave-02-scaffold.md) — Create files, write types, stub functions/systems, migrate callsites, wire plugins
3. [wave-03-ron-assets.md](wave-03-ron-assets.md) — Replace RON assets with new syntax

### Effect system (RED → GREEN per wave)
4. [wave-04-functions/](wave-04-functions/plan.md) — Non-system functions (EffectStack, walking, dispatch, commands, passive effects)
5. [wave-05-triggers/](wave-05-triggers/plan.md) — Trigger bridge systems + game systems
6. [wave-06-effects/](wave-06-effects/plan.md) — All 30 effects + tick systems + conditions
7. [wave-07-death-pipeline/](wave-07-death-pipeline/plan.md) — Death pipeline systems (apply_damage, detect_deaths, process_despawn)
8. [wave-08-integration/](wave-08-integration/plan.md) — Cross-domain end-to-end tests

### Domain migration (RED → GREEN per wave)
9. [wave-09-cell-domain/](wave-09-cell-domain/plan.md) — Cell domain → death pipeline
10. [wave-10-bolt-domain/](wave-10-bolt-domain/plan.md) — Bolt domain → death pipeline
11. [wave-11-wall-domain/](wave-11-wall-domain/plan.md) — Wall domain → death pipeline
12. [wave-12-breaker-domain/](wave-12-breaker-domain/plan.md) — Breaker domain → death pipeline

### Ship
13. [wave-13-standard-verification.md](wave-13-standard-verification.md) — Standard Verification Tier (remove #[expect], commit gate)
14. [wave-14-full-verification.md](wave-14-full-verification.md) — Full Verification Tier (pre-merge gate)
15. [wave-15-commit-merge.md](wave-15-commit-merge.md) — Commit, merge, push

## TDD Gates

Each RED → GREEN wave contains both phases in a single doc:
1. **RED**: Write failing tests. Tests must compile AND fail.
2. **GREEN**: Implement to pass tests. Do NOT modify tests.

| Wave | Domain |
|------|--------|
| 4 | Functions (EffectStack, walking, dispatch, commands) |
| 5 | Triggers (bump, impact, death, bolt-lost, node, time) |
| 6 | Effects (30 effects, tick systems, conditions) |
| 7 | Death pipeline (apply_damage, detect_deaths, process_despawn) |
| 8 | Integration (cross-domain flows) |
| 9 | Cell domain migration |
| 10 | Bolt domain migration |
| 11 | Wall domain migration |
| 12 | Breaker domain migration |

## Parallelism

Within each wave, independent domains can run in parallel:
- Wave 5: bump, impact, death, bolt-lost, node, time are independent
- Wave 6: passive, spawner, tick, condition, message-based are independent groups
- Wave 7: apply_damage, detect_deaths, process_despawn are independent
- Waves 9-12: strictly sequential (cell → bolt → wall → breaker)

Across waves: strictly sequential. Do not start wave N+1 until wave N is complete.
