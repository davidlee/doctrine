# Inquisition — ADR-005 (shipped-knowledge-tiering)

*Adversarial review before acceptance. Status under interrogation: `proposed`.*

> Bring the accused into the light. The decision is plausible — and plausibility
> is the favoured cloak of heresy. We presume guilt.

## Target

`.doctrine/adr/005/adr-005.{md,toml}` — the Decision, its Invariants, Consequences,
Verification, and five Open Questions. The premise interrogated: *that skills
broadly restate CLI shapes and storage mechanics, warranting a corpus-wide
migration.*

## Charges

### C1 — The duplication premise is overstated; the migration scope it licenses is heresy of excess

**Doctrine violated:** no-parallel-implementation invoked as *justification*, yet
the evidence does not support the breadth. "Find duplication before writing" cuts
both ways — it also forbids inventing migration work that the corpus does not need.

**Confessed under cross-examination:**
- 20 skills, ~9,978 words total. CLI-verb mentions: ~50, of which the bulk are
  `doctrine slice/spec/memory show|validate|new` — *naming which verb in context*
  (legitimate routing), not reproducing flag syntax.
- The single heaviest accused, `spec-product` (8 mentions), already reads:
  `Real CLI surface (ask doctrine spec --help; do not guess flags)` —
  **it already obeys the ADR's prescription.**
- The storage-rule "restatement" is a one-line *pointer* in 7 skills
  (`canon`, `execute`, `notes`, `phase-plan`, `plan`, `slice`, `inquisition`),
  e.g. `notes:14` — "live progress lives in the state tree, never in authored
  files." That is correct routing, not parallel implementation.

**Risk:** the Consequence "audit every skill … de-duplicate" commits the slice to
touching all 20 skills for a payoff that lives in perhaps 2–3 lines. Twenty edited
skills is twenty chances to drift, for near-zero DRY gain.

**Sentencing:** the de-dup leg shall name *specific file:line* offenders that
reproduce **flag syntax, option tables, or storage mechanics prose beyond a
named pointer** — and fix only those. No 20-skill sweep. The breaking-wheel
awaits any agent who "normalises" a skill that was already a clean router.

### C2 — OQ-4 left open makes the central Verification check unfalsifiable

**Doctrine violated:** vague acceptance criteria (a mortal sin of `/canon`).

**Confessed:** Verification #1 — *"A skill carries no CLI command shape or
storage-tier mechanic that isn't in a reference doc / --help."* But OQ-4 admits
the line between "restate" and "a minimal inline reminder" is **undrawn**. A check
with no boundary cannot pass or fail; it is theatre.

**Risk:** the slice inherits an untestable gate (the SL-020 C2/C3 lesson — undrawn
lines breed false charges later).

**Sentencing:** draw the line in the Decision before acceptance. Proposed canon —
a skill **MAY** name a verb and cite a rule by its name (a pointer); a skill
**MUST NOT** reproduce flag syntax, option/enum tables, or storage-tier mechanics
as prose. OQ-4 resolved, not deferred.

### C3 — PUSH-tier "bloat risk" is overstated because the ADR double-counts already-shipped rules

**Confessed:** the Negative consequence frets the boot snapshot "risks bloat …
must be policed for compactness." Yet `install/routing-process.md` (the push
digest, 305 words) **already carries** the storage read-rule and the use-the-CLI
guardrail (landed commit 8206b67). The only genuinely *new* PUSH content ADR-005
adds is **reference forms**.

**Risk:** the slice plans PUSH work that is mostly done, and budgets bloat-policing
for a ~100-word delta on an 819-word snapshot (~12%).

**Sentencing:** scope the PUSH leg to the **delta only** — a compact reference-forms
block (entity-id pad rule, bare doc-local enums, the VT/VA/VH criteria modes).
State in the slice that storage-rule + use-CLI are *already resident*, not new.

### C4 — OQ-1 understated: shipping the glossary dangles a cross-reference and risks shipping 9 unintended specs

**Confessed:** `doc/glossary.md` opens — *"see [entity-model](entity-model.md)."*
The embed set is `install/` `plugins/` `memory/`; `doc/` is **not** embedded.
OQ-1 frames the choice as relocate-vs-embed but omits two facts:
1. Embedding `doc/` wholesale ships **10 evergreen build-specs** (drift-spec,
   reservation-spec, slices-spec …) into every client — almost all unwanted.
2. Shipping `glossary.md` alone leaves its `entity-model.md` link **dangling** in
   the client install.

