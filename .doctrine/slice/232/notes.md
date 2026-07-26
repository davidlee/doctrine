# Notes SL-232: Corpus-aware memory verify gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-26 · design (inherited, not started) · 9f8cf40b

### Produced

Nothing yet — this slice has done no design work of its own. Everything below is
**inherited** from SL-230 by DEC-027 at RV-307 round 8.

- `design.md` — the gate half of SL-230's design, carried verbatim so that eight
  adversarial rounds of measured evidence are not re-derived. **NOT locked.**
- `slice-232.md` — scope authored at split time; six objectives.
- REV-034 — SPEC-007 + REQ-147 amendment. Proposed; `SL-232 needs REV-034`.
  Applied at close so spec and code turn over together.

### Learned

Inherited lessons, all from RV-307. Method, not trivia — this slice inherits the
failure modes along with the design.

- **A tool property is a claim needing a falsifier, not a premise.** "Stable",
  "total", "deterministic" must each be probed by varying the local state the
  instrument reads. Measuring that a discriminator *works* is not evidence it is
  *stable*. Two of eight rounds went on this (F-25 → F-31).
- **Register the falsifier before running the probe** (F-17, F-23) — a probe that
  cannot distinguish the two outcomes proves nothing. Committed three times, twice
  by the party that had just named it.
- **Totality claims over path resolution have failed three times, differently each
  time** (F-26, then F-32, then F-37). Enumerate the reachable shapes and probe
  each *before* stating a rule over them.
- **The integration sweep has three tiers**: normative prose, routing-record
  bodies (F-34), and **queried metadata** — titles, slugs, tags (F-39). A prose
  sweep reaches one of three; `grep` over the design reaches one.
- **Ask what general rule a finding instantiates — then check the domain that rule
  holds over** before promoting it (F-26/F-27: the fix for over-specificity was
  over-generality).

### Open

**Two inherited blockers. These are the design work; start here.**

- **RV-307 F-37 (blocker)** — the premise that a non-resolving entry contributes
  nothing is **false**. Three reproduced routes (git 2.54.0) contribute while
  bypassing canonicalisation and read clean against a dirty target: unresolved
  `..` alias, sparse checkout, and a `scope.paths` literal whose filename contains
  `*`. Falsifies § 5.2 steps 4–5 and I9. Underneath it: path-vs-pattern is
  inferred from *characters* when the schema already records which field the entry
  came from.
- **RV-307 F-36 (blocker)** — DEC-020 requires `validate` to raise every
  non-contributing entry; D11 leaves `validate` no contribution probe at all.
  Needs a second per-entry git path plus a corpus-wide continuation policy
  (F-29's shape). The ruling is applied normatively, not mechanistically.

**Two inherited majors.** **F-38** — NUL / newline scope entries escape E11/E13;
NUL cannot cross the argv boundary so it yields no git exit code at all, and
newline splits E7's report. **F-39 limb 1** — code-only wording at D9 and two
§ 5.2 sites, contradicting § 4's claim-not-code boundary.

**Three inherited contests** — returned by the raiser, concerning text that moved
here, so they are this slice's to answer: **F-25** (a stamp covering less than the
declared evidence), **F-26** (I9 totality and the class collision), **F-32**
(prefix splitting and probe abort).

**Open questions.**

- **OQ-6 — the reason this slice exists.** Should non-contribution ever refuse, and
  on which entries? DEC-020 deferred it. The answer that survives cloning is a
  **declared** boundary, not a derived one → a schema change.
- **OQ-2 / QUE-175** — does claim-surface drift feed retrieve-side ranking, or only
  `validate`? Gates **IMP-317**, which closes `wont-do` if the answer is no.
  SL-230 could only defer this; this slice must answer it.
- **OQ-3 / QUE-173** — body digest. **IMP-318** — persist attested coverage.
  DEC-020 argues these and OQ-6 are one schema change; that is an argument, not a
  measurement (**OQ-A**).
- **OQ-5** — should the *source* leg narrow to declared scopes too? Not taken in
  SL-230; reopenable here.
- **OQ-B** — `validate`'s contribution mechanism and its continuation policy.

**Risks.** R-A (the surface may not be totalisable from git alone — enumerate then
probe), R-B (`validate`'s probe undesigned), R-C (SL-230's R4 runs unmitigated
until this lands — sequence accordingly), R-D (hostile scope input incomplete),
plus inherited R6 / R7 / R8.

**No ledger yet.** RV-307 stays attached to SL-230 (append-only; it reviewed that
document). Open a fresh RV when this design is ready for adversarial review, and
seed it from the findings above. `design.md` § 10 carries the inherited view
organised by state rather than by round.
