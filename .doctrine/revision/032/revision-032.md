# REV REV-032 — Zero-rescue dispatch funnel state machine and git read verbs

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

### Decision (bottom line)

> Dispatch funnel **position** becomes persisted, per-phase, authoritative state;
> every funnel verb is **legality-gated on that position** — it refuses out-of-order
> execution and names the expected next verb. A `dispatch next` verb emits the
> single prescribed action from the current position (the orchestrator prompt
> collapses to "run `next`, do what it says, report-and-halt on refusal"). Every
> funnel git **read** is a first-class read verb over the object-db/ref primitives;
> **no funnel read shells raw git**, which makes the coord working-tree state
> irrelevant and dissolves the ISS-234 reverse-diff without any auto-sync. This is
> RFC-016 Cluster 2 (moves A + E) — the RFC's undelivered core.

This REV amends the two tech specs that govern the dispatch funnel — **SPEC-021**
(orchestrator process) and **SPEC-022** (git interaction model) — to carry the
zero-rescue mechanism as **forward-intent (`pending`) requirements**. It does not
implement anything; the descending slice does, and reconciles the requirements to
`active`. Cluster 1 (moves B/C, false-red/refusal elimination — SL-224/SL-225)
already landed and is the cleaned baseline this cluster's benchmark measures
against.

### The target: zero-rescue

