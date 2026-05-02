# Spec Workflow

> **Team mode**: when an agent team is active (see `.claude/rules/team-mode.md`), the spec writers/reviewers are persistent team members. Spec revision loops happen peer-to-peer between the writer and reviewer; on approval, the reviewer triggers the next-stage agent (writer-tests / writer-code) directly. The orchestrator only sees the kickoff and the final approval milestone. Briefing requirements below still apply — spec writers in team mode get the same context they need in standard mode, just delivered via messages instead of launch prompts.

Read this before creating or reviewing specs. The pipeline produces a **test spec → failing tests → code spec → production code** sequence: the test spec is written first, gets reviewed clean, drives writer-tests, passes the RED gate, and only THEN is the impl spec written — with the failing tests on disk as primary input.

See `.claude/rules/spec-format-tests.md` and `.claude/rules/spec-format-code.md` for spec templates and quality rules.
See `.claude/rules/tdd.md` for the TDD cycle that specs feed into.
See `.claude/rules/session-state.md` for session-state update requirements at each step.
See `.claude/rules/delegating-to-subagents.md` for the full pipeline flow.

## Spec File Location

All specs are written to `.claude/specs/` (gitignored). Naming convention:
- Test spec: `.claude/specs/<wave>-<feature>-tests.md`
- Implementation spec: `.claude/specs/<wave>-<feature>-code.md`

The orchestrator provides the exact file paths when launching spec agents.

## Briefing Spec Writers

Every spec writer prompt must include the following. The goal is that the agent never needs to ask the orchestrator a question that could have been answered upfront. Incomplete briefings produce incomplete specs, which cost a full revision loop to fix.

### Required in Every Briefing

| Item | Why |
|------|-----|
| **Feature description** | What the feature does, in plain language. Not a task list — the intent. |
| **Scope boundaries** | What's in and what's explicitly out. Name the excluded concerns. |
| **Domains involved** | Which domains this feature touches (bolt, breaker, cells, effect, etc.) |
| **Design decisions already made** | Any choices the orchestrator has made — don't leave the spec writer to guess between options. |
| **Spec file path** | The exact `.claude/specs/<name>-{tests,code,goal}.md` path to write to. |

### Required When Applicable

| Item | When | Why |
|------|------|-----|
| **Research results** | A research wave ran | Spec writers can't see what researchers found unless you tell them. Include the key findings, not the full report. |
| **Relevant design docs** | Feature connects to a specific design doc | Point to the file path — e.g., "See `docs/design/effects/piercing.md` for the design." |
| **Relevant architecture docs** | Feature touches scheduling, messages, or cross-domain wiring | Point to the file path — e.g., "See `docs/architecture/messages.md` for message conventions." |
| **Known constraints or interactions** | Feature interacts with existing systems in non-obvious ways | State the interaction explicitly — e.g., "This runs after `clamp_bolt_speed` in FixedUpdate — the order matters." |
| **Existing code to reference** | The domain has established patterns the spec should follow | Point to the specific file — e.g., "Follow the pattern in `src/effect_v3/effects/shockwave/`." |

### Additional Briefing Items for the Code Spec Writer

The code spec writer is launched **after the RED gate passes**. By then the failing tests exist on disk. Add these to the briefing:

| Item | Why |
|------|-----|
| **Test spec file path** | `.claude/specs/<wave>-<feature>-tests.md` — the agent reads it to understand the contract in prose. |
| **Failing test file paths** | The exact test file(s) writer-tests produced (e.g., `src/foo/systems/bar/tests/skip_row.rs`). The agent MUST read these — they are the contract, not the test spec. |
| **Failing test function names** | Optional but useful — list the test fn names so the impl spec can reference them by name. |
| **RED gate confirmation** | "Tests compiled and failed at <date>." Confirms the impl spec writer is operating on real artifacts. |

### What NOT to Include

- Full file contents — the agent reads files itself. Provide paths, not content.
- Implementation opinions — the spec writer decides how to structure the spec. Provide constraints, not solutions.
- Previous conversation context — the agent has no memory of earlier discussion. If a decision was made in conversation, state the decision, not "as we discussed."

## Phase 1 — Research and Test Spec

Resolve unknowns and produce the test spec first. See `.claude/rules/sub-agents.md` for the full agent directory.

1. Launch applicable **research agents** in parallel (see Research Agents in `sub-agents.md`)
2. Launch **planning-writer-specs-tests** per wave (in parallel across waves)
   - Each writes its test spec to `.claude/specs/<wave>-<feature>-tests.md`
   - Each returns a compact summary + file path to the orchestrator
   - **Update session-state** after each completes
3. Launch **planning-reviewer-specs-tests** as each test spec completes (in parallel)
   - Each reads its test spec; returns BLOCKING/IMPORTANT/MINOR findings
   - **Update session-state** after each completes
4. Test-spec revision loop (see below) until every test spec is clean

## Phase 2 — RED

After every test spec is clean, drive to RED:

1. Launch **writer-tests** per wave (in parallel) — reads its test spec
2. Launch **reviewer-tests** as each writer-tests completes (in parallel)
3. Single **runner-cargo** RED gate after ALL reviewer-tests pass

## Phase 3 — Code Spec

Only AFTER the RED gate passes are code specs written. The failing tests on disk are the contract.

1. Launch **planning-writer-specs-code** per wave (in parallel across waves)
   - Brief with: test spec path + failing test file path(s) + RED gate confirmation
   - The agent MUST read the failing tests, not just the test spec
   - Each writes its impl spec to `.claude/specs/<wave>-<feature>-code.md`
   - **Update session-state** after each completes
2. Launch **planning-reviewer-specs-code** as each code spec completes (in parallel)
   - Reviewer cross-checks the impl plan against the actual failing tests, not just the test spec
   - Returns BLOCKING/IMPORTANT/MINOR findings
   - **Update session-state** after each completes
3. Code-spec revision loop until every code spec is clean

## Phase 4 — GREEN

After every code spec is clean, drive to GREEN:

1. Launch **writer-code** per wave (in parallel) — reads its impl spec AND the failing tests
2. Single **runner-cargo** GREEN gate after ALL writer-codes complete
3. Route any failing tests back to writer-code via fix spec hints (see `.claude/rules/routing-failures.md`)
4. **Update session-state** after each agent notification

## Full Sequence

```
test spec  →  review test spec (loop)
            →  writer-tests  →  reviewer-tests  →  RED gate (runner-cargo)
            →  code spec    →  review code spec (loop)
            →  writer-code  →  GREEN gate (runner-cargo)
            →  REFACTOR
```

## Spec Revision Loop

The same loop applies to test specs (in Phase 1) and code specs (in Phase 3):

1. Triage findings (dismiss false positives, note valid issues)
2. Send valid feedback back to the appropriate **spec writer** to update the spec file in place
3. Re-launch the appropriate **reviewer** if needed (skip if only MINOR findings remain)
4. Only proceed to the next phase once every relevant spec is confirmed clean
5. **Update session-state** after each revision completes

**Never launch writer-tests with an unreviewed or uncorrected test spec.** **Never launch writer-code with an unreviewed or uncorrected code spec.** The cost of a bad spec propagating downstream is high (rework). The cost of one revision loop is low.

## Passing Specs to Writers

When launching writers, pass the spec file path (not the spec content):
- writer-tests: "Read your test spec from `.claude/specs/<name>-tests.md`"
- writer-code: "Read your implementation spec from `.claude/specs/<name>-code.md`. The failing tests are at `<failing test paths>` — read them too; they are the authoritative contract."

This keeps the orchestrator's context lean. Writers read the full spec (and tests, for writer-code) from disk.
