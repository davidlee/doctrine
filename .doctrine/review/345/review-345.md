# Review RV-345 — reconciliation of SL-244

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed.** The primary tree at `edge`, HEAD `a891463b`. SL-244 ran
**solo**, not dispatched — there is no candidate interaction branch, and the
`review/*` / `phase/*` evidence-ref caveat does not apply. Evidence is the eight
recorded phase boundaries in `.doctrine/state/slice/244/boundaries.toml` and the
landed tree they span (`d50777cd..3aed3918`).

**Lines of attack.**

1. **Is the mechanical conformance signal itself trustworthy?** SL-244 recorded
   a memory *during its own PHASE-06* saying a clean conformance read can mean
   *nothing was checked*. First move is to audit the registry before believing
   anything it reports — every boundary row checked against the phase's actual
   commit span, and pollution from interleaved foreign commits named rather than
   read as scope creep.
2. **Did the declared design targets all get delivered?** The `undelivered` cell
   is the slice's own promise list. `design.md` sec-6 states an obligation to
   `SPEC-029` in as many words; a declared target that never moved is either a
   dropped deliverable or a stale declaration, and the audit must say which.
3. **Do the criteria the slice itself flagged for adjudication hold?** `notes.md`
   § Open hands `/audit` a numbered list of departures from the letter of
   `EX-`/`VT-` rows (PHASE-01 ×2, PHASE-03 ×2, PHASE-04 VA-1, PHASE-05 VT-8,
   VA-3, EX-6b) plus one criterion whose evidence is *recorded, not reproducible*
   (PHASE-02 VA-1). Each is a finding candidate; a departure the slice argued
   well is `aligned`, not skipped.
4. **What did the slice ship broken and know about?** `EX-14` broke the snapshot
   shape once, deliberately and with the user's price agreed. `ISS-315` reports
   a second, undeclared break in the same class. The invariant to pin: a closed
   vocabulary that retires a member owes its own history a read path, and the
   slice establishes that pattern (`serde(alias)`) at one site while omitting it
   at another.
5. **Is the shipped corpus reachable and self-consistent?** Nine contract
   narratives, one generated diagram, twenty manifest rows — the whole point of
   objectives 5 and 6 is that a reader with no source can answer *what does this
   edge require*. Verified at the surface a client repo would use, not at the
   embed.

**Invariants held to.** ADR-001 (leaf ← engine ← command, `design_run` crate
out-degree zero); STD-001 (single-source named constants — the macro's table is
the single source and every projection is derived from it); POL-002 (no host
convention in the engine); ADR-009 (lifecycle: phases complete → `audit`); the
project's no-parallel-implementation rule, which the slice's own `VA-2` rows
already hunt for and which this audit re-runs across phase boundaries rather
than within one.

## Synthesis

### The closure story

SL-244 set out to make a design-run gate condition state its own contract, and
it delivered a bigger thing than that: the condition vocabulary, the rules that
discharge it, the act records that satisfy it and the evaluator that reads it
are now one macro-generated table, and every other surface in the system is a
projection of that table rather than a second copy of it. Eight phases, all
`completed`; the full gate is green (`doctrine check gate`, 116 suites, zero
clippy warnings); all 40 `VT-` rows PASS under `slice verify-vt`; conformance
reads 42 conformant, 1 undelivered, 68 undeclared of which none is code.

The structural claim worth recording is the single-source one. Nine conditions,
nine narrative assets, twenty publication rows, one refusal remedy, one
stage-entry receipt and one published stage diagram all derive from `CONTRACTS`,
and the slice made disagreement *unrepresentable* rather than tested-for: a
lawful edge the macro's source omits fails to compile, a vocabulary change breaks
the artefact golden until the file is re-rendered, and `remedy()` and the
receipt's injected discharge are asserted equal over every row rather than a
sample. That is `STD-001` reaching further than usual, exactly as the design
said it would at line 17.

The acceptance test is `VH-1`: a reader who has seen no source can say what
`exploring → inquiring` requires and how to discharge it. It holds at the
surface a client repo would actually use — `doctrine library show
reference/design-run-stages.md` returns the diagram, `library tree` lists the
`conditions/` corpus, and the shipped corpus cites no repo-private `DEC-`/`SL-`/
`ISS-` id (checked with a positive control, which fires on `design.md` and not
on the corpus). The measured problem the slice opened with — 33 reads of
`src/design_run/**` in one design run, four of them hunting gate conditions in
source — has its answer shipped rather than described.

### What the audit found, and the shape of it

Nine findings, all terminal: two fixed in-audit, one delegated to `/reconcile`,
one to a new backlog item, two tolerated with named owners, three
read-and-dismissed. No blockers, so the close-gate does not bite.

The nine are not evenly weighted, and the audit should say which matter. `F-8`
is the one worth acting on and it is not about this slice. `F-2` is the one this
slice owes. `F-4` is the one it shipped knowingly. The rest are hygiene.

