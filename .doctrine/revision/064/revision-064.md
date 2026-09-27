# REV REV-064 — Review ledger v2 amends ADR-007 and the SPEC-003 inventory

Revision — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

RFC-032 (Review ledger effectiveness) led to SL-268, which landed the review
ledger's second schema. That schema adds a turn journal, `amend` and `reopen`,
closed dispositions with a separate `route`, a `done` that needs a concluded
pass, fail-safe reads of corrupted vocabulary, and a `prime` that degrades
instead of failing. The mechanism is now described in the new container tech
spec SPEC-032 (Review ledger). Three ADR-007 decisions no longer match what
shipped:

- **D-C5** knows no journal, no `amend` or `reopen`, and no `route`, and leaves
  disposition free-text.
- **D-C8** derives `done` from finding statuses alone. `done` now also needs the
  raiser's `conclude`, which `raise` and `reopen` clear.
- **D-C10** describes a reviewer-authored `domain_map` distillation. The cache
  that shipped is a content-hash model over the target slice's selectors, and
  it degrades when there are none.

This revision rewrites those three decisions toward decisions only (IMP-481's
agreed boundary: ADR-007 keeps the decisions, SPEC-032 holds the mechanism). It
also carries the **knock-on wording** that keeps ADR-007 consistent with them:
the "Schema shape" block, the transition graph, D-C2's await sentence, the D-C4
clarification's list of single-owner edges, and the Consequences sentence on the
D-C9b reverse lookup. The knock-on changes are wording only. The decisions they
sit in (D-C2, D-C4, D-C9b) are unchanged.

It also revises SPEC-003's container inventory. SPEC-029 (Design run engine) was
already missing from it (DEC-317), and SPEC-032 is new. The structured
`responsibilities[0]` list and the Overview bullets each gain both.

SPEC-032 is **not** a change row. It is authored directly under the storage
rule, because the Revision grammar cannot create specs (the REV-035
precedent). It was authored after the code landed, so that it describes what
shipped rather than what was planned (DEC-321).

RFC-032's decision frontier D12 (the reverse index is not needed until
measured) closes IMP-479. The close-gate sentence below records that the corpus
scan stays.

Deliberately **left unchanged**:

- D-C9a ("done only when every finding is terminal"). It remains true as a
  necessary condition, and D-C8 now states the full rule.
- The "Resolved" sections, which are the record of earlier reviews.
- The Verification section, whose assertions still hold.

This REV is applied at reconcile. `modify` rows are hand-landed, so each
**After** block below is the exact replacement text.

## Change 1 — ADR-007 (modify, primary)

### D-C5 — replace the whole bullet

**Before:**

````markdown
- **D-C5 — Schema and finding mutability.** *Finding identity* is append-only:
  findings are never deleted or renumbered, and `id` / `title` / `detail` /
  original `severity` are raiser-owned and fixed. *Lifecycle fields* mutate
  through CLI-mediated transitions — the responder owns `disposition` /
  `response`; the raiser owns the verify / contest / withdraw transitions. Field
  ownership keeps keys disjoint. Severity is reviewer-idiomatic:
  `blocker | major | minor | nit`.
````

**After:**

````markdown
- **D-C5 — Schema, finding mutability, and the turn journal.** *Finding
  identity* is append-only: findings are never deleted or renumbered, and `id` /
  `title` / `detail` / original `severity` are raiser-owned and fixed.
  *Lifecycle fields* mutate only through CLI-mediated acts. The responder owns
  `disposition` / `route` / `response` (`dispose`, `amend`); the raiser owns the
  verify / contest / reopen / withdraw transitions. Field ownership keeps keys
  disjoint. Severity is reviewer-idiomatic: `blocker | major | minor | nit`.

  Every act is **journalled**. A finding act appends a `[[finding.turn]]` row,
  and `conclude` appends a `[[review.turn]]` row. The row records the act, the
  role, and the act's note or answer, and it is written in the same edit that
  moves the state. The current-state fields are what gates read; the journal is
  the append-only record of how they got there. `amend` (responder, answered →
  answered) revises an answer, and `reopen` (raiser, verified → contested) hands
  a verified finding back. `amend`, `contest` and `reopen` require a note.
  `disposition` is closed on write (`aligned | fix-now | design-wrong |
  follow-up | tolerated`) and open on read, so legacy values stay readable. An
  optional `route` (`review | demonstrate | probe | control | owner-fix`), on the
  same terms, records where the answer sends the finding. Keys added to the
  schema default on read, and no ledger is migrated. The mechanism and the full
  act table are in SPEC-032.
