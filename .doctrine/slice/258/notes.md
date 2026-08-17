# Notes SL-258: Governing commitment vertical experiment

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design session 2026-08-17 — settled positions

Design run `dr-01a00e5b`. Recorded here rather than on the inquiry nodes:
`body` is inert on `inq-` subjects (section-only), and resolution requires a
disposition the user has not yet ratified.

**inq-1 — what carries a commitment.** None of the three researched candidates
(new kind / `DEC` reuse / REV-admitted rows). A commitment is a **slice-local
text block in a per-slice commitments file, doc-local ids, convention-enforced**.
Commitments die with the slice — designs are slice-local and there is no reason
to promote entities retrospectively out of closed slices — so durability,
cross-slice citation and id-collision all dissolve. `DEC` stays what it is.
Authority inside a slice: durable canon (ADR/POL/STD/SPEC/REQ) > the slice's
governing set > design prose. Prose elaborates **downward**; any narrowing,
excepting or conditioning of what a block imposes is a new block, not prose.

**inq-2 — admission.** No new gate, and **no change to what lock ratifies**:
the design-run machinery is expensive to change and several large slices from
settled, so retargeting its acceptance is scope bloat. Admission is a runbook
step — consult the commitments file, pin each commitment against an `EX`.
Superseded en route: `src/commands/design.rs:1359-1366` (the design run's
acceptance attestation is the only route to an `accepted` `DEC`) is true but no
longer load-bearing, since `DEC` is not the carrier.

**inq-3 — admission against cited evidence.** Survives as a runbook line, not a
mechanism: when writing a commitment, check it against the evidence the design
already cites. Basis: `SL-213`'s `P8` (normative prose contradicting the
design's own pinned prototype goldens) pre-existed in the governing tier.

**inq-5 — discharge pins.** An `EX` cites the commitments it discharges,
hand-authored in `plan.toml` by convention. Rides `R-a` (plan.toml is
hand-authored; the tool reads, never rewrites) instead of fighting it, and
`R-c` (EX rows are not entities) stops mattering — no relation is minted.
Open: the exact spelling.

**inq-6 — vacuity.** Two read-the-file checks, no machinery: a commitment with
no `EX` pin (`T7`'s `R6` — honestly vacuous and silently dropped are
indistinguishable to a link, so absence must surface) and an `EX` with no
commitment (a plan obligation with no governing reason).

**inq-7 — semantic diff.** `git diff <lock>..HEAD -- <commitments file>`,
instructed in the reconcile skill. No renderer, anchor object, digest or
watermark. Of the three things a diff could show: membership/content churn is
this git diff and is cheap-when-needed; letter-vs-implementation is not
computable and stays a human judgement with somewhere to land; copy drift is
the real target and is addressed by citation-instead-of-restatement. Edge case
(user): blocks are not strictly immutable — a block can change in place without
superseding, which the git diff does catch.

**inq-8 — challenge / refine.** Instruction text or a skill, optional
bash/python helper. **Open: where a challenge lands.** A separate ledger hides
historical annotations as well as a graph would, keeping the commitments file
short — candidate sinks are an appended ledger section, a sibling file, or git
history alone. Preserve `Δ8`: these are the missing bottom rungs of the existing
ladder (notes flag → RV finding → REV) — cheaper than an RV finding and
*consumable*, unlike `SL-213`'s `ADJUDICATE AT AUDIT` flag that sat unread in
notes until audit.

**inq-11 — the vertical.** All five scoped legs exercised, **none in Rust**.
Deferred as researched: adaptation envelope, inert-check drift (RFC-029's
proving seam), the `VT`→`EX` binding (RFC-029 owns it).

