---
name: audit
description: Use after a slice's phases are implemented, when the task is now evidence, conformance, and reconciliation against the design — disposition every finding on a reconciliation review ledger (the RV kind) before closure.
---

# Audit

You are running the reconciliation loop: does the work match its design and
governance, and is every gap consciously dispositioned before reconciliation?

The audit stage runs on a **review ledger** — the RV kind (`RV-NNN`, ADR-007). The
shared ledger mechanics (open + prime, raise, dispose + resolve, the severity and
disposition vocab, synthesis, the close-gate, the parent-tree caveat) live in
`.doctrine/review-ledger.md` — **read it; this skill does not repeat the verbs.** 
What follows is the audit *lens*: the facet, the modes, the scope, the evidence 
the reconciliation loop demands, and the audit-specific harvest and closure tail.

Findings are append-only to the ledger and field-owned, "no undispositioned 
findings before close" is enforced by the binary
(the close-gate teeth), and the audit prose becomes the review's `## Synthesis`.

> **Dispatched slice — review the candidate surface, not the raw evidence.** When
> the slice was driven by `/dispatch`, `review/*` and `phase/*` are immutable
> evidence refs (R2); audit/repair runs against the **candidate interaction
> branch** published by `doctrine dispatch candidate create` (see `doctrine
> dispatch candidate status`). Record which surface you reviewed in the ledger
> `## Brief` (F-2), and link the admitting RV via `doctrine dispatch candidate
> admit --review RV-NNN`.

Inputs:

- the slice's implemented phases and their verification evidence
- `design.md` (canonical), `slice-nnn.md`, `plan.toml`
- relevant ADRs and tech specs (see `/canon`)

## Tool preference

If your harness supports MCP tools and doctrine's MCP server is connected
(you see `review_new`, `review_raise`, `review_dispose`, `review_verify`,
`review_prime`, `review_list`, `review_show`, `review_status` in your
tool list), **prefer these MCP tools over the CLI** — they return machine-parseable
JSON text in the MCP content block and eliminate shell overhead. Every review verb has
an MCP equivalent.

## Audit lens

