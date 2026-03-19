# Delegated Implementation

All implementation goes through the delegated pipeline. The main agent is the orchestrator — it describes features, reviews outputs, routes failures, and handles shared wiring. The **planner-spec** → **planner-review** → **writer-tests** → **writer-code** pipeline produces the code.

## The Flow

See `.claude/rules/tdd.md` for the authoritative RED → GREEN → REFACTOR cycle definition.

```
1. Main agent describes the feature in plain language
2. Launch planner-spec to produce all specs (behavioral + implementation, per domain)
3. Launch planner-review to pressure-test specs
4. Main agent reviews planner-review feedback, then sends feedback back to planner-spec to revise
5. Repeat 3–4 until planner-review confirms specs are clean (usually one revision)
6. Main agent reviews final specs, creates shared prerequisites
7. Launch ALL writer-tests in parallel                              ── RED phase
8. Launch runner-tests to verify tests compile and fail             ── RED gate
9. As each passes RED gate: review, launch writer-code              ── GREEN phase
10. After ALL writer-codes complete: launch verification wave        ─┐
11. Route Phase 3 failures through fix agents                        │ REFACTOR phase
12. Run /simplify on changed code                                    │
13. Repeat 10–12 until all agents pass and /simplify is clean        │
14. Main agent handles wiring (lib.rs, game.rs, shared.rs)          ─┘
15. Update session-state.md
```

**Spec revision loop**: planner-review produces BLOCKING/IMPORTANT/MINOR findings. The main agent triages findings (some may be false positives), then sends the valid feedback back to planner-spec with instructions to produce corrected specs. Do NOT launch writer-tests until the spec revision loop is complete. Do NOT skip this step even for "obvious" specs.

## Writing a Test Spec (for writer-tests)

These formats are consumed by planner-spec (which writes them) and writer-tests (which reads them).

### Format

```markdown
## Test Spec: [Domain] — [Feature]

### Domain
src/[domain]/

### Behavior

1. **[Behavior name]**
   - Given: [concrete initial state with specific values]
   - When: [action or trigger]
   - Then: [expected outcome with specific values]
   - Edge case: [boundary condition]

2. **[Behavior name]**
   - Given: ...
   - When: ...
   - Then: ...

### Types (if new types are needed)
- `TypeName` — [description, fields, derives]

### Messages (if new messages are needed)
- `MessageName { field: Type }` — sent by [system], consumed by [system]

### Reference Files
- `src/[domain]/systems/existing_system.rs` — follow this test pattern
- `src/[domain]/components.rs` — existing types to use

### Scenario Coverage
- New invariants: [InvariantKind variants to add, or "none — existing invariants cover this"]
- New scenarios: [scenario RON files to add, or "none — existing scenarios exercise this"]
- Layout updates: [existing layouts to update with new cell types / features, or "none"]

### Constraints
- Tests go in: [specific file path]
- Do NOT test: [anything explicitly out of scope]
```

### Rules for Good Test Specs

1. **Use concrete values, not descriptions.** "Bolt at position (0.0, 50.0) with velocity (0.0, 400.0)" — not "a bolt moving upward."
2. **One behavior per numbered item.** Don't combine multiple behaviors into one description.
3. **Include the edge case inline.** Don't leave edge cases for the writer-tests to discover.
4. **Name the types.** If new components or messages are needed, name them and describe their fields.
5. **Point to reference files.** The writer-tests needs existing patterns to match.
6. **Scope explicitly.** State what's in scope and what's out of scope.
7. **Consider scenario coverage.** State whether existing invariants cover the feature, whether new invariants are needed, and whether new scenario RON files should be added for chaos/stress testing.

### Example: Good Test Spec

