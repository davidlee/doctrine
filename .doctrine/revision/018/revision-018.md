# REV REV-018 — OQ-D close is shipped confinement, not the positive marker

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Makes the OQ-D framing honest. ADR-012 (RV-023 F-2) deferred the dispatch
impersonation gap (ADR-011 D6/M2) and named **IMP-065's positive coordination
marker** its "real close." Two findings collapse that claim (SL-181 design §0–§1;
RV-199, heresy confessed):

1. **A positive marker cannot close it.** Worker identity is a presence-only file +
   optional `DOCTRINE_WORKER` env — cooperative flags, not enforced boundaries
   (RSK-014; ADR-006 D2a already concedes "the fence … is the funnel + the jail,
   **not** a fail-closed CLI floor"). A capable worker `cd`s into the coordination
   tree or forges the marker trivially.
2. **The genuine close — confinement — has shipped.** SL-182 (Linux/bwrap claude
   arm, **ro shared `.git`**), SL-183 (macOS Seatbelt), SL-185 (subprocess parity).
   Where a backend runs, worker ref-mutation is blocked at the OS floor, stamped or
   not.

An interim anti-accident branch-check guard was designed under SL-181, inquisited,
and **retired**: RV-199 **F-1** (blocker) proved its predicate
(`is_coordination_worktree = is_linked && branch startswith "dispatch/"`) *false-
allows* the worker fork — which is minted on `dispatch/<name>` unconditionally — so
the mechanism is void on the one (unconfined) arm that needed it; everywhere else the
confinement floor or clone topology already closes the gap (design §1 table). **No
code ships** (design §3); the value is the honest reframe.

This REV therefore, per SL-181 design §2:

- **(a) Retracts** "the positive coordination marker (IMP-065) is the **real close**
  of OQ-D."
- **(b) Records the genuine close as shipped:** confinement — SL-182 / SL-183 /
  SL-185. The prior "closable, not-yet-landed" framing is superseded by **landed**
  enforcement.
- **(c) Reclassifies the residual (RSK-014):** OQ-D is **enforcement-closed on every
  arm that runs a confinement backend**. The bounded residual is only the degenerate
  cases, each self-limiting: no-backend → SL-182 hook fail-closed (worker runs
  nothing); standalone clone → corruption contained to a disposable object store. No
  cooperative-marker residual remains, because no marker is claimed.
- **(d) Records the guard disposition:** an anti-accident branch-check was designed,
  inquisited (RV-199 F-1, blocker), and retired — the confinement floor subsumed it.

Scope of the staged delta: two `modify` rows (ADR-012 headline; ADR-006 D2a/D2b
notes). Both targets are owner-locked (VH); this REV is the sanctioned amendment
path, applied as a **surfaced-for-manual** hand-edit at **reconcile** — not now, not
by hand-edit outside the REV. The IMP-065 close (reframed/superseded) and the
RSK-014 downgrade are the REV's *effects*, landed at reconcile (design D4).

### Change row 1 — modify ADR-012 (headline)

**Edit 1.1 — Consequences › Residual identity gap (line ~197).**

> before: `Bounded by the OQ-D plan-gate below; the real close is the positive marker (IMP-065).`
>
> after: `Bounded by the OQ-D plan-gate below. The real close is **shipped confinement** — SL-182 (Linux/bwrap, ro shared \`.git\`), SL-183 (macOS Seatbelt), SL-185 (subprocess parity) — which blocks worker ref-mutation at the OS floor, stamped or not. The positive marker (IMP-065) is **retired**, not the close (SL-181; RV-199 F-1).`

**Edit 1.2 — Decisions formerly open › OQ-D (lines ~215–222).** Append after the
existing block (the historical decision is preserved; the close is corrected):

> add: `**OQ-D close (SL-181, 2026-07-03).** The positive coordination marker (IMP-065) was found unable to close OQ-D — cooperative flags are not boundaries (RSK-014). The genuine close is **enforcement**: shipped confinement (SL-182/183/185) blocks worker ref-mutation at the OS floor. An interim anti-accident guard was designed, inquisited (RV-199 F-1, blocker — the coordination-signal predicate false-allows the worker fork), and **retired**. IMP-065 is superseded by confinement; residual is enforcement-closed on confined arms, bounded and self-limiting elsewhere (no-backend fail-closed; standalone clone contained).`

**Edit 1.3 — OQ-D plan-gate note (line ~259).**

> before: `… *not* as proof it catches \`gc\`/\`sync\` impersonation (that close is IMP-065).`
>
> after: `… *not* as proof it catches \`gc\`/\`sync\` impersonation (that close is **shipped confinement** — SL-182/183/185; IMP-065 retired, SL-181).`

**Edit 1.4 — References (line ~267).**

> before: `- Deferred redress: IMP-065 (positive coordination-tree marker, OQ-D).`
>
> after: `- OQ-D close: **shipped confinement** (SL-182/183/185). IMP-065 (positive coordination-tree marker) **retired** — SL-181, RV-199 F-1.`

### Change row 2 — modify ADR-006 (D2a/D2b notes)

Additive notes only; the historical D2a/D2b reasoning (owner-locked, VH) stands.

**Edit 2.1 — D2a (after line ~93, the `worker-mode-floor-decision.md` note).**

> add: `**D2a note (SL-181, 2026-07-03) — the enforcement floor has landed.** The 2056 re-weigh concluded "the marker is the positive signal; the funnel + jail + IMP-052 are the fence" *because no fail-closed CLI floor was justified*. Confinement now supplies the floor at the OS layer, not the CLI: SL-182 (Linux/bwrap, **ro shared \`.git\`**), SL-183 (macOS Seatbelt), SL-185 (subprocess parity). Where a backend runs, a worker — stamped or not — cannot mutate shared refs. The cooperative marker was never the fence; the OS floor now is.`

**Edit 2.2 — D2b (after the D2b note block, lines ~113–122).**

> add: `**D2b note (SL-181, 2026-07-03) — the residual gap is enforcement-closed on confined arms.** Raw-tree confinement, "deferred to sandbox/harness work" here, has **shipped** (SL-182/183/185). The marker-absence transitional assumption's blast radius is now bounded to the degenerate cases, each self-limiting: no-backend → SL-182 hook fail-closed (worker executes nothing); standalone clone → contained to a disposable object store. The SL-181 anti-accident guard for the unconfined arm was retired (RV-199 F-1); it added nothing the floor did not already give (SL-181 design §1).`

**Edit 2.3 — References (lines ~350–351).**

> before: `IMP-065 — the deferred positive coordination marker (OQ-D real close).`
>
> after: `IMP-065 — positive coordination marker, **retired** (SL-181); OQ-D real close = **shipped confinement** (SL-182/183/185).`

## Reconcile narrative (SL-181)

Applied at reconcile 2026-07-03 (RV-232 audit clean, VH-1 accepted). All rows
were `modify` (surfaced-for-manual); landed by hand-edit under the authored-truth
honour model:

- **ADR-012 (modify, primary)** — edits 1.1–1.4 landed: Consequences › Residual
  identity gap retracts "the real close is the positive marker" → shipped
  confinement; the OQ-D-close note appended to *Decisions formerly open* (historical
  decision preserved); the OQ-D plan-gate note and References line corrected.
- **ADR-006 (modify)** — additive notes 2.1–2.3 landed: D2a "enforcement floor has
  landed" note; D2b "residual enforcement-closed on confined arms" note; References
  line (IMP-065 retired; OQ-D close = confinement). Historical D2a/D2b reasoning
  (owner-locked, VH) stands untouched — the notes are additive.
- **Prose correction (recorded):** edit 2.1's draft read "The 2056 re-weigh"; landed
  as "The **SL-056** re-weigh" — an obvious id typo for the SL-056 PHASE-05 G2
  revert this very D2a block documents. Corrected under the honour model; substance
  unchanged.

Governance effects of the landing (design D4): **IMP-065** closed (obsolete —
superseded by confinement, not delivered); **RSK-014** resolved (mitigated —
downgraded from "unsolved" to enforcement-closed-on-confined-arms with a bounded,
self-limiting residual). No `src/` change (design §3).
