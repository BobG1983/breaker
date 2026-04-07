---
name: Scenario runner vocabulary exemption
description: Game vocabulary (Breaker/Bolt/Cell/etc.) is not enforced in breaker-scenario-runner tooling code
type: feedback
---

`breaker-scenario-runner` is tooling code. Game vocabulary compliance (Breaker/Bolt/Cell/Node/Bump/Flux) is not enforced here — user explicitly noted this at review start.

**Why:** The scenario runner is infrastructure for testing the game, not game code itself. Forcing game terms into test infrastructure names would be confusing.

**How to apply:** Skip the vocabulary section for any file under `breaker-scenario-runner/`. Still flag genuinely vague names (data, info, temp, flag) that hurt readability.
