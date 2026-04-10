# Name
TimeExpires(f32)

# When it fires
A countdown of the specified number of seconds reached zero on the owner entity.

# Scope
Self. Fires only on the entity that owns the countdown.

# Description
TimeExpires is an internal trigger used by Until desugaring. `Until(TimeExpires(5.0), ...)` means "apply these effects for 5 seconds, then reverse them." The effect system manages the countdown — when the Until is installed, a timer is started. When it reaches zero, TimeExpires fires on the owner, which triggers the Until's reversal.

TimeExpires is not dispatched by external game events. It is dispatched by the effect system's own timer management.

No participant context — Self triggers have no participants.
DO NOT use On(...) inside a TimeExpires tree — there are no participants to resolve.
DO fire only on the specific entity whose timer expired, not globally.