````

### D-C8 — replace the whole bullet

**Before:**

````markdown
- **D-C8 — Derived status function.** The review's overall status derives from
  finding statuses together with `await`, never hand-edited:

  ```text
  empty ledger                  → status = active, await = raiser   (raiser goes first)
  any open/answered/contested   → status = active, await = derived role
  all verified/withdrawn        → status = done,   await = none
  ```

  where the awaited role is: any `open`/`contested` ⇒ responder; else any
  `answered` ⇒ raiser. The empty ledger is `active` (not vacuously `done`), so an
  implementation can never mistake "no findings yet" for completion. This makes
  the kind born free of the status-rollup divergence SL-009 surfaces for slices.
````

**After:**

````markdown
- **D-C8 — Derived status function.** The review's overall status and `await`
  derive from the finding statuses and the pass's `concluded` marker. They are
  never hand-edited, and they are recomputed on every read, never latched:

  ```text
  any open/contested (or unknown) finding          → status = active, await = responder
  else any answered                                → status = active, await = raiser
  else all terminal (empty included), unconcluded  → status = active, await = raiser
  else all terminal and concluded                  → status = done,   await = none
  ```

  So `done ⇔ every finding terminal ∧ concluded`. The empty ledger is `active`,
  never vacuously `done`, so an implementation can never mistake "no findings
  yet" for completion. `concluded` is the one stored pass-level fact, because a
  pass finishing is an event that no finding set can express: a clean pass and a
  pass never run look the same. The raiser sets it with `conclude`, which takes
  a required `--basis` (what the pass examined) and records it as a
  `[[review.turn]]`. Open findings do not block `conclude`. `raise` and `reopen`
  clear the marker in the same write that journals them, so `done` needs a fresh
  `conclude` after the last raise or reopen. An out-of-vocabulary finding status
  reads non-terminal and is disclosed, never guessed (SPEC-032). This keeps the
  kind free of the status-rollup divergence SL-009 surfaces for slices.
````

### D-C10 — replace the whole bullet

**Before:** the D-C10 bullet in full, from `- **D-C10 — Warm-cache: uniform,
self-scaling reviewer-context cache.**` through `Lives beside the baton
(runtime, gitignored, regenerable — D-C1/D-C2 tier).`. It opens:

````markdown
- **D-C10 — Warm-cache: uniform, self-scaling reviewer-context cache.** Every RV
  carries a runtime-tier cache of the reviewer's *learned* model of the subject —
  a `domain_map` (area → purpose → explored paths), plus invariants and
  risk areas — populated on a `prime` step and reused across review rounds. It is
  a **distillation** the reviewer authors, not the inference-layer prompt/token
  cache: …
````

It goes on to key staleness on content hashes of "the `domain_map` paths", and
defers a worktree-aware staleness model to slice design.

**After:**

