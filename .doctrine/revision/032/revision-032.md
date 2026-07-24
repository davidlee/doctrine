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
> single prescribed action for the **per-phase import→verify→conclude→reap
> sub-funnel** (the orchestrator's phase-loop prompt collapses to "run `next`, do
> what it says, report-and-halt on refusal"); candidate/close/audit sourcing stays
> out of the oracle's scope until a sibling revision. Every
> funnel git **read** is a first-class read verb over the object-db/ref primitives;
> **no funnel read shells raw git**, and a no-pathless-commit / safe-commit guard
> bounds every coord-tree write — so the ISS-234 reverse-diff can no longer commit
> mass reversions, closed without any auto-sync. This is RFC-016 Cluster 2 (moves A
> + E) — the RFC's undelivered core.

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
`active` at reconcile. The `active` requirements the new mechanism touches
(REQ-287, REQ-293, REQ-294, REQ-318) are **not** amended here: modifying an active
requirement to describe unbuilt behaviour would present target behaviour as shipped
and breach the retrospective charter (RV-300 F-1 — the move REV-030 declined for
SPEC-022). Their §-prose reconciliation defers to a ship-time sibling revision,
when evidence exists.

### Move A — the funnel becomes a state machine (SPEC-021)

The code scope confirms move A is genuinely new, built on strong existing seams:

- **Persisted funnel position is new.** `dispatch_next_ready` computes *phase*
  readiness (which phase to dispatch a worker for) from per-phase `PhaseStatus`; it
  knows nothing of funnel position (imported? concluded? reaped?). Position is
  *derived* today (`ReceiptStatus::ConcludeIncomplete`, and `NextGuidance` — an
  already-deterministic 7-state sequencer) and **stored nowhere, enforced nowhere**.
  → **FR-008**: position persisted per-phase, advancing through explicit
  transitions that **include verification** as an authoritative, evidence-carrying
  step faithful to REQ-287's ordering (spawned → worker-committed → imported →
  **verified** → concluded → reaped). A sequence without a `verified` state cannot
  let `next` choose verify-vs-conclude, nor let the gate refuse conclude-after-
  unverified (RV-300 F-2). FR-008 also requires the run-state record to be a
  **single-writer authority with crash-safe idempotent recovery** — the durable,
  spec-altitude half of OQ-2. It does **not** pick the concrete home (extend
  `boundaries`/`journal` vs a new record) or the CAS/concurrency contract; those
  are slice design (RV-300 F-5).
- **Legality gates are new.** `dispatch_import` / `dispatch_conclude_phase` /
  `dispatch_reap` each check only *local* invariants (scope belt, CAS, landed-
  oracle) — none reads a position and refuses "illegal here, run X next."
  → **FR-009** (primary): every funnel verb legality-gated on position; refuses
  out-of-order and names the expected next verb; **conclude refuses after skipped
  or failed verification** (OQ-1: `next` prescribes **and** verbs refuse — both).
- **`dispatch next` promotes `NextGuidance` from advisory → prescriptive.**
  `NextGuidance`/`select_guidance` already know the expected action; they only
  print it. But `select_guidance` routes the **full seven-state guidance domain**
  (PrepareReview, Audit, Integrate, …), not just the phase sub-funnel. → **FR-010**:
  `dispatch next` emits the single prescribed action **for the per-phase
  import→verify→conclude→reap sub-funnel only**; candidate/close/audit sourcing
  (active REQ-317, OQ-4) is **outside the oracle's scope** until a sibling revision
  settles it, so the "prompt collapses to `run next`" claim is scoped to that
  sub-funnel (RV-300 F-6).
- **One machine, each transport projects into it.** The funnel write tools are
  already transport-agnostic (routed only by `(root, slice)`); the codex/pi
  subprocess arm is not yet forced through the same gated funnel (it uses `worktree
  fork --worker` + CLI `record-boundary`/`sync`). → **FR-011**: one state machine
  owns the shared transition semantics; each transport (main-thread, subprocess,
  confined-orchestrator) **projects into that single authority**, recording the
  same transitions whether committing directly or through mediation. Per-transport
  *altitude* stays with the existing REQ-291 / REQ-335 (reconciled at ship-time),
  keeping topology and shared-semantics independently verifiable (RV-300 F-5); the
  loose "three doors" framing is dropped.

REQ-287's ordered-contract discipline shifts from an **orchestrator-held prose
contract** to a **machine-enforced gate** — but that is a change to an **active**
requirement, so it is **not** modified here (see Not in this REV); it reconciles at
ship-time (RV-300 F-1/F-3).

### Move E — no shell git in the funnel (SPEC-022)

