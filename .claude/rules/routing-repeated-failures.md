# Circuit Breaking

When fix attempts repeatedly fail, stop retrying and escalate.

After **3 failed attempts** at the same failure → stop routing → move to Stuck → use `/debug` to systematically investigate root cause before surfacing to user.

## What Counts as 1 Attempt

- writer-code with fix spec
- writer-tests → writer-code cycle
- Main agent inline fix + rerun

## What Resets the Counter

- User provides new direction or changes spec
- Failure changes character (different error, different test, different file)
- `/debug` identifies a new root cause (resets to 0 with the new understanding)

## What NOT to Do

Do not: keep trying variations, weaken tests, escalate to different agent types hoping for luck.
