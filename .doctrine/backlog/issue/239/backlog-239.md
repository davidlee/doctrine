# ISS-239: Three tech specs carry no product descent

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observation

SPEC-017 (Tech-spec spine, active) governs `descends_from` as "the single-valued
cross-family link from a tech spec to the `PRD-NNN` capability it realises", and
grants exactly one lawful absence — the **root-context exception**: a
`c4_level = "context"` spec (SPEC-003) "descends from no single `PRD-NNN`
capability — it is the synthesis above the product capabilities, not a
realisation of one." SPEC-017 scopes that exception explicitly: "a container- or
component-level spec with null lineage is still an **unplaced** spec, not a root."

Three tech specs sit outside both the norm and the exception:

| Spec | C4 level | parent | descends_from |
|---|---|---|---|
| SPEC-004 Entity engine | container | SPEC-003 | *(none)* |
| SPEC-013 CLI surface | container | SPEC-003 | *(none)* |
| SPEC-016 Governance kinds (POL/STD) | component | SPEC-004 | *(none)* |

Every other tech spec carries descent: 22 of 26 do, and the remaining one is
SPEC-003 itself (the lawful context root). So by SPEC-017's own wording these
three are *unplaced*, not rooted.

Surfaced incidentally while establishing the placement convention for a new
graph-projection tech spec under [[CHR-046]]; not part of that chore's scope.

## Two distinct sub-cases

These are not one defect with one fix.

**(a) SPEC-016 — a missing PRD, not a missing field.** Every *other* kind-surface
component descends from the PRD for its kind: SPEC-005 → PRD-008 (ADRs),
SPEC-014 → PRD-001 (Slices), SPEC-015 → PRD-009 (Backlog), SPEC-019 → PRD-010
(Epistemic and Governance Records). The pattern is exceptionless except SPEC-016,
and the reason is that **no PRD governs policies and standards as a capability.**
POL/STD are load-bearing (POL-002 and STD-001/STD-002 are in force and project
into the boot snapshot) yet have no product intent above them. So the fix is
either a POL/STD product spec, or a judged decision that PRD-010 already covers
them and SPEC-016 should descend from it. Filling the field without making that
call would be fabricating lineage.

**(b) SPEC-004 and SPEC-013 — substrate containers with no capability above
them.** The entity engine and the CLI surface are cross-cutting substrate that
*every* capability rides; neither realises a single product capability, and the
PRD corpus has no candidate (PRD-001..PRD-017 are all capabilities, none is
"entity engine" or "CLI"). This looks less like an authoring omission than a gap
in SPEC-017's exception: the root-context exception recognises that *something*
sits above the product capabilities, but admits only C4 context altitude. A
substrate container may be a second legitimate lineage-free shape.

## Candidate resolutions

Not yet judged — this item is the capture, not the decision.

1. **Widen SPEC-017's exception** to admit substrate containers as lawfully
   lineage-free, with a marker distinguishing "substrate" from "unplaced" so the
   silence stays readable. Cheapest, and it makes the corpus self-describing.
2. **Author the missing PRDs** — a POL/STD product spec for (a), and possibly
   platform-substrate PRDs for (b). Most faithful to the descent model, most
   authoring cost, and risks minting PRDs that exist only to satisfy a field.
3. **Re-point SPEC-016 at PRD-010** if the judgement is that governance records
   already cover POL/STD. Resolves (a) alone, cheaply, if it is honest.
4. **Add a `validate` leg** flagging a container/component spec with null
   `descends_from`, so the condition is mechanically visible rather than
   discovered by hand-tabulating 26 TOMLs. Complements whichever of 1–3 lands,
   and prevents silent recurrence.

Note (4) is worth doing regardless: nothing currently detects an unplaced spec.
`spec validate` FK-checks the descent edge when present but does not require one,
so the gap is invisible to `doctrine doctor` and to the close gate.

## Neighbours

- SPEC-017 — the spine spec whose placement rule this tests.
- [[CHR-046]] — where it surfaced; that chore's new tech spec follows the
  convention correctly (component → parent SPEC-004, descends PRD-016).
- SPEC-006 — spec composition machinery, owner of the `spec validate` FK pass
  that resolution (4) would extend.
