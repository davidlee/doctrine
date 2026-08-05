# REV REV-048 — reconcile SL-244

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

`SL-244` declared `SPEC-029` a design target and stated the obligation in
`design.md` sec-6 in as many words — the spec "owns the gate table and is revised
by this slice regardless". No revision landed, and it was the slice's sole
remaining `undelivered` conformance row. Raised as `RV-345` `F-2`, delegated to
`/reconcile` by the ledger's `## Reconciliation Brief`, and discharged here.

**The revision is additive.** `D1` ("the gate table is a `const fn` table, not a
matcher") stands, and nothing in the spec body is falsified. The one thing the
audit expected to be a *correction* was not one: `SL-244`'s `design.md:19`
characterised the spec as describing evidence as payload-claimed, and it never
did — the claimed-evidence model lived in `SL-233`'s design prose and in source
(`grep -in claim spec-029.md` returns one unrelated hit at line 124, against a
positive control of three for `gate table`). So the spec is *silent* on how a
condition is satisfied, and this revision gives it the answer rather than
replacing a wrong one. `design.md:19` was corrected in the same reconcile pass.

**No requirement status moves.** `SPEC-029`'s members (`REQ-428`..) and
`PRD-019`'s `REQ-422` / `REQ-425` were checked; none is falsified or newly
satisfied by this slice.

## Reconcile narrative (SL-244)

- [`RV-345` `F-2`] — `SPEC-029` `responsibilities` gains the five mechanisms
  `SL-244` built, each a projection of one macro-generated source rather than a
  second copy of it:
  1. **the gate contract table** — every condition paired with its contract
     (kind, subject binding, reach, discharging act), keyed by the closed
     `Advance` forward relation rather than a `(Stage, Stage)` pair;
  2. **the attested-act ledger** (`CheckpointAct` / `AgentDeclaration`) replacing
     the existential scan over evidence rows, with satisfaction classified in two
     kinds — derived or attested — and no caller-claimed tier;
  3. **the per-run `ReviewPolicy`** and the reviewer lanes it requires;
  4. **the `ReviewPass` `RV`** minted on entry to `reviewing` through the
     journalled-intent seam, its facet and target derived from the run;
  5. **the condition contract corpus and the stage-entry receipt** —
     `design-prompts/conditions/` (nine narratives) plus the generated diagram.
- [`RV-345` `F-2`] — the spec **cites** the published address
  `reference/design-run-stages.md` rather than holding a copy, per `DEC-127`: the
  spec is the repo-private artefact and the diagram the reachable one, so the
  private document points at the public one and not the reverse.

Landed by hand (a `modify` row is surfaced-for-manual at `revision apply`) in
both tiers — the structured `responsibilities` array in `spec-029.toml` and its
mirroring summary in `spec-029.md` § Responsibilities, where the `DEC-127`
citation sits with `D1`.
