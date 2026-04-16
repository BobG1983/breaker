# Spec Workflow

Read this before creating or reviewing specs. This is the process that produces clean specs before the RED phase begins.

See `.claude/rules/spec-format-tests.md` and `.claude/rules/spec-format-code.md` for spec templates and quality rules.
See `.claude/rules/tdd.md` for the TDD cycle that specs feed into.
See `.claude/rules/session-state.md` for session-state update requirements at each step.

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
| **Cross-spec file paths** | Other specs in the same wave exist or are in progress | Provide paths so reviewers can cross-check alignment — e.g., "Implementation spec at `.claude/specs/wave1-piercing-code.md`." |

### What NOT to Include

- Full file contents — the agent reads files itself. Provide paths, not content.
- Implementation opinions — the spec writer decides how to structure the spec. Provide constraints, not solutions.
- Previous conversation context — the agent has no memory of earlier discussion. If a decision was made in conversation, state the decision, not "as we discussed."

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
