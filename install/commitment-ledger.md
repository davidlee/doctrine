<!-- Shipped reference. Published, not projected: there is no copy on disk in an
     installed project — read it as `lib:reference/commitment-ledger.md`. This
     is guidance for planners and plan reviewers. Nothing reads the commitment
     ledger yet and nothing checks it; the schema below is the only contract. -->

# The commitment ledger

How a planner turns a locked design into a plan whose exit criteria demonstrably
cover what the design committed to — and how a reviewer, and later an auditor,
can check that. The commitment ledger is a per-slice file that lists every
commitment the design makes, quoted, next to the plan criteria that satisfy it.

Naming: always say "commitment ledger". A bare "ledger" means the review ledger
(`lib:reference/review-ledger.md`), which this document builds on for the plan
review. For ids and the verification taxonomy, see `lib:reference/glossary.md`.

## Purpose

A plan can fail its design in two ways that no per-criterion read catches:

- **Unclaimed commitment.** The design says something must hold, and no exit
  criterion in any phase claims it. Nothing is wrong with any single criterion;
  the gap is only visible against the whole design.
- **Cited-but-wrong criterion.** A criterion cites the right design point and
  restates it wrongly: a count, a name, a set membership or a condition is off.
  Both sides look plausible, and the drift is only visible by comparing the
  words.

The commitment ledger makes both checkable. It quotes the design instead of
paraphrasing it, so the second class reduces to comparing a criterion's wording
with a quote. It names, for every commitment, the criteria that satisfy it, so
the first class reduces to finding a commitment with none. It also gives
ordering needs and later variances a place to live, linked to the commitment
they concern, rather than scattered across decision records, notes and history.

```text
design.md (locked)
      |  extract: one quoted row per commitment
      v
commitment-ledger.toml
      |  satisfy: link rows to phase exit criteria; add proofs
      v
plan.toml
      |  cross-check: coverage, content, proportionality, entry criteria
      v
plan review: two reviews in parallel (commitments, sequencing)
      |
      v
/phase-plan -> /execute and /consult (variances) -> /audit
```

## When

Required for any plan with more than one phase. A single-phase plan may skip
it and says so in one line of `plan.md`. The trigger reads only the plan's own
content.

## The commitment ledger

`commitment-ledger.toml` lives in the slice folder beside `plan.toml`. It is
authored: committed, diffable, and reviewed with the plan. Nothing parses it
yet. Its schema is the example below, and the `schema` marker line keeps a later
reader possible without a migration.

```toml
schema = "commitment-ledger/1"

[[extracted]]                      # one per design section read at extraction
section = "sec-3"
fingerprint = "7c052c403fa9"       # from `doctrine design show --format prompt`

[[commitment]]
id = "C-NN"                        # author-assigned, immutable, never reused
kind = "explicit"                  # explicit | implicit
source = { section = "sec-3", heading = "The commitment ledger" }
quote = '''
verbatim design text, as many lines as it takes
'''
satisfied_by = ["PHASE-NN/EX-n"]   # phase-qualified, always
how = "one line: how that criterion meets the quote"
needs = []                         # optional: ids of commitments this one needs
gated_by = ["PHASE-NN/EN-n"]       # optional: entry criteria of relying phases
verdict = "proportionate"          # proportionate | weak | excessive | deferred
note = ""

[[commitment]]                     # a routed design-review finding
id = "C-MM"
kind = "explicit"
source = { review = "RV-NNN", finding = "F-n" }
quote = '''
the finding's whole response: its criterion sketch and placement constraint
'''
satisfied_by = ["PHASE-NN/EX-n"]
how = "criterion cites the finding inline, on the earliest phase the constraint allows"
verdict = "proportionate"

[[proof]]                          # a verification row names the exits it proves
criterion = "PHASE-NN/VT-n"
proves = ["PHASE-NN/EX-n"]

[[variance]]                       # append-only; see Variances
commitment = "C-NN"
class = "realisation"              # realisation | intent
change = "what now differs, and why"
basis = "DEC-NNN"                  # or a consult outcome, or the user's turn
phase = "PHASE-NN"                 # or "plan" before execution
retires = false                    # intent only: true ends the commitment
successor = ""                     # with retires: the replacing id, or "" if withdrawn
```

