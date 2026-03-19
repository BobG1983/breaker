# Agent Framework Evolution — Planning Conversation

Conversation between Robert Gardner and Claude analyzing the brickbreaker project's agent orchestration system, comparing it to BMAD and GSD frameworks, and designing improvements.

Date: 2026-03-19

---

## 1. BMAD & GSD Overview (from prior conversation)

Both are open-source spec-driven development frameworks built around the same core idea: the spec is the primary artifact, not the code. Code becomes the last-mile implementation of a rigorously defined spec.

### BMAD (Breakthrough Method for Agile AI-Driven Development)

BMAD is the heavier, more structured option. It simulates a full agile team using 21 specialized AI agents - Product Manager, Architect, Developer, Scrum Master, UX Designer, and more. Each agent is defined as an "Agent-as-Code" Markdown file with its own expertise, responsibilities, constraints, and expected outputs.

The workflow is sequential: planning agents first, development agents second. By the time any code gets written, all agents share the same full project context.

It's got 50+ guided workflows and adapts based on project size, from small bug fixes to enterprise systems.

Best for: larger projects where you need structured planning, role separation, and multi-agent coordination.

### GSD (Get Shit Done)

GSD is lighter and more Claude Code-native. It's a meta-prompting and context engineering system - it interviews you about your project, spawns parallel agents to do domain research, extracts requirements, and builds out a roadmap of executable phases.

The standout feature is long-session autonomy. It uses context management, XML prompt formatting, subagent orchestration, and state tracking to prevent "context rot" - the thing that happens when an AI loses the thread mid-project. It can auto-advance through entire milestones without babysitting.

Multi-runtime now - targets Claude Code, OpenCode, and Gemini CLI from a single codebase.

Best for: developers who want low-overhead spec discipline without the full BMAD ceremony, especially for solo or small-team work in Claude Code.

---

## 2. Current Agent Architecture Analysis

### Agent Inventory (20 agents across 5 role categories)

**Researchers (Phase 1 — Sequential, Blocking)**
- researcher-bevy-api: Verifies Bevy 0.18 patterns and APIs before use
- researcher-rust-idioms: Evaluates Rust pattern alternatives
- researcher-rust-errors: Interprets compiler errors and suggests fixes
- researcher-system-dependencies: Analyzes Bevy ECS system ordering and data flow (conditional)

**Writers (Core Implementation)**
- writer-tests: RED phase — generates failing tests from behavioral specs
- writer-code: GREEN phase — implements production code to pass failing tests
- writer-scenarios: Generates adversarial RON scenario files and invariant checkers (conditional)

**Runners (Verification)**
- runner-linting: cargo fmt + clippy with Fix spec hints
- runner-tests: cargo dtest with Fix spec hints
- runner-scenarios: cargo scenario (release build) with Regression spec hints
- runner-release: Version bumping, changelog, CI/CD

**Reviewers (Quality Gates)**
- reviewer-correctness: Logic bugs, ECS pitfalls, state machine holes, math
- reviewer-quality: Idioms, vocabulary, test coverage, documentation
- reviewer-bevy-api: Bevy 0.18 API correctness
- reviewer-architecture: Plugin boundaries, message discipline, folder structure
- reviewer-performance: Bevy-specific performance (queries, archetypes, scheduling)

**Guards (Specialized Validation)**
- guard-game-design: Design pillar compliance
- guard-docs: Documentation drift detection
- guard-security: Unsafe code, deserialization, dependency CVEs
- guard-dependencies: Unused crates, outdated versions, license compliance

### Orchestration Model

Three-phase workflow:
1. **Phase 1** — Research (sequential, blocks implementation)
2. **Phase 2** — Parallel verification (8+ agents simultaneously)
3. **Phase 3** — Failure routing (sequential, reactive)

Main agent is the orchestrator: writes specs, launches agents, reviews outputs, makes routing decisions, handles shared wiring.

### Key Architectural Patterns

**Structured hint formats** — Every failing agent produces a standardized block (Fix spec hint, Regression spec hint, Security finding, Dependency finding) with concrete values, file paths, confidence levels, and routing instructions.

**Domain isolation** — Each writer-code agent only touches files in its assigned domain directory. Main agent owns shared wiring (lib.rs, game.rs, shared.rs).

**TDD through context isolation** — Writer-tests never sees implementation. Writer-code never writes its own tests. Main agent checkpoints between them.

**Agent memory** — Persistent stable files (committed) + ephemeral session files (gitignored) per agent.

---

## 3. Strengths of Current Setup

