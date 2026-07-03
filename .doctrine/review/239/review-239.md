# Review RV-239 — reconciliation of SL-193

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance (post-implementation audit). **Self-audit** — one agent
drives both roles via `--as`.

**Surface reviewed.** Dispatched slice. No `dispatch candidate` branch was minted;
the reviewed surface is the **`review/193` impl bundle** (tip `fadaef5e`, the
single commit that lands at `/close`), materialised into a detached worktree for
independent build/test. `review/*` + `phase/*` are immutable evidence refs (R2).
Note: `edge` (primary tree) does **not** yet carry the bundle — it is behind by
the impl bundle, which includes both the code and two authored corrections
(design F5 amendment, e2e selector). All land together at `/close`.

**Lines of attack.**
1. **Mechanical conformance** — `slice conformance` algebra: undeclared /
   undelivered / conformant against the `design-target` selectors.
2. **D4 behaviour-preservation (load-bearing)** — REV-019's whole thesis is that
   the engine (`src/hymns.rs`) is untouched and the fix lives entirely in the
   projector. Hold the slice to: `src/hymns.rs` production diff empty (test-only),
   full resolver/loader/e2e suites green **unchanged**, no frozen golden edited.
3. **F1 compile gate** — both `#[expect(dead_code)]` attrs (the `project_starters`
   fn and the `HymnsSection.expose` field) removed; `warnings = "deny"` ⇒ an
   unfulfilled lint expectation is a hard error, so a green clippy gate is the proof.
4. **F5 idempotence** — forward-step 4 must render its sidecars onto disk *before*
   the step-3 agent-render loop consumes the disk hymn corpus, or single-pass
   install doubles (the ISS-206 regression). Verify placement + design capture.
5. **Verb-semantics claim** — design asserts `prompt explain` demonstrates
   single-emit/suppression. Explain vs resolve: which verb applies the `replaces`
   graph? (The e2e implementer flagged a gap in-comment.)
6. **Target behaviour** — all 5 exposed slots single-emit corpus-wide (not just
   `role/worker`); projected self-`replaces` sidecars legal (INV-3, no
   NonTopReplacer/cycle); write-if-absent per-file (D2); no magic string (D3/STD-001).

**Invariants held:** INV-2 (only `replaces` suppresses), INV-3 (`replaces` legal
only on unique-most-specific active snippet), D4 (engine unchanged), POL-002
(host-agnostic sidecar), STD-001 (single-sourced off `Slot::path`).

**Independent verification (detached worktree on `review/193`):**
- project_starters units: **7/7 green** (VT-1..7).
- expose resolver goldens (`src/hymns.rs`, tests-only): **2/2 green**.
- e2e (`prompt check` legality + 5-slot single-emit over built binary): **2/2 green** → **VT shape gate 11/11**.
- full bin suite **2993 pass / 0 fail**; e2e_prompt suite **9 pass / 0 fail** (behaviour-preservation, unchanged).
- clippy `--bin doctrine` clean; zero `#[expect(dead_code)]` on the two sites (F1 gate green).
- `src/hymns.rs` diff = one hunk, entirely inside `mod tests` (D4 confirmed).
- Empirical verb check: `prompt explain --role worker` prints **both** `role/worker`
  Framework(rank=1) **and** User(rank=2 ★ WINNER) — raw ranked set, no suppression;
  `prompt resolve --role worker` emits the user body **once**. Suppression is a
  resolve-time operation; explain is a pre-suppression diagnostic.

## Synthesis