Today the funnel cadence lives as **prose the orchestrator must hold** (SPEC-021
REQ-287's eight-step ordered contract) plus a **memory corpus of recovery idioms**
recalled mid-run. The LLM diagnoses, sequences, and recalls — ~40–49k tokens of
ceremony context per phase plus rescue-archaeology (RFC-011 evidence). Zero-rescue
reserves LLM judgment for genuine judgment (conflict content, red-verify triage,
scope) and moves the invariants into verbs: the tool tells you what's next, you do
it, you halt on refusal.

### Posture: forward-intent requirements in a retrospective spec

SPEC-021/022 are **retrospective** ("describe shipped behaviour; coverage
reconciled, never inferred"). REV-030 correctly refused to amend SPEC-022 *ahead*
of the code. This REV does not violate that charter: it does not rewrite the
retrospective prose to *claim* unbuilt behaviour is shipped. It **adds `pending`
forward-intent requirements**, which the dual-posture rule expressly permits
(planned stays distinguishable from verified), following the standing precedent
that **SPEC-021 already hosts a `pending` requirement — REQ-335** (confined-
orchestrator altitude). The new requirements ship `pending`; the slice flips them
`active` at reconcile. Existing `active` requirements are touched only where the
new mechanism changes their meaning (REQ-287, REQ-293, REQ-294, REQ-318), and those
§-prose paragraphs are swept in the same change (requirement-entity/§-prose drift
is silent otherwise).

### Move A — the funnel becomes a state machine (SPEC-021)

The code scope confirms move A is genuinely new, built on strong existing seams:

- **Persisted funnel position is new.** `dispatch_next_ready` computes *phase*
  readiness (which phase to dispatch a worker for) from per-phase `PhaseStatus`; it
  knows nothing of funnel position (imported? concluded? reaped?). Position is
  *derived* today (`ReceiptStatus::ConcludeIncomplete`, and `NextGuidance` — an
  already-deterministic 7-state sequencer) and **stored nowhere, enforced nowhere**.
  → **FR-008**: position persisted per-phase (spawned → worker-committed → imported
  → concluded → reaped), authoritative not derived.
- **Legality gates are new.** `dispatch_import` / `dispatch_conclude_phase` /
  `dispatch_reap` each check only *local* invariants (scope belt, CAS, landed-
  oracle) — none reads a position and refuses "illegal here, run X next."
  → **FR-009** (primary): every funnel verb legality-gated on position; refuses
  out-of-order and names the expected next verb (OQ-1: `next` prescribes **and**
  verbs refuse — both).
- **`dispatch next` promotes `NextGuidance` from advisory → prescriptive.**
  `NextGuidance`/`select_guidance` already know the expected action; they only
  print it. → **FR-010**: `dispatch next` emits the single prescribed action.
- **One machine, three doors.** The funnel write tools are already transport-
  agnostic (routed only by `(root, slice)`); the codex/pi subprocess arm is not yet
  forced through the same gated funnel (it uses `worktree fork --worker` + CLI
  `record-boundary`/`sync`). → **FR-011**: one funnel state machine across main-
  thread / subprocess / confined arms; generalises SL-199's confined-arm machine
  (OQ-2). Relates REQ-335, whose confined-orchestrator altitude folds into the
  unified machine when this lands.

REQ-287's ordered-contract discipline moves from an **orchestrator-held prose
contract** to a **machine-enforced gate** (report-and-halt becomes the verb's
refusal, not the operator's judgment) — a `modify`, with the §-prose swept.

### Move E — no shell git in the funnel (SPEC-022)

The code scope enumerated OQ-7. Nine funnel read idioms **already have `pub(crate)`
primitives** in `src/git.rs` and need only a verb surface — `git show <ref>:<path>`
→ `read_path_at`, `rev-parse` → `resolve_ref`, `merge-base --is-ancestor` →
`is_ancestor`, `git cherry` → `git_cherry` (already wrapped by `dispatch_reap`),
`diff --name-only` → `changed_paths`, trunk-ladder → `trunk_commit`, `ls-tree` →
`blob_oid_at`, `worktree list` → `list_worktrees`, `status` (tracked) →
`tree_clean`. Seven are genuine gaps blocking a clean prohibition: isolation
detection (`--git-dir` vs `--git-common-dir`), an **untracked-aware** clean gate
(`tree_clean` ignores untracked, the commit-before-spawn guard does not), three-dot
content diff, `branch --show-current`, `git log`, `check-ignore`, and the ISS-234
`git restore` — which is a working-tree **write**, out of read-verb scope by
category.

- → **FR-010** (SPEC-022): a first-class git read-verb surface over the object-db/
  ref primitives; no funnel read shells raw git. Coverage is enumerated (OQ-7); the
  seven gaps above are the build list. Extends the REQ-318 precedent (object-db
  sourcing, "never the working filesystem") — `modify` REQ-318's §-prose to name
  the surface.
- → **FR-011** (SPEC-022): the funnel is working-tree-free, so coord tree state is
  irrelevant; because reads go through verbs, the orchestrator never shells raw git
  against the coord tree and the **ISS-234 reverse-diff is dissolved** — no `git
  restore` dance, and explicitly **not** an interim auto-sync (IDE-028 is the wrong
  path). The one residual write idiom is retired by the funnel not touching the
  tree, not by a mirror.

### Open questions — settled here vs deferred to the slice

Settled at spec altitude by this REV: **OQ-1** (`next` prescribes + verbs refuse —
both), **OQ-2** (one machine, three doors; run-state home), **OQ-7** (read-verb
coverage — the enumeration above is the spine). Deferred to the descending slice as
implementation/measurement: **OQ-3** (bundle export/ingest metadata), **OQ-5**
(memory-blind benchmark harness — measured against the Cluster-1 baseline), **OQ-6**
(which dispatch memories retire vs remain as rationale).

### The change payload

Ten `[[change]]` rows. SPEC-021: introduce FR-008/009/010/011 (`pending`); modify
REQ-287 (cadence now machine-enforced), REQ-293 (reads via verbs), REQ-294
(checkout-import retires to in-verb fallback). SPEC-022: introduce FR-010/011
(`pending`); modify REQ-318 (read-verb surface extends object-db sourcing). Prose
rows are surfaced-for-manual at `revision apply`; the introduce rows are minted via
`spec req add` at apply and reconciled `active` by the slice.

### Not in this REV (deliberately)

- **OQ-4** (candidate auto-sourcing — default `close_target` ← repaired
  `review_surface`, the one real SPEC-022 vocabulary cut) is **scoped out**. It is
  move-C-adjacent (operator-carried contract → verb; move C already landed in
  Cluster 1), separable from the state-machine/no-shell-git core, and belongs to a
  sibling REV so this revision stays tight. Flagged for the user.
- **REQ-335** is not transitioned here; it stays `pending` and reconciles with
  FR-011 when the unified machine lands.
- **The move-D tail** (IMP-174/201/304 — lineage rows) is framed by RFC-016/IMP-311,
  not amended here; orthogonal to A/E.
- **The benchmark** (OQ-5) and **memory retirement** (OQ-6) are slice deliverables,
  not spec prose.

### Review provenance

_Pending._ To be cross-checked by an external adversarial pass (codex, GPT-5.5)
against the code scope before `revision approve`/`apply`.
