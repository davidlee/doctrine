# Review RV-052 — design of SL-080

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This Inquisition arraigns the **design** and **plan** of SL-080 — the reconcile
skill + audit/reconcile seam disentanglement. The charges press five lines:

1. **Routing hygiene under ADR-009 F14** — does the plan's PHASE-04 ordering
   create a shipped-not-reachable window where routing points at a skill before
   install confirms it?
2. **Disposition vocabulary gap** — when D5 strips `design-wrong` from audit,
   what terminal disposition conveys "finding correct, write delegated to
   reconcile"?
3. **Stale-prose remnants** — does PHASE-03's retune explicitly remove the old
   "design was wrong" pre-check from close SKILL.md, or does it risk leaving both
   models in tension?
4. **Routing chain coherence** — D7 adds the `/reconcile` row but does it update
   the existing audit→close row to audit→reconcile→close?
5. **Trigger precision** — is "audit complete" in the routing row precise enough,
   or does it invite misinterpretation of lifecycle state vs handoff artefact?

Governing canon: ADR-003 §7 (hard audit/reconcile edge), §8 (loop order);
ADR-009 F2/F14 (shipped-not-reachable), F12 (closure-seam topology);
review-ledger.md §4 (disposition vocab).

## Synthesis

> *Inquisitio peracta est. Quinque crimina detecta, quinque condemnata.*

Five heresies were uncovered in the design and plan of SL-080. None rise to
**blocker** — the architecture is sound, the seam is correctly drawn, and the
four-phase plan is coherent in its major strokes. But five taints mar the
parchment, and each demands penance before the slice may proceed with a clean
conscience.

### The charges and their sentence

**F-1 (major) — The Shipped-Not-Reachable Heresy.** The plan's PHASE-04 writes
the routing row into `install/routing-process.md` *before* `doctrine claude
install` confirms the skill is embedded. Should install fail, the boot snapshot
would route agents to a skill that does not answer — precisely the sin ADR-009
F14 forbids. **Penance:** reverse the intra-phase order: install first, verify,
then add the routing row and regenerate boot. Or harden EN-1 to gate on install
success, not mere file existence.

**F-2 (major) — The Dispositional Void.** Design D5 commands audit to cease
using the `design-wrong` disposition, but offers no replacement. The
review-ledger's five-term vocabulary has no slot for "finding confirmed, write
delegated to reconcile." Without explicit convention, the PHASE-02 implementer
faces a void and may leave audit mute before the reconciliation brief.
**Penance:** inscribe into D5 or PHASE-02 EX criteria the explicit convention:
audit raises the finding, disposes it `verified` (terminal — observation
confirmed), records the exact change in the reconciliation brief. The audit
SKILL.md retune must name its permitted dispositions: `aligned`, `fix-now` (code
only, within audit scope), `tolerated`, and the brief pathway. `design-wrong` is
anathema.

**F-3 (minor) — The Unclean Pre-Check.** Close SKILL.md line 17 still intones
`"design was wrong" findings already reconciled into design.md` — the very
phrase ADR-003 §7 amendment brands over-reach. PHASE-03's exit criteria
enumerate the new spec-coherence gate but never order the old pre-check
expunged. A narrow implementer might leave both, a palimpsest of old and new
models in unholy tension. **Penance:** add an EX criterion that explicitly
requires removal or rewriting of this pre-check to reference the reconciliation
brief and REV-done gate.

**F-4 (minor) — The Broken Chain.** The existing routing row reads `/audit →
/close`. D7 adds a standalone `/reconcile` row but leaves the old chain
uncorrected — audit still promises a direct path to close that no longer exists.
The boot snapshot would proclaim two contradictory truths. **Penance:** update
the existing audit row to `/audit → /reconcile → /close`.

**F-5 (nit) — The Imprecise Trigger.** "Audit complete" in the routing condition
whispers of lifecycle state, not the handoff artefact that genuinely gates
reconcile. An agent might check `doctrine slice show` rather than the RV ledger.
**Penance:** reword to "Slice exists, audit RV resolved, reconciliation brief
written."

### Standing risks

- **No REV→slice relation edge exists** (OQ-1). The collision guard in D4 step 4
  is a heuristic, not a query. A slug collision on `reconcile-sl-NNN` would
  require manual inspection. This is a **known gap**, not a design defect, and
  the design correctly flags it.
- **Self-audit + self-reconcile.** When the same agent drives both audit and
  reconcile (the usual case), the hard edge between identification and writing
  is maintained by discipline alone. The RV ledger's role assertion (`--as`) is
  cooperative, not a security boundary. No design flaw — but worth noting.

### Verdict

The design and plan of SL-080 are **doctrinally sound** — the audit/reconcile
seam is correctly drawn, the two write surfaces are well-chosen, and the four
phases map cleanly to the four scope items. The five heresies are **surface
taints**: ordering, vocabulary gaps, stale-prose hygiene, and routing precision.
Each has a concrete, low-cost penance. Applied, the design and plan are fit to
proceed to implementation.

> **HERESIS URITOR; DOCTRINA MANET**