The code scope enumerated OQ-7. Nine funnel read idioms **already have `pub(crate)`
primitives** in `src/git.rs` and need only a verb surface — `git show <ref>:<path>`
→ `read_path_at`, `rev-parse` → `resolve_ref`, `merge-base --is-ancestor` →
`is_ancestor`, `git cherry` → `git_cherry` (already wrapped by `dispatch_reap`),
`diff --name-only` → `changed_paths`, trunk-ladder → `trunk_commit`, `ls-tree` →
`blob_oid_at`, `worktree list` → `list_worktrees`, `status` (tracked) →
`tree_clean`. **Isolation detection is not a gap** — `is_linked_worktree`
(`src/worktree/shared.rs:48-58`, re-exported at `mod.rs:22`, already used by
marker/jail/provision/gc) implements exactly the `--git-dir` vs `--git-common-dir`
read; it needs **relocation/wrapping into the read-verb surface, not
reimplementation** (RV-300 F-7 — a parallel-implementation trap my first sweep
missed). Five genuine read gaps remain: an **untracked-aware** clean gate
(`tree_clean` ignores untracked, the commit-before-spawn guard does not), three-dot
content diff, `branch --show-current`, `git log`, `check-ignore`. Plus the ISS-234
`git restore`, which is a working-tree **write** — out of read-verb scope by
category, and handled by FR-011's guard below.

- → **FR-010** (SPEC-022): a first-class git read-verb surface over the object-db/
  ref primitives; no funnel read shells raw git. Coverage enumerated (OQ-7); the
  five gaps are the build list, and **existing seams are reused/relocated, not
  reimplemented**. Extends the REQ-318 precedent (object-db sourcing, "never the
  working filesystem") — but REQ-318 is **active**, so its §-prose reconciliation
  defers to the ship-time sibling REV, not a modify row here.
- → **FR-011** (SPEC-022): the funnel is working-tree-free **and every coord-tree
  operation is bounded by a no-pathless-commit / safe-commit guard**. Read verbs
  keep funnel *reads* from observing the reverse-diff; the guard closes the *write*
  side — because `dispatch_conclude_phase` still writes the coord runtime sheet
  before its object-db boundary commit (`src/mcp_server/dispatch.rs:374-392`), the
  tree is still operationally touched, so reads alone do **not** dissolve ISS-234
  (RV-300 F-4). ISS-234 is absorbed only when both hold. Still explicitly **not** an
  auto-sync (IDE-028 is the wrong path).

### Open questions — settled here vs deferred to the slice

Settled at spec altitude by this REV: **OQ-1** (`next` prescribes + verbs refuse —
both), **OQ-7** (read-verb coverage — the enumeration above is the spine). **OQ-2 is
framed, not fully settled** (RV-300 F-5): FR-008 requires a single-writer
authoritative run-state record with crash-safe idempotent recovery, and FR-011 names
the one-authority/per-transport-projection semantics — but the concrete record
*home* (extend `boundaries`/`journal` vs a new record) and the CAS/concurrency
contract are left to slice design. Deferred to the descending slice as implementation/
measurement: **OQ-3** (bundle export/ingest metadata), **OQ-5** (memory-blind
benchmark harness — measured against the Cluster-1 baseline), **OQ-6** (which
dispatch memories retire vs remain as rationale).

### The change payload

Six `introduce` `[[change]]` rows, all `pending`. SPEC-021: FR-008 (persisted
funnel position incl. verification + run-state authority), FR-009 (per-verb legality
gate; primary), FR-010 (`dispatch next` over the phase sub-funnel), FR-011 (one
authority, per-transport projection). SPEC-022: FR-010 (read-verb surface), FR-011
(working-tree-free + safe-commit guard). Each is minted via `spec req add` at
`revision apply` and reconciled `active` by the descending slice.

**No modify rows.** REV-032 is introduce-only after RV-300 F-1/F-3: amending the
four *active* requirements the new mechanism touches (REQ-287 cadence, REQ-293/294
git-reads, REQ-318 object-db sourcing) would present target behaviour as shipped and
breach the specs' retrospective charter — the exact move REV-030 declined for
SPEC-022. Those reconciliations are staged for a **ship-time sibling revision** at
slice close, when evidence exists, with reviewable before/after prose then.

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
- **The four active-requirement / §-prose reconciliations** (REQ-287/293/294/318)
  defer to a **ship-time sibling revision** at slice close (RV-300 F-1/F-3), not
  amended here.

### Review provenance

External adversarial pass: **codex (GPT-5.5), ledger [[RV-300]]**, cross-checked
against SPEC-021/022, RFC-016, ADR-006/012/013/014, REV-030, ISS-234, and the
dispatch/git source. Five source claims held under inspection (REQ-335 pending;
`dispatch_next_ready` computes phase readiness not funnel position; funnel verbs
enforce no global ordering; all nine named `src/git.rs` primitives exist; both specs
pass structural validation). Seven findings raised, **all seven adjudicated as
correct and integrated** above: F-1/F-3 (introduce-only; defer active modifies),
F-2 (verify transition in FR-008/009), F-4 (safe-commit guard in SPEC-022 FR-011),
F-5 (run-state authority named; OQ-2 downgraded; FR-011 split from topology), F-6
(FR-010 narrowed to the phase sub-funnel), F-7 (`is_linked_worktree` reuse, gap
inventory corrected to five). Dispositions recorded on the RV-300 ledger.