Rules:

- **`quote` is verbatim and never edited.** Change is recorded as a variance.
  A quote that ends mid-line closes its literal on its last line, so it carries
  no newline the source lacks. `source` names the design-run section id
  (`design.md` carries a stable `doctrine:section` marker per section) plus its
  heading; never a line number.
- **Two source kinds.** A *design* row quotes `design.md`. A *review* row quotes
  the response of a routed design-review finding — one routed to `demonstrate`,
  `probe` or `control`, whose criterion sketch and placement constraint stay on
  the review ledger by rule and are never copied into the design. Its `source`
  names the review and finding; its quote is the *whole* current response, and
  its check is equality with the response in
  `doctrine review show RV-NNN -f F-n`, not containment — so an amendment that
  appends to the response is a change.
- **The governing set** is every commitment not retired. An `intent` variance
  with `retires = true` ends its commitment: superseded when it names a
  `successor`, withdrawn when it does not. A retired row keeps its quote as
  history; satisfaction, coverage, quote checks and phase-sheet copying all read
  the governing set only.
- **`[[extracted]]` stamps what was read** — each section's fingerprint at
  extraction — so a later revision of the design is detectable per section (see
  Refresh after a design revision).
- **Implicit commitments** are those the design entails without stating — for
  example a behaviour it relies on staying unchanged. They carry
  `kind = "implicit"` and a `quote` of the passage that entails them.
- **`needs` is correctness only** — "C-04 cannot be correct until C-01 holds".
  There is no soft ordering field; phase order carries taste. `needs` is
  optional and expected to be sparse: omit it when the dependency is simply the
  whole prior phase. `needs` must be acyclic.
- **Proofs are a separate table.** Commitment to exit criterion lives on the
  commitment; exit criterion from verification criterion lives on the proof.
  Every coverage direction is derived from those two links, never authored
  twice.
- **Ids are author-assigned, immutable and never reused**, and every criterion
  is phase-qualified (`PHASE-NN/EX-n`): a bare `EX-n` is ambiguous across
  phases.
- **No link field goes into `plan.toml`.** The plan reader silently drops
  unknown fields, so a link authored there would read as recorded and be lost.

## Authoring procedure

The steps run inside `/plan`, after scaffolding.

1. **Extract** — before authoring phases. Walk `design.md` section by section;
   write one row per commitment, explicit or implicit, quoted. Routed
   design-review findings become review rows, quoting the finding's response.
   Stamp an `[[extracted]]` row per section read. A gap or contradiction found
   while extracting is a design defect: stop and return to `/design`.
2. **Satisfy** — while authoring `plan.toml`, fill each row's `satisfied_by`,
   `how` and `gated_by`, and add `[[proof]]` rows for each verification
   criterion. Use `needs` to construct the plan:
   - *sequencing* — a commitment's phase is never earlier than any commitment it
     needs;
   - *gates* — a `needs` edge crossing phases appears as `gated_by` on an entry
     criterion of the dependent phase;
   - *parallelism* — commitments with no `needs` path between them and disjoint
     surfaces are candidates for parallel phases. The commitment ledger names
     candidates; host directives decide whether to parallelise.
3. **Cross-check**, before the first commit:
   - *coverage* — over the governing set: a commitment no exit criterion
     satisfies; an exit criterion no proof proves; a verification criterion that
     proves nothing; an exit criterion that meets no commitment (an extraction
     gap or scope drift — resolve which);
   - *content* — each satisfying exit criterion's wording against the quote:
     counts, names, sets, conditions;
   - *proportionality* — set each row's `verdict`: `proportionate`; `weak` (the
     proof does not exercise the commitment, e.g. a keyword floor standing in
     for behaviour); `excessive` (more or brittler proof than the risk
     warrants); `deferred` (proved later or at audit — `note` says where and
     why). Never count tests;
   - *entry criteria* — every `gated_by` names an entry criterion on the relying
     phase that actually gates the commitment; every cross-phase `needs` edge
     has such an entry criterion; `needs` has no cycle and never points forward.

Fix what the cross-check finds in the plan (or the commitment ledger), then
commit both.

