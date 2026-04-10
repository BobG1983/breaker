# Name
On

# Parameters
- Participant: Which entity in the trigger event to redirect to
- Terminal: The operation to perform on that entity (Fire, Stamp, or Route)

# Description
On redirects an operation to a different entity instead of the Owner. By default, Fire targets the Owner — the entity whose effect tree is being walked. On lets you target a trigger participant instead.

On(ImpactTarget::Impactee, Fire(Vulnerable(2.0))) means "make the thing I just hit take double damage." On(DeathTarget::Killer, Fire(SpeedBoost(1.5))) means "give the killer a speed boost."

On takes a Participant (a named role in the trigger event) and a Terminal. The terminal can be:
- Fire(Effect) — execute an effect on the participant
- Stamp(Tree) — permanently install an effect tree on the participant
- Route(Tree) — install a one-shot effect tree on the participant

On(Owner) never appears — if you want to target the Owner, just use Fire directly.
