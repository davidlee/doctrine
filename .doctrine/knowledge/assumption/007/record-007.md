# ASM-007: Interpretation classes exhaustive

**INVALIDATED 2026-08-01 by SL-241 PHASE-04 step 0.** Retained as the record of
a claim that was posed, tested, and failed. Do not carry it forward, and do not
repair it — read *Why the shape failed*, below, before proposing a sixth class.

The claim was: the trigger classes in [[interpretation-surface]] are exhaustive —
every way untrusted content acquires agency on the trusted side falls into one of
them. **Five numbered classes, one with a git-level sub-class** (RV-340 F-13):
1 explicit execution · 2 build-system evaluation · 3 toolchain auto-load, **3g**
git-level auto-load · 4 path-shaped data · 5 resource shape. Six rows, five
classes.

The replacement claim is [[interpretation-surface-responsibility-split]]
(ASM-008). The taxonomy itself is **not** retired — CPT-001 remains active and
useful; only the exhaustiveness claim about it is dead.

## How it was tested

Exactly as this record specified: an independently-derived enumeration of a
second ecosystem's triggers, classified against the taxonomy, residue as the
falsifier. The protocol held up and is worth reusing.

- Enumeration authored at commit `beb4b665` by a context that had **not** read
  CPT-001 — 96 npm/TypeScript triggers across 11 surfaces, walked by ecosystem
  surface rather than by recalling the taxonomy.
- Classification appended at `61ea9f08`, after reading CPT-001. The diff between
  the two commits is **append-only** (190 insertions, 0 deletions), which is what
  evidences the independence.
- Full artifact: `.doctrine/slice/241/step0-enumeration.md`.

Result: 84 clean fits, 6 fits with named strain, 2 out of scope, and **4 residue
items**. The clean-fit bulk is real portability evidence for DEC-107/DEC-108 —
`binding.gyp` ≡ `build.rs`, `packageManager` ≡ `rust-toolchain.toml`, `.mise.toml`
≡ `.envrc` all land without stretching a class.

## The residue

- **R1 — prompt injection into an LLM reading arbitrary tree content.** Fixed-name
  agent files (`CLAUDE.md`, `AGENTS.md`) *are* class 3 and fit. The residue is the
  unbounded-carrier case: the interpreter is a model, the carrier is any file it
  reads, so the trigger path is not enumerable and the `interpret:` declaration
  format cannot express it. **Not a discovery** — [[capsule-threat-model-boundary]]
  (CON-005) and SL-241 design § 1.1 already name it as a known unbounded threat,
  with an operator ruling and a structural mitigation (refusals report
  trusted-side-computed tokens; artifact content is never relayed verbatim).
  It contradicts exhaustiveness while corroborating what the design already knew.
- **R2 — terminal escape sequences.** Untrusted content reaching a trusted-side
  *rendering* channel with its own execution grammar: OSC 52 clipboard writes,
  input-injection sequences, ANSI making a diff display differently from what it
  is. Not 1 (the interpreter is neither the binary nor the content — it is the
  display), not 2/3, not 4, not 5.
- **R3 — untrusted metadata interpolated into a trusted-side command.** Ref names,
  commit messages, PR titles. All five classes model **tree content**; metadata
  rides the same push without being tree content. Class 4 admits ref names only in
  their *path* aspect. The LLM-carrier half of this is anticipated by CON-005; the
  shell/template-injection half is not.
- **R4 — non-inert parsing.** `{"__proto__": {…}}` deep-merged into a config
  object mutates JavaScript's shared prototype, so *later, unrelated* trusted code
  behaves differently. Nothing is executed; the parse was supposed to be inert.
  Its neighbour `!!js/function` in YAML *is* class 2 — there the loader genuinely
  evaluates. Narrowest of the four, and not live for doctrine's own Rust control
  plane, but real for the client languages the taxonomy has to hold for.

**R2 and R4 are individually sufficient.** R1 and R3 were partly anticipated
elsewhere, so the evidence is thinner than the count of four suggests.

## Why the shape failed, not just the claim

The important finding is not that five classes missed four triggers. It is that
**"these classes are exhaustive" is the wrong shape of claim** (operator ruling,
2026-08-02).

It is a universal negative over an open-ended adversarial space, revisable by
anyone who thinks of something new. No amount of search settles it, so it can
only ever sit at low confidence — and its practical effect was to make an *empty*
residue look like progress, when an empty residue would mostly have measured the
searcher.

So: **do not add a sixth class and re-assert exhaustiveness.** That re-runs the
same error one class later. The residue items are recorded as evidence and as an
open question against the taxonomy ([[downstream-interpreter-classification]],
QUE-203) — not as a gap to be plugged and closed.

What the design actually load-bears on is *not* exhaustiveness. It is the
**universal / language-bound split** — classes 1–3 are the client project's to
declare, 3g/4/5 are doctrine's to ship — which is what makes POL-002 satisfiable,
and which ASM-008 now carries. Nothing in the residue touches that split. The
exhaustiveness claim was load-bearing for one closure sentence in SL-241
design § 9, and for nothing else.

A useful diagnostic for the next claim of this kind: *if this assumption were
false, what would I do differently?* For exhaustiveness the answer was "record a
new class" — bookkeeping, not a decision. For the responsibility split the answer
is "redesign who declares what", which is a real consequence.

## Related

- [[interpretation-surface]] — the taxonomy. Still active; only this claim died.
- [[interpretation-surface-responsibility-split]] — ASM-008, the replacement.
- [[downstream-interpreter-classification]] — QUE-203, the open question R1/R2
  raise, tracked rather than pre-resolved.
- [[capsule-threat-model-boundary]] — CON-005, which already named R1.
- [[interpretation-surface-ownership]] — DEC-099, what rests on the split.
- [[two-spike-fixtures]] — the portability control; that claim is unaffected, and
  was corroborated by the clean fits.
- SL-241 `.doctrine/slice/241/step0-enumeration.md` — the enumeration and
  classification in full.
- RV-340 F-8, F-13 — which sharpened the falsification protocol that worked.
