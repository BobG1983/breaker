# Name
CircuitBreaker

# Parameters
`CircuitBreakerConfig`

# Description
A charge-and-release mechanic. Each time the effect fires (typically on bump), an internal counter increments. When the counter reaches the required number of bumps, a reward fires: extra bolts are spawned and a shockwave bursts from the entity. The counter then resets and the cycle begins again. This creates a rhythmic burst pattern -- consistent bumping leads to periodic explosive payoffs. See [CircuitBreakerConfig](../configs/circuit-breaker-config.md).
