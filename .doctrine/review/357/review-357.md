# Review RV-357 — design of SL-251

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject.** `SL-251`'s design — `.doctrine/slice/251/design.md`, nine sections,
materialised from design run `dr-019ffb40-4533-7802-9f23-6da9cd92858d` at
revision 64. External adversarial pass, standing in for the human on detail.

**Why this pass exists.** Three self-review passes have already run; twenty
findings were raised and disposed across them, and the integration lives in the
sections themselves rather than here. Each pass found real defects, and each
found its predecessor's blind spot — which is the signal that a fourth internal
pass buys less than an outside one. Do not re-raise the disposed twenty.

### Primary line of attack — the probe nothing has reached

A governance re-read against the **finished** artefact. `ADR-001`, `STD-001` and
`POL-002` were applied while drafting; nothing has re-read the whole design
against them since the sections settled.

- **`ADR-001`** (module layering: leaf ← engine ← command, no cycles). `sec-2`
  "Where it sits", `sec-3`'s resolution — one region injected through a closed
  seam — and `sec-7` "Where the new module sits". Does the injection direction
  actually respect the layering, or does it smuggle a higher-tier vocabulary
  into a leaf that must not see it?
- **`STD-001`** (no magic strings — single-source named constants). The
  descriptions, token strings and shipped type names (`sec-2` "Naming, and why
  the type names ship"), `sec-5`'s fixed line shapes, `sec-6`'s pointer text.
- **`POL-002`** (platform independence from host-project conventions). Anything
  in the design that bakes doctrine-as-host-project convention into engine-tier
  behaviour.

### Secondary attacks

The reviewing obligation's surfaces: vague sections where a short sample would
remove the ambiguity; hidden assumptions; weak verification; missing code-impact
detail; prose that makes a human reconstruct the design from identifiers,
review history, or locally invented terminology; relationships, sequences, state
changes or ownership boundaries that need a diagram to be understood reliably;
diagrams that disagree with the prose or merely decorate an inventory of boxes.

### Where the bodies are likely buried

1. **`sec-8` pin 1's premise is argued, not exercised** — whether
   `assert_keys_described` stays one generic body across eleven types with
   different `Serialize` shapes is unproven. Known and accepted as a `/plan` or
   implementation-time discovery; the live question is whether the design
   *presents* it as more settled than it is.
2. **`sec-3`'s bound** is total over the payload and silent on stage
   admissibility ("And one thing it does not claim at all"). `ISS-362` is
   load-bearing: the design states the bound and defers the repair. The ordering
   argument is the whole point — attack it.
3. **`sec-5`'s cost claim** (`R5`, measured) and the three-consumer split — one
   generator, three renderings, and the assertion that `--help` is not a fourth.
4. **`sec-4`'s pin ladder** — "Where the pins do not reach" is the design's own
   honesty about the gap. Is it honest *enough*?
5. **`sec-6`'s three push points** are claimed to cover disjoint failures.
   Disjointness is an assertion, not a demonstration.

### Ground rules

- `.doctrine/slice/251/render-sample.md` is **evidence, not a golden**, and
  `sec-5` says so in terms. It predates the generator, differs in whitespace and
  ordering, and its `fnd-N` markers record what the rendering exposed rather
  than the current model. Divergence between it and `design.md` is not a
  finding.
- **Read the model from `sec-2` and `sec-3` as written**, not from any earlier
  revision. Revisions 62–64 moved it hard: the extern region is a
  `SelectorTable`, not an enum-form `TypeContract`; `UnknownKeys` has three
  variants (`StoredThenFlagged` is new, and `SelectorTable` is its only
  inhabitant); `WireType::Id` carries `&'static [IdKind]`; and `Cow` is gone
  from the model entirely. `sec-8` pin 1 is eleven-by-contract plus
  `SubmissionEnvelope` by root composition — twelve is not a miscount.
- **The design must stand alone.** Context required to understand or implement
  it may not live in review history. That is itself a line of attack.
- A pass that finds nothing is a result worth stating plainly, not a gap to fill
  with invented findings.
