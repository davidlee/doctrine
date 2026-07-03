# REV REV-019 — Exposed-slot override via self-replaces

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

### The defect (ISS-206)

SPEC-023 self-contradicts on exposed slots. Two clauses that cannot both hold:

- **Narrative (precedence §):** "the user wins only the *same-slot* tiebreak
  (the legitimate customisation)." Reads as: an exposed projected starter
  *overrides* its framework origin.
- **Mechanism (suppression §):** "Concatenation is the rule; `replaces` is the
  only suppression." Provenance in the precedence key is an **ordering** term,
  never a suppression term (`replaces_suppression`, `src/hymns.rs:392`). So the
  same-slot tiebreak only *reorders* the two twins — both still emit.

Result at an EXPOSED slot: the install projector writes a byte-identical editable
starter (`.doctrine/hymns/role/worker.md`) carrying no `replaces`; the framework
snippet and the user twin share slot + selector; the resolver emits **both**. The
narrative promised override; the engine delivers append.

The Concerns § "double-emit … wasteful, never incorrect" valve does NOT cover
this. That valve is for **author-chosen box intersections** — two overlapping
selectors an author picked. ISS-206 is a **seal/expose projection twin**: not an
author's box choice. And in the case the exposed-editable feature exists for —
the user *edits* the starter to `B'` — the output is `B` (framework) + `B'`
(edit): the customisation is appended to the very thing it meant to replace.
That is incorrect-against-intent, not merely wasteful.

No "wontfix by design" is available: there is no coherent design to defer to.
Closing wontfix ratifies a self-contradictory active spec.

### The fix — expose as the mirror of seal (self-`replaces`)

Affirm the override intent; deliver it through the **existing** suppression
mechanism, not a new one. The projector, when it writes an exposed editable
starter, also writes a sidecar setting `replaces` to the starter's **own slot**.

Seal and expose become symmetric single-emit suppression:

- **seal** → drop the *user* twin before matching (framework wins) — REQ-323, delivered.
- **expose** → keep the user twin; it self-`replaces` the *framework* origin (user wins) — this REV.

`replaces` stays the only suppression; the precedence key stays ordering-only;
INV-2 (append-unless-`replaces`) is untouched. Rejected alternative — "implicit
same-slot override" (equal-specificity higher-provenance suppresses) — mutates
the core compose rule to fix a projection-specific problem; not taken.

### Mechanics — pressure-tested against `src/hymns.rs` (unchanged engine)

Active set at exposed slot `S` = { `F`: framework, no-replaces; `U`: user,
`replaces=S` }:

- Seal filter (`hymns.rs:361`) drops user twins only on *sealed* slots; exposed
  ⇒ `U` survives (exactly why it doubles today).
- `is_unique_top_of_slot(U)` (`:435`): same slot ⇒ same band/specificity/alpha;
  provenance user>framework ⇒ `U` is strict-max, count-at-or-above = 1 ⇒ **unique
  top. INV-3 (unique-most-specific replacer) passes.**
- Self-edge `own==target` excluded from the cycle graph (`:447`) ⇒ no false cycle.
- Suppression loop (`:428`) suppresses every slot member where `j != r.carrier`
  ⇒ suppresses `F`, keeps `U`. Output = `U` only.

Both cases land: unedited `U.body==B` → single `B` (dedup); edited `U.body==B'`
→ `B'` only, framework `B` suppressed (**customisation genuinely wins**).

### Known gap (documented, out of scope)

A **hand-authored** snippet at an exposed slot with no projection sidecar carries
no self-`replaces` → still doubles. That is outside the projection mechanism this
spec describes. Not fixed here; the "implicit same-slot override" alternative
would be the only thing that covers it, and it is rejected above. Record as a
follow-up if demand appears.

## Change rows — before / after

### REQ-329 (primary) — suppression via `replaces`

Extend, do not replace. `replaces` remains the only suppression and the
precedence key remains ordering-only. Add: a projected exposed editable starter
carries a `replaces` targeting its own slot, so the user overlay suppresses its
framework origin — the expose-side mirror of seal. Amend the SPEC-023 suppression
§ and the precedence-§ narrative line accordingly.

- **Before (precedence §, spec-023.md ~L106):** "the user wins only the
  *same-slot* tiebreak (the legitimate customisation)."
- **After:** the same-slot tiebreak is *ordering only*; on an **exposed** slot,
  the projected editable starter additionally carries a self-`replaces`, so it
  *suppresses* (not merely outranks) its framework origin — override delivered by
  `replaces`, the mirror of seal.

- **Before (Concerns §, spec-023.md ~L217):** "Double-emit at box intersections
  … wasteful, never incorrect. Accepted as the duplication valve's cost."
- **After:** scope this explicitly to **author-chosen** overlapping selector
  boxes. Seal/expose **projection twins** are NOT this case: an exposed projected
  starter self-`replaces` its origin (single-emit); only author-authored box
  intersections ride the accepted valve.

### REQ-323 — seal enforcement

State the seal/expose symmetry. Seal drops the user twin before matching
(framework wins by active exclusion); expose is its complement — the user twin is
kept and self-`replaces` the framework twin (user wins). Both resolve to a single
emit.

### REQ-322 — corpus loader / install-time projection

The projector, when writing an exposed editable starter, also writes the sidecar
`replaces=<own slot>` so the projected artifact is a genuine suppressing overlay,
not an appending twin. (`interactions.toml`: "install-time projection of exposed
starters" is the touched mechanism.)

## Provenance

Originates from ISS-206 (backlog issue). Descends from the SL-186/SL-187 locked
designs whose narrative already carried the same contradiction now enshrined in
SPEC-023.
