# Spec Workflow

Read this before creating or reviewing specs. This is the process that produces clean specs before the RED phase begins.

See `.claude/rules/spec-formats.md` for the spec templates and quality rules.
See `.claude/rules/tdd.md` for the TDD cycle that specs feed into.
See `.claude/rules/session-state.md` for session-state update requirements at each step.

## Spec File Location

All specs are written to `.claude/specs/` (gitignored). Naming convention:
- Test spec: `.claude/specs/<wave>-<feature>-tests.md`
- Implementation spec: `.claude/specs/<wave>-<feature>-code.md`

The orchestrator provides the exact file paths when launching spec agents.

## Phase 1 — Research and Spec Creation

Before writing any code, resolve unknowns and produce specs. See `.claude/rules/sub-agents.md` for the full agent directory.

1. Launch applicable **research agents** in parallel (see Research Agents in `sub-agents.md`)
2. Launch **planning-writer-specs-tests** and **planning-writer-specs-code** in parallel per wave
   - Each writes its spec to `.claude/specs/`
   - Each returns a compact summary + file path to the orchestrator
   - **Update session-state** after each completes
3. Launch **planning-reviewer-specs-tests** and **planning-reviewer-specs-code** in parallel
   - Each reads its spec file + optionally the other spec for cross-alignment
   - Each returns a review with BLOCKING/IMPORTANT/MINOR findings
   - **Update session-state** after each completes

## Spec Revision Loop

After reviewers produce findings, the main agent:

1. Triages findings (dismiss false positives, note valid issues)
2. Sends valid feedback back to the appropriate **spec writer** to update the spec file in place
3. Re-launches the appropriate **reviewer** if needed (skip if only MINOR findings remain)
4. Only proceeds to writer-tests once BOTH specs are confirmed clean
5. **Update session-state** after each revision completes

Both reviewers produce BLOCKING/IMPORTANT/MINOR findings. Do NOT launch writer-tests until the spec revision loop is complete for BOTH specs. Do NOT skip this step even for "obvious" specs.

**Never launch writer-tests with unreviewed or uncorrected specs.** The cost of a bad spec propagating through writer-tests → writer-code is high (rework). The cost of one revision loop is low.

## Passing Specs to Writers

When launching writer-tests and writer-code, pass the spec file path (not the spec content):
- writer-tests: "Read your test spec from `.claude/specs/<name>-tests.md`"
- writer-code: "Read your implementation spec from `.claude/specs/<name>-code.md`"

This keeps the orchestrator's context lean. Writers read the full spec from the file.
