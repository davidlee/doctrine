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
fresh-as-of: 2026-08-15 · design/reviewing (run rev 50) · ef47e756b

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
- `fnd-10`..`fnd-19` raised on the run (rev 50) from that diff — **undisposed**,
  four blocking. Ids only; the run holds the text.

### Learned

- Triage errors corrected in the governance table above: `ADR-001` (`design_run`
  is leaf out-degree 0, not engine) and `ADR-019` (engaged on the publication
  leg, not the embed-root leg).
- `DEC-224` body carried a wrong line ref; corrected to `guard.rs:430`.
- `WireFacetValue` is `#[serde(untagged)]` — a fifth tagging mode the `sec-2`
  model initially missed.
- `knowledge.rs:860-875,1028` — `facet_fields` / `FieldShape` already publish a
  contract-shaped description of the knowledge tier; `sec-3`'s `Extern` injects
  it. Enforced post hoc by `doctor_checks.rs:161`, not at the write seam.
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

### Open

- **Section attestations** — all nine outstanding. Human review is the v1
  default (`reviewing.md`); the run will not lock without them.
- ~~**Owed artefact: a worked full-closure rendering.**~~ — **delivered
  2026-08-15**, `.doctrine/slice/251/render-sample.md`. What it settled: the
  closure itself is right (twelve structs, fourteen enums, membership, the
  three-refuse/nine-discard split, the thirteen-vs-nine asymmetry, acyclicity —
  all confirmed against source). What it broke is one tier down.
- **`fnd-10`..`fnd-19` undisposed — four blocking, and they gate the lock.**
  Four are **expressibility**: `sec-2`'s model cannot state an untagged variant
  (`fnd-11`), per-variant tagging where `External` is non-uniform (`fnd-12`), a
  map's key type or a key set that depends on a sibling field (`fnd-13`), or the
  name of an inlined variant type (`fnd-14`). Three are **rendering**:
  `render_json` cannot be the derived `Serialize` (`fnd-10`), the variant line
  has no multi-key form (`fnd-17`), eleven blocks render against twelve closure
  types (`fnd-16`). Three are **facts**: a third scalar and a sixth file
  (`fnd-15`), a second internal- and a second external-tagged enum (`fnd-18`),
  and a `sec-2`/`sec-3` contradiction over `CreateRecord.kind` (`fnd-19`).
  `fnd-13` and `fnd-19` are forks with real alternatives, not corrections.
- ~~`DEC-228`'s pin is not achievable as recorded~~ — corrected 2026-08-15 at
  source: the pin parses the constant's JSON arm alone after placeholder
  substitution, keeping `concat!(JSON_ARM, PROSE)` so the length assertion still
  covers what renders. `sec-8` pin 8 matches.
- `sec-3`'s `Extern` soft spot is now discharged in design by `sec-8` pin 5 (the
  two set-equalities); it remains the ladder's only test-only rung.
- `ISS-346` duplicates `ISS-333` (minted 2026-08-12 vs 2026-08-09) — merge so
  closure lands on one id.
- `mem.fact.design-run.apply-payload-vocabulary` wrongly claims `Declaration`
  does not deny unknown fields; it does (`submission.rs:123`).
