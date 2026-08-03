# EVD-012: Exploring gate forces canonical content into design prose

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The datum

The `exploring → inquiring` boundary requires evidence for
`governing-context-recorded` and `initial-concerns-recorded`
(`src/design_run/gate.rs:162-167`). Evidence is an `EvidenceDeclaration
{ condition, subject }` (`src/design_run/submission.rs:486-489`), and the shell
resolves `subject` to a fingerprint **against `next.sections` only** —
`Refusal::UnknownNode` for anything else (`src/design_run/run.rs:1471-1478`).

Sections are `design.md` prose. A run at `exploring` normally has none, because
sections are a drafting artefact. So the edge is unpassable until the caller
mints a prose section for the evidence to point at. `tests/e2e_design_state.rs`
shows the intended path and no other: declare `sec-1` (line 834), then bind both
conditions to `sec-1` (line 856). Declarations carry no stage guard, so this is
legal — it is simply what the mechanism requires.

Observed on `SL-244`'s own run (`dr-019fc4dd-e049-7db0-8cea-9af8ff970810`,
revision 25) after the user attested both conditions and the attestation could
not be recorded.

## Why this is a governance violation, not merely awkward

The user's assessment, recorded as the reading this slice proceeds under.

The governing context of a design *is* a set of canonical entities — for
`SL-244`: `DEC-101`, `DEC-102`, `DEC-066`, `DEC-067`, `STD-001`, `SPEC-029`,
`ADR-001`. Each holds its content canonically in its own TOML+MD pair. The
initial concerns are likewise either their own records or slice-local prose that
already exists in `.doctrine/slice/244/notes.md`.

The gate cannot bind clearance to any of those. It can bind only to a
`design.md` section — so the run's evidence that governing context was recorded
attaches to a **restatement** of canonical content, not to the canonical content.
That is duplication of canonical data into prose, which the storage rule forbids
in terms (authored TOML is the queried tier; prose is never the source of a fact
the system reads) and which `STD-001` forbids by the same principle one level
down. The mechanism does not merely permit the violation, it is the only way to
pass the edge.

The design can hold the duplication to citation-plus-judgement — name the ids and
state the applicability reasoning, which is genuinely slice-local content, rather
than restate the entities. That narrows the violation. It does not dissolve it:
clearance is still bound to a prose subject whose fingerprint moves for reasons
unrelated to whether governing context was in fact recorded.

## Why it is also poorly conceived, independent of governance

Three defects, separable:

1. **The subject is unconstrained.** Nothing relates the subject to the
   condition. Any section clears any claimed condition. A section titled
   "Appendix" clears `governing-context-recorded` exactly as well as one that
   contains the governing context.
2. **The name asserts what the check cannot see.** `governing-context-recorded`
   reads as a semantic fact; the check is a fingerprint match on an arbitrary
   subject. The gap between the two is the whole of the defect class this slice
   was cut to close.
3. **The bootstrap is inverted.** A stage-1 boundary is satisfiable only by a
   stage-3 artefact, so the cheapest compliant move is to mint a placeholder —
   the gate's incentive runs towards the fake artefact, not away from it.

## Bearing on the design

Direct mechanical support for `DEC-126` (what the gate should check) and for
`DEC-120`'s classification (derived / attested / claimed). It converts an
argument reached by reasoning into an in-tree fact, and it supplies the
strongest available case that a condition needs a **subject rule** — the
question `OQ-5` / `ISS-286` raised and the triage judged plausibly separable.

Where `DEC-121` lands — the exploring pair becomes attested user checkpoints —
this evidence says the checkpoint must also name what it is attested *against*,
or the new mechanism reproduces the old one's indifference to its subject.
