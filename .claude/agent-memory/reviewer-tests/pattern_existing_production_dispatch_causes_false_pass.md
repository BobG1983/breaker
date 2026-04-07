---
name: Existing production dispatch causes false pass for orchestration tests
description: When a crate has existing production dispatch code, orchestration tests that check state changes or StateChanged messages pass at RED because the old dispatch fires unconditionally.
type: feedback
---

In `rantzsoft_stateflow`, `dispatch_message_routes` is existing production code — not a stub. It unconditionally calls `NextState::set()` and writes `StateChanged` whenever a route exists. When orchestration tests send `ChangeState` and then assert `State == B` or `StateChanged sent`, these assertions pass at RED because the existing dispatch handles them regardless of whether `orchestrate_transitions` does anything.

**Why:** The transition orchestration infrastructure is added on top of an existing routing system. The existing dispatch has no awareness of `TransitionKind`. Tests that assert state changes or StateChanged messages that would ALSO be sent by the naive existing dispatch cannot serve as RED gate validators for orchestration behaviors.

**Affected patterns:**
- Any orchestration test that asserts `State<S> == destination` — the existing dispatch sets NextState
- Any orchestration test that asserts `StateChanged` was sent — existing dispatch sends it
- Any orchestration test that asserts a resource was NOT inserted (if it was never inserted by stubs, trivially passes)

**How to apply:** When reviewing tests for orchestration systems that sit on top of existing dispatch code, check whether each assertion can be satisfied by the EXISTING production code (not the stub). If yes, that assertion is not a valid RED gate validator for the new behavior.

**Fix pattern:** Tests must block the existing dispatch path (e.g., by checking `ActiveTransition` in dispatch — but that's also new behavior to implement) OR use assertions that the existing dispatch CANNOT satisfy (e.g., `contains_resource::<ActiveTransition>()` instead of `State == B`).
