# Name
BoltLostOccurred

# When it fires
A bolt falls off the bottom of the playfield and is lost.

# Scope
Global. Fires on every entity that has BoundEffects or StagedEffects.

# Description
BoltLostOccurred is global because "losing a bolt" is a game-level event, not a collision between two entities. Both the bolt and the breaker (and all other entities) need to react — breaker loses a life, bolt may trigger last-resort effects, other entities may respond.

Even though BoltLostOccurred is global, the trigger context carries which bolt was lost and which breaker lost it. This means On(BoltLost(Bolt)) and On(BoltLost(Breaker)) CAN resolve inside a BoltLostOccurred tree, unlike other global triggers where participant context is empty.

DO populate the bolt and breaker in the trigger context.
DO fire before despawning the lost bolt — its trees need to evaluate.
