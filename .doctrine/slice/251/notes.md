# Notes SL-251: Acts carry their own payload contract

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage (exploring, 2026-08-13)

`explore.triage`'s record. Evidence lives in `research/research.md`; this is the
shape of the decision space, not a restatement of it.

### Constraining governance

| Rule | How it binds |
|---|---|
| `DEC-123` | contract *structure* rides a const table beside the existing tables; no third loader. |
| `DEC-122` / `DEC-102` | if prose ships it ships sealed — an override of a claim the engine enforces is false, not different. Keeps `IMP-372`'s override seam out. |
| `DEC-101` | totality template; a closed set describing itself narrows nothing. |
| `STD-001` | two renderings must render from one source, not restate each other. |
| `DEC-064` | orthogonal while the contract is *fetched*; re-binds the moment it is injected into the turn envelope. |
| `ADR-001` | **corrected 2026-08-15.** The triage said "const table = engine tier". It is not: `layering.toml:31` classifies `design_run` as **leaf, out-degree 0, "std + serde derive only"**. So the contract table and its renderer must be pure and import nothing from the crate; asset embedding, publication and CLI attachment sit above it in command tier. This shapes where the generator lives, it does not block it — `condition_vocabulary!` already lives in `design_run/gate.rs` under the same constraint. |
| `ADR-019` | **corrected 2026-08-15.** The triage said "not engaged". True of the *embed-root* leg — `install/` is already a RustEmbed root, so no `flake.nix` graft — but `DEC-224`'s published doc and `DEC-226`'s `customization = "fixed"` engage the **publication** leg: a `publication/manifest.toml` entry with a `CustomizationStatus` parsed fail-closed (`publication.rs:394`). Publication policy is engaged; asset-policy independence is what keeps the graft out. |
| `SPEC-029` | one line of prose ("start, show, apply, resume") goes stale if a fifth verb ships. Prose update, not an amendment. |

### Open questions

The inquiry map holds them: `inq-1` (what the contract consists of) over `inq-2`
renderings / `inq-4` row richness / `inq-5` drift pin / `inq-7` discoverability,
with `inq-3`, `inq-6`, `inq-8`, `inq-9` downstream.

Scope `OQ-1` is re-framed by evidence: not authored-vs-generated but **tier 2
(serde-key-set equality against an exhaustive no-`..` literal) vs tier 3
(single-source generation)**. Tier 1 — the hand-written test-copy set-equality
the payload axis has today — is excluded: it is blind to a new field, which is
the failure it claims to prevent.

Scope `OQ-2` gains a third option research surfaced: a generated, embedded,
published reference doc on `artifact.rs`'s pattern — the only form that reaches
an installed client project with no source to read.

### Risks

- **R1 (scope)** — a second description that can go stale. Answered by the pin
  choice at `inq-5`, not by discipline.
- **R2 (scope)** — fetchable but not found. `X5` gives a cheap answer: refusals
  already name payload fields, and `SL-244`'s `Contract::remedy` renders remedy
  text from the const table, so refusal and contract cannot disagree.
- **R3 (new)** — tier 3's measured cost: clippy does not lint tokens a macro body
  wrote (`gate.rs:459-462`). A generated contract table trades drift-proofness for
  lint blindness over the generated region.

### Assumptions

- **A1 (scope)** — no v1 envelope wire change. Now supported twice over: the
  envelope runs a byte budget and the existing worked example is capped at 1024
  bytes, so a full act contract cannot ride the turn regardless of `DEC-064`.
- **A2 (new)** — `delegation`'s absence from `WRITER_ACTS` is correct for that
  table's purpose (it answers "does this payload write", and the proposal channel
  must not). So the contract surface must not key off it.

### Shaping decisions already made by evidence

- Sealed, not overridable, if prose ships (`DEC-102` + the neighbouring axis's
  `customization = "fixed"` condition assets).
- ~~The unknown-key hole is exactly one level deep~~ — **false, superseded
  2026-08-15 by the self-review pass (`fnd-8`).** Those three types are the only
  ones that deny; the other nine wire structs discard silently. `ISS-333`'s
  discharge can still be stated precisely at close, but it is a statement about
  three types refusing and nine not.

### Corrections found while exploring

- `mem.fact.design-run.apply-payload-vocabulary` states "neither `ApplyRequest`
  nor `Declaration` denies unknown fields". The second half is **wrong** —
  `Declaration` carries `deny_unknown_fields` (`submission.rs:123`). Correct the
  memory at harvest.
- The same memory cites `ISS-346` for the silent-discard; the slice scope cites
  `ISS-333`. Reconcile which is canonical before close.

