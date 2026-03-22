---
name: Use research agents instead of Explore for specialized tasks
description: Use dedicated research agents (researcher-git, researcher-codebase, researcher-impact, etc.) instead of Explore agents when the task matches a researcher's specialty
type: feedback
---

Use dedicated research agents for their specialties — don't default to Explore for everything.

- **researcher-git** for git history analysis (not Explore reading git log)
- **researcher-codebase** for tracing data flow and behavior chains
- **researcher-impact** for finding all references before modifying a type
- **researcher-system-dependencies** for system ordering and query conflicts
- **researcher-bevy-api** for Bevy API verification
- **researcher-rust-idioms** for idiomatic Rust patterns
- **researcher-crates** for evaluating dependency options

Explore is for quick file finding and keyword searches only.

**Why:** Each research agent has specialized memory, domain knowledge, and search strategies that produce better results than generic exploration. The user called this out multiple times — it's a strong preference.

**How to apply:** Before launching an Explore agent, check if a research agent matches the task. During Phase 1 planning, use researcher-git for history, researcher-codebase for behavior tracing, researcher-impact for modification impact analysis.