**Closure story.** SL-193 delivers REV-019's locked design faithfully: expose
becomes the single-emit mirror of seal, delivered entirely in the projector
(`src/install.rs`) with the engine (`src/hymns.rs`) untouched. Independent
build + test on the `review/193` bundle confirms the whole verification basis —
11/11 VT shape gate, full suites green unchanged (2993 + 9), clippy clean, both
`#[expect(dead_code)]` attrs removed under `warnings = "deny"`. The mechanism is
sound: `project_starters` writes per-file write-if-absent (`.md` preserved,
sidecar backfilled), single-sourced off `slot.path()`, sealed-slot guarded, with
`create_dir_all` ahead of the no-mkdir `write_atomic` (F2). Forward-step 4 renders
early — before the step-3 agent-render loop consumes the disk corpus — so a single
`install` pass is idempotent (the F5 correction, surfaced at execution, captured in
design.md's adversarial-review entry). The 5 orphan sidecars are backfilled by
running the wired producer (D5), not by hand.

**Findings (3, all minor, all terminal).** One genuine defect: the design PROSE
names `prompt explain` as the verb that demonstrates single-emit/suppression
(F-1). It does not — `explain` prints the raw ranked active set (both twins,
provenance-ordered); only `prompt resolve` applies the `replaces` graph. The
implementation is correct (verifies over `resolve`, with an in-test comment
flagging the gap); the design artifact lies. Delegated to /reconcile as a
per-slice design.md direct edit. F-2 (conformance `undeclared`) is a benign
integration artifact — the declaring selector lives on the impl bundle, not edge;
it resolves when the bundle lands. F-3 records the D4/F1/F5 gate confirmations.

**Standing risks / consciously accepted.** (a) The **hand-authored-no-sidecar**
general case still doubles — REV-019 known gap, explicit non-goal, follow-up only
on demand. (b) **Sidecar repair**: a user who deletes the `replaces` line from a
projected `.toml` gets a permanently doubling slot the write-if-absent projector
won't repair — accepted flip side of rejecting always-clobber (D2/iii); `prompt
check` surfaces it. (c) **Maintenance invariant**: a future embedded hymn that is
neither sealed nor exposed would be copied by base install as a doubling twin the
projector never sidecars — not a current gap (seal ∪ expose covers the corpus),
but a boundary to hold.

**Integration-topology note (RFC-011).** Two authored corrections (F5 design
amendment, e2e selector) rode into the impl bundle via mid-drive base-drift
re-forks rather than landing directly on edge. Net effect is correct — everything
lands together at /close — but it entangles authored governance state with the
code bundle and is the mechanical reason conformance-against-edge reads red. The
carry-forward (F-2) makes the /close land-the-whole-bundle requirement explicit.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md — F-1 verb correction (3 loci).** Replace `prompt explain` with
  `prompt resolve` wherever the doc claims it demonstrates single-emit /
  suppression / no-doubling: §Target behaviour ("Verified corpus-wide: `prompt
  explain` shows no exposed slot doubling", ~L92), §Verification/E2E ("`prompt
  explain` shows framework `role/worker` suppressed, not rank-ordered-but-present",
  ~L164-167), §Corpus check ("in-repo, `prompt explain` … shows no exposed slot
  double-emitting", ~L180-181). Reframe `prompt explain` as the pre-suppression
  diagnostic (prints the raw ranked active set; the `replaces` graph is applied at
  resolve). `prompt resolve` (and `prompt check` for legality) are the verbs that
  demonstrate the fix.

### Governance/spec (REV)
- None. No ADR/SPEC/REQ divergence surfaced; REV-019's design is delivered as
  locked, engine untouched.

### Off-surface (note only — NOT a reconcile write target)
- **plan.toml PHASE-02 EX-2** carries the same `prompt explain` wording as F-1.
  `plan.toml` criteria are immutable-append (boot rule) — not a reconcile
  direct-edit surface. Recorded as a design/plan-accuracy note; do not edit the
  criterion.

### Carry-forward to /close (NOT reconcile)
- Land the `review/193` bundle's **authored** files (design.md F5 amendment,
  slice-193.toml e2e selector) onto edge/main together with the code — not the
  code alone — or edge's design/selector stay stale and conformance stays red
  (F-2). The e2e selector is already correctly authored on the bundle; no re-add
  needed, only the landing.

## Reconciliation Outcome

### Direct edits applied
- **design.md — F-1 verb correction (3 loci), on `edge`** (drives RV-239 F-1):
  §Target behaviour (~L92), §Verification/E2E (~L165-167), §Corpus check
  (~L180-181). `prompt explain` → `prompt resolve` wherever the doc claimed the
  verb demonstrates single-emit / suppression / no-doubling; `prompt explain`
  reframed as the **pre-suppression diagnostic** (raw ranked active set, both
  twins present) and `prompt check` cited for legality. The Problem-statement use
  (~L10-12) was **left unchanged** — it correctly uses `explain` to show both
  twins *surviving* (the doubling), which is exactly what a pre-suppression
  diagnostic shows; it is not one of the three claim-loci.

### REVs completed
- **None.** No governance/spec divergence surfaced (brief §Governance/spec: none).
  REV-019's design is delivered as locked, engine untouched — no ADR/SPEC/REQ
  edit owed.

### Withdrawn / tolerated
- **RV-239 F-2** (`aligned`): benign integration artifact, no reconcile write. The
  conformance `undeclared` is an edge-lags-bundle effect; the declaring selector
  already exists on the `review/193` bundle. Resolves when the bundle lands —
  carry-forward to /close (below), not a reconcile surface.
- **RV-239 F-3** (`aligned`): positive gate confirmation (D4 engine-untouched,
  F1 compile gate, F5 idempotence, D3/STD-001, POL-002). No remediation.

### Off-surface note (NOT edited)
- **plan.toml PHASE-02 EX-2** carries the same `prompt explain` wording as F-1.
  `plan.toml` criteria are **immutable-append** (boot rule) — not a reconcile
  direct-edit surface. Left as-authored; recorded as a design/plan-accuracy note
  only. If a corrected criterion is wanted, it is an append via /plan, not an
  edit here.

### Carry-forward to /close
1. **Land the bundle's genuine delta only — cherry-pick, NOT whole-tree.** The
   `review/193` bundle (`fadaef5e`) forked from an **old** merge-base
   (`1b5e3b4a`); `edge` (`63d5c576`) has advanced far past it. `git diff edge
   review/193` therefore shows the bundle *reverting* a mass of later edge work
   (this RV-239, REV-020, SL-191 progress, memories, backlog transitions). A
   whole-tree integration would destroy that. /close must land **only** the
   bundle's real additions:
   - code: `src/hymns.rs` (test-only prod-empty), `src/install.rs`,
     `tests/e2e_prompt_resolve_golden.rs`
   - 5 sidecars: `.doctrine/hymns/{harness/claude, model/anthropic/claude-sonnet-4,
     model/deepseek/_default, role/orchestrator, role/worker}.toml`
   - authored corrections: `design.md` **F5 block** (+15 lines, added at ~L254,
     before ## Open questions) and `slice-193.toml` **e2e selector** + status.
2. **F-1 edits (this pass, on edge) are disjoint from the bundle's design.md F5
   hunk.** F-1 loci are ~L92/L165/L180; the F5 block lands at ~L254. Apply the
   bundle's design.md change as the **+15 F5 hunk** (patch/cherry-pick of that
   region), **never a blind whole-file overwrite** of design.md — an overwrite
   would clobber the F-1 correction and re-stale the file. Disjoint regions ⇒
   clean apply.
3. On `slice-193.toml`: the bundle flips status `reconcile` → `started` (stale
   base). /close owns the terminal status transition — take the bundle's
   **selector** addition, not its status value.

Reconcile pass complete — every finding terminal, one direct-edit item applied,
zero REV surface. Handoff to /close.