Four of the eight cluster on one theme, and it is the theme worth carrying out
of this slice: **the slice's own verification signals were less trustworthy than
the code they were reporting on.** `F-1` — the PHASE-08 boundary row was
truncated to its last commit, so conformance called the phase's headline
deliverable undelivered and `verify-vt` could not attribute the criterion
asserting it was reachable. `F-3` — with that row truncated, the slice's one
genuinely undeclared code path was invisible; it only surfaced once the registry
was correct. `F-7` — a criterion whose subject is gitignored runtime state left
evidence that cannot be re-derived, and the window in which it could have been
re-run closed at PHASE-05. Nothing in this cluster falsifies the implementation.
All of it degrades the audit's ability to *know* the implementation is sound,
which is a different and quieter failure.

The initial reading of `F-1` was that the practice memory SL-244 recorded at
PHASE-06 — "a clean read can mean nothing was checked" — was written and then
not adopted three phases later. **Root-causing it (`F-8`) overturned that.** The
check *was* run: the binding's boundary-span advisory fired at the PHASE-08 flip
and was acted on. It prescribed `record-delta --commit <tip>`, and `--commit`
records exactly `[S^, S]` — so following the advisory verbatim truncated a
seven-commit phase to one. The advisory returns early when a span is under two
commits, so it fires *only* where its own remedy is wrong. There is no input for
which the printed command is correct.

That is worth stating plainly because it inverts the lesson. This was not an
agent skipping a step; it was an agent following the tool's own instruction into
a trap, and trading a *visible* error (a too-wide boundary over-reports into a
cell readers dismiss) for an *invisible* one (a too-narrow boundary silently
drops real deliverables and criteria). Filed as `ISS-317`. The practice memory
was right all along — its step 3 distinguishes the two modes correctly — and the
CLI contradicts it at the exact moment an agent is reading the CLI.

The second theme is narrower and equally durable: **retiring a member from a
closed serde vocabulary retires it from the write path, not the read path**
(`F-4`). The slice demonstrates the correct handling twice — the
`evidence_invalidated` alias on `ChangeEvent::ActInvalidated`, and the
`RecoveryIntent` subject alias removed at `29c12e05` and restored at `7e2b768d`
once it broke this repo's live runs — and omits it once, at PHASE-04's
retirement of `IntegratedReviewRecorded`. The result is that the binary this
slice ships cannot read the design run this slice conducted. Blast radius was
measured rather than assumed: of four live runs, only SL-244's own fails.

### Tradeoffs consciously accepted

- **The snapshot shape broke once, deliberately** (`EX-14`), priced with the
  user on 2026-08-04. SL-243 owes a five-act hand repair at its next forward
  move. The criterion's requirement — that the break happen *once*, with the
  refusal naming the repair — held.
- **A user-visible render change shipped**: `SectionRow` no longer carries
  `clearances=`, which retired with the store it read. Stated in `EX-11` rather
  than discovered in a diff, and it takes an uncapped list out of the envelope
  budget in exchange.
- **One assertion is deferred to `IMP-392`** with its attribution recorded, as
  `VA-3` required: a `Conducted` disposition admitted over a *concluded* ledger,
  which needs a marker no verb sets. The other two clauses `VA-3` predicted might
  land live did land, and the notes correct the record where the prediction only
  half held.
- **`review.findings` is write-only** after PHASE-05 and is left standing;
  adjudicated at `VA-2` and noted on `IMP-392`, where its removal is now a plain
  deletion rather than a migration of a live consumer.
- **No external adversarial round saw the repairs.** `RV-344` pass 2 was
  specified and, on the user's call, not run — its four lines of attack were
  carried into implementation instead. This audit re-ran the
  parallel-implementation line across phase boundaries and found no second
  spelling of a comparison, a derivation or a vocabulary; that is one lens, not
  the whole pass, and the residual is the user's stated position rather than an
  oversight.

### Standing risks

1. **The boundary-repair path is a live trap** (`F-8`, `ISS-317`). It will
   truncate the next multi-commit phase boundary someone repairs, and truncation
   is silent — until it is fixed, any conformance verdict in this repo may be
   clean because nothing was measured. This is the highest-value thing this
   audit found, and it is not about SL-244.
2. **The undeclared cell's noise floor hides real rows** (`F-5`, `F-3`). 68 of
   69 rows were authored knowledge or a concurrent agent's commits. A reader who
   skims that cell will miss the one code path in it — which is what happened.
3. **`ISS-315` leaves this slice's own design run unreadable** (`F-4`). Harmless
   to authored state, but it is the run governing the work now closing.
4. **One criterion's evidence is unre-derivable** (`F-7`), and the pattern that
   produced it — a `VA-` row whose subject is gitignored state — is not unique to
   PHASE-02.

### Available simplification, not owed work

The review-policy path is the last writer act that validates an
`AcceptanceDeclaration` without binding it (`F-6`). PHASE-05 built the machinery
that would let it, but the policy is a scalar on the run header with no record
to hold an attestation, so migrating it means giving the policy a record. That
is a design change with its own cost; recorded here so that whoever revisits
`ReviewPolicy` finds the reasoning rather than re-deriving it.

