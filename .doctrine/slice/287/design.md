<!-- doctrine:section sec-1 -->
## What changes and why

Plans for complex designs lose commitments in translation. On one measured
plan review (`RFC-027` § G4, `SL-251`), 2 of 9 defects were design commitments
no criterion claimed, and 5 of 9 were criteria that cited the right design
point and restated it wrongly. A citation-only cross-check scores the second
class as covered.

This slice ships, as guidance only, a procedure the user has run informally in
other doctrine repos:

```text
design.md ──extract──► commitment ledger ──► plan.toml (EX)
                         │  quote (verbatim)      │
                         │  satisfied_by ─────────┘
                         │  [[proof]] VT ──proves──► EX
                         │  [[variance]] (append-only)
                         ▼
          cross-check: coverage · content · proportionality
                         ▼
          plan review: RV `commitments` ∥ RV `sequencing`
                         ▼
          phase-plan → execute/consult (variances) → audit
```

- The ledger quotes the design rather than paraphrasing it, so the
  cited-but-wrong class becomes checkable by comparison.
- It is the interim home of the `VT`→`EX` link (`REQ-468`, unbuilt), shaped so
  a later move into `plan.toml` is a copy.
- It carries variances to audit, which today live in DEC records, `notes.md`
  prose and git history with no link to the commitment they changed.

No engine, schema, verb or check changes. One test is added (§ Code impact).
Decisions: `DEC-415`–`DEC-420`. Evidence: `.doctrine/slice/287/research/research.md`.

<!-- doctrine:section sec-2 -->
## The commitment ledger

`commitment-ledger.toml` in the slice folder — **authored** tier, committed and
reviewed with the plan (`DEC-415`). Nothing parses it; its schema is the
shipped reference doc's example, and the marker line keeps a later reader
possible without a migration.

```toml
schema = "commitment-ledger/1"

[[commitment]]
id = "C-01"                        # author-assigned, immutable, never reused
kind = "explicit"                  # explicit | implicit
source = "design.md § The commitment ledger"
quote = '''
verbatim design text, as many lines as it takes
'''
satisfied_by = ["PHASE-02/EX-1"]   # phase-qualified, always
how = "one line: how that EX meets the quote"
gated_by = ["PHASE-04/EN-2"]       # optional: later phases relying on it
verdict = "proportionate"          # proportionate | weak | excessive | deferred
note = ""

[[proof]]                          # a verification row names the exits it proves
criterion = "PHASE-02/VT-1"
proves = ["PHASE-02/EX-1"]

[[variance]]                       # append-only; see § Variances
commitment = "C-01"
class = "realisation"              # realisation | intent
change = "what now differs, and why"
basis = "DEC-NNN"                  # or a consult outcome, or the user's turn
phase = "PHASE-02"                 # or "plan" before execution
```

Rules:

- **`quote` is verbatim and never edited.** Change is recorded as a variance.
  `source` names a design heading, not a line (headings survive edits).
- **Implicit commitments** are those the design entails without stating — e.g.
  a behaviour it relies on staying unchanged. They carry `kind = "implicit"`
  and a `quote` of the passage that entails them.
- **Proofs are a separate table** in `REQ-468`'s shape. Commitment → EX lives
  on the commitment; EX ← VT lives on the proof; every coverage direction is
  derived from those two, never authored twice.
- **No link field goes into `plan.toml`.** The plan reader drops unknown fields
  silently today (`RFC-029` OQ-1), so authoring one there would read as
  recorded and be lost.
- Name it "commitment ledger" everywhere; bare "ledger" means the review ledger.

<!-- doctrine:section sec-3 -->
## Authoring procedure

**Trigger** (`DEC-417`): required for any plan with more than one phase. A
single-phase plan may skip it and says so in one line of `plan.md`. The trigger
reads only the plan's own content (`POL-002`).

**Steps**, inside `/plan` (current steps 3–9):

1. **Extract** — after scaffolding, before authoring phases. Walk `design.md`
   section by section; one row per commitment, explicit or implicit, quoted.
   Routed design-review findings (the existing step-3 sub-procedure) become
   rows too, with `source` naming the finding. A gap or contradiction found
   while extracting is a design defect: stop and return to `/design`, as step 2
   already requires.
