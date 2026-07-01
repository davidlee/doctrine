# Review RV-207 — reconciliation of SL-184

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation audit of SL-184 (rename `memory find` → `search` + adopt shared
listing spine). Subject reviewed: the dispatch worktree branch `SL-184/phase-01`
(`provenance = solo`), phases PHASE-01 (35ee40fa) + PHASE-02 (e297262), plus the
audit-remediation commit 1397d2b4. RV-206 (code-review facet) precedes this audit
and holds the code findings; RV-207 carries the reconciliation lens.

Lines of attack:

1. **Mechanical conformance** — do the `design-target` selectors match what git
   actually touched? (`slice conformance`.)
2. **Design fidelity** — does the implementation match `design.md`, including the
   pagination clamp (§2), column model (§2), and MCP mapping (§3)?
3. **Governance/origin drift** — is IMP-220 (the `originates_from`) still accurate
   after the design's silent-alias override of its stderr-notice proposal?
4. **RV-206 remediation state** — were its `fix-now` findings actually applied, and
   is each finding sound?

Invariants held: behaviour-preservation gate (existing suites green unchanged);
storage rule (no drift between design prose and shipped code); no scope creep
beyond declared selectors.

## Synthesis

**Closure story.** SL-184 is a clean, mechanically-faithful two-phase change:
PHASE-01 renamed the `find` surface to `search` across CLI, internal, MCP, and
tests; PHASE-02 replaced the hand-rolled `format_find_table` with the shared
`listing::render_columns` spine (15 columns, 8 default, colour + comfy-table).
Conformance is exact — 7/7 declared selectors delivered, zero undeclared, zero
undelivered. The green baseline (build + clippy zero-warnings + full suite) holds
at the remediation tip.

**RV-206 handoff.** The prior code-review (RV-206) was ledger-only: its three
`fix-now` findings were dispositioned but never applied to code. This audit closed
that gap in commit 1397d2b4 — F-1 (stale `find` doc prose) and F-2 (wrong
`page_size` constant in the search truncation notice) fixed. F-2 was a genuine
behavioural defect: for `--offset > 0` without `--limit`, the notice reported
"page size 5" (the retrieve default) instead of the number shown.

**F-5 was a false finding.** RV-206 F-5 proposed removing `.min(ranked.len())`
from the pagination end-bound as "redundant." It is not: `slice.get(offset..end)`
returns `None` (→ empty) when `end > len`, so with `--limit` unset (`end ==
usize::MAX`) the clamp is what keeps any rows visible. Applying F-5 emptied all
search output and the existing `writer_capture_run_search` test caught it
immediately; design §2 (line 100) independently mandates the clamp. Reverted and
documented inline. Lesson for reviewers: `Vec::get(range)` does not saturate.

**Tradeoffs consciously accepted (RV-206 tolerated, unchanged).** F-3 (JSON path's
intermediate `Vec<&Candidate>` — negligible, shared `From` impl with the MCP
path); F-4 (`--columns` appears in `retrieve --help` — design-acknowledged, help
text already says "ignored by retrieve"); F-6 (`search_columns()` built per call —
`Candidate<'a>`'s non-static lifetime precludes a `LazyLock`; cost invisible for a
one-shot CLI). All sound.

**Standing risks:** none. The rename is proven by tests; the listing spine is a
well-exercised shared path (REC/REVIEW surfaces).

**Audit-process note (not a slice defect).** During evidence-gathering the auditor
flipped both phases to `completed` from the main tree; this re-stamped the
source-delta registry with degenerate `start==end==HEAD` boundaries, clobbering
the real worktree-recorded ranges and making conformance mis-report every selector
as undelivered. Recovered with `slice record-delta`. Captured as memory
`mem.pattern.doctrine.phase-complete-clobbers-boundary` (high severity) so the
foot-gun is not repeated.

## Reconciliation Brief

### Per-slice (direct edit)
- None. `design.md` is accurate to the shipped implementation (including the §2
  `.min(ranked.len())` clamp the code retains). No prose sync required.

### Governance/spec (REV) — none
- No ADR / spec / REQ / standard / policy drift. The change is CLI/UX-local.

### Backlog (direct edit, delegated to /reconcile)
- **IMP-220 §1** — supersede the stale alias premise: "hidden alias, prints a
  redirect notice to stderr" → **silent** clap alias (no stderr notice), per
  SL-184 design §1. Edit IMP-220's scope text to match what shipped (RV-207 F-1).
- **IMP-220 lifecycle** — SL-184 `originates_from` IMP-220; mark IMP-220 fulfilled
  / resolved at close (Axis B, ADR-018) once SL-184 lands on main.