**inq-12 — process wiring, now the whole slice.** Skill masters live at
`plugins/doctrine/skills/<id>/SKILL.md`, a RustEmbed root
(`src/install.rs:18-20`), materialised to `.doctrine/skills/` and **copied**
(not symlinked) into `.agents/skills/` and `.claude/skills/`. An installed copy
is derived and is clobbered by the next install — edits land on the plugins
master, and the change still needs `cargo build` (re-embed) then
`doctrine install`. Zero logic change, **not** zero build cycle. Open: which
skills change, and whether the authoring discipline rides a skill, a template
or a standard.

**inq-13 — measurement. Drift stays the primary concern (user).** An earlier
draft of this note demoted drift-catching in favour of "does lock ratify
anything"; that over-read `Δ2`. What `Δ2` exhausted is *retrospective*
measurement — record completeness correlates with apparatus weight, so absence
of recorded drift proves nothing after the fact. Prospective measurement is
tractable, and this apparatus is what makes it so: the commitments file is a
fixed `before` at lock, the reconcile git diff is the `after`, and the subject
slice's own audit is the independent check. The load-bearing number is the
**catch ratio** — of the commitment-shaped findings the subject slice's audit
raises, how many were already surfaced by a challenge during the drive versus
discovered fresh at audit. That is the quantity the six case studies were
trying to estimate retrospectively and could not.

Secondary, kept but not promoted: designs are often literally unreadable after
several adversarial rounds and the user does not read them, so lock arguably
ratifies nothing today. Worth observing during the experiment; not the measure
the slice is judged on. Legibility in general is explicitly *not* this slice's
target.

**inq-14 — governance instruments.** Leaning defer, unratified. `OQ-3` was
adjudicated at scoping (canon as ADR/POL/Spec via a Revision) but presumed a
mechanism slice. A convention-only experiment should not mint canon before it
has results; instruments become the follow-on if measurement supports the
discipline.

**Pruned.** `inq-4` (behaviour over a half-populated claim tier — presupposed a
machine check over a durable claim tier; neither exists). `inq-9` (ledger reuse
vs transplant — no Rust ledger is built, so `R2` cannot occur; the live residue
moves to `inq-8`).

**Retracted en route, so it does not get re-litigated.** Fingerprint-bound
attestation as a ratification device (`src/design_run/attestation.rs:242`) —
lock already means ratification, and the answer to riders is brevity, not a
seal. A reified governing-set anchor for the diff's "before" side — dates and
git suffice. And the seed's closing thesis that documents are mere authoring
surfaces: inverted, then re-inverted to entities-over-prose *within* the slice.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-17 · design/exploring (run `dr-01a00e5b`, rev 5) · 576bd48ae

### Produced
- Design run `dr-01a00e5b` — 14 inquiry nodes, 12 blocking, `inq-4`/`inq-9` pruned.
- `ISS-447` — hymn stage band unreachable.
- This file's "Design session 2026-08-17 — settled positions" section.
- Two friction observations (`design apply` needs-ordering; no read verb for
  inquiry question text).

### Learned
- Delivery vehicle for this slice: the hymn cascade's `project` band. See
  `ISS-447` for the stage-band half and `src/design_run/prompt.rs:4-9` for why
  the design-prompts store is closed to it.
- The scoped five-leg vertical narrowed to convention-only, zero Rust. Positions
  per node in the settled-positions section above.

### Open
- Undisposed blocking inquiries: `inq-1`, `inq-2`, `inq-3`, `inq-5`, `inq-6`,
  `inq-7`, `inq-8`, `inq-10`, `inq-11`, `inq-12`, `inq-13`, `inq-14`.
- Four user calls gate the exit from `exploring`: project-vs-stage band
  (`inq-12`), the challenge sink (`inq-8`), whether to record `governed_by`
  ADR-005 / POL-002, and the `governance-confirmed` + `graph-reviewed`
  attestations.
- `inq-1` is to be disposed `create` (a `DEC`, user-agreed); the other eleven
  `non-durable`.
- No subject slice named yet for the experiment — user is finding one. Closure
  needs it (`inq-13`).