## Self-review pass (2026-08-15, run rev 41–43)

Nine findings raised on the run (`fnd-1`..`fnd-9`), all dispositioned; the
integration is in the sections themselves. Ids only — the run holds the text.

- **Facts, not judgement, dominated.** Five of the nine were claims the draft had
  never checked against source: the top-level key count (thirteen, not twelve —
  and "nine act fields" had silently reproduced `WRITER_ACTS`'s row count, which
  is the very asymmetry Objective 3 exists to resolve), the struct closure
  (twelve, listed, against a stated thirteen), two enums missing from the closure
  (`ReviewPolicy`, `Reviewer`), a fifth file the closure crosses, and a deeper
  chain through `DelegationAct::Propose`.
- **Two were modelling gaps.** `Dispose` is internally tagged with a *newtype*
  variant, so `CreateRecord`'s keys inline beside `form` — the model had no way
  to say that, and `sec-5`'s sample would have taught the wrong shape.
  `VariantContract` was referenced and never defined.
- **One was load-bearing and wrong.** "The hole is exactly one level deep" — only
  three of twelve wire structs deny unknown fields; nine discard silently,
  `TraversalDeclaration` and its `cursor` among them.

**What a further pass should probe.** Three areas this one did not reach:

1. **The renderers.** `sec-5` fixes the line shape and `sec-2` the model, but no
   full rendering of the real closure has been written down — `--format prompt`
   over twelve structs and fourteen enums is where `R5` (size) and any remaining
   presentation gaps would show. A worked fragment would settle both.
2. **`sec-8`'s pins against a real fixture.** The pin ladder is argued, not
   exercised. Whether `assert_keys_described` can stay one generic body across
   twelve types with different `Serialize` shapes is unproven.
3. **Governance re-read at the finished artefact.** `ADR-001`, `STD-001` and
   `POL-002` were applied while drafting; nothing has re-read the whole design
   against them since the sections settled. That is the pass an external
   adversarial reviewer is best used for.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-16 · **closed, then reconciled a second time** · two audits,
`RV-362` (12 findings) and `RV-361` (9), the first recovered from an unmerged
capsule ref after close and its four delegated findings discharged

### Produced

**Execution, PHASE-01..07 — all seven complete.**