````markdown
- **D-C10 — Warm-cache: a content-hash cache over the target's selectors.**
  Every RV may carry a runtime-tier reviewer-context cache, filled by `prime` and
  reused across review rounds. It records content hashes over the **target
  slice's declared selector path-set**. It amortises the re-check cost across
  rounds and agents that defines an adversarial loop, which the inference-layer
  prompt cache (ephemeral, single-context) cannot survive. It is facet-agnostic
  and self-scaling: a thin target tracks few paths. (Prior art: spec-driver
  `supekku:workflow.review-index@v1`.)

  **Staleness keys on content hashes of the tracked paths**, not a bare
  `{phase, head}`, so drift in uncommitted and gitignored files is seen as well
  as committed drift. Staleness is a signal `status` reports, never a gate.
  Literal selectors that do not name a regular file (directories, symlinks) are
  skipped and listed. When the target has no path-set (it is not a slice, or the
  slice declares no selectors), `prime` **degrades**: it tracks nothing, removes
  any earlier cache, and says why, rather than failing (STD-003). The cache is
  hashed in the invoking tree, and review verbs refuse a worker fork (D-C1). There
  is no reviewer-authored distillation: the `domain_map` prose tier is retired.
  The cache lives beside the baton (runtime, gitignored, regenerable — D-C1/D-C2
  tier). Mechanism: SPEC-032.
````

### Knock-on (wording only) — D-C2, one sentence

**Before:**

````markdown
- **D-C2 — Authored ledger is truth; baton is a regenerable cache.** `await` is a
  pure function of finding statuses; the runtime baton caches it and holds
  non-derivable bookkeeping. On loss it is recomputed from the authored ledger.
````

**After:**

````markdown
- **D-C2 — Authored ledger is truth; baton is a regenerable cache.** `await` is a
  pure function of the authored ledger (finding statuses and the pass's
  `concluded` marker, D-C8). The runtime baton caches it with the CAS key
  (D-C4a), and on loss it is recomputed from the authored ledger. Turn counters
  derive from the ledger's turn journal (D-C5), not from the baton.
````

### Knock-on (wording only) — D-C4 clarification, the single-owner edge list

**Before:**

````markdown
  (append-only, raiser-owned); the single-owner edges (`dispose`/`verify`/`contest`/
  `withdraw`) are gated by the per-finding predicate.*
````

**After:**

````markdown
  (append-only, raiser-owned); the single-owner edges (`dispose`/`amend`/`verify`/
  `contest`/`reopen`/`withdraw`) are gated by the per-finding predicate.*
````

### Knock-on (wording only) — the "Schema shape" block

**Before:** the `review-NNN.toml` block under `### Schema shape`, from
`[review]` through the `[[finding]]` row ending
`response    = "..."        # responder-owned, mutable`.

**After:**

````markdown
```toml
# review-NNN.toml — authored. Findings append-only; await/status derived, not stored.
[review]
facet     = "design"   # closed enum: scope|design|plan|phase-plan|implementation|code-review|reconciliation
raiser    = "reviewer" # role labels, also accepted as --as aliases
responder = "author"
concluded = true       # the raiser concluded the current pass (D-C8); raise/reopen clear it
rounds_base   = 0      # counter seed, written once at the first journalled write
contests_base = 0

[[review.turn]]        # review-level journal: conclude only (D-C5)
act  = "conclude"
role = "raiser"
note = "..."           # the --basis

[target]                # outbound relation (ADR-004): RV-NNN ──reviews──▶ ref
ref   = "SL-024"        # single canonical id of the subject (reconciliation: the spec, e.g. PRD-010)
phase = "PHASE-03"      # optional scoped qualifier — phase-scoped facets only

[[finding]]
id          = "F-1"        # raiser-owned, fixed; bare doc-local id, append-only
status      = "verified"   # open|answered|contested|verified|withdrawn
severity    = "major"      # raiser-owned, fixed
title       = "..."        # raiser-owned, fixed
detail      = "..."        # raiser-owned, fixed
disposition = "fix-now"    # responder-owned; closed on write, open on read
route       = "review"     # responder-owned, optional; closed on write, open on read
response    = "..."        # responder-owned, current answer

[[finding.turn]]           # append-only journal (D-C5), file order
act  = "raise"
role = "raiser"

[[finding.turn]]
act         = "dispose"
role        = "responder"
disposition = "fix-now"    # snapshot of what this turn answered
route       = "review"
response    = "..."

[[finding.turn]]
act  = "verify"
role = "raiser"
```
````

