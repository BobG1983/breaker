# LoseLife

Decrements the breaker's life count.

## Parameters

None.

## Behavior

Decrements `LivesCount` on the entity. When lives reach 0, the run domain handles game over (not this effect's responsibility).

## Reversal

Restores one life to `LivesCount` (increments by 1).