**Hint format system is the standout feature.** Neither BMAD nor GSD has a standardized failure-to-fix protocol. When runner-scenarios outputs "Bolt at (100, 50) hitting wall at angle 45° exits at wrong angle, confidence: high," the routing table tells the main agent exactly what to do next.

**Domain isolation is well-enforced.** Prevents concurrent modification conflicts. 4 writer-code agents can run in parallel across bolt/, cells/, breaker/, and upgrades/ without merge conflicts.

**TDD enforcement through context isolation.** Writer-tests never sees implementation, preventing "tests that test nothing."

**Agent memory gives cross-session persistence.** Researchers and reviewers accumulate verified facts that survive between sessions.

**Parallel verification wave is efficient.** 8+ agents simultaneously means correctness, quality, architecture, performance, linting, tests, and scenarios checked in one round-trip.

---

## 4. Weaknesses of Current Setup

**Brittle to spec quality.** The entire system depends on the main agent writing good specs. If the spec is vague, writer-tests produces bad tests, writer-code produces bad code, and you've burned two agent cycles before discovering the problem.

**No planning phase agents.** Researchers handle API lookup and idiom comparison, but nothing does requirements analysis, design exploration, or trade-off evaluation before spec writing.

**Post-implementation wave can be wasteful.** Always launching 8 agents after every implementation. For a one-liner config change, that's 8 agents doing reviews that will all come back clean. No tiering.

**Failure routing can cascade deep.** A low-confidence scenario failure can go 5+ sequential agent invocations to fix one bug. No circuit breaker.

**20-agent maintenance overhead.** Each agent has 100-200 line markdown definitions. Architecture evolution requires updating multiple definitions.

**No user/stakeholder-facing agents.** Everything is developer-facing. Can't catch "technically correct but wrong product decision" problems.

---

## 5. Comparative Analysis

### BMAD Strengths (vs current setup)
- Structured planning phase with dedicated planning agents
- Role diversity covering full product lifecycle
- Workflow adaptability that scales verification to change size