The sentence after the block is unchanged.

### Knock-on (wording only) — the transition graph

**Before:**

````markdown
Transition graph (each edge is single-owner; status function in D-C8):

```
  (none) --raise[raiser]--> open --dispose[responder]--> answered --verify[raiser]--> verified*
                              ^                              |
                              +------ contest[raiser] -------+
   contested --dispose[responder]--> answered
   open|answered --withdraw[raiser]--> withdrawn*
```
````

**After:**

````markdown
Transition graph (each edge is single-owner; status function in D-C8):

```
  (none)          --raise[raiser]-------> open
  open|contested  --dispose[responder]--> answered
  answered        --amend[responder]----> answered     (note required)
  answered        --verify[raiser]------> verified*
  answered        --contest[raiser]-----> contested    (note required)
  verified        --reopen[raiser]------> contested    (note required)
  open|answered   --withdraw[raiser]----> withdrawn*
  review level:     conclude[raiser]      sets concluded (--basis required)

  * terminal. raise and reopen clear concluded.
```
````

### Knock-on (wording only) — Consequences / Negative, the close-gate bullet

**Before:**

````markdown
- The close-gate (D-C9b) needs a **reverse** lookup over the outbound `reviews`
  edge ("active reviews targeting this ref with an unresolved `blocker`"). No
  general reverse-relation index exists today (relations are outbound-only,
  ADR-004); the gate is a corpus scan over RV `[target].ref` — feasible, but built,
  not reused.
````

**After:**

````markdown
- The close-gate (D-C9b) needs a **reverse** lookup over the outbound `reviews`
  edge ("active reviews targeting this ref with an unresolved `blocker`"). No
  general reverse-relation index exists (relations are outbound-only, ADR-004),
  so the gate is a corpus scan over RV `[target].ref`, costing O(number of
  reviews) per close. A reverse index or a materialised status is not built
  until a measurement shows the scan costs (IMP-479, closed as not needed until
  measured).
````

## Change 2 — SPEC-003 (modify)

### Structured `responsibilities[0]` (in `spec-003.toml`)

**Before:**

````text
Name the containers Doctrine is composed of — the entity engine, spec composition, memory, the observation ledger, id lifecycle, install, skills distribution, boot, the prompt cascade, dispatch/worktree, the priority engine, reconciliation, and the CLI surface — and the C4 decomposition that arranges them under one whole-system context.
````

**After:**

````text
Name the containers Doctrine is composed of — the entity engine, spec composition, memory, the observation ledger, id lifecycle, install, skills distribution, boot, the prompt cascade, dispatch/worktree, the design run engine, the review ledger, the priority engine, reconciliation, and the CLI surface — and the C4 decomposition that arranges them under one whole-system context.
````

### Overview container bullets (in `spec-003.md`)

**Before:** the inventory goes straight from dispatch to the priority engine:

````markdown
- **Dispatch & worktree** (SPEC-012) — the isolation and orchestrator-sole-writer
  machinery for concurrent work.
- **Priority engine** (SPEC-001) — the derived, explainable "what next" view over
  the entity graph.
````

**After:** two bullets are inserted between them:

````markdown
- **Dispatch & worktree** (SPEC-012) — the isolation and orchestrator-sole-writer
  machinery for concurrent work.
- **Design run engine** (SPEC-029) — the durable, recoverable coordination state
  behind a slice's design workflow: stage gates, attested acts, and a bounded
  read model.
- **Review ledger** (SPEC-032) — the turn-based adversarial review ledger: its
  act table, derived status, and the blocker gates other kinds close on.
- **Priority engine** (SPEC-001) — the derived, explainable "what next" view over
  the entity graph.
````
