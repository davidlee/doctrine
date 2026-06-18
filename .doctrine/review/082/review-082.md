# Review RV-082 — design of SL-101

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

The Inquisition probes SL-101's design against the sanctified doctrine of
SPEC-020 — the Estimation facet tech spec. The accused claims descent from
SPEC-020 and PRD-014, yet its own confession reveals two mortal deviations:

1. **The unit default.** SPEC-020 FR-003 decrees `high_caffeine_hours` as the
default estimation unit. The accused's design (§3.3) inscribes `espresso_shots`.
This is not a trivial renaming — it is a direct contradiction of the governing
spec the slice itself cites as authority. The Inquisition demands: by what right
does a design overrule its own charter?

2. **The Value facet — a bastard child of no known parentage.** SPEC-020 covers
the Estimate facet alone. No PRD-014 passage, no SPEC-020 section, no separate
spec — nothing in the sanctioned doctrine — authorises a `ValueFacet` with unit
`magic_beans`. A facet bred in the darkness of `slice/101/`, claiming space in
`src/value.rs`, the `dtoml.rs` config surface, and the `SliceDoc` without a
single line of product intent or technical specification to sanctify it.

The lines of interrogation are these:

- Does the default estimation unit match SPEC-020 FR-003? (I-1)
- Does every facet in the design trace to a governing PRD+SPEC? (I-2)
- Do the leaf modules honour ADR-001 purity? (I-3)
- Is the config wiring `#[serde(default)]` tolerant? (I-4)
- Is the parse forward-compatible per SPEC-020 NF-003? (I-5)
- Are there estimate/value reads in any workflow predicate? (I-6)
- Do the scope and design contradict each other or the governing spec? (I-7)

The accused shall answer for every deviation. The pyre awaits.

## Synthesis

The Inquisition has examined the accused — SL-101's design — against the
sanctified word of SPEC-020, ADR-001, and ADR-004. Four charges are raised.
Two are **blockers**; the design must not proceed to plan or implementation
until they are resolved.

### F-1 (BLOCKER) — The unit of apostasy

The accused confesses `espresso_shots` where SPEC-020 commands
`high_caffeine_hours`. This is not a trivial word-choice — it is a design
talking over its own governing spec. The constant is wrong in three places:
the design's `DEFAULT_ESTIMATION_UNIT`, the scope document's FR-003 prose,
and the install template. **Penance:** Replace all three with
`high_caffeine_hours`. One `sed` invocation. No design discussion needed.

### F-2 (BLOCKER) — The bastard facet

The `ValueFacet` has no father in PRD-014 and no mother in SPEC-020.
It is a stowaway in a slice chartered for estimation. The `magic_beans` unit
is whimsy without sanction. **Penance:** Either strike the Value facet from
SL-101 entirely and file a backlog item for its proper doctrinal parentage,
or produce a governing PRD+SPEC and amend the slice's relationships.
The "two facets in one slice" framing is false advertising until both have
authority.

### F-3 (minor) — The test index is garbled

The verification alignment table (§8) points FR-004 at E16 (a forward-compat
test) and NF-003 at E15 (a confidence-bounds test). The implementer who
follows the index will examine the wrong tests. **Penance:** Remap
FR-004 → E4,E17 and NF-003 → E16,V7. Fix the dangling E15 reference in §11.
Three line edits.

### F-4 (minor) — The phantom round-trip claim

E17 claims unknown keys survive serialize→parse round-trip, but
`EstimateFacet` carries no `_extra` field. The struct cannot honour the
claim. This is not a code defect — SPEC-020 says v1 persists no inferred
fields, so losing extras at serialize is *correct* — but the test claim
is a false promise that will waste an implementer's time. **Penance:**
Correct E17 to test parse-tolerance only. Correct §3.4 prose from "survive
round-trip" to "tolerated at parse; v1 does not persist."

### What was found clean

The Inquisition finds no heresy in:
- **ADR-001 layering** — both modules are correctly placed as pure leaf tier.
- **Forward-compat mechanism** — `#[serde(flatten)] _extra` on the raw struct
  correctly tolerates unknown keys (NF-003 satisfied at parse).
- **Config tolerance** — `#[serde(default)]` on `dtoml.rs` fields follows
  the existing pattern.
- **Non-blocking guarantee** — no estimate/value reads appear in workflow
  predicates (NF-001 structural).
- **Custom Deserialize** — the pattern is coherent and keeps `SliceDoc` clean.

### Standing risks

- The Value facet's uncertain fate: if it is extracted to its own spec+slice,
  the SL-102/SL-103 display and graph slices must account for both facets
  independently.
- The `magic_beans` unit is an orphan — even if a Value spec is authored,
  the unit name may change, adding churn to downstream slices.

### Verdict — amended by the User's ruling

The Inquisition's zeal was sharp but its aim was ahead of the doctrine.
The User hath spoken:

- **F-1** — `espresso_shots` stays. SPEC-020 shall be amended during
  reconciliation to match the design. The Inquisitor mistook a vanguard
  for a heretic.
- **F-2** — The Value facet is no bastard. It too shall be sanctified
  retroactively — a governing PRD+SPEC to be authored and SPEC-020 amended
  during reconciliation. `magic_beans` is the chosen unit.

**The design is the forward-looking authority; the specs follow.** F-3
(test index) and F-4 (round-trip claim) remain fix-now — hygiene that
costs nothing and saves implementer confusion.

All four findings terminal. The review is **done**. The pyre is doused;
the accused walks free, chastened only in its test documentation.

> **HERESIS URITOR; DOCTRINA MANET**
