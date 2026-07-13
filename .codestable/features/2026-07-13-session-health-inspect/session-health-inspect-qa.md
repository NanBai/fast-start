---
doc_type: feature-qa
feature: 2026-07-13-session-health-inspect
status: passed
date: 2026-07-13
---

# session-health-inspect QA

| Command | Result |
|---------|--------|
| cargo test --lib | 106 passed |
| pnpm build | passed |

Unit: missing_cwd/empty_summary/cache_limited、file bytes、OpenCode row missing。

Verdict: **passed**
