# Review RV-235 — reconciliation of SL-187

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Reviewed surface (F-2).** Candidate interaction branch `candidate/187/review-001`
(tip `a3e84ba8`, worktree `cand-187-review-001`), published by `dispatch candidate
create` off `refs/heads/main` after `refresh-base` merged 84 commits of trunk drift
into `dispatch/187`. This is the audit surface per the dispatched-slice rule — not
the raw `dispatch/187` evidence ref. Impl bundle = `review/187` (`0c2caada`) over
current trunk.

**Lines of attack.**
1. **Behaviour-preservation gate** (the design's named gate): the only intended disk
   delta is one additive universal-hymns section; existing boot + dispatch + onboard
   suites must stay green after the 84-commit trunk merge. Probe: full `just gate`.
2. **INV-D1 axis-invariance** — `prompt resolve` disk artifact identical across
   `--role`/`--harness` (PHASE-04 VT-2); model band demonstrably absent from `boot.md`
   (PHASE-01 VT-1).
3. **Onboarding inline** (PHASE-02) — shipped ∪ local `onboarding`-tagged bodies
   inline, local-wins, deterministic order, footer round-trip retired; no model content.
4. **doctrine_onboard contract swap** (PHASE-03) — drops `Onboarding Memories`, emits
   model-band self-ID.
5. **Per-harness wiring** (PHASE-04) — pi extension + Claude/Codex SessionStart exec
   `prompt resolve --role orchestrator`, not `boot --emit`.
6. **Path-conformance** — `slice conformance` undeclared/undelivered algebra, read on
   the candidate authored surface (current selectors incl. the a299cf27 PHASE-04
   declaration), cross-checked against the edge mirror to separate real drift from the
   un-integrated-selector split-brain.

**Evidence gathered.**
- `just gate` on the candidate: **3703 passed / 0 failed** (behaviour gate satisfied
  post-merge).
- `slice verify-vt 187`: **6/6 VT criteria PASS** the existence/shape gate (all four
  phases).
- `slice conformance 187`: **0 undelivered, 6 conformant, 5 undeclared** (dispositioned
  as findings below).

## Synthesis

**Closure story.** SL-187 delivers the SL-186 resolver world to live agents. All four
phases landed on `dispatch/187` with genuine delivery commits whose end-oids match the
source-delta registry (P01→`574afd7f`, P02→`ae23771c`, P03→`757b71db`, P04→`12f8f87e`).
The audit ran against the candidate interaction branch after `refresh-base` folded 84
commits of trunk drift into the dispatch branch (single conflict, in the RFC-011
instrumentation file, union-resolved — no code conflict). Post-merge the full gate is
**green (3703/0)**, so the design's named **behaviour-preservation gate** holds across
the merge: existing boot + dispatch + onboard suites survive the additive
universal-hymns section unchanged, and the six VT criteria all pass the existence gate.

**Conformance algebra.** Zero undelivered (every design-target selector matched real
edits). Six conformant. Of five undeclared paths, three are non-findings —
`.doctrine/rfc/011/case-notes.md` is RFC-011 instrumentation (excluded by nature) and
`.doctrine/memory/items/mem_019ef1ae…` is the PHASE-02 `onboarding` tag on the shipped
memories (EX-1, authored/intended). The two remaining code paths (`cli.rs`,
`e2e_prompt_resolve_golden.rs`) are the **un-integrated-selector split-brain** (F-2):
declared on the delivery, invisible on the edge mirror until integrate. The one genuine
gap is **F-1** — `install/model-band.md`, a delivered/wired/tested asset with no
selector.

**Standing risks / tradeoffs consciously accepted.**
- The runtime phase-completion flags (`phase-NN.toml`) were `planned` on entry — the
  disposable runtime state did not survive the session clear, so `prepare-review` and
  `conformance` refused until the flags were restored from the verified registry+commit
  evidence. A recurring dispatched-audit-after-clear tax (harvested below), not a
  delivery defect.
- No true on-model-change ceiling for the model band (design Non-Goal, deferred). The
  floor directive in `model-band.md` is explicitly best-effort — "no correctness
  invariant depends on it" — so its degradation mode is benign.
- Neither finding is a blocker; the close-gate is satisfiable.

## Reconciliation Brief

### Per-slice (direct edit)
- **slice-187.toml selectors (F-1)** — add a design-target selector for the delivered
  asset: `doctrine slice selector add 187 "install/model-band.md" --intent design-target`
  (note: "PHASE-01 model-band floor asset — Static embed wired in boot.rs"). Clears the
  sole genuine `undeclared` and makes conformance fully green.

### Governance/spec (REV)
- None. No design, spec, ADR, policy, or standard drift surfaced; the implementation
  matches design intent across all four phases.

### Resolved by integration (no write)
- **F-2** — the `cli.rs` + `e2e_prompt_resolve_golden.rs` design-target selectors
  (a299cf27) land on trunk when `dispatch sync --integrate` projects slice-187.toml.
  No reconcile action; noted so integrate is not misread as introducing scope creep.

## Reconciliation Outcome

### Direct edits applied
- **slice-187.toml (RV-235 F-1)** — added design-target selector `install/model-band.md`
  (`slice selector add 187 … --intent design-target`). Conformance on edge now: **0
  undelivered, 7 conformant, 4 undeclared** — the 4 remaining are all dispositioned:
  the two `onboarding`-tag/instrumentation authored paths (aligned) and the two F-2
  split-brain code paths (`cli.rs`, `e2e_prompt_resolve_golden.rs`) whose selectors
  arrive with the impl bundle at integrate.

### REVs completed
- None. No governance/spec drift surfaced; brief carried no REV items.

### Withdrawn / tolerated
- None. F-1 verified + remediated (selector added); F-2 verified + aligned (resolves at
  integrate, no write).

Reconcile pass complete — handoff to /close. The code has NOT yet landed on trunk;
close performs the candidate close_target + admit + `sync --integrate --trunk main`.
