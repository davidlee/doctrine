# IMP-476: Unify review read shapes across CLI and MCP (findings vs finding)

The CLI's `review show --json` nests findings under `review.finding` (singular);
the MCP `review_show` tool returns `Showed.findings` (plural). Findings also carry
an empty `summary` while `detail` can hold a multi-KB inlined entity dump.

Observed: obs `01a0d801`, `019fac2f`. See RFC-032 `research.md` F1/F10.

Fix: one shape for both transports, a stable finding `summary`, and a decision on
whether `detail` should carry inlined entity dumps.
