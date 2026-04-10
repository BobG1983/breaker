# Name
Route

# Parameters
- Tree: The effect tree to install

# Description
Route is a one-shot terminal that installs an effect tree on another entity. The tree is consumed after its trigger matches once — it doesn't re-arm like a Stamped tree does.

Route only appears inside On() — you redirect to a participant, then Route a tree onto them. The classic example is the powder keg pattern: When(Impacted(Cell), On(ImpactTarget::Impactee, Route(When(Died, Fire(Explode(...)))))) means "when I hit a cell, give that cell a one-shot 'explode when you die' effect." If the cell dies, it explodes and the routed tree is consumed. If the bolt hits another cell, it gets its own separate powder keg.

Contrast with Stamp inside On() — Stamp permanently installs a tree that re-arms. Route installs a tree that fires once and is gone.