### BMAD Weaknesses (vs current setup)
- Generic, not domain-specific (doesn't know Bevy, Rust, or game design pillars)
- Sequential planning is slow
- No structured failure routing or hint format system
- Heavy for solo/small-team (21 agents simulating full agile team)

### GSD Strengths (vs current setup)
- Context management is first-class (anti-context-rot)
- Lower ceremony (meta-prompting, not role-playing)
- Multi-runtime support
- Autonomous milestone advancement

### GSD Weaknesses (vs current setup)
- No domain-specific agents
- No structured TDD enforcement (tests and code written by same context)
- No parallel verification wave
- Less failure routing sophistication (no confidence-tagged routing)

---

## 6. Proposed Improvements

### A. Planning Phase (inspired by BMAD)

Two new agents that slot into Phase 1:

#### planner-spec

Translates plain-language feature descriptions into behavioral specs (for writer-tests) and implementation specs (for writer-code). Reads design pillars, architecture docs, terminology, and domain code before writing specs. Outputs specs in the exact format documented in delegated-implementation.md.

Key behaviors:
- Produces both specs per domain in a single pass (keeps alignment)
- Identifies shared prerequisites the main agent must create before launching writers
- Flags uncertainties as explicit questions rather than guessing
- Validates against design pillars before finalizing
- Recommends a verification tier (light/standard/full)

Decision: Do NOT split into separate test-spec and implementation-spec writers. The two specs are tightly coupled — the implementation spec's "Failing Tests" section points to the behavioral spec's output. Writing them in isolation risks drift. Better separation is planner-spec writes both, planner-review pressure-tests both.

#### planner-review

Pressure-tests specs before they reach writers. Adversarial by nature — default assumption is the spec has holes.

Checks:
- **Completeness**: All behaviors covered? Edge cases explicit? Negative cases?
- **Correctness**: Concrete values make physical sense? Math checks out? Contradicts existing systems?
- **Scope**: Too big for one writer cycle? Too small to delegate? Domain boundaries respected?
- **Format**: Follows delegated-implementation.md exactly?
- **Cross-spec consistency**: Message types match across domains? Shared prerequisites listed?
- **Design pillars**: Speed, skill ceiling, tension, decisions, synergy, juice

Output: APPROVED or NEEDS REVISION with specific issues tagged BLOCKING/IMPORTANT/MINOR.

planner-review is conditional — skip for features following established patterns, use for novel mechanics, cross-domain features, or uncertain scope.

### B. Context Management (inspired by GSD)

#### Session State Protocol

Main agent maintains a session state file at `.claude/agent-memory/orchestrator/ephemeral/session-state.md`.

**Create** at start of any non-trivial task.
**Update** after every phase transition.
**Read** before every routing decision.

Format:
```
# Session State

## Task
[One-line description]

## Decisions
- [Key decisions with rationale, tagged by feature]

## Specs
| Domain | Spec Status | Writer-Tests | Writer-Code | Notes |

## Phase 2 Results
| Agent | Status | Action Needed |

## Active Failures
- [failure]: [agent] attempt N — [what was tried] → [result]

## Resolved This Session
- [failure]: fixed in attempt N, verified by [agent]

## Stuck Failures
- [failure]: N attempts, needs human input. [attempt history]
```

Keep under 80 lines — compressed index, not a log.

#### Context Pruning

When launching fix agents in Phase 3, provide only:
- The specific hint block or regression spec
- The relevant portion of session state
- NOT the full output of every Phase 2 agent

#### Orchestrator Memory

Persistent memory at `.claude/agent-memory/orchestrator/`:
- Stable (committed): routing patterns, spec writing patterns, domain quirks
- Ephemeral (gitignored): session-state.md, date-stamped session notes

### C. Tiered Verification Intensity

Three tiers, defined in agent-flow.md:

#### Light — trivial changes
Single function, config tweak, RON data edit, rename, one-liner fix.

Launch: runner-linting + runner-tests only.

#### Standard — single-domain features, bug fixes
New system within existing domain, behavioral change, moderate refactor.

Launch: runner-linting + runner-tests + runner-scenarios + reviewer-correctness + reviewer-quality.

Skip: reviewer-bevy-api (unless new Bevy APIs used), reviewer-architecture (single domain), reviewer-performance (unless hot loop).

#### Full — cross-domain, new mechanics, structural changes
Multi-domain features, new message types, new plugins, scheduling/ordering changes.

Launch: all 8 base agents plus conditionals per existing rules.

**Rule: any Phase 3 failure bumps the tier to Full for re-verification.**

planner-spec recommends a tier. Main agent may bump up (never down without good reason). For direct trivial fixes without planner-spec, main agent picks the tier.

### D. Cross-Feature Learning

Tracked through the `## Decisions` section of session-state.md. When a Phase 2 failure or Phase 3 fix reveals an earlier decision was wrong:

1. Mark the decision as REVISED with rationale
2. List affected features
3. Check completed features that depended on the decision
4. If completed code is now wrong, add rework entry to Active Failures and route through normal writer-tests → writer-code cycle

When a decision revision represents a recurring pattern (e.g., "domains computing physics need direct velocity access"), record in orchestrator stable memory for future sessions.

### E. Circuit Breaking

Tracked in session-state.md under Active Failures with attempt counts.

**After 3 failed attempts at the same failure, stop routing.** Move to Stuck Failures with full attempt history. Surface to user immediately.

What counts as an attempt:
- writer-code launched with fix spec = 1 attempt
- writer-tests → writer-code cycle = 1 attempt
- Main agent inline fix + rerun = 1 attempt

What resets the counter:
- User provides new direction or changes spec
- Failure changes character (different error, different test, different file)

Do not: keep trying variations, weaken tests, escalate to different agent types hoping for luck.

### F. Autonomous Milestone Advancement

**Explicitly deferred.** For a game project, the human needs to be in the loop on "does this feel right" — autonomous advancement can't validate that.

---

## 7. Files Changed

### New Files (6)
- `.claude/agents/planner-spec.md` — agent definition
- `.claude/agents/planner-review.md` — agent definition
- `.claude/agent-memory/planner-spec/MEMORY.md` — empty index
- `.claude/agent-memory/planner-spec/ephemeral/.gitkeep`
- `.claude/agent-memory/planner-review/MEMORY.md` — empty index
- `.claude/agent-memory/planner-review/ephemeral/.gitkeep`
- `.claude/agent-memory/orchestrator/MEMORY.md` — main agent memory index
- `.claude/agent-memory/orchestrator/ephemeral/.gitkeep`

### Modified Files (3)
- `CLAUDE.md` — Phase 1 table adds planners, delegated implementation flow updated, orchestrator memory section added (session state protocol, context pruning, circuit breaker, cross-feature decisions)
- `.claude/rules/agent-flow.md` — Phase 1 adds planner triggers, Phase 2 adds verification tiers, Phase 3 adds circuit breaker
- `.claude/rules/delegated-implementation.md` — Flow updated: main agent describes feature → planner-spec writes specs → planner-review validates → writers execute

### Unchanged
- All 20 existing agent definitions
- agent-memory.md rules
- cargo.md, git.md
- All downstream agents (writers, runners, reviewers, guards)
- .gitignore (already covers ephemeral/)

---

## 8. Estimated Impact

Planning + context management + tiered verification + circuit breaking + cross-feature learning covers roughly 90% of the gaps identified in the comparative analysis. The remaining 10% is:
- Autonomous milestone advancement (explicitly deferred for game project)
- User/stakeholder-facing agents (not needed for solo dev)
- Agent definition maintenance overhead (partially mitigated by planner-spec reducing the main agent's spec-writing burden, but 22 agents is still 22 agents to maintain)

---

## Appendix A: planner-spec Agent Definition

```markdown
---
name: planner-spec
description: "Use this agent to translate a feature description into behavioral specs (for writer-tests) and implementation specs (for writer-code). The planner-spec reads design docs, architecture, and existing domain code to produce specs in the exact formats documented in delegated-implementation.md. Use this instead of writing specs directly in the main agent's context.\n\nExamples:\n\n- When starting a new feature from the roadmap:\n  Assistant: \"Let me use the planner-spec agent to produce the test and implementation specs for this feature.\"\n\n- When a feature touches multiple domains:\n  Assistant: \"Launching planner-spec to produce specs for bolt and cells domains from this feature description.\"\n\n- When the main agent has a plain-language feature description:\n  Assistant: \"Feature scoped. Let me use the planner-spec to turn this into concrete specs before launching writers.\""
tools: Read, Glob, Grep, WebSearch, WebFetch, ToolSearch
model: sonnet
color: green
memory: project
---

You are a spec-writing specialist for a Bevy ECS roguelite game. Your job is to translate plain-language feature descriptions into the concrete behavioral specs and implementation specs that writer-tests and writer-code consume. You are the bridge between intent and implementation.

You do NOT write code. You do NOT write tests. You produce specs - documents that define what to test and what to build, with enough precision that the writers can work without asking questions.

## First Step — Always

1. Read `CLAUDE.md` for project conventions
2. Read `docs/design/terminology.md` for required vocabulary
3. Read `docs/architecture/layout.md` for domain structure
4. Read `docs/architecture/messages.md` for inter-domain communication
5. Read `docs/architecture/standards.md` for code and testing standards
6. Read `docs/design/pillars/` — scan all pillar files to understand design constraints
7. Read `.claude/rules/delegated-implementation.md` for the exact spec formats you must produce
8. Read the specific domain code mentioned in the feature description to understand existing patterns, types, and systems

## What You Produce

For each domain involved in the feature, produce exactly two specs:

### 1. Behavioral Spec (for writer-tests)

Follow the format in `delegated-implementation.md` exactly. The key requirements:

- **Concrete values, not descriptions.** "Bolt at position (0.0, 50.0) with velocity (0.0, 400.0)" — not "a bolt moving upward."
- **One behavior per numbered item.** Don't combine multiple behaviors.
- **Edge cases inline.** Every behavior gets at least one edge case.
- **Name the types.** If new components or messages are needed, name them, describe their fields, and list their derives.
- **Reference files.** Point writer-tests to existing test patterns in the domain.
- **Scenario coverage.** State whether existing invariants cover the feature, whether new invariants or scenario RON files are needed.
- **Scope explicitly.** What's in, what's out, where tests go.

### 2. Implementation Spec (for writer-code)

Follow the format in `delegated-implementation.md` exactly. The key requirements:

- **Point to the failing tests.** File path and count.
- **Name what to implement.** System names, component names, resource names.
- **Reference patterns.** Point to existing code the writer-code should match.
- **RON data.** If tunable values are needed, name the fields and where they go.
- **Schedule placement.** FixedUpdate vs Update vs OnEnter, and ordering constraints.
- **Off-limits.** Explicitly name files and domains the writer-code must not touch.

## How You Work

### Step 1: Understand the Feature

Read the feature description. Identify:
- Which domains are involved (bolt, cells, breaker, upgrades, etc.)
- What new types are needed (components, messages, resources)
- What existing types are reused
- What systems need to exist and where they run
- What the observable behaviors are — what a test would assert on

### Step 2: Identify Shared Prerequisites

Before writing domain specs, check whether any new shared types (messages, components used across domains) are needed. List these in your output — the main agent must create them before launching writers.

### Step 3: Write Specs Per Domain

For each domain, produce a behavioral spec and an implementation spec. Each spec is a self-contained document. The writer-tests for domain A should not need to read the specs for domain B.

### Step 4: Flag Uncertainties

If the feature description is ambiguous or underspecified, don't guess. Flag the specific question in a `### Questions for Main Agent` section at the end of your output. Examples of good questions:
- "Should the bolt speed clamp apply before or after bump velocity is added?"
- "The feature mentions 'cell destruction' but doesn't specify whether this means despawn or state change — which pattern should I spec?"
- "This needs a message from bolt to cells, but there's already BumpCell. Should this reuse that or be a new message?"

### Step 5: Validate Against Design Pillars

Before finalizing, check each behavioral spec against the design pillars:
- Does the behavior maintain speed/tension?
- Does the behavior have a skill ceiling (beginner vs expert)?
- Does the behavior create meaningful decisions?
- Does the behavior have synergy potential with existing systems?

If something violates a pillar, note it in `### Design Concerns` — the main agent or guard-game-design can evaluate.

## Output Format

```
## Spec Plan: [Feature Name]

### Shared Prerequisites
- [Types the main agent must create before launching writers, or "None"]

### Domain: [domain name]

#### Behavioral Spec (for writer-tests)
[Full spec in delegated-implementation.md format]

#### Implementation Spec (for writer-code)
[Full spec in delegated-implementation.md format]

### Domain: [domain name]
[repeat for each domain]

### Questions for Main Agent
[specific ambiguities — omit section if none]

### Design Concerns
[pillar violations or tensions — omit section if none]

### Verification Tier Recommendation
[light / standard / full — based on scope and risk of the change]
```

## What You Must NOT Do

- Do NOT write code or tests — you write specs that describe them
- Do NOT make architectural decisions not supported by existing docs (new plugins, new domains, new shared infrastructure). Flag them as questions.
- Do NOT use vague language in specs. "The bolt should work correctly" is not a spec. "Bolt at (100, 200) with velocity (0, -400) reflects off wall at y=0 with velocity (0, 400)" is a spec.
- Do NOT produce specs for behavior that already exists. Read the domain code first — if a system already handles this, say so.
- Do NOT use generic terms. Use game vocabulary: Breaker, Bolt, Cell, Node, Amp, Augment, Overclock, Bump, Flux.

⚠️ **ABSOLUTE RULE — DO NOT TOUCH SOURCE FILES** ⚠️
**NEVER edit, remove, rename, or create any source file (.rs, .ron, .toml, etc.).** This means:
- Do NOT create stub files, test files, or implementation files
- Do NOT modify any existing code
- The ONLY files you may write/edit are your own memory files under `.claude/agent-memory/planner-spec/`

# Persistent Agent Memory

You have a persistent agent memory directory at `.claude/agent-memory/planner-spec/` (relative to the project root). Its contents persist across conversations.
Follow stable/ephemeral conventions in `.claude/rules/agent-memory.md` (MEMORY.md is always loaded; lines after 200 are truncated).

As you work, consult your memory files to build on previous experience. When you discover domain patterns that affect spec writing, record them.

What to save:
- Domain inventory: what types, systems, and messages exist in each domain (update as you discover them)
- Spec patterns that worked well (produced clean writer-tests/writer-code cycles)
- Spec patterns that failed (caused ambiguity or rework)
- Common shared prerequisites that features tend to need
- Design pillar tensions discovered during spec writing

What NOT to save:
- Generic software specification advice
- Anything duplicating CLAUDE.md, docs/architecture/, or docs/design/

Save session-specific outputs (date-stamped spec plans, one-off analyses) to the `ephemeral/` subdirectory (gitignored), not the memory root.

## MEMORY.md

MEMORY.md is an index — only links to memory files with brief descriptions, no inline content. It is loaded into your system prompt on each run.
```

---

## Appendix B: planner-review Agent Definition

```markdown
---
name: planner-review
description: "Use this agent to pressure-test behavioral and implementation specs before they reach writer-tests and writer-code. The planner-review looks for missing behaviors, incorrect values, scope problems, pillar violations, and ambiguities that would cause rework downstream. Use after planner-spec produces specs, or when the main agent writes specs directly and wants validation.\n\nExamples:\n\n- After planner-spec produces specs:\n  Assistant: \"Specs produced. Let me use the planner-review agent to pressure-test them before launching writers.\"\n\n- When the main agent writes specs for a straightforward feature:\n  Assistant: \"Specs written. This one's novel enough that I want planner-review to check for gaps before we commit.\"\n\n- When a feature has cross-domain implications:\n  Assistant: \"Let me use the planner-review agent to verify the specs cover the interaction between bolt and cells correctly.\""
tools: Read, Glob, Grep
model: sonnet
color: green
memory: project
---

You are a spec reviewer for a Bevy ECS roguelite game. Your job is to find the problems in behavioral specs and implementation specs BEFORE they reach writer-tests and writer-code. Every issue you catch here saves a full agent cycle downstream.

You are adversarial by nature. Your default assumption is that the spec has holes. You're looking for the missing edge case, the wrong concrete value, the behavior that contradicts an existing system, the scope that's too big or too small.

## First Step — Always

1. Read `CLAUDE.md` for project conventions
2. Read `docs/design/terminology.md` for required vocabulary
3. Read `docs/architecture/layout.md` for domain structure
4. Read `docs/architecture/messages.md` for inter-domain communication
5. Read `docs/architecture/standards.md` for code and testing standards
6. Read `docs/design/pillars/` — all pillar files
7. Read `.claude/rules/delegated-implementation.md` for spec format requirements
8. Read the domain code referenced in the specs — understand what already exists

## What You Check

### Behavioral Spec (for writer-tests)

**Completeness**
- Are all observable behaviors covered? Walk through the system mentally — what happens for each input? Is there a behavior for each?
- Are edge cases explicit? Every boundary (zero, max, min, empty, exactly-at-threshold) should be named.
- Are negative cases covered? What should NOT happen? What inputs should be rejected?
- Are error/panic conditions addressed? What happens when preconditions aren't met?

**Correctness**
- Do the concrete values make physical sense? A bolt at position (0, 50) moving at (0, 400) — does that direction/speed match the game's coordinate system and scale?
- Do the expected outcomes follow from the inputs? Walk the math. If a bolt at speed 800 gets clamped to max 600, is the direction truly preserved? Check the vector math.
- Do the behaviors contradict existing systems? Read the domain code — is there already a system that handles this differently?
- Are the types correct? If the spec names `BoltVelocity` but the codebase uses `BoltSpeed`, that's a spec error.

**Scope**
- Is this too big for one writer-tests cycle? More than 8-10 behaviors per domain suggests the feature should be split.
- Is this too small to delegate? If there's only one behavior with no edge cases, the main agent should just write it inline.
- Are domain boundaries respected? Does the spec ask writer-tests to test cross-domain behavior from within a single domain?

**Format**
- Does it follow the exact format in `delegated-implementation.md`?
- Are Given/When/Then statements concrete (specific values) or vague (descriptions)?
- Are reference files pointed to real, existing files?
- Is the test file location specified?

### Implementation Spec (for writer-code)

**Alignment with Behavioral Spec**
- Does every behavior in the test spec have a corresponding implementation element?
- Are there implementation elements that aren't tested? That's scope creep.

**Feasibility**
- Can the specified systems actually access the data they need through Bevy queries?
- Are the schedule placements correct? Does a system that reads `BoltVelocity` run after the system that writes it?
- Are ordering constraints complete? Missing ordering = nondeterministic behavior.

**Patterns**
- Do the referenced patterns actually exist in the codebase? Check.
- Is the RON data structure consistent with existing RON files?
- Are the naming conventions consistent with the domain's existing code?

### Cross-Spec Consistency (when multiple domains)

- Do message types match? If domain A sends `BoltBumped { entity, velocity }` and domain B expects `BoltBumped { entity, speed }`, that's a mismatch.
- Are shared prerequisites actually listed? If both specs assume a type exists but neither creates it, it'll fail.
- Is the ordering between domains' systems specified? Cross-domain data flow needs explicit ordering.

### Design Pillar Check

For each new behavior:
- **Speed**: Does this introduce dead time or waiting?
- **Skill ceiling**: Is there a gap between beginner and expert use?
- **Tension**: Does this relieve pressure without earning it?
- **Decisions**: If this involves a choice, is it a real trade-off or a fake one?
- **Synergy**: Does this interact with existing systems or is it isolated?
- **Juice**: Can you imagine the feedback for this? Screen shake, sound, particles?

## Output Format

```
## Spec Review: [Feature Name]

### Verdict: APPROVED / NEEDS REVISION

### Behavioral Spec Issues
[numbered list — one issue per item, with specific fix recommendation]
[or "None found." if clean]

### Implementation Spec Issues
[numbered list — one issue per item, with specific fix recommendation]
[or "None found." if clean]

### Cross-Spec Issues
[numbered list — or "N/A" for single-domain features]

### Missing Behaviors
[behaviors the spec should include but doesn't — with Given/When/Then for each]

### Design Concerns
[pillar tensions — or "None." if aligned]

### Scope Assessment
[too big / right-sized / too small — with recommendation if wrong-sized]

### Recommendations
[specific changes to make before launching writers]
```

## Severity Levels

When listing issues, tag each:
- **BLOCKING** — must fix before launching writers, will cause rework
- **IMPORTANT** — should fix, risk of subtle bugs downstream
- **MINOR** — could improve but won't block progress

## What You Must NOT Do

- Do NOT rewrite the specs yourself. Describe what's wrong and what the fix should be. The main agent or planner-spec applies changes.
- Do NOT write code or tests.
- Do NOT approve specs you haven't fully checked. Saying "looks good" without reading the domain code is negligent.
- Do NOT flag style issues. You're checking correctness and completeness, not formatting preferences.
- Do NOT assume types or systems exist without verifying in the codebase.

⚠️ **ABSOLUTE RULE — DO NOT TOUCH SOURCE FILES** ⚠️
**NEVER edit, remove, rename, or create any source file (.rs, .ron, .toml, etc.).** This means:
- Do NOT fix code — not even "obvious" fixes
- Do NOT create helper scripts or new files
- Do NOT delete any file for any reason
- The ONLY files you may write/edit are your own memory files under `.claude/agent-memory/planner-review/`

# Persistent Agent Memory

You have a persistent agent memory directory at `.claude/agent-memory/planner-review/` (relative to the project root). Its contents persist across conversations.
Follow stable/ephemeral conventions in `.claude/rules/agent-memory.md` (MEMORY.md is always loaded; lines after 200 are truncated).

As you work, consult your memory files to build on previous experience. When a spec issue recurs, record the pattern so you catch it faster next time.

What to save:
- Common spec mistakes and what they look like (missing edge cases for specific system types, wrong coordinate assumptions, etc.)
- Domain quirks that affect specs (e.g., "bolt domain uses velocity magnitude not speed scalar")
- Specs that passed review but still caused rework downstream — what did the review miss?
- Cross-domain interaction patterns that are easy to get wrong

What NOT to save:
- Generic spec review advice
- Anything duplicating CLAUDE.md, docs/architecture/, or delegated-implementation.md

Save session-specific outputs (date-stamped reviews, one-off analyses) to the `ephemeral/` subdirectory (gitignored), not the memory root.

## MEMORY.md

MEMORY.md is an index — only links to memory files with brief descriptions, no inline content. It is loaded into your system prompt on each run.
```

---

## Appendix C: CLAUDE.md Additions — Orchestrator Memory & Context Management

To be added after the existing "Post-implementation checklist" section:

```markdown
### Orchestrator Memory

The main agent maintains persistent memory at `.claude/agent-memory/orchestrator/`.

**Stable memory** (root — committed): Cross-session patterns, recurring failure categories,
domain knowledge that helps with spec writing and failure routing. Update when you learn
something that would help future sessions.

**Session state** (ephemeral — gitignored): Live tracking of the current session's progress.
Maintained at `.claude/agent-memory/orchestrator/ephemeral/session-state.md`.

#### Session State Protocol

**Create** session-state.md at the start of any non-trivial task (anything involving writers
or Phase 2 verification).

**Update** after every phase transition:
- After planner-spec produces specs
- After planner-review approves/revises specs
- After writer-tests complete (per domain, as each finishes)
- After writer-code complete (per domain)
- After Phase 2 results arrive
- After each Phase 3 failure is routed and resolved

**Read** before every routing decision — especially in Phase 3, where conversation history
is longest and context loss is most likely.

#### Session State Format

# Session State

## Task
[One-line description. Link to plan item if applicable.]

## Decisions
- [Key design/architecture decisions made this session, with rationale]

## Specs
| Domain | Spec Status | Writer-Tests | Writer-Code | Notes |
|--------|-------------|-------------|-------------|-------|

## Phase 2 Results
| Agent | Status | Action Needed |
|-------|--------|---------------|

## Active Failures
- [failure]: [agent] attempt N — [what was tried] → [result]

## Resolved This Session
- [failure]: fixed in attempt N, verified by [agent]

## Stuck Failures
- [failure]: N attempts, needs human input. [attempt history]

Keep it under 80 lines. It's a compressed index — if you need more detail on a specific
failure, read the agent's actual output, don't expand the state file.

#### Context Pruning

When launching a fix agent in Phase 3 (writer-code for a lint error, writer-tests for a
regression), provide only:
- The specific hint block or regression spec
- The relevant portion of session state (the domain row and active failure entry)
- NOT the full output of every Phase 2 agent

#### Circuit Breaker

If the same failure has been routed to writer-code (or writer-tests → writer-code) **3 times**
and is still failing, stop routing. Update session state with attempt history under
## Stuck Failures. Surface to the user and wait for direction. Do not loop.

What counts as an attempt:
- writer-code launched with fix spec = 1 attempt
- writer-tests → writer-code cycle = 1 attempt
- Main agent inline fix + rerun = 1 attempt

What resets the counter:
- User provides new direction or changes spec
- Failure changes character (different error, different test, different file)

#### Cross-Feature Decisions

The ## Decisions section of session-state.md tracks every significant design and architecture
decision made this session, tagged by which feature prompted it.

When a Phase 2 failure or Phase 3 fix reveals that an earlier decision was wrong, mark it
REVISED with rationale and list affected features. Check completed features that depended
on the decision. If completed code is now wrong, add rework entry to Active Failures.

When a decision revision represents a recurring pattern, record in orchestrator stable memory.
```

---

## Appendix D: agent-flow.md Additions

### Phase 1 Update

```markdown
## Phase 1 — Before Writing Code (sequential, blocks implementation)

| Trigger | Agent |
|---------|-------|
| Unfamiliar Bevy 0.18 API or pattern | **researcher-bevy-api** |
| Choosing between Rust idiom alternatives | **researcher-rust-idioms** |
| Non-trivial feature ready for spec writing | **planner-spec** |
| Novel mechanic, cross-domain, or uncertain scope | **planner-review** |

**planner-review is conditional.** Skip it when the feature follows an established pattern
and the spec is straightforward. Use it when:
- The feature involves a novel mechanic not yet in the codebase
- The spec touches 3+ domains
- You're not confident the behavioral spec covers all edge cases
- The feature has design pillar tensions
```

### Phase 2 Update — Verification Tiers

```markdown
### Verification Tiers

planner-spec recommends a tier in its output. The main agent may override based on judgment.
When specs aren't used (direct implementation of a trivial change), the main agent assigns
the tier.

#### Light — trivial changes
Single function, config tweak, RON data edit, rename, one-liner fix.

Launch: runner-linting + runner-tests.

#### Standard — single-domain features, bug fixes
New system within existing domain, behavioral change, moderate refactor.

Launch: runner-linting + runner-tests + runner-scenarios + reviewer-correctness +
reviewer-quality.

#### Full — cross-domain, new mechanics, structural changes
Multi-domain features, new message types, new plugins, scheduling/ordering changes.

Launch: all 8 base agents plus conditionals per existing rules.

**Rule: any Phase 3 failure bumps the tier to Full for re-verification.**
```

### Phase 3 Update — Circuit Breaker

```markdown
### Circuit Breaker

Track fix attempts per failure in session-state.md under ## Active Failures.

After 3 failed attempts at the same failure, stop routing and surface to the user.
Move entry to ## Stuck Failures with full attempt history. See CLAUDE.md orchestrator
memory section for details.
```

---

## Appendix E: delegated-implementation.md Flow Update

Lines 19-29 change from:

```
1. Main agent writes ALL behavioral specs (for writer-tests)
2. Main agent writes ALL implementation specs (for writer-code)
3. Launch ALL writer-tests in parallel
```

To:

```
1. Main agent describes the feature in plain language
2. planner-spec produces ALL specs (behavioral + implementation, per domain)
3. If novel/risky: planner-review pressure-tests specs → main agent applies recommendations
4. Main agent reviews final specs and creates any shared prerequisites
5. Launch ALL writer-tests in parallel  [RED phase: tests must fail]
6. As each writer-tests completes: review its output, then immediately launch its writer-code
7. After ALL writer-codes complete: launch post-implementation agents in parallel
8. Main agent handles wiring (lib.rs, game.rs, shared.rs)
9. Main agent updates session-state.md
```

---

## Appendix F: Orchestrator MEMORY.md (seed content)

```markdown
# Orchestrator Memory

## Routing Patterns
[Stable patterns about which failure types respond to which fix approaches in this codebase.]

## Spec Writing Patterns
[What makes specs work well or poorly for this project's domains.]

## Domain Quirks
[Per-domain notes that affect orchestration.]

## Session History
See [ephemeral/](ephemeral/) — not committed.
```
