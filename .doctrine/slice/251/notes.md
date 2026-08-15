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
- The unknown-key hole is exactly one level deep — `deny_unknown_fields` holds at
  `Declaration`, `CheckpointActDeclaration`, `AgentActDeclaration`. `ISS-333`'s
  discharge can therefore be stated precisely at close.

### Corrections found while exploring

- `mem.fact.design-run.apply-payload-vocabulary` states "neither `ApplyRequest`
  nor `Declaration` denies unknown fields". The second half is **wrong** —
  `Declaration` carries `deny_unknown_fields` (`submission.rs:123`). Correct the
  memory at harvest.
- The same memory cites `ISS-346` for the silent-discard; the slice scope cites
  `ISS-333`. Reconcile which is canonical before close.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-15 · design/drafting (run rev 32) · b3e64183f

### Produced

- `DEC-219` `DEC-221` `DEC-224` `DEC-225` `DEC-226` `DEC-227` `DEC-228` `DEC-229`
  — the inquiry's eight decisions. `DEC-229` partially supersedes `DEC-221`
  (enums only; struct half stands).
- Run sections `sec-1`..`sec-5` — declared, all 5 outstanding review.
- Scope reconciled: `OQ-1` `OQ-2` `OQ-3` `R2` resolved against their records;
  Objective 2's "authored, not derived" contradiction corrected.

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

### Open

- `sec-6`..`sec-9` undrafted (discoverability · code impact · verification ·
  assumptions). All five declared sections still outstanding review.
- `sec-3`'s `Extern` soft spot: no compile-time pin that the shell supplies a
  sub-contract per `Extern`, or that it matches `facet_fields`. `sec-8` owes the
  two tests.
- `ISS-346` duplicates `ISS-333` (minted 2026-08-12 vs 2026-08-09) — merge so
  closure lands on one id.
- `mem.fact.design-run.apply-payload-vocabulary` wrongly claims `Declaration`
  does not deny unknown fields; it does (`submission.rs:123`).