**Subject is always the slice — target-ladder rung 1.** An audit targets its slice
(the `--target`) and never degrades to prose; the closure-grade trigger is
satisfied by definition (it gates the slice's `audit→reconcile→done`). Do not
re-derive the subject — open the RV against the slice.

**Facet is `reconciliation`.** That is the lifecycle aspect this stage
interrogates. Posture, if any, rides `--raiser`, never a new facet (`review-ledger.md`
§2).

**Audit mode** — pick one:

- **conformance** — post-implementation audit tied to a slice (the usual case).
- **discovery** — backfill or existing-code investigation.

**Self-audit (the usual case).** When you are both reviewer and author, drive both
roles with `--as <role>` — the raiser raises/verifies/withdraws, the responder
disposes. This is cooperative role assertion, not a security boundary (ADR-007;
`review-ledger.md` §4).

**Disposition convention (audit-specific).** Audit's permitted dispositions are:
`aligned` (observation correct, no change needed), `fix-now` (code fix within
audit scope — never a spec or governance edit), `tolerated` (explicit accepted
drift with rationale), and `verified` with a reconciliation-brief link for
spec/governance changes delegated to `/reconcile`. Audit must **never** use
`design-wrong` or `follow-up` for spec/governance items — those belong to the
reconcile write surface. Every finding stays `verified` (the observation is
confirmed); the *remediation* is reconcile's job and is recorded separately — do
not mutate a finding to `fixed`/`remediated`.

## Process

1. **Open the ledger for the slice** (replaces authoring `audit.md`): open a
   `reconciliation`-facet RV targeting the slice, then fill the ledger's `## Brief`
   with the lines of attack (what this audit probes and the invariants it holds the
   slice to). Verbs and flags: `review-ledger.md` §1–§2. Loose notes are
   insufficient for closure-grade work — findings belong in the ledger.

   The old `domain_map`/`prime` seeding is **gone** (RFC-004 / SL-147): the
   hand-authored area map was a dead authoring tax. The mechanical drift signal now
   comes from `slice conformance` (step 2), computed from recorded source-deltas —
   no curation.
2. **Gather evidence** (the audit's divergent work):
   - prepare subject: do NOT change the main repository branch; use a worktree
     instead, if necessary.
   - **Run `doctrine slice conformance <id>` and read the algebra** — the
     mechanical path-conformance delta between what `design.md` declared
     (`design-target` selectors) and what git actually touched (recorded
     source-deltas). It reports three cells:
     - **undeclared** (highest signal) — paths edited but not in any
       `design-target` selector. Each is a finding candidate: scope creep, a
       missed design update, or an undocumented touch.
     - **undelivered** — `design-target` selectors that matched no actual edit.
       Declared-but-not-delivered: dropped work or a stale design.
     - **conformant** — count of paths that matched (each with its selector).

     Conformance is **necessary, not sufficient**: it says *where to look*, never
     *whether it passes*. Treat undeclared/undelivered as leads to disposition, not
     auto-findings. If it reports `unavailable` (empty registry) or `incomplete`
     (a completed phase carries no row — the F-2 backstop), that gap is itself a
     finding: the registry was not recorded as phases landed; bootstrap with
     `doctrine slice record-delta <id> PHASE-NN --start <oid> --end <oid>` or note
     the partial coverage — never read a partial registry as clean.
   - run the tests/checks the design and plan require, **plus `doctrine check gate`**;
   - inspect observed behaviour against `design.md` and the phase `VT-` criteria;
   - note where behaviour and design diverge — each divergence is a finding.
3. **Raise + dispose every finding** on the ledger per `review-ledger.md` §3–§4.
   Hold the audit line on the **anti-escape pressure**: do not pick **follow-up**
   for spec/governance findings — those go to the reconciliation brief with
   `verified`; for code findings, do not pick **follow-up** merely because the fix
   is large; do not normalise **tolerated** without a real rationale; and do not
   downgrade a true **blocker** to dodge the close-gate. If the right route is
   ambiguous after reading `design.md` and governance, stop and `/consult`.
4. **Synthesize.** Write the audit's reasoning as the review's `## Synthesis`
   (append it to `review-NNN.md`) — the closure story, the standing risks, the
   tradeoffs consciously accepted (the prose the old `audit.md` carried).
5. **Write the reconciliation brief.** Append a dedicated `## Reconciliation Brief`
   section to `review-NNN.md` — separate from `## Synthesis`. This is the
   structured handoff from audit to `/reconcile`, mapping every spec/governance
   finding to its target and the intended write surface (D3):

   ```markdown
   ## Reconciliation Brief

   ### Per-slice (direct edit)
   - design.md §3: the eviction model changed from edge-at-a-time to per-SCC —
     update prose to match implementation.

   ### Governance/spec (REV)
   - ADR-006 §D5: branch-point staleness description is wrong → REV modify
   - REQ-077: cordage scale target verified at 50k nodes → REV status active
   ```

   Build the brief from every non-aligned, non-tolerated finding that touches
   design or governance. Group by write surface (per-slice direct edit vs.
   governance/spec REV). Each entry cites the finding id and describes the exact
   change needed.
6. **Harvest (audit tail).** Harvest durable risks, decisions, and gotchas from the
   disposable runtime phase sheets into `notes.md`; promote reusable facts via
   `/record-memory`; capture durable follow-up **work** the audit surfaced — risks,
   issues, chores — as backlog items with `backlog new` (the work / knowledge /
   decision boundary: `using-doctrine.md`).
7. **Hand off to reconcile.** Once the reconciliation brief is written, the ledger
   is resolved, and every finding is terminal, hand off to `/reconcile`. Do NOT
   hand off directly to `/close` — reconcile is the sole writer of reconciled
   truth; close only confirms the outcome. Record the lifecycle move:
   `doctrine slice status <id> reconcile` (bare number) — the binary refuses it
   while a blocker is unresolved (D-C9b).

## Outcomes

- Audit evidence is a structured RV ledger (`review-NNN.toml` + the review's
  `## Synthesis` + `## Reconciliation Brief`), not a hand-made `audit.md`.
- Every finding ends terminal with an explicit disposition (or is withdrawn).
- No unresolved `blocker` remains — the close-gate would refuse it.
- The reconciliation brief maps every spec/governance finding to its target and
  write surface.
- `/reconcile` receives a complete, actionable brief — not raw findings.
