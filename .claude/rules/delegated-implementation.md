# Delegated Implementation

Guidelines for using the **test-writer** and **code-writer** agent pair to implement features with TDD-enforced quality and context isolation.

## When to Use

Use delegated implementation when:
- **2+ independent domains** need implementation in the same phase
- **Domain boundaries are clearly defined** in architecture docs
- **The behavioral spec can be written in ~10-20 lines per domain** without ambiguity
- **The work is self-contained** within a single domain directory

Do NOT use delegated implementation when:
- **Cross-cutting changes** touch multiple domains or shared files (`lib.rs`, `game.rs`, `shared.rs`)
- **Exploratory work** where the API shape isn't proven yet
- **New domain creation** that requires wiring in `lib.rs`/`game.rs` (do the wiring yourself, then delegate the internals)
- **Single system additions** where spec-writing overhead exceeds just writing the code
- **Complex Bevy API uncertainty** — use bevy-api-expert first to resolve, then delegate

## The Flow

```
1. Main agent writes behavioral spec (test-writer)
2. Main agent writes implementation spec (code-writer)
3. Launch test-writer(s) — parallel if multiple domains
4. Main agent REVIEWS test output (mandatory checkpoint)
5. Launch code-writer(s) — parallel if multiple domains
6. Launch post-implementation agents (test-runner, correctness-reviewer, etc.)
7. Main agent handles wiring (lib.rs, game.rs, shared.rs)
```

### The Checkpoint Is Mandatory

Between steps 4 and 5, the main agent MUST review the test-writer's output:
- Do the tests capture the intended behavior?
- Are concrete values correct?
- Are edge cases covered?
- Are there ambiguities the test-writer flagged?

If tests are wrong, fix them or re-spec before launching the code-writer. Bad tests produce bad implementations.

## Writing a Test Spec (for test-writer)

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

### Constraints
- Tests go in: [specific file path]
- Do NOT test: [anything explicitly out of scope]
```

### Rules for Good Test Specs

1. **Use concrete values, not descriptions.** "Bolt at position (0.0, 50.0) with velocity (0.0, 400.0)" — not "a bolt moving upward."
2. **One behavior per numbered item.** Don't combine multiple behaviors into one description.
3. **Include the edge case inline.** Don't leave edge cases for the test-writer to discover.
4. **Name the types.** If new components or messages are needed, name them and describe their fields.
5. **Point to reference files.** The test-writer needs existing patterns to match.
6. **Scope explicitly.** State what's in scope and what's out of scope.

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

## Writing an Implementation Spec (for code-writer)

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

1. **Point to the failing tests.** The code-writer reads them first.
2. **Name what to implement.** System names, component names, resource names.
3. **Point to reference patterns.** The code-writer should match existing code.
4. **Specify schedule placement.** FixedUpdate vs Update vs OnEnter.
5. **Specify ordering.** After which system sets, before which.
6. **State what's off-limits.** Especially shared files and other domains.

## Parallel Domain Implementation

When implementing multiple domains simultaneously:

1. Write ALL test specs first (one per domain)
2. Launch ALL test-writers in parallel (each gets its own spec)
3. Review ALL test outputs (checkpoint)
4. Write ALL implementation specs (one per domain, now informed by the actual tests)
5. Launch ALL code-writers in parallel
6. Launch post-implementation agents

### Safety Requirements for Parallel Execution

- Each agent ONLY touches files within its assigned domain directory
- The main agent handles all shared file modifications (`lib.rs`, `game.rs`, `shared.rs`, `mod.rs` at crate root)
- If two domains need a new shared type, the main agent creates it before launching agents
- If a domain needs a message from another domain, the main agent ensures the message type exists before launching agents

## Model Override

Both agents default to Sonnet. Override to Opus via the Agent tool's `model` parameter when:

| Situation | Override? |
|-----------|-----------|
| Standard domain implementation | No — Sonnet is sufficient |
| Complex state machines with many transitions | Consider Opus for test-writer |
| Physics/math with subtle edge cases | Consider Opus for test-writer |
| Novel Bevy patterns not seen elsewhere in codebase | Consider Opus for code-writer |
| Simple CRUD-like systems or config loading | No — Sonnet is sufficient |