2. **Satisfy** — while authoring `plan.toml`, fill each row's `satisfied_by`,
   `how` and `gated_by`, and add `[[proof]]` rows for each verification
   criterion.
3. **Cross-check** (`DEC-416`), before the first commit:
   - *coverage* — a commitment no EX satisfies; an EX no proof proves; a
     verification criterion that proves nothing; an EX that meets no commitment
     (an extraction gap or scope drift — resolve which);
   - *content* — each satisfying EX's wording against the quote: counts, names,
     sets, conditions;
   - *proportionality* — set each row's `verdict`: `proportionate`; `weak` (the
     proof does not exercise the commitment, e.g. a keyword floor standing in
     for behaviour); `excessive` (more or brittler proof than the risk
     warrants); `deferred` (proved later or at audit — `note` says where and
     why). Never count tests.
   - *entry criteria* — every `gated_by` names an EN on the relying phase that
     actually gates the commitment; a later phase relying on an earlier
     commitment with no such EN is a gap.

Fix what the cross-check finds in the plan (or the ledger), then commit both.

<!-- doctrine:section sec-4 -->
## Plan review

For multi-phase plans this replaces `/plan` step 7 ("consider plan
critically"); its findings feed the existing revise steps 8–9 (`DEC-418`).

Two RVs at facet `plan`, opened in parallel, each with its own raiser label
(`review new` takes one raiser label per ledger, and two ledgers avoid turn
contention). Reviewers should be fresh contexts, not the plan's author. Review
writes are refused inside dispatch workers, so this runs in a primary or
solo tree.

| raiser | reads | checks |
|---|---|---|
| `commitments` | `design.md`, `commitment-ledger.toml`, `plan.toml` | missed commitments (implicit especially); quote fidelity; EX wording vs quote; coverage; verdicts |
| `sequencing` | `design.md`, `plan.toml`, the tree | phase order and dependencies; EN gates do gate what later phases need; each phase ends green; phase sizing; each VT's `test_file` can reach what the test needs |

The split maps onto the measured defect classes: `commitments` owns
*unclaimed* and *cited-but-wrong*; `sequencing` owns *needs a tree check*.
Findings follow the review-ledger protocol unchanged.

<!-- doctrine:section sec-5 -->
## Variances and lifecycle touchpoints

A variance is a change against a specific commitment after extraction
(`DEC-420`). It is appended, never edited in place, and the `quote` stays.

- **`realisation`** — how the commitment is met changed; intent holds. A
  recorded `basis` suffices.
- **`intent`** — what must be true changed. Always needs the user or a design
  revision first; the `basis` cites that act.

This is a thin, default form of a design autonomy envelope (intent escalates,
realisation is recorded); a per-commitment envelope is `SL-258` OQ-1's
question, not this slice's.

Touchpoints — each skill gets a pointer to `lib:reference/commitment-ledger.md`,
not a restatement (`ADR-005`):

| skill | pointer |
|---|---|
| `/plan` | build the ledger (§ Authoring procedure); run the plan review (§ Plan review) |
| `/phase-plan` | copy the phase's ledger rows' quotes into the runtime phase sheet, so the worker reads the design's words |
| `/execute` | a change to how or whether a commitment is met is recorded as a variance |
| `/consult` | a consult outcome that changes a commitment is recorded as a variance, citing the DEC |
| `/audit` | read the ledger: each commitment's quote, its variances, and where it was finally satisfied |

Between design lock and planning, extraction is the first close reading:
defects return to `/design`; a different realisation chosen at planning is a
variance with `phase = "plan"`. The cost of returning to design is `IMP-525`.

<!-- doctrine:section sec-6 -->
## Governance and guidance

- **`ADR-005`** — one normative home: `install/commitment-ledger.md`, published
  as `lib:reference/commitment-ledger.md`. Skills route to it. It must be
  pointed at by a skill to be reachable; five skills do.
- **`ADR-019`** — publication needs a `publication/manifest.toml` entry, or
  `lib:` citations do not resolve.
- **`ADR-024` / `POL-002`** — shipped text grounds only on prose, `lib:`
  addresses, skill names and CLI verbs. No `SL-`/`RFC-`/`DEC-`/`REQ-` ids, no
  `src/` paths; the defect classes are described, not cited.
- **`ADR-007`** — the plan review is the existing RV kind at facet `plan`; no
  new facet, kind or multi-raiser mechanism.
- **`STD-002` / `DEC-101`** — "commitment", never "obligation"; ledger ids are
  author-assigned and immutable; criteria are phase-qualified.
- **`DEC-101` authoring rule** — guidance, never presented as an enforced check.
- **Storage rule** — the ledger is authored TOML. `lib:reference/glossary.md`'s
  slice-directory layout and `lib:reference/using-doctrine.md`'s storage example
  gain it, and the glossary distinguishes it from the review ledger.

Also corrected while touching `/plan` step 4: its claim that `specs` /
`requirements` stay empty "(no registry yet)" is false — plans populate them
and the reader drops them (`RFC-029` § 5). The edit states only what is true:
relations are written with `doctrine link`; the in-file tables are not read.

<!-- doctrine:section sec-7 -->
## Code impact

| path | change |
|---|---|
| `install/commitment-ledger.md` | new reference doc: purpose, trigger, ledger format (one fenced TOML example), procedure, cross-check, plan review, variances |
| `publication/manifest.toml` | `[[entry]]` for `reference/commitment-ledger.md` |
| `src/install.rs` | test beside `review_ledger_is_shipped`: the doc ships, and its fenced TOML example parses |
| `plugins/doctrine/skills/plan/SKILL.md` | routes at extraction (step 3), satisfaction and cross-check (step 4), review (step 7); step-4 `specs`/`requirements` line corrected |
| `plugins/doctrine/skills/phase-plan/SKILL.md` | one pointer (step 5, filling the sheet) |
| `plugins/doctrine/skills/execute/SKILL.md` | one pointer (step 10, design ambiguity / variance) |
| `plugins/doctrine/skills/consult/SKILL.md` | one pointer (outcome capture) |
| `plugins/doctrine/skills/audit/SKILL.md` | one pointer (step 2, gather evidence) |
| `install/glossary.md` | slice-directory layout row; "commitment ledger" term |
| `install/using-doctrine.md` | storage-tier slice example |

Installed copies (`.doctrine/skills/`, `.claude/skills/`) refresh via
`doctrine install` from the built binary; they are not edited by hand.

<!-- doctrine:section sec-8 -->
## Verification

- **VT** — `src/install.rs` test: `commitment-ledger.md` is embedded and
  published, and the fenced `toml` block in it parses. Keeps the shipped example
  honest without a reader.
- **VT** — `doctrine check gate` green: existing shipped-asset, reachability and
  skill-dedup tests stay green unchanged.
- **VA** — shipped-text sweep over every edited `install/` and `plugins/` file
  with the full id-prefix set: no repo-private ids or `src/` paths.
- **VA** — `doctrine library show lib:reference/commitment-ledger.md` resolves
  after `cargo build` + `doctrine install`; each of the five skills' pointers
  names that address.
- **VA** — read-through: the doc and pointers agree with `DEC-415`–`DEC-420`;
  no skill restates the procedure.

The trial on a real plan is a follow-up, tracked in `RFC-029` § 9, not a
closure condition.

<!-- doctrine:section sec-9 -->
## Risks and residuals

- **Extraction becomes the paraphrase site.** Mitigated by verbatim quotes and
  the `commitments` reviewer checking quote fidelity; the follow-up trial
  measures whether cited-but-wrong actually shrinks.
- **Ceremony.** The multi-phase trigger may be too broad for small plans;
  revisit with trial evidence.
- **Unenforced.** Nothing checks the ledger exists or is current. Accepted:
  `DEC-101`'s authoring rule puts this in guidance until a consumer earns a
  check (`IMP-409`, `SL-258`).
- **Drift from `plan.toml`.** Criteria ids are immutable, so ledger references
  do not rot; renamed or withdrawn criteria still need a ledger edit.
- **Overlap with `SL-258`.** That slice mechanises extraction and admission;
  this one is its unmechanised precursor and evidence source.

