# Scenario Runner: Verbose Violation Log File

## Summary
Write verbose violation output to a temp file after all scenarios complete, so detailed violation info is always available without re-running with `-v`.

## Context
`ViolationLog` is an in-memory Resource (`Vec<ViolationEntry>`) that lives only during the scenario run. Violations print to stdout (compact by default, verbose with `-v`). When the process exits, violation details are lost unless captured by terminal. Agents and humans often need to inspect full details after a run without re-running.

## Scope
- In: `VerboseViolationLog` accumulation, temp file output via `std::env::temp_dir()`, path printed in `print_summary`, verbose output for every scenario with violations (not just failures)
- Out: UI changes, log rotation, changing the existing stdout output format

## Dependencies
- Depends on: Nothing
- Blocks: Nothing

## Notes
1. Accumulate verbose violation output for every scenario (regardless of `-v` flag)
2. After all scenarios complete, write to temp file (`std::env::temp_dir()` — cross-platform)
3. In `print_summary`, always print: `violation log: /tmp/scenario-violations-<timestamp>.log`
4. File contains verbose output for every scenario with violations (including expected violations)

## Status
`ready`