## Reconciliation Brief

### Per-slice (direct edit)

- **`design.md:19`** — the governance-context line characterises `SPEC-029` as
  describing "evidence as payload-claimed". It does not: `grep -in claim
  spec-029.md` returns one unrelated hit (line 124, about test seams), against a
  positive control of three hits for `gate table`. The claimed-evidence model
  lived in SL-233 design prose and in source, never in the spec. Correct the line
  to say what `SPEC-029` actually owes — that it owns the gate table and is
  silent on how a condition is satisfied — so the spec revision below is scoped
  as additive. (`F-2`.)

- **`notes.md` § Open** — three entries are now settled by this audit and should
  read as settled rather than as questions for a future reader. (i) "`/audit`
  should decide whether PHASE-05's act record adopts the attestation (sheet D2)"
  → it does; `CheckpointAct`'s constructor routes the declaration through
  `AcceptanceAttestation::bind`, and PHASE-03's `EX-5` departure is `aligned`.
  (ii) "`ISS-315` — `design show 244` is still broken" → confirmed still live
  against the built binary, blast radius measured at one run of four, and the
  in-tree precedent for the fix named. (iii) the four criterion departures held
  open for `/audit` (PHASE-01 ×2, PHASE-03 `VT-1`, PHASE-08 `D2`) → all
  `aligned`, each verified at its named file and line. (`F-6`, `F-4`, `F-9`.)

- **`notes.md`** — the PHASE-03 departure entry names
  `Refusal::SectionsUnreviewed`. The type is `Cause::SectionsUnreviewed`, a
  member of the `Vec<Cause>` an `Unmet` carries; there is no such `Refusal`
  variant. Worth correcting because a reader will grep the name they were given.
  (`F-9`.)

- **`notes.md` § Open** — add the PHASE-08 boundary-row truncation, its two
  spurious signals, the correction command, and its root cause (`ISS-317`: the
  binding's own advisory prescribes the one `record-delta` mode that truncates a
  multi-commit span). The next reader of this slice's conformance output should
  find the repaired numbers *and* the reason, not re-derive either. (`F-1`,
  `F-8`.)

### Governance/spec (REV)

- **`SPEC-029` (tech, Design run engine) — REV modify.** Declared a
  `design-target` selector; `design.md` sec-6 § *Where the citation points*
  states the obligation in as many words ("owns the gate table and is revised by
  this slice regardless"). No revision landed; it is conformance's sole remaining
  `undelivered` row. The revision is **additive** — D1 ("the gate table is a
  `const fn` table, not a matcher") stands, and nothing in the spec body is
  falsified. What is owed:
  - `responsibilities` gains the mechanisms this slice built: the const contract
    table keyed by the closed `Advance` forward relation; the attested-act ledger
    (`CheckpointAct` / `AgentDeclaration`) that replaced the existential evidence
    scan, with its two-kind classification (Derived / Attested, no Claimed tier);
    the per-run `ReviewPolicy` and its required reviewer lanes; the `ReviewPass`
    RV minted on entry to `reviewing` through the journalled-intent seam; and the
    condition contract corpus with its stage-entry receipt.
  - a citation of the published address `reference/design-run-stages.md`, per
    `DEC-127` — the spec is the repo-private artefact and the diagram the
    reachable one, so the private document points at the public one and not the
    reverse.

  No requirement status moves: `SPEC-029`'s members (`REQ-428`..) and `PRD-019`'s
  `REQ-422` / `REQ-425` were checked and none is falsified or newly satisfied by
  this slice. (`F-2`.)

### Not brief items — recorded so `/reconcile` does not adopt them

- `ISS-315` (`F-4`) is owned by the backlog item, not by a slice artefact. Its
  resolution decides what a closed change-event vocabulary owes its own history,
  which is a governance question with three live candidates — a `/consult`, not
  a reconcile edit.
- `ISS-317` (`F-8`) is a defect in `src/state.rs`'s phase-binding advisory,
  pre-existing and outside SL-244's selectors and design. Filed, related to
  `IMP-175`, and deliberately not repaired in-audit: fixing it here would be the
  undeclared cross-subsystem edit conformance exists to catch.
- The `slice-244.toml` selector registry (`F-3`) and the
  `.doctrine/state/slice/244/boundaries.toml` row (`F-1`) were repaired in-audit
  through their own verbs. Runtime state and the selector registry are not
  reconcile write surfaces.
- `src/design_run/run.rs` and `src/design_run/submission.rs` comment corrections
  (`F-6`) landed in-audit as `fix-now`. Gate re-run green afterwards.
- No `plan.toml` edit is proposed anywhere in this brief. `PHASE-NN` and
  `EN-`/`EX-`/`VT-` ids are immutable-append and off the reconcile surface; the
  three criteria this slice amended mid-flight (PHASE-04 `VT-4`, PHASE-06
  `VT-5`→`VT-6`, PHASE-07 `VT-2`→`VT-4`) all appended rather than renumbered and
  each carries its dated rationale and a `DEC-` id.
