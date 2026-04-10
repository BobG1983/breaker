# Name
Once

# Parameters
- Trigger: The game event to listen for
- Tree: What to evaluate when the trigger matches

# Description
Once is a one-shot gate. It works exactly like When, but after the first trigger match it removes itself. It will never fire again.

Once(PerfectBumped, Fire(Explode(...))) means "the next time a perfect bump happens, explode — but only once." After that first perfect bump, the Once entry is gone.

Once doesn't care about reversibility — it's just a gate that happens to self-destruct. The inner tree can be anything. Once(PerfectBumped, Until(Died, Fire(SpeedBoost(1.5)))) is valid — "on the next perfect bump, gain a speed boost that lasts until death."
