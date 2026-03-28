---
name: Stub with partial real logic — position/required hardcoded to sentinel
description: writer-tests sometimes writes stubs that hardcode Vec2::ZERO and false rather than todo!() — this compiles and the required=false test passes immediately, violating RED
type: feedback
---

When stubbing a system that must populate new message fields, writer-tests may implement the send site with hardcoded sentinel values (e.g., `position: Vec2::ZERO, was_required_to_clear: false`) instead of `todo!()`. This causes:
- The test for `was_required_to_clear=false` to PASS at RED (because false is the default sentinel)
- Only the `position` and `was_required_to_clear=true` tests to fail as intended

**Why:** The writer-tests agent treats "struct must compile" as license to fill fields with neutral/zero values. But if any neutral value matches a test assertion, that test passes at RED — breaking the RED gate guarantee.

**How to apply:** Flag any stub where a hardcoded field value (Vec2::ZERO, false, 0, Entity::PLACEHOLDER) matches a test assertion. The test for that concrete value will PASS instead of failing. That is a BLOCKING finding: it means the RED gate cannot detect regressions for that behavior.
