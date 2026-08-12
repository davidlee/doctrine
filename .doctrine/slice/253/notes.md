# Notes SL-253: Conformance verdict kernel

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-13 · design/reviewing · rev 79 (head a7c7bfe14) · four
external passes integrated, plus one author self-audit; `RV-354` awaits the
raiser on `F-1` and `F-5`

### Produced

- `EVD-021` — the vacuous-admission finding.
- `EVD-022` — the pre-split nineteen-row baseline, captured in-jail at
  `4662e64eb`. The pre half of `DEC-199`'s bracket; uncapturable later.
- `DEC-195` — verdict shape: reduced floor, published profile, named admission axis.
- `DEC-196` — the seam cuts at row identity: kernel keeps identity and judgement,
  construction goes to the payload.
- `DEC-197` — the kernel classifies as a leaf (**required exit criterion**).
  Placement rider **re-cut 2026-08-12**: `BackendId` *and* `Availability` now
  move into the kernel, which then imports no other `doctrine-control` module.
  Crate extraction declined for this slice on scope; `IMP-404` holds it.
- `DEC-198` — row identity splits a closed floor from an open profile.
- `DEC-199` — row verdicts are the preservation bar; instruments and artefacts.
  Layer 2 **re-cut 2026-08-12**: the closed list of three permitted differences
  becomes a **closed transformation contract** — a total map from the pre-split
  artefact's lines to the post-split artefact's, two totality clauses, seven
  rules, and a whole-output golden test as its instrument. The stale
  `capsule-check needs no bwrap` claim is withdrawn in the same cut.
- `DEC-200` — test bands are carved before the split, two bands.
- `DEC-201` — one REV, four payloads; criterion 3's text narrows to the floor.
- Design run `dr-019ff3ff-b227-7030-a75b-65981efd87ca`, revision 49, stage
  `reviewing`, all eight inquiry nodes resolved, all ten sections drafted and
  materialised, `reviewing` runbook cleared.
- Findings `fnd-1` … `fnd-9` on the run — the agent hostile pass, raised at rev
  43, dispositioned at rev 45, integrated and materialised at rev 46.
- Design-local rulings `D7` (floor reading), `D8` (fronts payload-side), `D9`
  (source-side rename), `D10` (value-only kernel entry point), and force `F7`
  (three enumerations of the backward references have each been wrong). All in
  `design.md`; none banked as a `DEC`, on `D1`–`D6`'s precedent.
- Friction observation `019ff53c-902e-7ef2-b45b-685b241313a4` — `design
  materialise` renders sections in declaration order, so `design.md` reads
  1–5, 10, 6–9.
- Research artefact at `.doctrine/slice/253/research/` (runtime tier, gitignored;
  `raw/governance.md`, `raw/codemap.md`, `research.md`).
- Friction observation `019ff42b-2655-7aa3-b042-0a83272a328c` — `design apply`
  silently absorbs unknown payload keys. Now filed as `ISS-346` and banked as
  `mem_019ff439bede7fb29c7c09b7fd76d893` (the apply payload vocabulary).
- `IMP-427` — the deferred live-`bwrap`/neutral third test band.
- Findings `fnd-10` … `fnd-14` on the run — the **second** review pass, raised at
  rev 50. Three blocking (`fnd-10` `ArmResult` drags `backend::Termination` into
  the kernel; `fnd-11` front rendering escapes § 9.1's closed licence; `fnd-12`
  `Table::front_of` has no route to `main.rs`), two nits (`fnd-13` stale `§ 2.4`
  count in `P2`; `fnd-14` two off-by-one cites). **All five dispositioned and
  integrated at revs 51–57**; `design show` will not display them, so read
  `[[review.finding]]` in the runtime `design.toml`.
- Design-local rulings `D11` (the payload returns a `FrontCatalog` beside the
  verdict) and `D12` (the kernel takes `ArmJudgement`; diagnostics stay
  payload-side), added by the second pass. Same treatment as `D1`–`D10`: in
  `design.md`, not banked as `DEC`s.
