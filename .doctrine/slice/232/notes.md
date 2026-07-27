# Notes SL-232: Corpus-aware memory verify gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-27 · design (decisions taken; `design.md` NOT yet rewritten) · c00d6705

### Produced

- **DEC-053** — claim surface is built from the index, never the filesystem. The
  replacement for `design.md` § 5.2's ordered algorithm.
- **DEC-054** — `validate`'s two unknowns unify; **ISS-257 absorbed**
  (`SL-232 fulfils ISS-257`), designed as one mechanism with F-36 under a new
  objective 7.
- **QUE-175 answered `yes`** on measurement; **IMP-317 split** — limb (a) taken
  here on objective 4, limb (b) retained and rescoped. IMP-317's body H1 also
  corrected (it still named the constructor F-27 rejected).
- **`slice-232.md` rescoped** — objectives 2, 3, 4, 5 rewritten; objective 7
  added; risks R-E, R-F, R-G added; OQ-A answered `no`.
- **`research/probes/`** — seven executable probes plus README, each with
  falsifiers registered in-header. The evidence base for every measured claim
  above, committed so it can be re-run rather than re-derived.
- **RFC-011 case notes** — three entries (ledger findings unreadable via CLI;
  no sink for design-round probe scripts; `slice show` drops a `fulfils` edge).

**Not yet produced — the bulk of the remaining work.** `design.md` is still the
inherited text and now **contradicts** the scope document in several places. It
must be rewritten, not patched.

### Decisions taken this round

- **Index-first replaces the ordered algorithm** (DEC-053). Emit by field of
  origin, expand via `ls-files -s -z`, resolve mode-`120000` matches from index
  blobs, guard aborts lexically.
- **ISS-257 absorbed and unified with F-36** (DEC-054), anchored on REV-041.
- **OQ-A answered `no`** — the declared boundary is an authored input; IMP-318
  and QUE-173 are machine-written outputs. Sequenced, not merged. **R8 stays open
  after this slice.**
- **OQ-2 answered `yes`** (QUE-175) — 13 of 43 scoped+attested memories mis-moded.
- **Declared boundary shape** — a parallel assertion over entries that stay in
  `scope.paths`/`globs`; chosen because it is *falsifiable* and never subtracts
  from the claim surface (I8 holds). Three alternatives rejected on the record.

### Learned

**This round.**

- **When a rule keeps failing over a domain, suspect the *instrument*, not the
  rule.** F-26, F-32 and F-37 were three failed totality claims over path
  resolution. They were one defect: a filesystem oracle (`realpath`) answering an
  index question. The fourth repair would have failed too. The generalisable
  move was to find the reformulation that makes the failing taxonomy
  **non-load-bearing** — not to enumerate harder.
- **Ask what the finding's *underneath* is before fixing what it reports.**
  F-37's fourth observation (path-vs-pattern inferred from characters when the
  schema records the field) had longer reach than its three routes.
- **Read the audit of the slice you were split from.** SL-230's RV-313 / REV-041
  / ISS-257 all postdate DEC-027 and directly falsify D11 and R7. Nothing in the
  handover pointed at them; the user's question did.
- **A `None` means two things — check whether the guard makes that safe.** The
  hypothesis that `GitFacts::default()` and a failed probe collide was
  **refuted**: `staleness()` branch 1 is guarded by the same predicate
  `git_facts` gates on. Recorded because refuted hypotheses are evidence too.
- **The correct shape is often already one module away.** `coverage.rs`'s
  `None => Unknown` and `retrieve`'s B18 per-candidate contract both predate
  ISS-257's remedy and should be ridden, not re-invented.
- **The sweep has a fourth tier.** F-39 named three (normative prose,
  routing-record bodies, queried metadata). A body's own **title-shaped H1** is a
  fourth — IMP-317's heading still named the rejected model after the retitle
  moved the queried field and the slug.
- **State a fix's value by what it makes impossible, not by what it fixes
  today.** Index-first adds **zero** live coverage for declared scopes. Claiming
  otherwise would be the overclaim F-25/F-33 punished.
- **Stamp every corpus figure with its HEAD** (RV-313 F-1 — denominator drift
  broke a design-time absolute at execution).

