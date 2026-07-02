# Review RV-232 — reconciliation of SL-181

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation-facet audit (conformance mode) of SL-181, a **REV-only** slice.
Reviewed surface: the **in-tree** main branch (`edge`) — not dispatched, no
candidate branch. Sole deliverable: REV-018 (author + approve), landing at
reconcile against ADR-012 + ADR-006. No `src/` code (design §3).

Lines of attack — the invariants this audit pins the slice to:

1. **Deliverable-vs-design coherence.** Does REV-018's payload + rationale carry
   *exactly* design §2 (a)–(d) — retract the overclaim, record shipped confinement
   (SL-182/183/185), reclassify the RSK-014 residual, record the guard disposition
   — and do its two `modify` rows target the right entities (ADR-012 headline,
   ADR-006 D2a/D2b)? Invariant: the REV is a faithful, complete encoding of the
   locked design, no more and no less.
2. **Zero-code conformance.** Design §3 asserts a zero `src/` touch-set. Does the
   conformance algebra confirm no `src/` edit? What does it report for the REV
   entity files, and is that a drift finding or the sanctioned deliverable?
3. **Write-surface discipline (D3/D4).** Is REV-018 **approved-but-unapplied** —
   i.e. did execute correctly stop short of the governance write, leaving apply +
   the IMP-065 close + RSK-014 downgrade to reconcile? Invariant: no owner-locked
   ADR prose was hand-edited under this slice.
4. **The load-bearing claim.** The REV rests on "where a confinement backend runs,
   worker ref-mutation is blocked at the OS floor (ro shared `.git`)" — SL-182/183/
   185. Is that claim sound and are those slices actually `done`? (Re-confirmed at
   plan time; re-pinned here.)

## Synthesis

**Verdict: reconcile-ready, no blockers.** SL-181 did exactly what a REV-only slice
should — it produced one faithful governance artefact and stopped at the write
boundary. Three findings, all terminal-verified.

The **coherence** line (F-5) holds cleanly: REV-018's two `modify` rows target
ADR-012 (headline) and ADR-006, and its rationale encodes design §2 (a)–(d) with the
before/after excerpts reconcile will apply mechanically. The **load-bearing claim** —
confinement blocks worker ref-mutation at the ro-`.git` floor — was re-pinned against
live state: SL-182/183/185 all `done`. The reframe rests on shipped enforcement, not
a promise.

The **conformance** signal (F-4) is a model-vs-artefact mismatch, not drift: a
code-delta conformance algebra has no `design-target` selector for authored
governance output, so a REV-only slice's deliverable *necessarily* reads as
undeclared. The real code-impact gate — `git diff --stat -- src/` — is empty; design
§3's zero touch-set is honoured. The behaviour gate is N/A by design (§4:
conformance + coherence, not behaviour); `check quick` green (12/12) only confirms
the pre-existing tree is unbroken.

**Standing risk (accepted, not closed here):** the whole slice's value is the
*honesty* of the reframe, and honesty is human-judged — VH-1 was accepted at execute
before approval. The residual OQ-D gap is now enforcement-closed on confined arms and
bounded/self-limiting elsewhere (RSK-014, to be downgraded at reconcile); no
cooperative-marker residual remains because no marker is claimed. This is a genuine
narrowing, but it is a *governance-prose* claim — its proof is the confinement slices,
audited under their own ledgers, not re-litigated here.

The one substantive item (F-6) is not a defect but a **handoff**: the governance
truth on disk is unchanged because apply is reconcile's job (design D3/D4). That is
the reconciliation brief.

## Reconciliation Brief

Handoff to `/reconcile`. Every item below is the *pending effect* of the
already-approved REV-018; none is a per-slice direct edit.

### Per-slice (direct edit)
- None. `design.md`, `slice-181.md`, `plan.*` all match the implemented outcome
  (REV-only, guard retired). No prose to reconcile against the code — there is no
  code.

### Governance/spec (REV) — apply REV-018 (F-6)
REV-018 is approved + unapplied. `revision apply REV-018` surfaces its two `modify`
rows for manual hand-edit (spec/ADR prose is surfaced-for-manual, not auto-applied):

- **ADR-012** (modify, headline) — apply edits 1.1–1.4 from `revision-018.md`:
  retract "the real close is the positive marker (IMP-065)" → shipped confinement
  (SL-182/183/185); append the OQ-D-close note to *Decisions formerly open*; fix the
  OQ-D plan-gate note and the References line.
- **ADR-006** (modify) — apply additive notes 2.1–2.3: D2a "enforcement floor has
  landed" note; D2b "residual enforcement-closed on confined arms" note; References
  line (IMP-065 retired; OQ-D close = confinement).

### Governance effects of the REV landing (reconcile records these)
- **IMP-065** — close as **reframed/superseded** by confinement (design §2, D4). The
  positive-marker proposal is retired, not delivered.
- **RSK-014** — downgrade to "enforcement-closed on arms running a confinement
  backend; bounded, self-limiting residual (no-backend fail-closed; standalone clone
  contained)" (design §2c, D4).

### Not in scope for reconcile
- No `src/` change, no test, no memory correction mandated by findings. Durable
  harvest (below) is judgment-gated, not a reconcile obligation.

## Reconciliation Outcome

Reconcile pass complete 2026-07-03 (VH-1 accepted before apply).

### Ledger hygiene (pre-reconcile gate)
- RV-232 carried three non-terminal findings the audit left open: **F-1**
  (`check gate` red — cordage denylist host-path root-resolution panic, independent
  of REV-only SL-181; tracked ISS-205), **F-2** (dup of verified F-4), **F-3** (dup
  of verified F-6). All three **withdrawn** → every finding now terminal.

### Direct edits applied
- None. `design.md`, `slice-181.md`, `plan.*` already matched the implemented
  outcome (REV-only, guard retired). No code, no prose to reconcile against code.

### REVs completed
- **REV-018** (`oq-d-close-is-shipped-confinement-not-the-positive-marker`): `done`.
  Applied → 2 surfaced `modify` rows hand-landed: **ADR-012** edits 1.1–1.4,
  **ADR-006** D2a/D2b additive notes 2.1–2.3. One draft id typo ("2056" → "SL-056")
  corrected under the honour model. Covers RV-232 F-6 (governance truth now written).
  Narrative in `revision-018.md`.

### Governance effects (design D4)
- **IMP-065** → `closed · obsolete` — positive-marker approach superseded by shipped
  confinement (retired, not delivered).
- **RSK-014** → `resolved · mitigated` — downgraded from "unsolved / cooperative-only
  off the bwrap arm" to enforcement-closed on every confined arm (SL-182/183/185);
  bounded, self-limiting residual (no-backend fail-closed; standalone clone
  contained).

### Withdrawn / tolerated
- F-1, F-2, F-3 withdrawn (see Ledger hygiene). F-4/F-5/F-6 verified at audit.

Reconcile pass complete — handoff to /close.
