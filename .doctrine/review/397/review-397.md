# Review RV-397 — implementation of SL-268

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Bounded guidance review (VA-1, DEC-320) of the shipped corpus after the T1
guidance rewrite (commit 5c8be28f2), checked against the landed binary.

**Scope.**
- `install/**/*.md` and `plugins/doctrine/skills/**/SKILL.md`. The
  review-touching subset is read in full: `install/review-ledger.md`,
  `install/design-prompts/reviewing.md`, the audit, close, code-review,
  inquisition, plan (routed-findings step) and reconcile (read-inputs step)
  skills, plus the review hunks of `install/routing-process.md`. The rest is
  grep-scanned for review verbs, flags and states, and only the hit context is
  read.
- The MCP `review_*` tool descriptions and input schemas (13 tools), taken
  from a live `tools/list` dump.
- The shipped review signpost memory T1 rewrote, and two local memories T1
  flagged as stale.

**Oracle.** `./target/debug/doctrine review <verb> --help` built at HEAD, the
`tools/list` schema, and the code where the help text is silent: the act table,
derived status, the fork guard, the design-run lock gate and the review_show
projection.

**Lines of attack.**
- Every review verb, flag, value and argument shape in the guidance must match
  `--help`. This covers required `--note` on contest, amend and reopen,
  required `--basis` on conclude, and the closed disposition and route sets.
- Every stated gate must match the binary: done, close-gate, lock gate and the
  fork refusal.
- No guidance may still teach a retired form, such as a `route:` prefix,
  "no reopening", or a free-text disposition.
- MCP descriptions must match their schema and must not cite repo-private ids
  (ADR-024).
- Internal coherence: a design pass concludes with instrument-routed findings
  still `answered`, while done needs every finding terminal plus concluded.