**Risk:** silent over-ship (heresy of leakage) or a broken client-facing link.

**Sentencing:** OQ-1 must resolve to a *curated* ship, not "add `doc/` to embed."
Either relocate `glossary.md` (and any doc it hard-links) under `install/`, or
embed a named allow-list. Resolve the `entity-model` link: inline, drop, or ship.

### C5 — The tier-2 "CLI / hand-editing reference" risks being a parallel implementation

**Doctrine violated:** no parallel implementation; find the existing seam first.

**Confessed:** `install/rules/AGENTS.md` already exists as the client-facing
governance stub — 3 lines: *"This repository is governed by doctrine … doctrine
--help."* And `install/templates/*` already encode the hand-editing shapes
(every authored entity's TOML+MD skeleton). The ADR mandates authoring a *new*
reference doc without enumerating what it holds that `--help` + glossary +
templates do not.

**Risk:** a third surface that splits client-facing usage knowledge across
AGENTS.md / the new doc / glossary — the very fragmentation this ADR claims to cure.

**Sentencing:** before authoring, enumerate the reference doc's **unique payload**
(what is *not* in `--help`, glossary, or templates — e.g. *which verb for which
intent*, edit-preserving rules, the read-via-`show` discipline at length). If the
payload is thin, grow `AGENTS.md` instead of birthing a sibling. Justify or fold.

### C6 — Verification is 3/4 manual "review check"; relapse will be silent

**Confessed:** of four Verification items, three are "review check during the
migration" — human eyes, no gate. Only the embed/ship presence and the
boot-snapshot assertion are automatable.

**Sentencing:** make the two automatable items **VT** (embed test: glossary +
reference present in a fresh install; boot assertion: reference-forms block
present). The de-dup item becomes **VA** carrying the explicit C2 line. No
naked "review check" as the sole guard against relapse.

## Questions (interrogatories — answer before acceptance)

1. **OQ-1/C4:** relocate `glossary.md` under `install/`, or named-allow-list embed?
   And the `entity-model.md` link — inline, drop, or co-ship?
2. **OQ-4/C2:** ratify the MAY-pointer / MUST-NOT-reproduce line as canon?
3. **OQ-5/C3:** append the reference-forms block to `routing-process.md` (one
   Static asset — precedent: the read-rule already lives there), confirmed?
4. **OQ-3:** rename `routing-process.md` → "workflow" doc, or leave as-is and just
   add the block? (Recommend: leave — a rename churns every `@`-import and hook
   reference for cosmetic gain.)
5. **OQ-2:** read-at-runtime vs compile-time embed for tier-2 docs — given the
   rust-embed footgun, does the slice accept compile-time embed (status quo) or
   change the loading model? (Recommend: keep compile-time; the footgun is a
   known dev-loop cost, not a shipping defect.)
6. **C5:** state the reference doc's unique payload, or fold into `AGENTS.md`?

## Pronounce Judgement

**Not heresy in its thesis — the tiering by access pattern is sound doctrine, and
the glossary's unshipped state is a true, confessed defect.** But the ADR is
**tainted by overscope and deferred decision**: it licenses a 20-skill migration
the evidence does not warrant (C1), rests a central gate on an undrawn line (C2),
double-counts already-shipped PUSH rules as new work (C3), understates a ship
hazard (C4), and risks a parallel reference surface (C5).

The decision may be **accepted only once OQ-1, OQ-4, OQ-5 are resolved into the
Decision body** and the migration scope is bound to evidence. Accept-as-written
would pass ambiguity to the slice — and the slice would burn for the ADR's sins.

## Sentencing (ordered penance)

1. **Resolve OQ-4 into the Decision** as the MAY/MUST-NOT line (C2). *Verify:* the
   line is quotable as a VA criterion.
2. **Resolve OQ-1 into the Decision** as a curated ship + the `entity-model` link
   disposition (C4). *Verify:* no `doc/`-wholesale embed; no dangling link.
3. **Resolve OQ-5** → append-to-`routing-process.md` (C3). *Verify:* PUSH leg
   scoped to the reference-forms delta only.
4. **Re-scope the de-dup leg** to named file:line offenders (C1). *Verify:* the
   slice lists targets; no blanket "audit every skill."
5. **Adjudicate C5** — reference doc unique payload, or fold into `AGENTS.md`.
6. **Promote the two automatable checks to VT** (C6).

Then, and only then, may the binding move `proposed → accepted`. Until the
confessions are entered into the Decision, the wheel turns slow.

> **HERESIS URITOR; DOCTRINA MANET**
