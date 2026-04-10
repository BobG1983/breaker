# Name
During

# Parameters
- Condition: The state to watch (NodeActive, ShieldActive, ComboActive)
- Scoped Tree: Effects to apply while the condition is true

# Description
During is state-scoped. It watches a condition, and while that condition is true, its inner effects are active. When the condition becomes false, the effects are reversed. If the condition becomes true again, the effects reapply. This can cycle indefinitely.

During(NodeActive, Fire(SpeedBoost(1.5))) means "while the node is active, the entity is 1.5x faster. When the node ends, the speed boost is removed."

Because During reverses its effects, the immediate children must be reversible. You can't put During(NodeActive, Fire(Explode(...))) because an explosion can't be un-exploded. However, you CAN nest a When inside During: During(NodeActive, When(PerfectBumped, Fire(Explode(...)))) is valid — the reversal removes the When listener, not the individual explosions that already happened.

During stays on the entity permanently — conditions can cycle on and off and the effects toggle accordingly.