```markdown
## Test Spec: Bolt — Speed Clamping

### Domain
src/bolt/

### Behavior

1. **Bolt speed stays within min/max bounds after bump**
   - Given: Bolt with velocity magnitude 800.0, BoltMinSpeed(200.0), BoltMaxSpeed(600.0)
   - When: speed clamping runs
   - Then: velocity magnitude is 600.0, direction unchanged
   - Edge case: velocity exactly at max (600.0) — should remain unchanged

2. **Bolt speed increases to minimum if too slow**
   - Given: Bolt with velocity magnitude 100.0, BoltMinSpeed(200.0), BoltMaxSpeed(600.0)
   - When: speed clamping runs
   - Then: velocity magnitude is 200.0, direction unchanged
   - Edge case: velocity exactly at min (200.0) — should remain unchanged

3. **Zero velocity is not clamped**
   - Given: Bolt with velocity (0.0, 0.0)
   - When: speed clamping runs
   - Then: velocity remains (0.0, 0.0)

### Types
- Uses existing `BoltVelocity`, `BoltMinSpeed`, `BoltMaxSpeed` from `src/bolt/components.rs`

### Reference Files
- `src/bolt/components.rs` — existing unit tests for BoltVelocity methods

### Scenario Coverage
- New invariants: `BoltSpeedInRange` already exists — covers this
- New scenarios: none — existing chaos scenarios exercise speed clamping
- Layout updates: none

### Constraints
- Tests go in: `src/bolt/systems/clamp_bolt_speed.rs` (new file, system + tests)
- Do NOT test: integration with bump system (tested separately)
```

### Example: Bad Test Spec (don't do this)

```markdown
## Test Spec: Bolt Speed

The bolt should have its speed clamped. Make sure it works for all edge cases.
Use the existing bolt components.
```

Problems: no concrete values, no explicit behaviors, no edge cases, no file references, no scope.

## Writing an Implementation Spec (for writer-code)

### Format

```markdown
## Implementation Spec: [Domain] — [Feature]

### Domain
src/[domain]/

### Failing Tests
- `src/[domain]/[file].rs` — [N] tests to satisfy

### What to Implement
- [System/component/resource name]: [one-line description]
- [System/component/resource name]: [one-line description]

### Patterns to Follow
- Follow the pattern in `src/[domain]/systems/[existing].rs`
- [Specific convention or constraint]

### RON Data (if applicable)
- Add fields to `assets/[file].ron`: [field names and types]
- Add to defaults: [defaults resource path]

### Schedule
- [System] runs in [FixedUpdate/Update/OnEnter(State)]
- [Ordering constraints: after X, before Y]

### Constraints
- Do NOT modify: [files explicitly off-limits]
- Do NOT add: [features explicitly out of scope]
```

### Rules for Good Implementation Specs

1. **Point to the failing tests.** The writer-code reads them first.
2. **Name what to implement.** System names, component names, resource names.
3. **Point to reference patterns.** The writer-code should match existing code.
4. **Specify schedule placement.** FixedUpdate vs Update vs OnEnter.
5. **Specify ordering.** After which system sets, before which.
6. **State what's off-limits.** Especially shared files and other domains.

## Parallel Domain Implementation

When implementing multiple domains simultaneously:

1. planner-spec produces ALL specs upfront (test spec + implementation spec for each domain)
2. Launch ALL writer-tests in parallel **as background agents** (`run_in_background: true`)
3. When each writer-tests completes (notified automatically): run RED gate (runner-tests), then launch its writer-code — do NOT wait for other writer-tests still running
4. When ALL writer-codes have completed (they produce code only — no build verification): launch post-implementation verification per tier
5. Main agent handles wiring (lib.rs, game.rs, shared.rs)

### Safety Requirements for Parallel Execution

- Each agent ONLY touches files within its assigned domain directory
- The main agent handles all shared file modifications (`lib.rs`, `game.rs`, `shared.rs`, `mod.rs` at crate root)
- If two domains need a new shared type, the main agent creates it before launching agents
- If a domain needs a message from another domain, the main agent ensures the message type exists before launching agents
