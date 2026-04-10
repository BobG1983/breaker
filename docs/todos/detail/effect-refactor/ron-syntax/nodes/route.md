# Name
Route

# Parameters
- RouteType: Where to install the tree — Bound (permanent) or Staged (one-shot)
- Tree: The effect tree to install

# Description
Route installs an effect tree on another entity. It only appears inside On() — you redirect to a participant, then Route a tree onto them.

Route has two modes controlled by RouteType:

**Bound** — permanently installs the tree. It re-arms after each trigger match, just like a definition-level Stamp. Use when you want a lasting effect on the participant.

When(Impacted(Cell), On(Impact(Impactee), Route(Bound, When(Died, Fire(SpeedBoost(2.0)))))) means "when I hit a cell, permanently give that cell a 'speed boost on death' effect."

**Staged** — installs a one-shot tree that is consumed after its trigger matches once.

When(Impacted(Cell), On(Impact(Impactee), Route(Staged, When(Died, Fire(Explode(ExplodeConfig(...))))))) means "when I hit a cell, give that cell a one-shot 'explode when you die' effect." If the cell dies, it explodes and the routed tree is consumed.