- `b36cd84f1` (PHASE-01), `836662efa` and `8d2845ac5` (PHASE-02 — the second
  records `VT-2`'s waiver reason), `017c59742` (PHASE-03). `4663ce278` is an
  incidental `skills-lock.json` hash refresh that falls inside PHASE-02's
  recorded boundary; `00295be85` carries thirteen friction observations.
- `cc64a24c2` (PHASE-04), `01f757571` (PHASE-05), `cba1f257f` (PHASE-06),
  `d7001375c` (PHASE-07), each preceded by its plan amendment — `34935b318`
  (PHASE-05 `VT-3`), `be5605b2d` (PHASE-06 `VT-1` re-sited, `DEC-226` pinned),
  `015099119` (PHASE-07 `EX-10`). `b90577f75` harvested; `eca2c9a11` carries the
  PHASE-04..07 friction observations. `be679d544` is the capsule driver's own
  commit and is `RV-361` `F-7`.
- Landed on `edge` at `eddb971c1`, which also resolved an id collision: `edge`
  and `sl-251` both allocated backlog ids 434 and 366 after the merge base, so
  this slice's two items were re-minted as `IMP-438` and `ISS-439`. Anything in
  this slice's record naming `IMP-434` before that merge means `IMP-438`.
- `doctrine check gate` exit 0 on the merged tree. `doctrine slice verify-vt 251`
  is **clean, two waivers, no FAIL** — `PHASE-06/VT-1` named the implementation
  token `CARGO_MANIFEST_DIR` that the golden legitimately routes through
  `test_support::repo_root()`. `RV-361` `F-1` adjudicated the FAIL `aligned` and
  left the criterion red; `RV-362` `F-1` retired it with its reason and appended
  `VT-3` on the seam the code actually uses, which is what cleared it.
- **Two audits, not one.** `RV-362` ran first (2026-08-16 04:31, twelve findings)
  on the capsule ref `refs/capsule/d/heads/work`, which was never merged; `RV-361`
  ran second (18:51, nine) on `edge`, blind to it, and the slice closed under
  `RV-361` alone. The first ledger was recovered afterwards and landed as
  `RV-362` — id 358 was already `SL-238`'s design review, so the ledger is
  re-minted, not merged. Its four delegated findings were discharged in a second
  reconcile pass; see *Open*. The two agree everywhere they overlap.
- Seven criteria appended and one VT retired **at execution**: `PHASE-01/EX-9`;
  `PHASE-02/EX-10`, `EX-11`, `VT-4`; `PHASE-03/EX-10`, `EX-11`; `PHASE-07/EX-10`;
  `PHASE-02/VT-2` waived. Ids only — the whole rationale for each is authored in
  `plan.md`'s per-phase execution headings and in the criterion text in
  `plan.toml`. Criteria ids are immutable: append, never renumber.
- The design corrections `/reconcile` owes are enumerated in `plan.md` (the
  `## Notes` section plus the three execution headings). Not restated here, and
  not implementation work.

**Design run.**

- `DEC-219` `DEC-221` `DEC-224` `DEC-225` `DEC-226` `DEC-227` `DEC-228` `DEC-229`
  — the inquiry's eight decisions. `DEC-229` partially supersedes `DEC-221`
  (enums only; struct half stands).
- Run sections `sec-1`..`sec-9` — declared and materialised, all 9 outstanding
  review. `sec-2` amended during `sec-7` drafting (`Cow` slices +
  `WireType::Token`).
- Eleven `design-target` selectors recorded (`draft.selectors` runbook step).
- Self-review pass: `fnd-1`..`fnd-9` raised and disposed on the run; `sec-1`
  `sec-2` `sec-3` `sec-4` `sec-5` `sec-6` `sec-8` revised to integrate them.
  `RV-357` minted by the stage move — empty, and stale against the revised
  sections.
- `DEC-228` corrected at source (choice facet + body); the reviewing runbook's
  three steps discharged.
- Scope reconciled: `OQ-1` `OQ-2` `OQ-3` `R2` resolved against their records;
  Objective 2's "authored, not derived" contradiction corrected.
- `.doctrine/slice/251/render-sample.md` — the owed full-closure rendering.
  Evidence beside the design; no section, no attestation. Closure **derived from
  source** (rev `ef47e756b`) then diffed against `sec-3`, per `sec-3`'s own
  commitment to mechanical derivation. `R5` measured: **202 lines / 8 826 B**.
  **The plan cites it nowhere**, yet it is the authoritative per-key reference —
  load-bearing for PHASE-02 (`PHASE-02/EX-11` reads presence off it), and the
  thing to check before inventing a shape for PHASE-05's renderers.
- `fnd-10`..`fnd-20` raised from that diff and **all dispositioned** (rev 60);
  `fnd-19` withdrawn as mis-framed and superseded by `fnd-20`. Ids only; the run
  holds the text and the resolutions.
- **Second self-review pass integrated at rev 52–61** — `sec-2` `sec-3` `sec-4`
  `sec-5` `sec-7` `sec-8` `sec-9` revised. The model gained `VariantPayload`,
  `MapKey`, `TokenSource` and `ExternRegion`, and **lost** `Tagging::Bare` and
  `Tagging::NotTagged` (both derivable; `tagging` moved inside `TypeForm::Enum`).
  `R5` retired as measured rather than carried.
- **Third self-review pass integrated at rev 62–63** — `fnd-21`..`fnd-26` raised
  and all dispositioned; every section but `sec-1` revised. `sec-3`'s extern
  region stopped being a `TypeContract` and became a `SelectorTable`;
  `UnknownKeys` gained `StoredThenFlagged`; `WireType::Id` gained its admissible
  `IdKind` set; `sec-8` pin 1 split eleven-by-contract from the twelfth by
  root composition. `Cow` **removed** from the model entirely, along with its two
  `const fn` constructors and the `Box::leak` / const-fn-assembly alternatives.
  `A3` retired as false-premised. Ids only; the run holds the text.
- **`ISS-362` put to the run and answered in `sec-3`** — the contract states that
  it is total over the payload and silent on stage admissibility, and names the
  state machine as where that lives. Not a stage column: `CONTRACTS` is keyed by
  `ActKind` (the eight attestation acts), a disjoint vocabulary from the ten
  payload act fields, so there is no map to render and authoring one would
  promise refusal at a seam that fails open.

- **Neighbouring items folded in as closure statements at rev 64** — `sec-2` and
  `sec-8` record `ISS-355` (a *correct* submission prints no change row either, so
  the missing row is not a signal and the disclosure has to ride the contract
  rather than the response); `sec-6` records that `DEC-225`'s parse site is clean.
  Neither is a scope change.
- **`RV-357` concluded — external adversarial pass, five rounds, 14 findings, all
  verified (`done · await=none`).** Round 1 raised `F-1`..`F-12` (rev 66); round 2
  verified eight, contested four and raised `F-13` `F-14` (rev 67); round 3
  verified five and contested `F-6` (rev 68); rounds 4 and 5 contested `F-6` again
  (revs 69, 70). Every finding was checked against source before disposition; none
  was confabulated, and every point after round 1 was a defect in the *integration*
  rather than in the reviewed design. Ids only; the ledger holds the text.
  - **`F-6` alone took five rounds and four repairs**, each closing the case it
    was shown: the assertion's direction, then its domain, then empty containers,
    then an untagged `Shape` no key led to. Reworked at rev 71 as a class fix and
    reconciled at rev 72 — see Learned.
- Revs 68–72 landed on `sec-4` `sec-7` `sec-8` `sec-9`; `sec-1`..`sec-3`, `sec-5`,
  `sec-6` untouched since rev 67.
- `mem.pattern.review.general-rule-must-absorb-what-it-subsumes`
  (`mem_01a004d882d37b31b9cb2b67307db698`) — the generalise-then-sweep pattern,
  recorded from `F-6`.
  - `F-1` deleted `UnknownKeys::StoredThenFlagged` — the design had the facet
    write seam backwards. `sec-8` gained pin 10 (`F-4`, no repo-private id in
    shipped output) and lost pin 7's drift role (`F-3`, the pointer is
    single-sourced through `long_about = format!(…)`, so `STD-001` needed no
    deviation). `sec-4`'s `payload_variants!` now consumes six existing `as_str`
    authorities (`F-2`).
  - `sec-4`'s ladder table is the current index of what each rung holds; it
    gained rows for both `TypeContract::name` seams, split presence into `Sparse`
    and requiredness, and added the citation detector.

