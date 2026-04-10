# Name
On

# Parameters
- Participant: Which entity in the trigger event to redirect to
- Terminal: The operation to perform on that entity (Fire or Route)

# Description
On redirects an operation to a different entity instead of the Owner. By default, Fire targets the Owner — the entity whose effect tree is being walked. On lets you target a trigger participant instead.

On(Impact(Impactee), Fire(Vulnerable(VulnerableConfig(multiplier: 2.0)))) means "make the thing I just hit take double damage."

On(Death(Killer), Fire(SpeedBoost(SpeedBoostConfig(multiplier: 1.5)))) means "give the killer a speed boost."

On takes a Participant (a named role in the trigger event) and a Terminal. The terminal can be:
- Fire(Effect) — execute an effect on the participant
- Route(Bound, Tree) — permanently install an effect tree on the participant
- Route(Staged, Tree) — install a one-shot effect tree on the participant

On(Owner) never appears — if you want to target the Owner, just use Fire directly.

## Inside During/Until (scoped context)

When On appears as a direct child of During or Until, its terminal is a ScopedTerminal — Fire must use a ReversibleEffectType, not any EffectType. Route is unchanged.
