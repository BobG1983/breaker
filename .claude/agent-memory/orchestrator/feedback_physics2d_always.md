---
name: Always use rantzsoft_physics2d — never custom spatial/physics code
description: All spatial queries, collision detection, and physics logic must go through rantzsoft_physics2d. If capability is missing, add it. If performance is lacking, improve it.
type: feedback
---

ALWAYS use `rantzsoft_physics2d` (and `rantzsoft_spatial2d`) APIs for spatial queries, collision detection, constraint logic, and any physics-adjacent computation. NEVER write custom spatial or physics code in the game crate (`breaker-game`).

**Why:** The user wants all physics/spatial logic centralized in the reusable crates. Game-crate workarounds fragment the codebase, skip improving the shared library, and create maintenance burden.

**How to apply:**
- If a physics2d/spatial2d method exists for the task, use it.
- If the crate **lacks the capability** and it's within the crate's domain (spatial queries, collision, constraints, CCD, quadtree), **add the capability to the crate** rather than working around it in game code.
- If the crate's API **is not performant enough**, **improve the crate's performance** rather than bypassing it with a hand-rolled alternative.
- This applies to spec writing too — specs must mandate physics2d/spatial2d usage and must never spec custom spatial logic in the game crate.