### Learned

**Execution, PHASE-01..03 — traps a later phase will otherwise walk into.**

- **`doctrine slice record-delta 251 PHASE-NN --start <sha>^ --end <sha>` after
  each phase commit is mandatory, not bookkeeping.** `verify-vt` builds its
  attributable-file set from the source-delta registry and nothing else
  (`slice.rs:884-899`); with no row for a phase, every VT criterion of that
  phase reports `UNATTRIBUTABLE` however exactly its keywords and selectors
  match, because the file-in-delta test precedes the keyword test
  (`vtgate.rs:120-126`). Nothing in the plan or the skills says so. Recorded as
  [[mem.pattern.slice.record-delta-before-verify-vt]]. The `<sha>^..<sha>`
  single-commit form is the common case, not the rule — PHASE-02 spans two
  commits and its row is `4663ce278..8d2845ac5`.
- **`WireType::Token(TokenSource::Fixed(_))` has no reachable input yet.** No
  `Fixed` row exists in the shipped `PAYLOAD` table; every construction site is
  under `cfg(test)` (`payload_contract.rs:2413` in `EXEMPLAR_ROOT`, and
  `2649`/`2668` inside `every_extern_region_resolves_to_its_own_supply`).
  PHASE-04 supplies the first real one. The arm reads as dead code and is
  **deliberately kept** — `PHASE-01/EX-1` forbids a wildcard — and is commented
  as such in source. Do not delete it.
- **`payload_contract.rs`'s `EXEMPLAR_*` set is not the real table.** It
  declares the root `Refused` and exists as the input to
  `the_model_states_the_four_shapes_the_closure_forced`
  (`payload_contract.rs:2539`). It must never be merged into `PAYLOAD`.
- **The two `#[expect(dead_code)]` on `PAYLOAD_CONTRACT_POINTER` and
  `PAYLOAD_CONTRACT_PATH` (`payload_contract.rs:1532`, `1540`) must keep
  firing until PHASE-06/07 read them.** Under `warnings = "deny"` an `expect`
  that *stops* firing is itself a hard error, so removing each attribute is
  part of the phase that lands its first reader — not optional tidy-up, and not
  safe to do early. Each attribute's `reason` names the phase that owes it.
- **Line citations into `submission.rs` decay *within this slice*.** PHASE-02's
  fixtures displaced everything `design.md` and `plan.toml` cite by line, and
  PHASE-03 displaced them again: `PHASE-02/EX-10` cites
  `CheckpointActDeclaration` at 825 and `AgentActDeclaration` at 855, which are
  now 1043 and 1095; the `Declaration` / `CheckpointActDeclaration` /
  `AgentActDeclaration` triple recorded under *Payload facts* below has drifted
  the same way. Re-resolve every citation by grep. Never trust a cited number
  in this slice's artefacts, including the ones written to record the drift.
