# Name
Die

# Parameters
`DieConfig` — See [DieConfig](../configs/die-config.md)

# Description
Kills the entity. The entity enters the death pipeline -- domain-specific checks run (invulnerability, shields), death triggers fire (Died on the victim, Killed on the killer if any), and finally the entity is despawned. This is how one-shot walls work: a wall with When(Impacted(Bolt), Fire(Die(DieConfig()))) destroys itself when a bolt hits it.
