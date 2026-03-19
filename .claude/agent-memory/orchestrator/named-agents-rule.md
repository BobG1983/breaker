---
name: Always use named subagents
description: Never run own versions of guard-architecture, guard-game-design, runner-tests, or runner-scenarios — always use the named subagent directly
type: feedback
---

Never run your own versions of guard-architecture, guard-game-design, runner-tests, or runner-scenarios. Always use the named subagent directly via the Agent tool with the correct subagent_type.

**Why:** The user wants consistent, auditable output from the designated agents rather than ad-hoc inline checks.

**How to apply:**
- For architecture validation -> `subagent_type: "guard-architecture"`
- For game design validation -> `subagent_type: "guard-game-design"`
- For linting -> `subagent_type: "runner-linting"` (never run cargo fmt, cargo dclippy directly — always delegate)
- For test validation -> `subagent_type: "runner-tests"` (never run cargo dtest directly — always delegate)
- For gameplay scenario validation -> `subagent_type: "runner-scenarios"` (never run cargo scenario directly — always delegate)
