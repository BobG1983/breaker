---
name: Use research agents instead of Explore — STRICT RULE
description: NEVER use Explore when a dedicated research agent exists for the task. Only use Explore when NO research agent matches.
type: feedback
---

**STRICT RULE: Only use Explore when NO dedicated research agent exists for the task.**

When a research agent matches the task, ALWAYS use it instead of Explore:

- **researcher-git** for git history analysis
- **researcher-codebase** for tracing data flow and behavior chains
- **researcher-impact** for finding all references before modifying a type
- **researcher-system-dependencies** for system ordering and query conflicts
- **researcher-bevy-api** for Bevy API verification
- **researcher-rust-idioms** for idiomatic Rust patterns
- **researcher-crates** for evaluating dependency options

Explore is ONLY for: quick file pattern matching (`*.rs` in a directory), keyword searches where no researcher fits, or when no dedicated agent exists for the task.

**Why:** The user has corrected this 4+ times across sessions. Each research agent has specialized memory, domain knowledge, and search strategies. Explore is generic and produces weaker results for specialized queries. This is a strong, repeated, non-negotiable preference.

**How to apply:** Before every agent launch, ask: "Does a research agent match this task?" If yes, use it. If no research agent fits, then Explore is acceptable.