- **All twelve closure structs are defined in `src/design_run/submission.rs`;
  `attestation.rs` holds none of them** — `CheckpointActDeclaration` included.
  `attestation.rs` contributes the closure's *enums* plus `ReviewRef`, and it
  holds `CheckpointAct`, the near-namesake that makes the mistake easy. `sec-7`'s
  touch table and `sec-8` pin 1 are wrong about this, which is what made
  `PHASE-02/VT-2` void on contact rather than merely unmet — the criterion
  mandated fixtures in `attestation.rs` while naming a `submission.rs` type as
  its keyword. **Expect the same error to void a PHASE-04+ criterion**; treat it
  as evidence, adapt, append a criterion recording the disposition, and leave
  the correction to `/reconcile`.

**Design run.**

- Triage errors corrected in the governance table above: `ADR-001` (`design_run`
  is leaf out-degree 0, not engine) and `ADR-019` (engaged on the publication
  leg, not the embed-root leg).
- `DEC-224` body carried a wrong line ref; corrected to `guard.rs:430`.
- `WireFacetValue` is `#[serde(untagged)]` — a fifth tagging mode the `sec-2`
  model initially missed.
- `knowledge.rs:860-875,1028` — `facet_fields` / `FieldShape` already publish a
  contract-shaped description of the knowledge tier; `sec-3`'s `Extern` injects
  it. ~~Enforced post hoc by `doctor_checks.rs:161`, not at the write seam.~~
  **Wrong, and it was load-bearing — corrected via `RV-357` `F-1`.** It is
  enforced **at** the write seam: `plan_facet_edits` refuses
  `FacetEditRefusal::UnknownField` (`knowledge.rs:1233-1247`) from inside
  `plan_checkpoints`, before `execute_mint` and before an id is reserved
  (`commands/design.rs:985`). `doctor`'s inert-facet check is the **authored
  corpus**, and its doc comment says so: it is a warning there "because the
  design-run seam's equivalent is a refusal" (`doctor_checks.rs:141-148`).
  Reading a corpus-hygiene warning as a wire-seam behaviour is the mistake.
- The braced pattern `Variant { .. }` is uniform across unit / tuple / struct
  variants (compiler-verified) — the basis of `DEC-229`.
- `artifact.rs:373-375` — a generated-asset golden must read disk-source, never
  the embed (`install/` has no `rerun-if-changed`).
- `asset_source.rs:132` compels publication of any new `install/` asset.
- **Payload facts, traced against source rather than recalled** (the self-review
  pass; each had been stated wrongly in the draft): `ApplyRequest` carries
  thirteen top-level wire keys — three flattened plus **ten** act fields, against
  `WRITER_ACTS`'s nine rows. The wire closure is **twelve struct types and
  fourteen enums**, spanning five files (`submission`, `attestation`, `inquiry`,
  `traversal`, `mod`). Only **three** structs carry `deny_unknown_fields`
  (`Declaration` 123, `CheckpointActDeclaration` 824, `AgentActDeclaration` 854);
  the other nine discard silently, `TraversalDeclaration`/`cursor` included.
  `Dispose` is internally tagged with a **newtype** variant, so `CreateRecord`'s
  keys inline beside `form`. `DesignId` (`ids.rs:132`) and `ReviewRef`
  (`attestation.rs:475`) serialise as scalars and are not struct rows.
- **`IdKind` has eight members** (`ids.rs:30-52`), and `declare` admits **five**
  of them as a subject — inquiry, section, attestation, finding, checkpoint —
  refusing `dlg-` / `cpa-` / `agd-` as engine-allocated (`run.rs:1139-1150`).
- **clap's derive does take an expression** — `compare.rs:75` passes
  `clap::ArgGroup::new(…)` into `#[command(group = …)]`; of eighty `#[command]`
  attributes, 46 are `subcommand`, 30 `flatten`, and 4 others. ~~What blocks
  single-sourcing the `--help` pointer is that `long_about` *replaces* a doc
  comment and `concat!` cannot splice a `const &str` — a cost, not a limit.~~
  **Neither, in the end — corrected via `RV-357` `F-3`.** Both premises are true
  and the conclusion did not follow. `Command::long_about` takes
  `impl IntoResettable<StyledStr>`; `String` satisfies it through the blanket
  `impl<I: Into<StyledStr>>` (`resettable.rs:190`, `styled_str.rs:161`), so
  `long_about = format!("{CONST}…")` single-sources with no function and no
  `concat!`. Doc-comment methods emit **before** explicit attribute methods
  (`clap_derive item.rs:980`), so an explicit `long_about` wins while a doc
  comment still supplies `about`. Cost: two attribute lines.
