# Review RV-303 — design of SL-228

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

External adversarial inquisition (raiser label `inquisitor`; the raising arm is
**codex / GPT-5.5 via MCP**, per the project reviewer default) of the SL-228
design — `.doctrine/slice/228/design.md` — against:

- **Requirements**: SPEC-021 REQ-384/385/386/387 (FR-008..011), SPEC-022
  REQ-388/389 (FR-010/011) — all `pending`, minted by REV-032. Fidelity
  question per FR: does the named mechanism actually satisfy the statement?
- **Ground truth**: `.doctrine/slice/228/extraction.md` (as-built graph at
  `0603f11f`) — design deltas against it; extraction claims verify against code.
- **Governance**: ADR-001 (layering), ADR-006/012 (worktree/dispatch topology),
  ADR-011 (mechanism-over-prose, capability altitude), POL-002 (platform
  independence), STD-001 (no magic strings), STD-002 (naming).

Lines of interrogation (soft spots named up front, honestly):

1. §5 reverse-diff-set derivation — stated at design altitude; is it too
   hand-wavy to carry the fault-safety claim (R3, I2)?
2. D8 provable-prefix heal-forward vs REQ-387 single-authority — does healing
   at import weaken "one writer"?
3. §9 `ReceiptStatus` mapping — position × sheet matrix completeness.
4. Class-2 `Reap`: crash between `run_gc` and the record destroys the
   landed-oracle's evidence — is idempotent completion sound?
5. §7 hook chaining — global `core.hooksPath` case (operator has one), runtime
   resolution, worktreeConfig precedence.
6. §3 atomicity classes — is Class-1 truly windowless; are Class-2 heals
   provable from git facts alone?
7. Imprecision, hidden assumptions, weak verification (§11), scope drift.

Out of scope: re-litigating locked D1–D8 absent evidence of defect; the six
already-integrated internal findings (probe whether the fixes hold, don't
re-raise); `worker_commit` validate-skip legs (SL-225 DEC-003 — settled law).
