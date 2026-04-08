---
name: InvariantKind variant name mismatch pattern
description: Scenario RON files referencing variant names that do not exist in InvariantKind cause a parse failure and the scenario is silently skipped (not counted as FAIL)
type: project
---

## Pattern: InvariantKind name mismatch in RON

When a `.scenario.ron` file uses an `InvariantKind` variant name that does not match any variant in `breaker-scenario-runner/src/types/definitions/invariants.rs`, the runner emits a parse error at startup and skips that scenario entirely. The scenario does not appear in the results table and is not counted as a failure — it simply does not run. This is a scenario runner bug: a misspelled or non-existent variant causes silent coverage loss.

**Observed case (2026-04-08):**  
`guardian_slide_protection.scenario.ron` used `EntityLeaks` in its `disallowed_failures` list. The correct variant name is `NoEntityLeaks`. The error message is:

```
Failed to parse .../guardian_slide_protection.scenario.ron: 18:135-18:146:
Unexpected variant named `EntityLeaks` in enum `InvariantKind`,
expected one of `BoltInBounds`, ..., or `BreakerCountReasonable` instead
```

**Coverage impact:** The `bastion` layout appeared in the "Unused layouts" coverage gap report, confirming no scenario ran against it.

**Fix location:** `breaker-scenario-runner/scenarios/mechanic/guardian_slide_protection.scenario.ron` line 18 — rename `EntityLeaks` to `NoEntityLeaks`.

**How to apply:** When a layout appears in "Unused layouts" after a run that was supposed to have a new scenario for it, check the parse error output at the top of `cargo scenario -- --all` for a variant name mismatch.
