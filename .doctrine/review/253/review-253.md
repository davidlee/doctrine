# Review RV-253 — reconciliation of SL-203

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed:** solo (non-dispatch) — the two SL-203 commits on `edge`:
`30396eaf` (code + ratchet) and `bd74dfa5` (canon corrections + RSK-228 +
memory). Working tree carries only the `slice-203.toml` status flip to `audit`
and foreign untouched files.

**Lines of attack** — the invariants this audit holds SL-203 to:

1. **INV-1 (EX-1/VA-2)** — `mcp_server` production code imports zero
   `crate::commands` / `crate::install`. The whole point of the slice.
2. **INV-2 (EX-2/VA-1)** — `doctrine_onboard` output byte-identical; pure
   re-wiring, no behaviour change.
3. **Tangle ratchet (EX-3/VT-1)** — `[tangle_baseline] command` monotone-lowered
   to the *measured* value; gate green. Watch for the inherited-premise trap: the
   plan's 123→121 was falsified at execute-time (F-EXEC-1).
4. **Wiring guard (EX-4/VT-2)** — the injection is actually exercised (a known
   key renders; the empty producer fails), not a hollow section-presence check.
5. **Conformance** — the mechanical undeclared/undelivered delta: is any code
   path touched outside the design-target, and is any declared surface silently
   dropped?
6. **Non-goals respected** — forward edge untouched, `model_keys` not moved, no
   leaf module, no fan-out gate.

## Synthesis

**Closure story.** SL-203 landed exactly the change it scoped: the incidental
back edge `mcp_server::tools → commands::prompt::model_keys` is severed by
dependency inversion — a `ModelKeysFn` fn-pointer threaded through `McpConfig`
and bound once at serve-start. `mcp_server` production code is now corpus-agnostic
(F-1: all residual `crate::commands` references sit under `#[cfg(test)]` from
line 1401, passing the real `model_keys` as the injected pointer — exactly the
posture EX-1 permits). The decision (D-B injection, not extraction) is unchanged
and validated: `model_keys` stayed in `commands::prompt`, nothing moved, no new
same-tier edge was introduced.

Every acceptance criterion is met on live evidence: `command = 86` in
`layering.toml` (F-2), `doctrine check gate` exit 0, full suite green (F-4,
byte-identity holds by construction — the render body is unchanged but for the
call indirection), and the `onboard_wiring` guard has real teeth (F-3: an
injected known key renders as a bullet; the empty producer, which emits the
placeholder, fails the assertion — the F-1 inquisition penance).

**The one substantive divergence was already reconciled at execute-time.** The
VT-1 empirical measurement (deferred to execute-time by design F-4) falsified two
premises the design inherited from RSK-227: the `123` baseline was stale (live
pre-change count was 90), and severing the edge dropped the tangle by **−4**, not
the −2 a clean core-separate 2-cycle predicts — `mcp_server` was fused *into* the
23-node core SCC. This was caught mid-flight, routed through `/consult`, and
written into canon before the audit ran: design §1 carries the F-EXEC-1
superseding note, `mem.pattern.lint.mcp-server-entangled-with-core` records the
corrected topology model, and RSK-228 flags SL-204 (which inherits RSK-227's
now-suspect separable-SCC map) to re-derive its decomposition premise. So the
audit's job here is confirmation, not repair: **no spec/governance write remains
outstanding.**

**Standing risks / accepted tradeoffs.** Conformance reports one `undelivered`
selector — `tests/architecture_layering.rs` — and 21 `undeclared` paths. Both are
benign and consciously accepted (F-5, F-6): the undelivered gate-test file was
correctly scoped as the verification surface but needed no edit (the −4 ratchet
lives entirely in `layering.toml`'s data), and the 21 undeclared paths are all
authored `.doctrine/` artifacts (the memory harvest, RSK-228, and the canon
corrections) that sit outside the code-only design-target by construction — not
scope creep. The retained undelivered selector preserves the traceability that
the gate surface *was* scoped and verified; trimming it would misrepresent design
intent for a cosmetic green.

The wider caution SL-203 leaves behind is epistemic, not code: a static coupling
snapshot (RSK-227) mis-attributed one module's SCC membership. RSK-228 carries
that forward so SL-204 does not plan its larger integrity refactor on an unverified
map.

## Reconciliation Brief

**No reconciliation writes required.** The single design/canon divergence
(F-EXEC-1: stale baseline + wrong SCC topology) was reconciled into canon at
execute-time via `/consult`, ahead of this audit:

- **design.md §1** — F-EXEC-1 superseding note supersedes every
  `123`/`121`/`−2`/`core-separate`/`2-cycle` claim below it. *Already written*
  (commit `bd74dfa5`).
- **`mem.pattern.lint.mcp-server-entangled-with-core`** — records the corrected
  topology model. *Already recorded.*
- **RSK-228** — flags SL-204's inherited RSK-227 premise for re-derivation.
  *Already open.*

### Per-slice (direct edit)
- None. `design.md` already tells the truth (F-EXEC-1 note); `slice-203.toml`
  status advances to `reconcile` via the lifecycle verb, not a brief edit.

### Governance/spec (REV)
- None. No ADR/spec/policy/standard change is implied — ADR-001's layering model
  is unchanged; the ratchet is a data update the code commit already carried.

`/reconcile` should confirm the empty brief, advance status, and hand to
`/close`. All six findings are `verified` (terminal); no `blocker`; the
close-gate is clear.
