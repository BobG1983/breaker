# Name
When

# Parameters
- Trigger: The game event to listen for
- Tree: What to evaluate when the trigger matches

# Description
When is a repeating gate. It listens for a trigger, and every time that trigger fires, it evaluates its inner tree. After evaluation, When resets and listens again — it never runs out.

When(PerfectBumped, Fire(SpeedBoost(1.5))) means "every time a perfect bump happens, apply a speed boost." The speed boost fires on every single perfect bump, forever, as long as this effect tree is on the entity.

When can nest anything inside it — Fire, Sequence, another When, Once, On, even During or Until. When(PerfectBumped, When(Impacted(Cell), Fire(Shockwave(...)))) means "after a perfect bump, wait for the next cell impact, then fire a shockwave." The inner When is armed after the outer When matches, then consumed when it matches. The outer When re-arms on the next perfect bump.
