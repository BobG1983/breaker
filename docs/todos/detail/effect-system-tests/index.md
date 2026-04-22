# Effect-system test coverage

Test-infrastructure work for the effect_v3 / damage-pipeline stack. All entries are test additions or test-pattern migrations — no production code changes. Each file is a self-contained test addition; the index is the inventory.

## Why grouped

These were originally four separate `audit/remediations/` files with overlapping concerns around "make sure the effect system's wiring is actually verified." Bundling into one TODO gives a single review surface for test-harness work and lets the end-to-end tests share a harness with the more focused diamond-topology / deadline / ron-asset suites.

## Entries

| File | Purpose |
|------|---------|
| `end-to-end-integration-tests.md` | Cross-cutting end-to-end tests for effect_v3 tree dispatch — chip → resolve → stamp → walk → fire/reverse → stack aggregation → damage pipeline. Exercises the full hot path as a single harnessed scenario. |
| `deadline-integration-tests.md` | Integration tests for Deadline's code-driven form from TODO #6 — threshold crossing fires boosts on all bolts, late-spawn inheritance, node-exit reversal, stack-with-chip aggregation. |
| `diffusion-diamond-topology.md` | Regression test pinning Diffusion's flat-share-per-ring attenuation under diamond topology (D reachable via both B and C takes damage once, not twice). Companion to `update-docs/diffusion-flat-share-documentation.md`. |
| `ron-asset-tests-structural-only.md` | Migration pattern: convert every `tests/ron_asset.rs` from value-pinned assertions to structural-validity-only. Touches every mechanic that has a `ron_asset` test. |

## Dependencies

- `end-to-end-integration-tests.md` → TODO #0 (damage crate), #2 (mutators domain), #7 (dispatcher + code-driven protocols)
- `deadline-integration-tests.md` → TODO #6 (Deadline's code-driven form)
- `diffusion-diamond-topology.md` → TODO #0 (Diffusion becomes a `MessageMutator<DamageDealt<Cell>>` post-crate)
- `ron-asset-tests-structural-only.md` → rides with TODO #9 (RON tuning sweep), which is where most value-pinned assertions currently live

## Ordering

Land the individual entries in any order after their respective dependencies are in place. `ron-asset-tests-structural-only.md` can land in waves (one mechanic per branch) as part of the RON tuning sweep.

## Scope boundary

In scope:
- New test files
- Rewritten `ron_asset.rs` tests (value-pinned → structural)
- Test harness helpers shared across the new suites

Out of scope:
- Production code changes
- Design-doc updates (those live in TODO #0)
- RON value changes (TODO #9)