**Inherited lessons, all from RV-307.** Method, not trivia — this slice inherits
the failure modes along with the design.

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

**THE BIG ONE: `design.md` is stale and now contradicts `slice-232.md`.** It is
still the inherited SL-230 text. Decisions are taken; the document does not carry
them. Rewrite, do not patch — § 5.2 (replaced wholesale by DEC-053), § 5.4 (D11
falsified), § 5.5 (I9 must be re-expressed as an **outcome** property, not a
pre-emission one; E13 re-justified or folded), § 7 (D9/D10/D11), § 8 (R-A/R-B
restated; R-E/R-F/R-G added), and § 9 — **the whole T-matrix pins an algorithm
that no longer exists**.

**Inherited blockers.**

- **F-37 — ANSWERED** by DEC-053. Three routes reproduced, diagnosed as one
  defect, algorithm replaced. Probes at `research/probes/`.
- **F-36 — answered in shape** by objective 7 / DEC-054; **open in detail**. The
  probe mechanism and continuation policy still need designing (OQ-B), though
  B18's per-candidate-never-abort precedent largely supplies the latter.

**Inherited majors — both still open.**

- **F-38** — two distinct obligations, not to be conflated: *rejection* at the
  write verbs for argv-unrepresentable values (NUL yields no git process, so no
  exit code — outside E11/E13's taxonomy entirely), and *escaped framing* for
  anything reaching a report (newline splits E7 across lines). Live corpus
  population: **0**.
- **F-39 limb 1** — code-only wording at D9 and two § 5.2 sites, contradicting
  § 4's claim-not-code boundary. Will be swept by the rewrite; verify it was.

**Inherited contests.**

- **F-26 and F-32 — answered at the root** by DEC-053, not at the split point.
  Both contests charged that the shape taxonomy was stated over an unenumerated
  domain; the taxonomy is now non-load-bearing. Write the dispositions to say
  *that*, not "we fixed the prefix rule".
- **F-25 — still open.** OQ-A's `no` means IMP-318 is not built here, so **R8
  (an attestation does not record what it covered) survives this slice**. F-25's
  charge is answered *in part* — the declared boundary makes the shortfall
  authored rather than inferred — but a stamp still cannot distinguish full from
  partial coverage. Say so plainly; do not let objective 3 read as closing it.

**Open questions.**

- **OQ-6 — shape decided** (parallel assertion; see `slice-232.md` objective 3),
  **field name, type and validation rules still to design.** Live population: 20
  of 59 non-contributing entries would be declared.
- **OQ-B** — `validate`'s contribution mechanism and continuation policy.
- **E13's fate** — its mechanical-necessity basis is dissolved (aborts are now
  prevented lexically). Re-justify or fold into E7. Live population: **0**.
- **OQ-5** — should the *source* leg narrow to declared scopes too? Untouched;
  still reopenable. Nothing this round bears on it.
- **OQ-3 / QUE-173 (digest), IMP-318 (attested coverage)** — routed, explicitly
  **not built here** per OQ-A.
- **~~OQ-2 / QUE-175~~ — ANSWERED `yes`.** ~~OQ-A~~ — **answered `no`.**

**Risks.** R-A **discharged in method** and narrowed to R-E/R-F. **R-E** (index
bits `S`/`h` suppress the measurement; no pathspec approach closes it; live
population 0). **R-F** (case-insensitive collision unmeasured, not cleared).
**R-G** (absorbing ISS-257 widens the blast radius to a corpus-wide seam no
original criterion governed). R-B now shaped, not undesigned. R-C, R-D, R6, R7
(partially closing), R8 (**survives this slice**) as inherited.

**Governance debt — do not defer to close.** `validate` appears in SPEC-007
exactly **once** (the sentence REV-041 added), and REV-034's amendment inventory
was drawn for `verify` before objective 7 existed. Re-take it during design.
REQ-147's title is the retired contract verbatim, so F-39's queried-surface trap
applies to any requirement left unamended.

**No ledger yet.** RV-307 stays attached to SL-230 (append-only; it reviewed that
document). Open a fresh RV when this design is ready for adversarial review, and
seed it from the findings above. `design.md` § 10 carries the inherited view
organised by state rather than by round.