- Codex consultation (3 rounds, thread `019ff556-c847-7800-aa5d-27eb38931e0d`) —
  the repair for all three blocking findings, converged and recorded in the
  re-cut `DEC-197` and in **Open** below. Not an adversarial review; `RV-354`
  is still empty scaffolding and the external pass is still unspent.
- Friction observation `019ff550-6459-7002-bb5e-28945ff4c51b` — a raised finding
  is visible only as a change-log row, so `design show` reports zero outstanding
  while blocking findings await disposition.
- **`RV-354` — the external adversarial pass, now spent.** Codex (GPT-5.5),
  thread `019ff622-0e71-7a81-a449-d705a9ce4fd4`, four rounds: raise, then three
  verification rounds each attacking the repairs rather than accepting them.
  `F-1`/`F-3` blockers, `F-2` major, `F-4` minor, plus `F-5` major raised on the
  fourth pass. All five disposed `fix-now`; `F-2`/`F-3`/`F-4` **verified**,
  `F-1` contested three times and repaired four, `F-5` awaiting first
  verification. **Every contest was upheld against the artefact and the code —
  the repairs were the defective party, not the findings, on all five
  occasions.** `F-5` is the sharpest: it is a defect in a repair the author's own
  self-audit had just made.
- **Every `F-1` repair has been author-constructed and every one has been
  contested; the one finding that closed (`F-2`) closed on a remedy the reviewer
  wrote out verbatim.** Codex has supplied a diagnosis for `F-1` in every round
  and never a remedy. Five data points, not a law — but the row-trace repair at
  rev 76 is the least externally validated thing in the design, and its three
  predecessors were each wrong.
- Rev 78 closes two gaps round four left: `F-5`'s two options are not
  alternatives (a channel maximum ranges over per-return grades, so it
  presupposes them and is circular alone — § 7.2 now says the maximum is
  derived, not primitive), and § 7.3 records the `I11` alternative neither party
  raised — dissolve the observability rather than pin the order, by building the
  fixture eagerly before any closure runs. Not adopted; it narrows `I11` without
  removing the need for it, and is a hardening *on top of* `I11`, not instead. Findings and dispositions are on the ledger — read them
  with `doctrine review show RV-354 --format json`, since the table format
  summarises findings to a count.
- `D13` — the row runner returns `(ArmJudgement, ArmJudgement)`; the kernel
  adjudicates. `D10` narrowed with it. Its *generalisation* took three attempts
  and a self-audit, and now names **two** independent kinds of crossing (reported
  adjudication, type contamination) with **one** severity grade (reaching
  admission) over **six** location classes — the sixth being the complete
  callback interaction trace, pinned as `I11` in § 5.4. Each restatement failed
  structurally, and always in the same place: an *only where* on a test that
  could not carry it, then the grade as a peer of the kinds, then the grade
  attached to a callback instead of to a returned value. The seam analysis has
  been right since the first cut; only the location of the grade kept moving. Same treatment as `D1`–`D12`: in
  `design.md`, not banked as a `DEC`.
- `mem.pattern.testing.classify-the-expectation-before-trusting-the-assertion` —
  new, and the most portable thing the first verification round produced.
- `mem.pattern.review.repair-closes-a-subset-of-the-stated-class`
  (`mem_019ff6a77dec7b518fa063f1c3e24d5a`) — new, from the second verification
  round, and the sharpest lesson of the three. A repair derived from a sentence
  that enumerates a class closes one member and leaves the rest, because nobody
  re-reads the enumeration sitting three sentences away. § 5.1 listed three
  vacuities in one sentence; two consecutive `F-2` repairs closed the first and
  the third.
