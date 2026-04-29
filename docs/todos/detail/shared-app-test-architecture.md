# Shared App Test Architecture Investigation

## Summary
Investigate whether tests can share a single `App` instance via `LazyLock` in a shared testing util, with each test creating what it needs and cleaning up after itself — to reduce test suite runtime.

## Context
The test suite is getting slow (though Docker resource contention may be the current cause — investigate that first before assuming the architecture is the bottleneck). The hypothesis is that App construction overhead is significant enough that sharing a single App across tests in a module would yield meaningful speedup.

Proposed pattern: a `LazyLock<App>` (or `OnceLock<Mutex<App>>`) in a shared test util module. Each test inserts its required components/resources, runs assertions, then removes what it inserted. Tests would share a single initialized app rather than each constructing one from scratch.

**Critical open question**: Many tests call `app.update()`, tick `FixedTime`, or manipulate `Time` resources. In a shared App, these mutations persist across tests — frame count advances, time accumulates, tick state carries over. This may make the shared-App model unworkable for any test that does more than insert/query without advancing the schedule. Isolation is then only achievable for pure-query tests.

Possible mitigations to investigate:
- Snapshot + restore of time/tick state around each test (complex, fragile)
- Separate shared apps per test category (pure-query vs. schedule-advancing)
- Per-test App construction remains the model but App build time is profiled and reduced (e.g., fewer plugins registered in test harnesses)
- Parallel test execution at the OS level (Rust test runner already does this — check if there's a threading bottleneck)

## Scope
- In: feasibility investigation, benchmarking App construction time, prototype of the shared-App pattern in one test module
- Out: full migration (gate on investigation result)

## Dependencies
- Depends on: nothing — pure investigation
- Blocks: nothing

## Notes
- First confirm whether Docker resource contention is the actual bottleneck before investing in architecture changes. Profile `cargo all-dtest` with Docker idle vs. active.
- If App construction is < 10ms per test, the architecture change probably isn't worth it.
- Rust's test runner runs tests in parallel by default — check thread count and whether we're CPU-bound or App-init-bound.

## Status
`[NEEDS DETAIL]` — feasibility unknown; investigation required before any implementation decision
