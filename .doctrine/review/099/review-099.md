# Review RV-099 — reconciliation of SL-026

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject.** SL-026 — lazyspec read-only projection (`doctrine export lazyspec`).
Dispatched slice: 4 phase commits on `dispatch/026` (`0564488b`→`bd36940b`) plus the
INV-7 spec-date fix (`5de8e723`). **Reviewed surface (F-2):** the candidate
interaction branch `candidate/026/review-001` (`815d0804`) — the 3-way merge of the
impl-bundle `review/026` (`0c0dac5a`) onto current `main`, *not* the raw evidence
refs. The impl was authored against a base 29 commits behind main; the candidate is
the integration-true surface.

**Mode.** conformance (post-implementation audit). Self-audit: both roles via `--as`.

**Lines of attack.**
1. **INV-7 (the emergent finding).** The audit's reason to exist: specs carry no
   authored date on disk, so the projection emitted `date: ""` for all 34 specs — a
   hard break against lazyspec's mandatory `%Y-%m-%d` `DocMeta.date`. Consult
   2026-06-19 settled the source = spec toml **mtime** (lossy-v1, read-only). Verify:
   the fix holds over the *real* corpus (smoke), a regression test loads a *real
   dateless* scaffold (not a dated fixture), and the design records the decision +
   tradeoff honestly with IMP-108 as the durable-fix follow-up.
2. **Pure/impure split.** mtime is disk I/O — confirm it lives in the `load_spec`
   shell and `project()` stays pure (date injected as data). clock.rs must keep a
   single date formatter (no parallel formatter).
3. **Contract conformance.** INV-1..6 + edge label→RelationType map + wire strings
   still exact vs lazyspec source; golden + conformance pin them; the fix introduced
   no drift.
4. **Integration health.** Candidate merged cleanly (no conflict) onto a +29-commit
   main; gate green there (1913 tests, clippy clean); INV-7 holds over the live
   corpus (0/433 violations).
5. **No parallel impl / scope.** Projection rides SL-025 readers, SL-048
   `tier1_edges`, `listing::canonical_id`; backlog.rs delta is the PHASE-02
   `test_support` promotion (design §9 gap), not scope creep.
6. **Read-only.** No mutation path in the command; ADR-001 layering held.

## Synthesis

**Closure story.** SL-026 ships a conformance-tested, read-only JSON projection of
the doctrine corpus (`doctrine export lazyspec`). All four phases landed on
`dispatch/026`; a post-implementation live smoke over the real corpus surfaced the
one substantive divergence — **INV-7**: specs are the only kind with no authored date
on disk, so the projection emitted `date: ""` for all 34, a hard break against
lazyspec's mandatory `%Y-%m-%d` `DocMeta.date`. The clean, always-dated in-memory
fixtures hid the class. A consult (2026-06-19) settled the source as the spec toml's
filesystem **mtime**; the fix (`5de8e723`) injects it in the impure `load_spec` shell
via `clock::date_of_system_time`, leaving `project()` pure. A regression test now
loads a **real dateless scaffold** (not a dated fixture) so the suite catches the
class — the surface-parity intent the fixtures had undercut.

The audit reviewed the **candidate interaction branch** `candidate/026/review-001`
(`815d0804`) — the impl-bundle 3-way-merged onto current `main` (29 commits ahead of
the slice's authoring base). That merge was conflict-free and the merged surface is
green: 1913 tests, clippy clean, INV-7 0/433 over the live corpus. Five findings
raised, all disposed `aligned` and verified; no blockers.

**Standing risks (consciously accepted).**
- **mtime is checkout-unstable across clones** — honest last-changed for a read-only
  viewer, weak as provenance. The accepted lossy-v1 tradeoff (design §5.3); the
  durable fix (authored `created`/`updated` on the spec schema) is **IMP-108**, out
  of this read-only slice's scope.
- **lazyspec wire strings are version-fragile** vs upstream lazyspec source — pinned
  by golden + conformance; the fix introduced no drift.
- **`lint-js` cannot run** in ephemeral dispatch worktrees (no `node_modules`); the
  slice touches zero JS, so the Rust gate is the operative proof here.

**Lifecycle note.** Phase-04 tracking and the slice status had drifted in disposable
runtime state (per-worktree); reconciled to 4/4 and advanced `plan → audit` before
this ledger. The four phase commits are the authoritative record.

## Reconciliation Brief

Most reconciliation was applied **during the consult** (the user authorised handling
the emergent finding inline), so this brief is near-no-op — it records what already
landed plus the one follow-up. `/reconcile` should confirm, not re-edit.

### Per-slice (direct edit) — ALREADY APPLIED (commit `e00ebf32`)
- **design.md §5.3** — added the "Spec node `date`" paragraph: mtime source, the
  checkout-instability tradeoff, IMP-108 pointer. Matches `lazyspec.rs::spec_date`.
- **design.md INV-7** — amended: specs sourced from toml mtime, never empty; plan
  from owning-slice `updated`; all else from `created`.
- **slice-026.md / design Follow-Ups** — cite IMP-108.
- Confirm these read true against the candidate; no further per-slice edit expected.

### Governance/spec (REV)
- **None.** No ADR or REQ requires modification — the pure/impure split (ADR), layering
  (ADR-001), relation seam (SL-048), and read APIs (SL-025) were honoured, not changed.

### Follow-up work (already captured)
- **IMP-108** — authored `created`/`updated` on the spec schema (the durable fix).
  Filed during the consult; once landed, `spec_date`'s mtime fallback can be removed.

## Reconciliation Outcome

The brief's per-slice edits were applied **during the 2026-06-19 consult** (the user
authorised handling the emergent finding inline), so this reconcile pass is a
**confirmation, not a re-write**. Inspected each target (D9 — no new discovery) and
validated it reads true against the admitted candidate `candidate/026/review-001`.

### Direct edits applied (validated, already landed — commit `e00ebf32`)
- **design.md §5.3** — "Spec node `date`" paragraph (mtime source + checkout-instability
  tradeoff + IMP-108 pointer). Confirmed matches `lazyspec.rs::spec_date` (created →
  else toml mtime). (RV-099 F-1, F-2)
- **design.md INV-7** — "never empty … specs from toml mtime". Confirmed. (F-1)
- **slice-026.md Follow-Ups** — cites IMP-108. Confirmed.

### REVs completed
- **None.** No governance/spec change required — no ADR or REQ was modified by this
  slice; the relevant ADRs (pure/impure split, ADR-001 layering, SL-048 relation seam,
  SL-025 read APIs) were honoured as-is.

### Follow-up / tolerated
- **IMP-108** — durable spec-schema date fix, filed during the consult.
- **Tolerated standing risk:** mtime checkout-instability — accepted lossy-v1
  read-only tradeoff, recorded in design §5.3 and the synthesis; no write needed.

Reconcile pass complete — handoff to /close.