- Two existing memories strengthened rather than duplicated:
  `mem_019eda2fed8672d39214a5eeb3c86385` (pre-enumerated maps, broadened to a
  repair's own footprint) and
  `mem.pattern.rust.exhaustive-destructure-pins-hand-written-mappings` (the
  construction-side `..` sibling). Both already existed under scopes too narrow
  to surface when they would have helped.
- Friction observations `019ff661-d3db-7550-ad84-29d319c6a0c8` (`design apply`
  takes whole-section bodies, so a surgical edit needs a scripted
  replace-with-assert-fires-once) and `019ff662-647b-7cd1-b6a0-1385c468d4bd`
  (`review show`'s table format drops finding bodies; json carries them), and
  `019ff6a6-b6e0-7f71-9403-28e1956c159b` (a bare `ls .doctrine/state/` buries the
  real subdirectories under ~220 `mem-surface-seen-*.txt` receipts).

### Learned

- The change surface is far smaller than 14,252 lines implies: one reducer, one
  production call site, two production consumers.
- Row titles mislead. `TrustedTerminationObservation` reads epistemic and is a
  file-size resource bound. Read the `Row`'s `delta`, not its name.
- Nested `bwrap` works inside the project jail, so `just capsule-verify` is
  runnable on the development host — `ISS-339`'s never-run-off-jail note is not a
  blocker for this slice's evidence (`EVD-022`).
- Capsule time-to-interactive is **~2 min** from `capsule-baseline`, excluding
  delete and provision which are quick (owner, 2026-08-12). This is the figure
  behind `DEC-199`'s bias toward running `capsule-verify` more often.
- The 186 tests are flat: one `#[cfg(test)]` at `conformance.rs:5339`, **no inner
  `mod` at all**. Symbol triage sizes the carve at ~67 needing judgement.
- **`just capsule-check` needs `bwrap` too**, superseding the earlier note that
  only `capsule-verify` does: `cargo test -p doctrine-control` includes tests
  asserting `availability() == Available` and provisioning real capsules
  (`conformance.rs:6915`, `:7244`, `:7453`), and `EX-14` forbids skipping. A
  host without `bwrap` has no instrument at all until `IMP-427` lands.
- **The backward-reference enumerations were wrong in the body, not the
  signature.** Both misses — `host.path_exists(SHELL)` at `:5299`,
  `host_descriptor()` at `:5280` — were inside `verify_over`, while every pass
  read its parameter list. The `leaf` classification catches an *import*, not a
  re-declared constant or a `std` call.
- **…and the fourth was in a type's *field*.** `fnd-10`: `ArmResult::Indeterminate`
  carries `termination: Termination` (`backend.rs:751`). Signature, body, field —
  three location classes, each found only after the previous was closed.
- **Observational equivalence decides project-vs-move at a seam**, and it is
  mechanical where "does this feel neutral?" has now been wrong four times. Two
  payload values are equivalent if substituting one for the other can never
  change the kernel's verdict. Where the quotient is *narrower* than the type,
  **project** (`ArmResult` → `ArmJudgement`: `row_verdict` never reads
  `termination`/`stdout`/`stderr`). Where the quotient *is* the type, **move**
  (`Availability`: every distinction is consumed and reproduced in the verdict,
  so a projection would be an isomorphic copy). Generalisable beyond this slice.
- **The `leaf` gate proves tier *direction*, not mechanism *neutrality*.** `I7`
  claims the second and cites the first. They coincided only because the six
  types `I7` enumerates all live in `conformance`; a `backend`-resident mechanism
  type walks through, since `backend` is itself `leaf`. Rust offers no per-module
  import restriction inside one crate — `clippy.toml`'s `disallowed-types` is
  workspace-scoped. The repair is a `harness = false` cargo test target that
  `#[path]`-includes the kernel into a synthetic crate; compilation is the
  assertion. `harness = false` is load-bearing: it stops `cfg(test)` activating
  and dragging `DEC-200`'s kernel test band in.
- **The recorded verdict is emitted text, never a serialised value.**
  `AdmissionVerdict` derives only `Debug, Clone, PartialEq, Eq`; `report`
  (`main.rs:97`) writes to **stderr**; `EVD-022`'s body is that transcript. So
  `DEC-156`'s "recorded verdict" means the rendering — which is what lets `D8`
  keep front labels out of the verdict, and what makes § 9.1 layer 2 the *only*
  byte-level instrument there is. `just capsule-verify` does not redirect, so
  re-capturing the baseline needs `2>&1`.
- **…and the fifth location class is the executable closure** (`RV-354` `F-1`).
  A closure crosses the seam in **two independent ways** and the reduction test
  finds only one: *authority delegation* (the return carries the adjudication
  out — happens wherever, and only where, something reduces over the return) and
  *type contamination* (the return names a mechanism type — indifferent to
  reduction, and `I7`/the compile probe's business). Both were live on `run_row`,
  which is why one fix looked like it closed one class. The captured environment
  remains invisible to every instrument in this design.
- **`D13` closes bypass, never fabrication.** A payload can report `(Held,
  Failed)` for a row it never ran and the kernel computes `Proven` faithfully.
  Fabrication is irreducible at this seam because the payload is the only layer
  that can run a probe. What shrinks is the surface — from *any verdict for any
  reason* to *the two arm judgements reported*.
- **A golden proves `actual == expected` and nothing about where `expected` came
  from.** Generalised and banked as
  `mem.pattern.testing.classify-the-expectation-before-trusting-the-assertion`:
  classify every authored expectation as observation-backed (must be derived),
  reality-checked (safe because asserted against a reality that already exists —
  the pre-split characterisation test), or intent-backed (no ground truth, stays
  reviewer-checked — § 9.2's key translation table).
- **A vacuity guard can itself be vacuous, and the first cut of this one was.**
  Pinning `E0433` does not subsume a positive sentinel: the forbidden import is
  independent of the include, so a `#[path]` aimed at any other valid file yields
  the same diagnostic with the kernel uncompiled — and a positive `grep` asserts
  containment, not equality, so a missing path emitting `E0583` *and* `E0433`
  passes. A control needs a symbol only the subject defines, and must reject
  every unexpected diagnostic.
- **`D2`'s compile-time guarantee is unchecked, and that is `F-2`'s class.**
  "Adding a floor member fails to compile" holds only while no construction site
  uses `..` and the type derives no `Default`. Both are ordinary edits, neither
  warns, either voids a guarantee `DEC-195` *requires*.
- **`ISS-326` is a live trap for `DEC-197`'s exit criterion** — the layering gate
  exempts edgeless modules from classification. The kernel escapes it only
  because `backend`/`conformance`/`transaction`/`main` import it, and the gate
  flags any unit appearing as an edge *target*.

### Open

- **Stage is `reviewing` (rev 70). The third (external) pass is integrated,
  including a verification round that overturned half its repairs, and a
  self-audit of those repairs.** What remains to lock, in order: every section
  attested (**all ten are `outstanding`** — the integration invalidated the lot
  again), the review pass dispositioned (`conducted` naming `RV-354`), and the
  owner's `design-accepted`. Human section review is the v1 default; do not
  invent a reviewer posture to discharge it.
- **`RV-354` is `active`, `concluded`, `await=raiser`, and this does not block
  the lock.** The run's gate reads *disposal*, and all four findings are
  disposed. The `outstanding 1 blocker, 1 major` the run reports is `F-1` and
  `F-2` awaiting the **raiser's verification** of their *second* repair — the
  lamp counts findings whose status is not `verified`/`withdrawn`, which an
  `answered` finding is not. Resuming the codex thread would clear it.
- **Whether to run a fourth round is genuinely open.** The argument for: two of
  four repairs failed verification, so the base rate is bad, and two of the
  things landed since are unattacked — `D13`'s two-crossing-kinds generalisation
  and the second cut of `F-2`'s recipe. The argument against: three codex
  invocations on one design, and the owner has twice steered against
  gold-plating.
- **Known defect in the record, not fixable in place.** The `F-1` disposition
  says "the five surviving `RowVerdict` returns" and then lists six categories
  while omitting a seventh occurrence (`design.md:1933`, `D13`'s description of
  the superseded shape). The ledger is append-only so the miscount stands. The
  *design* is correct — the enumeration was re-run and every hit classified; only
  the disposition's prose is wrong. Worth knowing before quoting it.
- **What a fourth pass should be told.** The prior has now been right four times
  running, so assume a **sixth** location class. § 10.1 carries the four earlier
  ones plus the closure; the closure's *captured environment* is explicitly still
  open and no instrument here sees it. Also unattacked: `D13`'s claim that
  authority delegation happens *wherever and only where* something reduces over a
  closure's return, and § 5.1's placement criterion, which § 10.2 has flagged
  through three passes as new and self-certifying and which nothing has yet run
  backwards over the calls already made.
- **§ 5.1's placement criterion is new and self-certifying**, which § 10.2 flags
  as its own attack surface: the adjudicative normal form and the
  observational-equivalence test were written *in response to* five wrong
  judgement calls, and have not been run backwards over the calls already made.
- `OQ-2` is closed by `DEC-195`; `OQ-1` closed narrow by the owner.
- Carried into `/plan` (each stated in its decision, gathered here):
  the kernel-unit `leaf` classification as an exit criterion (`DEC-197`);
  `capsule-check` every phase and `capsule-verify` as a phase exit criterion for
  payload-touching and arguable phases (`DEC-199`); the committed key translation
  table in the `RowId` phase and the pre-split characterisation test (`DEC-199`);
  the pre-split test-band reorganisation with ~67 tests needing triage
  (`DEC-200`); the REV as a phase carrying four payloads (`DEC-201`).
- Added by the first review pass, also for `/plan`: the `today` → `observed_at`
  rename is **source-side only**, the rendered `date=` key is unchanged (`D9`);
  and `just capsule-check` needs `bwrap` too, so neither instrument runs on a
  host without it until `IMP-427` lands.
- Added by the second pass, for `/plan`: the **transformation contract must be
  written before the `RowId` phase**, because the whole-output golden test
  consumes it and that phase is the first that can break it (`DEC-199` re-cut);
  the golden test lands committed in that same phase beside the key translation
  table, whose key and front columns *are* the contract's rule 4; and the compile
  probe needs a **negative control** at implementation (`DEC-197`) or it can be
  present and prove nothing.
- Added by the third pass, for `/plan` — three new work items, all small:
  a **one-shot stdlib-only Python transform** in `scripts/` that derives the
  golden from `EVD-022`'s transcript, run *before* the `RowId` phase so the code
  goes green against it, owing four self-checks (exhaustive classification, exact
  category counts, single consumption/production, duplicate rejection);
  the compile probe's **negative control** now specified rather than deferred — a
  second `harness = false` target behind `required-features`, naming a
  kernel-only symbol, with a `capsule-check` leg that inverts the exit status and
  asserts the diagnostic set is *exactly* `E0433`; and `Floor` **derives no
  `Default` and is never built with `..`**, which is what makes `D2`'s stated
  guarantee true rather than assumed.

## Agent hostile pass over the drafted design (stage `reviewing`)

Conducted at run revision 43 against `design.md` at `499c2ebbc`, with every code
claim re-read in the working tree. Nine findings raised as `fnd-1` … `fnd-9` on
the design run and all nine dispositioned at revision 45; the integrated design
materialised at revision 46.

Do not restate them here — they are queryable. `doctrine design show SL-253`
carries each finding, the section it concerns, and its resolution; `design.md`
§ 10.1 carries what they changed and § 7.2 `D7`–`D10` the rulings taken. The one
thing worth repeating out of band, because it binds later work rather than this
document: **three enumerations of what crosses the seam backwards have each been
careful, believed complete, and wrong** — two of five, then three of five. Both
misses were in `verify_over`'s body rather than its signature, and the `leaf`
classification the design had leaned on would have caught neither.

## What a further review pass should probe (runbook step `review.passes`)

Written after the first pass integrated at revision 46. A second pass **is**
warranted; the argument for it is not that the first found nine things but that
of the nine, the four that changed the design's shape were all found by *checking
the code against the prose* rather than by reasoning about the design. The first
pass was run by the design's author, so the one thing it structurally could not
do is disagree with the design's own framing.

`design.md` § 10.2 carries the concrete list — `D1`, `D10`, `D7`, `D8` and the
`Qualification::Ran` shape. What belongs here rather than there, because it is
about the *pass* and not the artefact:

- **An external reviewer is the right instrument, not `/inquisition`.** The open
  questions are judgement calls on shape (is `FloorReading` a wrapper too many?
  does the value-only entry point really close `F7`'s class?), not conformance
  to doctrine. `codex` MCP on the default model is the project's adversarial
  reviewer; `RV-354` is the ledger it would fill, and is currently empty
  scaffolding reading `done` with 0 findings — it needs reopening or replacing
  before it can carry a pass.
- **Give the reviewer the code, not only the design.** Three of the nine findings
  were prose-versus-tree mismatches that no amount of reading `design.md` alone
  would surface. A pass confined to the document will systematically miss that
  class, which is the class this slice has the worst record on.
- **`D10` is the highest-value target and the least independently checked.** It
  was taken under review pressure, in one move, resolving three findings at once.
  That is exactly the shape of a ruling that over-reaches: ask whether the
  payload composing `Availability` hides the shell precondition rather than
  relocating it, and whether a caller can now build an `Availability::Available`
  that lies.
- **Do not re-litigate the numbers.** `P1` and § 3.4 answer *this is more than a
  few hundred lines*, and the owner has ruled the figures are not a criterion.

## Design surface triage (runbook step `explore.triage`)

### Constraining governance

| authority | what it binds here |
|---|---|
| `ADR-020` | the authority floor, unamended; step 5 admission is the control plane's alone |
| `DEC-190` | the split, and the `RV-352` behaviour-preservation baseline it must reproduce |
| `DEC-191` | stop collapsing; authority is a floor, assurance a per-front profile |
| `DEC-189` | row membership does not port; rows 10/12/13/14 lose their delta under a hypervisor |
| `DEC-194` | the qualification rename, which must land *with* the split |
| `DEC-195` | the verdict's shape and the floor's membership |
| `POL-002` facet 3 | `Unavailable { missing, remedy }` survives the rename as descriptive absence |
| `ADR-001` | `backend` is leaf, `conformance` is engine — a kernel depending on `BackendId` is a new edge |
| `STD-001` | the verb string and exit constants stay single-source through the rename |

### Shaping decisions taken

- **Floor membership is `{Property::DeniedCanonicalStateAndCredentials}`** — row 3
  alone. Row 8 was considered and rejected on inspection (`DEC-195` alternatives).
- **The floor is reduced by exhaustive match over a closed enum**, not `.all()`
  over a `Vec`, so `EVD-021`'s vacuous path becomes unrepresentable rather than
  externally guarded.
- **Fronts are open grouping metadata, never a reduction target.** Reducing per
  front would reproduce the vacuity one level up, since `DEC-189` guarantees
  empty fronts. The review pass placed them **wholly in the payload** (`D8`):
  the kernel neither reduces over them nor validates them.
- **Admission is named in the taxonomy and carries no outcome field**, on the
  `Unrowed`/`Reading` precedent — equal footing without a provable claim.
- **The seam cuts at row identity** (`DEC-196`). Kernel keeps identity and
  judgement; `Row`/`Delta`/`ArmShape`/`Under`/`Arm`/`PropertyRemoval`/
  `AuthorityGrant`/`ConformanceBackend` are construction and go to the payload.
- **The kernel classifies as a leaf** (`DEC-197`) — imports `std` plus two types
  from `backend`, and not `host` at all after the review pass moved the shell
  check and the descriptor read out (design `D10`). **Carry to `/plan` as two
  required exit criteria**: the architecture gate classifies the kernel unit
  `leaf` and passes, *and* the kernel's entry point takes values and closures
  only (`I10`) — the gate is a check on the import graph and would not have
  caught either reference the pass found.
- **Row identity splits closed floor from open profile** (`DEC-198`). Thirteen
  of fourteen `Property` members become payload-minted constants; `Axis` stays
  closed.
- **Row verdicts are the preservation bar** (`DEC-199`). `EVD-022` is the
  pre-split half of the bracket. **Carry to `/plan`**: `just capsule-check` green
  every phase; `just capsule-verify` a phase exit criterion for every
  payload-touching phase and by default for arguable ones; a committed key
  translation table in the phase that changes `RowId`; a pre-split
  characterisation test carried through the split.
- **Test bands are carved before the split** (`DEC-200`) — two bands, kernel and
  payload, as a pure reorganisation in the same pre-split phase. ~67 of 186 tests
  need individual triage; the phase plan carries that number. The live/neutral
  third band is deferred to `IMP-427`.
- **One REV, four payloads** (`DEC-201`) — `REQ-459` criterion 1 splits, criterion
  3's *text* narrows to "same floor, own profile" (an owner-approved scope
  widening), `REV-051`'s criterion-3 disposition is corrected to match, and
  `IMP-405`'s rename plus `CPT-002`'s threat priority land — the latter in
  `SPEC-030` § **Concerns**, beside "Security posture is structural".

### Risks

- Behaviour-preservation vs `DEC-191`: no record reconciles "suites green
  unchanged" with "the rendering deliberately changes". `inq-5`.
- Test carving is unbudgeted: 186 tests in one flat `mod tests`, no per-band
  sub-module, neutral and live-`bwrap` assertions interleaved. `inq-6`.
- The REV is wider than first scoped — it must correct `REV-051`'s applied
  criterion-3 disposition, not only `REQ-459`'s text. `inq-7`.

### Assumptions carried

- `ADR-020` is not reopened; `DEC-191` was built to land without it.
- The Firecracker row set is **derived, not implemented** (`OQ-1`, owner).
- `ADR-021` is `proposed`, so its two-site `unsafe` budget has directional
  weight only — but the kernel should add no `unsafe` regardless.

### Open questions found in passing

- **`DEC-189` may have missed a fifth row.** Row 8's control removes a file-size
  resource bound enforced via `RLIMIT_FSIZE` on the child (`ADR-021`'s first
  budgeted `unsafe` site). Under a hypervisor that limit is enforced by the
  *guest* kernel, so it is guest-internal and host-blind — the same reasoning
  that struck rows 13 and 14. Carried to `inq-8`; not resolved.

## Design-run state, and the disposition shape (resolved)

Run `dr-019ff3ff-b227-7030-a75b-65981efd87ca`, revision 7, stage `exploring`,
`resolved=2`, pin and cursor on `inq-3`.

`inq-1` and `inq-2` are dispositioned at revision 6 — `cp-1` and `cp-2`, both
`adopt DEC-195`. Revisions 4 and 5 remain no-op receipts from the two earlier
probes; nothing was corrupt and nothing needed unwinding.

**The shape, for the record.** A disposition is not a verb and not a run-level
payload key — it is a `declare` entry whose subject is a `cp-` checkpoint id:

    {"subject":"cp-1","disposes":"inq-1",
     "dispose":{"form":"adopt","record":"DEC-195"}}

`dispose` is the only spelling (EX-12); its four forms are `create` / `adopt` /
`unresolved` / `non-durable`. The authority is `ApplyRequest` in
`src/design_run/submission.rs`, which is where the rest of the vocabulary lives
too — `stage`, `acceptance`, `discharge`, `delegation`, `checkpoint_act`,
`agent_declaration`, `review_policy`, `adopt_authored`.

**Why it could not be found from the tool.** Neither `ApplyRequest` nor
`Declaration` denies unknown fields, so a guessed key is dropped and the
submission still succeeds — revision bumped, receipt written, no events, no
state change, and the same `revision N stage <stage>` line a real mutation
prints. Filed as `ISS-346` (with the serde `flatten` constraint on the obvious
fix noted); friction observation `019ff42b-2655-7aa3-b042-0a83272a328c`; durable
memory `mem_019ff439bede7fb29c7c09b7fd76d893`. **Read an apply's event rows —
an empty set is a refusal wearing a success.**
