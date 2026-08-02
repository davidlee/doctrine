# Review RV-323 — design of SL-233

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Pre-reading is the complete marker-grammar sketch
(`.doctrine/slice/233/sketches/marker-grammar.md`); `plan.toml` PHASE-06 EN-2,
EX-1..EX-12 and VT-1..VT-4; `design.md` §§5.2, 5.3 (rules 1–3), 5.4, 5.5 and
9.2; the projection-bounds sketch's § *The layer rule* and § *The second rule:
provenance*, which this sketch claims to obey; and the incumbent code the
sketch is about — `src/commands/design.rs` (`MARKER_OPEN`/`MARKER_CLOSE`,
`authored_sections`, `authored_section_digests`, `render_document`,
`materialise`, `plan_checkpoints`), `src/design_run/ids.rs`, and
`src/design_run/run.rs::checkpoint_disposition`.

The review holds the sketch to EN-2's seven-part entrance contract — **an
answer must settle a question, not merely name it** — under EX-10's constraint
on how (a) and (d) may be answered.

The author is the orchestrator; the sketch is therefore reviewed by someone who
did not write it, per EN-2.

Lines of attack:

- **Falsify the governing claim** — "materialise-then-parse is the identity on
  the section map". Find a document, or a body, for which the round trip loses
  or reassigns a section and no refusal fires. The escaping scheme in (c) claims
  to be a bijection on marker-shaped lines; test that claim against bodies whose
  lines are marker-shaped under one direction's definition and not the other's,
  and against ids that are legal in the shape test but not in `DesignId::parse`.
- **Attack the refusal partition in (e) for totality**, which the sketch itself
  names as its second-least-confident claim. Five refusals are asserted to cover
  every way a document can fail to describe its run. Construct a mangled
  document that falls through all five, or that fires two under an evaluation
  order the sketch did not fix. Blank-line and whitespace-only edge cases at the
  head of the document are explicitly in scope.
- **Test (a)'s length answer against EX-10(a)** — the grammar's maximum encoded
  length is claimed to be exactly `DESIGN_ID_BYTES` = 32, satisfying the bound
  at equality. Check that the claim is *derived* rather than asserted, per the
  provenance rule the sketch invokes, and that the charset argument (three
  requirements, `[A-Za-z0-9_-]` the maximal satisfying set) actually excludes
  every byte that could terminate or split a marker.
- **Test (d) against EX-10(c)** — collisions for ids that are distinct but share
  a long common prefix, not only exact duplicates. The sketch claims structural
  immunity via whole-token comparison. Verify no prefix, abbreviation, or
  normalisation survives anywhere in the parse or lookup path, including
  `authored_section_digests`' `BTreeMap` collection and `SectionGroup::find`.
  The case-fold and leading-zero refusal-to-normalise is a deliberate choice —
  attack it if the cost is misjudged.
- **Cross-examine (b)'s retitle answer** against DEC-066 fingerprint-bound
  invalidation and §5.3's watermark rules. The sketch's load-bearing negative is
  that a section id is never title-derived; check that nothing in the model,
  the wire, or the incumbent makes a slug-derived id reachable anyway, and that
  the `Section.title` / heading dual source cannot desynchronise silently.
- **Audit (f)'s evidence for over-claiming.** It rests on one formatter at one
  version in one jail, and the sketch distinguishes "the marker map survives"
  from "the bytes survive". Check that no claim elsewhere in the sketch quietly
  assumes byte survival, and that the two parser obligations the measurement
  produced (no adjacency dependence; mandatory right-trim) are actually
  sufficient for what was measured.
- **Attack § *The incumbent*'s nine-defect table**, which the sketch flags as
  its highest-value and least-verified section. Each row is a reading of code,
  not an executed test. Find a row that is wrong — already handled elsewhere in
  the call path — or a tenth defect in the same files that the table missed.
- **Check EX-11 and EX-12 are load-bearing rather than decorative.** EX-11
  claims id order is live in `render_document`; EX-12 claims the annotation
  spelling is wire-only with no internal producer and that its removal is
  therefore a cleanup rather than a breaking change. Both are factual claims
  about the code and both are falsifiable. VA-NC2 further claims the annotation
  test can be made to red as a *wrong acceptance* before the fields are removed
  — check that ordering is actually achievable.
- **Hold the sketch to its own rules.** It invokes the projection sketch's layer
  rule and provenance rule. Find a bound, a constant, or an enforcement
  guarantee in this sketch that is asserted without derivation, or that acts at
  the wrong layer. The projection sketch's history is that the defect hides in
  the *repair*, not the original — and that sketch's falsifier now stands for
  the pair of rules.
