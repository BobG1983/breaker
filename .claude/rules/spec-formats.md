# Spec Formats

Templates and quality rules for behavioral specs (consumed by writer-tests) and implementation specs (consumed by writer-code). Produced by planner-spec.

See `.claude/rules/spec-workflow.md` for the revision loop specs must pass before reaching writers.

## Test Spec Format (for writer-tests)

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
- Self-test scenarios: For each new InvariantKind, a self-test scenario in `scenarios/self_tests/`
  with `expected_violations: Some([NewVariant])` that intentionally triggers the invariant
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
- Self-test scenarios: none — `BoltSpeedInRange` already has a self-test
- Layout updates: none

### Constraints
- Tests go in: `src/bolt/systems/clamp_bolt_speed.rs` (new file, system + tests)
- Do NOT test: integration with bump system (tested separately)
```

### Example: Bad Test Spec (DO NOT DO THIS)

```markdown
## Test Spec: Bolt Speed

The bolt should have its speed clamped. Make sure it works for all edge cases.
Use the existing bolt components.
```

Problems: no concrete values, no explicit behaviors, no edge cases, no file references, no scope.

## Implementation Spec Format (for writer-code)

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
