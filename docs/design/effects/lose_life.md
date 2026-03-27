# LoseLife

Decrements the breaker's life count.

## Parameters

None.

## Behavior

Decrements `LivesCount` on the entity. When lives reach 0, sends `RunLost`.

## Reversal

No-op. Life loss is fire-and-forget.
