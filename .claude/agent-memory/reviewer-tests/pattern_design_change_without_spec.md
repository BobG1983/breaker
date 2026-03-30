---
name: Design change introduced in tests without backing spec
description: writer-tests adds new types/systems (e.g. PendingBreakerEffects) that represent a design divergence from the spec, with no spec file covering the new behavior
type: feedback
---

When a writer-tests output introduces new types or systems not mentioned in any spec file, and those types contradict existing spec behaviors (e.g., a new deferred-dispatch resource for a target that the spec says is dispatched directly), flag all of those tests as BLOCKING.

**Why:** This surfaced in the runner+dispatch bug fix session where `PendingBreakerEffects` and `apply_pending_breaker_effects` appeared in test files with no spec. The new tests also contradicted the existing `initial_effects_breaker_target_pushed_to_effect_chains` test that was spec-backed. Two tests for the same system with opposite assertions cannot both be RED — one will trivially pass.

**How to apply:** When reviewing, check every new type name used in tests against all spec files in `.claude/specs/`. If a type is used in tests but absent from specs, flag BLOCKING and note the contradiction with any existing spec behavior that covers the same domain.
