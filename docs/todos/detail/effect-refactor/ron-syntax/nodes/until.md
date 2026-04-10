# Name
Until

# Parameters
- Trigger: The game event that ends the effect
- Scoped Tree: Effects to apply until the trigger fires

# Description
Until is event-scoped. It applies its inner effects immediately, then waits for a trigger. When the trigger fires, the effects are reversed and the Until entry removes itself.

Until(Died, Fire(SpeedBoost(1.5))) means "speed boost right now, reversed when I die." Until(TimeExpires(3.0), Fire(DamageBoost(2.0))) means "double damage for 3 seconds."

Like During, the immediate children must be reversible (because they'll be reversed when the trigger fires). And like During, nesting a When relaxes this rule.

Unlike During, Until is one-shot — once the trigger fires and effects are reversed, the Until is gone. It doesn't cycle.
