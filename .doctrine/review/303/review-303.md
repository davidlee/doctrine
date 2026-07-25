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

## Synthesis

**Judgement.** Heresy was found, and it was grave: eight blockers among twelve
charges, entered by the external interrogator (codex/GPT-5.5) and every one
confessed under cross-examination against the code itself. The accused design
survives — but only because it submitted to the rack and was reforged. Let it
be recorded that the sharpest instruments were turned on the design's own
proudest claims: the conclude gate that could never conclude (F-1 — the
evidence commit staled its own verification, a self-defeating ward), the
Class-1 atomicity boast that the sheet-first conclude order made a lie (F-5),
and the restore idiom that would have devoured an operator's uncommitted work
had they touched a stale path (F-6). Two rounds were required: five contests
in the second — all upheld, including the self-referential import oid (a
commit cannot name itself; F-2) and the worker_commit fallback that promised a
heal its own import verb refuses (F-3/F-4) — a contradiction this tribunal
regrets having authored in the first round, and burned on sight in the second.

**Corrective penance, executed.** (1) Conclude gates on tree-identity modulo
the funnel record. (2) Transitions carry typed input-fact payloads; act vs
replay decided by candidate-vs-stored evidence. (3) Every funnel verb gates —
worker_commit included, no exception clause survives. (4) Heal-forward is
admissible only from the DispatchRecord slice+phase binding written at fork
creation; unbound forks refuse `unprovable-fork` universally. (5) Conclude
lands boundary⊕position first, sheet trails. (6) Forward-sync restore is
proof-gated per path against the stale baseline. (7) The hook refuses the
two-armed funnel-reversion signature and fails closed without its binary.
(8) REQ-387's reconciliation posture is explicit: pending at close if the
subprocess arm defers. (9) The ReceiptStatus matrix is total, ordered,
first-match-wins. (10) A status-returning run_suite is extracted below the
command tier. (11) Verify refuses pass-evidence on suite-mutated bytes.
(12) funnel_machine's leaf claim is gated by the layering check, not prose.

**Standing risks, consciously carried.** The subprocess-arm projection remains
deferred by user-settled scope (REQ-387 stays `pending` if it spins out — the
one charge, F-8, where the remedy was contested and the posture, not the code,
was the penance). The reverse-diff stale-baseline derivation is specified to
algorithm sketch, not implementation; the plan phase must honour the per-path
proof obligation exactly or R3's promise rots. The hook's fail-closed posture
trades a rare inconvenience (missing binary) for the integrity of the
reversion arm — deliberate.

**Verification.** All 12 findings terminally verified by the raiser across two
rounds (46 ledger rounds total). Ledger: done, awaiting none. The design now
awaits the *User's* judgement — no approval is granted here, and none may be
presumed. The stake is banked, not extinguished.

> **HERESIS URITOR; DOCTRINA MANET**