- **The error class both of the above belong to** — an unverified claim about
  what a *tool* cannot do, load-bearing on a design decision. `A3` was the first
  instance (clap can't take an expression → bought an unnecessary pin); its
  *cost* estimate was the second (→ nearly bought an unnecessary `STD-001`
  deviation). Both read as settled reasoning on the page, and both cost minutes
  to check. Cite the source whenever asserting a limit of clap, serde or cargo.
- **Requiredness is a read-path property; serialization is never its oracle.**
  `ApplyRequest.declare` carries `#[serde(default)]` with **no**
  `skip_serializing_if` (`submission.rs:939`), so it serialises as `[]` while
  being optional on input — any pin reading "present in a minimal value" gets it
  wrong in one direction or the other. The correct probe is removal: delete a key
  from a full payload and see whether `from_value` refuses. (`RV-357` `F-6`,
  which caught this in a *repair*, not in the original design.)
- **A structural key-set check is not a nominal type check.** Comparing a
  `Named` edge against its target's keys catches every swap the current closure
  can express — the eleven contract-bearing structs have eleven distinct
  key-name sets — but it is total by a property of the closure, not by
  construction (`RV-357` `F-5`). Extended at rev 72: an enum target is told apart
  by its **token** set instead, and the thirteen tokened enums have thirteen
  distinct ones; `WireFacetValue` is untagged and told apart by shape.
- **A repair written as a *case* buys one round.** `F-6`'s four repairs were each
  correct about what they were shown and each reopened somewhere else, because
  each named the places a property currently appears rather than where the model
  *declares* it. What ended it was a change of instrument — an exhaustive match on
  `sec-2`'s model with no wildcard arm, plus one coverage equality — and the
  self-check that found the largest hole (`Named` edges to **enum** targets had no
  defined check at all) came from reading the repair against the model's own
  enumerations, not from a sixth review round. Full pattern, including the
  follow-on sweep a generalisation owes the text it subsumes:
  [[mem.pattern.review.general-rule-must-absorb-what-it-subsumes]].
- **The run's revision moves when review findings land**, not only when a payload
  is applied: `RV-357`'s twelve raises took the run 64 → 65 with `changes: none
  since revision 64` and a new `review_outstanding` line. An `adopt_authored`
  built against the pre-review revision is refused as a conflict; re-read and
  resubmit.
- **Adopting hand-edited design prose, last section included**: the whole-file
  `sha256` is the run's watermark, and each section's fingerprint is the `sha256`
  of the bytes **after its marker line** — not from the marker itself — up to the
  next marker, less exactly one trailing newline. For the final section that means
  dropping the file's own trailing newline. Positive-control the recipe against an
  **unchanged** section every time before submitting: the marker-inclusive reading
  of this same sentence reproduces nothing, and it cost a round trip to notice.
  No verb reports the observed digests — captured as a friction observation.
- **`design_run` const tables tolerate a `Named` graph over drop-glue types** —
  compiled as a probe rather than assumed, when the `Cow` question was live.
- **`ISS-361`'s reported mechanism is not at the parse site** — verified against
  source: `apply` parses with `?` before `admit` (`design.rs:1503-1504`) and
  `admit` (`run.rs:207`) is pure, so a top-level parse failure mints no receipt
  and bumps no revision. The fitting reconstruction is a parse that *succeeded*
  through `ISS-333`'s hole, then a corrected resubmission reusing its
  `submission_id` meeting `Refusal::SubmissionReplayed` (`run.rs:222-226`). The
  residual defect — that refusal naming no remedy — is `IMP-390`'s fourth
  candidate and a Non-Goal here. `DEC-225` inherits no bug.

**Execution, PHASE-04..07 — swept from the phase sheets and commits at the first
audit (`RV-362`), since the sheets are runtime and three of them were never
written back.**

- **A VT's `test_file` was mis-sited four times in this slice, and the fourth
  would have gone GREEN.** `PHASE-01/VT-1`, `PHASE-03/VT-3` and `PHASE-04/VT-1`
  each named a leaf file for a test that must name command-tier symbols — a
  layering violation `just check` reds loudly. `PHASE-06/VT-1` was different: a
  leaf-sited golden can only pass `extern_fixture()`, PHASE-05's two-row stub, so
  it would have pinned the *shipped, published* document to a test fixture and
  passed. `DEC-140` (verification evidence lives at the tier that can produce its
  subject) decided this in the general case on 2026-08-04; `/plan` does not
  enforce it. Carried out of the slice as `IMP-438`. **A defect class that usually
  fails loudly and occasionally fails silently is the one worth mechanising** —
  vigilance calibrates to the usual case.
- **A grep-keyword VT is a proxy, and a correct DRY refactor can defeat it.**
  `PHASE-06/VT-1` also named `CARGO_MANIFEST_DIR`; the golden reaches that
  property through `crate::test_support::repo_root()` (`test_support.rs:31`,
  runtime env var, not `env!`). Spelling the var a second time to satisfy the
  grep would be an `STD-001` violation raised to pass a test. Retired at audit,
  `VT-3` appended on the seam actually used. `RV-362` `F-1`.
- **`sec-` is LIVE WIRE VOCABULARY in the shipped contract, not a design-section
  leak.** PHASE-06 planned `!shipped.contains("sec-")` as the `POL-002` probe; it
  reds a *correct* document, because the rendering carries `{id(sec-): text}` and
  `id(inq-|sec-|att-|fnd-|cp-)` — which a client needs in order to send a
  well-formed payload. The probe needs a following **digit**, matching
  `cites_a_repo_private_id`'s own prefix+hyphen+digit shape. Re-checked at audit:
  zero repo-private ids and zero `sec-[0-9]` in the 210-line shipped document.
- **Doctrine replaces clap's help renderer, so reasoning from clap's behaviour is
  reasoning about a renderer this project does not run.** `main.rs:285-292` routes
  every `--help` and every `help` subcommand through `render_subcommand_help`,
  which read `get_about()` alone (`cli.rs:1427`); `long_about` occurred **zero**
  times in the tree. `PHASE-07/EX-4`'s named mechanism would have rendered
  nothing. This is a third instance of the class this slice named after `A3` —
  *an unverified claim about what a tool does, load-bearing on a design decision*
  — and the first two were also about clap. The fix widened the touch-set to
  `cli.rs` and the widening was **measured**: 281 help renderings before and
  after, 251 byte-identical, 27 differing by exactly one restored trailing full
  stop, 3 by a PID in a pre-existing panic (`ISS-439`, raised from that sweep —
  three broken `--help` renderings had been sitting in plain sight because
  whole-tree help sweeps are not something this project does).
- **The golden must read disk, and PHASE-06 found the sheet's stated REASON for
  that false while the conclusion stood.** The sheet said an `install/`-only edit
  does not trigger a rebuild; it does, here. Execution replaced the argument with
  an experiment instead of keeping a true conclusion on a false premise:
  redirecting `CARGO_MANIFEST_DIR` at one already-built binary reds the golden
  while the embed is byte-identical across both runs. The doc comment now carries
  the reason that survives the evidence.
- **The `#[expect(dead_code)]` ladder ran in both directions, one phase at a
  time.** `extern_contracts()`'s gate came *off* at PHASE-06 (first production
  caller); `PAYLOAD_CONTRACT_PATH`'s became `cfg_attr(not(test), …)` at the same
  phase (first `cfg(test)` reader); `PAYLOAD_CONTRACT_POINTER`'s was *deleted* at
  PHASE-07 (production readers). Under `warnings = "deny"` an `expect` that stops
  firing is itself a hard error, so each move belongs to the phase that lands its
  first reader — never early, never optional tidy-up. See
  `mem.fact.build.dead-code-fires-only-under-test-compile` for why nothing
  accumulates silently: the gate runs the test compile.