**Orchestration seam.** The steps divide cleanly, which makes them an
opportunity for delegation proportionate to the slice's scope: the planner may
extract alone, hand extraction to one delegate or one per design section, and
have a separate reviewer check the transcription (quotes, satisfaction sites) or
the sequencing implications. This guidance does not prescribe the split or the
mechanism — that is where host directives (project or harness conventions)
govern how the procedure is applied. It names the seam and the
proportionality, nothing more. The plan review below is the same seam at review
time.

## Plan review

For multi-phase plans this replaces `/plan` step 7 ("consider plan
critically"); its findings feed the existing revise steps 8–9.

Open two reviews at facet `plan`, in parallel, each with its own raiser label
(`doctrine review new` takes one raiser label per ledger, and two ledgers avoid
turn contention). Findings follow `lib:reference/review-ledger.md` unchanged:
there is no new facet, kind or multi-raiser mechanism. After authoring,
`plan.toml` owns the realised sequence and the commitment ledger's `needs` is
its rationale; the `sequencing` reviewer checks they agree. Reviewers should be
fresh contexts, not the plan's author. Review writes are refused inside
dispatch workers, so this runs in a primary or solo tree.

| raiser | reads | checks |
|---|---|---|
| `commitments` | `design.md`, `commitment-ledger.toml`, `plan.toml` | missed commitments (implicit especially); quote fidelity; criterion wording vs quote; coverage; verdicts |
| `sequencing` | `design.md`, `plan.toml`, `commitment-ledger.toml`, the tree | phase order agrees with `needs`; missed `needs`; parallel candidates; entry criteria do gate what later phases need; each phase ends green; phase sizing; each verification criterion's `test_file` can reach what the test needs |

The split follows the defect classes: `commitments` owns *unclaimed* and
*cited-but-wrong*; `sequencing` owns the ordering problems that need a look at
the tree.

## Variances

A variance is a change against a specific commitment after extraction. It is
appended, never edited in place, and the `quote` stays.

- **`realisation`** — how the commitment is met changed; intent holds. A
  recorded `basis` suffices.
- **`intent`** — what must be true changed. Always needs the user or a design
  revision first; the `basis` cites that act.

This is a thin, default form of a design autonomy envelope: intent escalates,
realisation is recorded.

Where variances arise: `/execute` records a change to how or whether a
commitment is met; `/consult` records an outcome that changes a commitment,
citing the decision; `/audit` reads the commitment ledger — each commitment's
quote, its variances, and where it was finally satisfied.

### Refresh after a design revision

A revised and relocked design invalidates the commitment ledger silently: rows
may quote text that no longer exists, and commitments added in revised sections
have no row.

- **Detect** — at the locked-exit handoff to `/plan`, and at the start of each
  `/phase-plan`, the commitment ledger is stale if any of these holds:
  - the current section-id set (`doctrine design show --format prompt`) differs
    from the `[[extracted]]` set — a section added or removed;
  - a recorded fingerprint differs from the current one;
  - a governing design row's quote no longer occurs verbatim in `design.md`;
  - a governing review row's quote no longer equals its finding's response;
  - a routed finding on the design review's review ledger has no governing row.

  Without a managed design run, the fallback is the `doctrine:section` markers
  in `design.md` for the section set, plus the quote check and a diff of
  `design.md` since the commitment ledger's last commit.
- **Refresh, scoped to what changed** — extract each added or changed section,
  each unrowed routed finding and each amended response. Retire each governing
  row whose quote no longer matches, or whose section was removed, with an
  `intent` variance (`basis`: the design revision, or the amended review
  response): `retires = true` plus the `successor` id when the commitment was
  restated, no successor when it was dropped. Add rows for new commitments.
  Re-run the cross-check on affected rows; re-stamp `[[extracted]]` to the
  current section set and fingerprints.
- **The loop closes** — an `intent` variance raised during execution is what
  sends work back to design; the refresh after relock is where the commitment
  ledger confirms the design now says the new thing.

`/phase-plan` also copies the phase's governing rows' quotes into the runtime
phase sheet, so the worker reads the design's words.

### Between lock and planning

Between design lock and planning, extraction is the first close reading:
defects return to `/design`; a different realisation chosen at planning is a
variance with `phase = "plan"`.
