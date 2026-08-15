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
fresh-as-of: 2026-08-15 · design/reviewing (run rev 67) · e5c868ea0

### Produced

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
- **`RV-357` conducted — external adversarial pass, two rounds, 14 findings, all
  dispositioned.** Round 1 raised `F-1`..`F-12` (integrated at rev 66); round 2
  verified eight, contested `F-3` `F-5` `F-6` `F-12` and raised `F-13` `F-14`
  (integrated at rev 67). Every finding was checked against source before
  disposition; none was confabulated, and every round-2 point was a defect in the
  round-1 *integration* rather than in the reviewed design. Ids only; the ledger
  holds the text and the responses.
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
  construction (`RV-357` `F-5`).
- **The run's revision moves when review findings land**, not only when a payload
  is applied: `RV-357`'s twelve raises took the run 64 → 65 with `changes: none
  since revision 64` and a new `review_outstanding` line. An `adopt_authored`
  built against the pre-review revision is refused as a conflict; re-read and
  resubmit.
- **Adopting hand-edited design prose, last section included**: the whole-file
  `sha256` is the run's watermark, and each section's fingerprint is the
  `sha256` of the bytes between its marker and the next, less exactly one
  trailing newline. For the final section that means dropping the file's own
  trailing newline — the uniform rule reproduces all nine (positive-controlled
  against the unchanged `sec-1`).
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
- **Adopting hand-edited design prose**: `sha256` of the text between marker
  lines, less the trailing newline, reproduces a section's stored fingerprint
  (validated with a positive control against an unchanged section). No verb
  reports the observed digests — captured as a friction observation.

### Open


- **Two obligations open, both the user's** — section attestations (all nine
  outstanding; human review is the v1 default per `reviewing.md`) and the review
  pass disposition. Every section but `sec-1` has been revised at rev 52–67, so
  nothing carried over from before is reusable. `sec-1` alone has been untouched
  since rev 52 and is still outstanding for that earlier reason.
- **`RV-357` is now conductable but not yet terminal** — 14 findings, all
  dispositioned, `await=raiser`. Round 2's verifications stand; the four contests
  and two new findings were re-dispositioned at rev 67 and have **not** been
  re-verified. A third raiser round closes them, after which the pass can honestly
  be named *conducted* rather than waived. The codex thread carrying rounds 1 and
  2 is `01a0048a-b948-7bf1-9ee6-f5d3fbc93fa1`.
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
  instead. Deriving `ALL` from the enum is the knowledge tier's repair and wants a
  `ISS-364`.
- **The nominal-identity residue on `Named` edges** — `sec-8` pin 2 is structural,
  total over today's closure only because no two structs share a key-name set. The
  recorded escalation is a `payload_struct!` mirroring `sec-4`'s enum instrument;
  not taken.
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