- **An audit parked on an unmerged ref is an audit that did not happen.** The
  capsule driver committed `RV-362` to `refs/capsule/d/heads/work` and never
  merged it; fourteen hours later a second audit ran on `edge` with no way to
  know the first existed, and the slice closed on the second alone. Nothing was
  lost in the end — the ledger was recovered — but four delegated findings sat
  undischarged through `done`, and the recovery cost a full second reconcile
  pass. The lesson is not about capsules: **any ledger whose reachability depends
  on a ref nobody merges is invisible to the lifecycle that gates on it.**

### Open

- **`/reconcile` obligations are now enumerated on `RV-361`'s Reconciliation
  Brief, not only in `plan.md`.** Every phase added one at execution and
  `plan.md`'s per-phase headings remain the reasoning; the brief is the
  actionable list, because `plan.toml` is immutable-append and is not a reconcile
  write surface. Query one of those two, never a count carried in prose.
  **Discharged at reconcile** — `RV-361`'s `## Reconciliation Outcome` records
  what landed; nothing is owed.
- **The `attestation.rs` trap was real, reached the audit, and is fixed** —
  `design.md` §7 attributed closure-struct fixtures to a file that defines none,
  which `PHASE-02/VT-2`'s waiver caught at execution and the design never
  absorbed. Corrected at both sites plus the selector registry (`RV-361` `F-2`);
  `slice conformance 251` now reports **undelivered 0, conformant 12**.
