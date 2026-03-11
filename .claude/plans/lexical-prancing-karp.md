# Plan: Simplify ron-lsp step in test-runner

## Context

ron-lsp is already installed. User will add `~/.cargo/bin` to PATH. The current test-runner step has an unnecessary `$HOME/.cargo/bin` fallback — simplify to just call `ron-lsp` directly.

## Changes

### 1. Simplify ron-lsp command in test-runner
**File**: `.claude/agents/test-runner.md` (line 29)

Replace:
```
RON_LSP=$(which ron-lsp 2>/dev/null || echo "$HOME/.cargo/bin/ron-lsp"); [ -x "$RON_LSP" ] && "$RON_LSP" check . 2>&1 || echo "ron-lsp not installed, skipping RON validation"
```

With:
```
ron-lsp check . 2>&1
```

## Verification
1. Test-runner step 2 uses plain `ron-lsp check .`