- **`RV-362`'s brief was discharged after close, in a second reconcile pass**
  (2026-08-16, post-`21746a11d`). Two of its four delegated findings were already
  satisfied by `RV-361`'s pass — `F-5`/`F-6`, the selector registry, and two of
  `F-8`'s five design.md claims. The rest were owed and are now landed:
  `PHASE-06/VT-1` retired with `VT-3` appended (`F-1`, which is what cleared
  `verify-vt`); `SPEC-029`'s command family corrected from four verbs to six by
  `REV-054` (`F-7`); the three remaining `design.md` claims — the `const fn`
  count in `sec-4`, the single `Id`/`declarable` rule in `sec-8`, and `contract`
  as the fifth rather than sixth `DesignCommand` variant (`F-8`); `ISS-346`
  closed `duplicate` against `ISS-333` (`F-9`); the
  `mem.fact.design-run.apply-payload-vocabulary` falsehood about `Declaration`'s
  `deny_unknown_fields` corrected at source (`F-10`); and the orphaned
  `.doctrine/workflows/drive-slice.js` pruned (`F-12`). `F-11`'s stray root-level
  `capsule-*.md` files were already gone by another route.
- **Nothing about this slice is owed.** What remains open is owned elsewhere:
  `ISS-333` on its serde axis, `IMP-390` on its other three faces (both linked
  `fulfils … degree = partial`), and the four items carried out — `IMP-438`,
  `IMP-439`, `ISS-439`, `ISS-440`. The standing risks below are recorded, not
  scheduled.

- **Two obligations open, both the user's** — section attestations (all nine
  outstanding; human review is the v1 default per `reviewing.md`) and the review
  pass disposition, which `RV-357`'s terminal ledger now makes nameable as
  *conducted* rather than waived (but read the rev 71–72 caveat below first).
  Every section but `sec-1` has been revised at rev 52–72, so nothing carried over
  from before is reusable. `sec-1` alone has been untouched since rev 52 and is
  still outstanding for that earlier reason.
- **Revs 71–72 post-date the review pass.** `RV-357` is terminal (14/14 verified),
  but `F-6` was verified against **rev 70**; the class rework and its
  reconciliation landed after. The run reports this itself as
  `review_pass STALE`. The rework is self-checked against `sec-2`'s enumerations
  and no reviewer has seen it — weigh before disposing the pass as *conducted*.
  The codex thread carrying all five rounds is
  `01a0048a-b948-7bf1-9ee6-f5d3fbc93fa1`.
- **`sec-8` pin 1's premise is still unexercised** — whether
  `assert_keys_described` can stay one generic body across eleven types with
  different `Serialize` shapes is argued, not proven, and it now also carries the
  `TypeContract::name` comparison (`F-9`) and feeds the removal probe's fixtures
  (`F-6`). A `/plan` or implementation-time discovery, not a design defect, but a
  reviewer who reads `sec-8` as fully exercised will be wrong.
- **`RecordKind::ALL` is hand-maintained and nothing forces a new variant into
  it** (`knowledge.rs:60-68` vs `159-169`; the incumbent test iterates `ALL`, so
  it is green over a stale one). Out of scope here — `sec-3` states the barrier at
  its real strength and `sec-8` pin 5 takes its oracle from an exhaustive match
  instead. Deriving `ALL` from the enum is the knowledge tier's repair, captured as
  `ISS-364`.
- **The nominal-identity residue on `Named` edges** — `sec-8` pin 2 is structural,
  total over today's closure only because no two targets present identically
  (eleven distinct key-name sets, thirteen distinct token sets, one untagged type
  told apart by shape). The recorded escalation is a `payload_struct!` mirroring
  `sec-4`'s enum instrument; not taken.
- **`ISS-362` is now load-bearing on `sec-3`** — the design states the bound and
  defers the repair. If `ISS-362` lands a payload-act-to-stage guard as data, the
  stage column becomes derivable and `sec-3`'s subsection should be revisited.
- ~~`DEC-228`'s pin is not achievable as recorded~~ — corrected 2026-08-15 at
  source: the pin parses the constant's JSON arm alone after placeholder
  substitution, keeping `concat!(JSON_ARM, PROSE)` so the length assertion still
  covers what renders. `sec-8` pin 8 matches.
- `sec-3`'s `Extern` soft spot is discharged in design by `sec-8` pin 5's two
  equalities over one-and-a-half compile barriers — the supply is closed and the
  facet vocabulary is derived, the kind list is not (above).
- `ISS-346` duplicates `ISS-333` (minted 2026-08-12 vs 2026-08-09) — merge so
  closure lands on one id.
- `mem.fact.design-run.apply-payload-vocabulary` wrongly claims `Declaration`
  does not deny unknown fields; it does (`submission.rs:123`).
