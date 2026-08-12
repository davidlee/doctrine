# Notes SL-253: Conformance verdict kernel

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-12 · design/reviewing · eeabca359

### Produced

- `EVD-021` — the vacuous-admission finding.
- `EVD-022` — the pre-split nineteen-row baseline, captured in-jail at
  `4662e64eb`. The pre half of `DEC-199`'s bracket; uncapturable later.
- `DEC-195` — verdict shape: reduced floor, published profile, named admission axis.
- `DEC-196` — the seam cuts at row identity: kernel keeps identity and judgement,
  construction goes to the payload.
- `DEC-197` — the kernel classifies as a leaf; `BackendId` stays in `backend.rs`.
- `DEC-198` — row identity splits a closed floor from an open profile.
- `DEC-199` — row verdicts are the preservation bar; instruments and artefacts.
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

### Open

- **Stage is `reviewing` (rev 46).** The agent pass is raised, dispositioned and
  integrated; scope reconciled. Outstanding to lock: every section attested (all
  ten currently `outstanding`, and seven changed at rev 44 so a prior attestation
  would have been stale anyway), the review pass dispositioned — `conducted`
  naming an RV, or `waived` with a reason — and the owner's `design-accepted`.
  `RV-354` exists as empty scaffolding (`done`, 0 findings, still untracked) and
  is not yet the pass that would be named.
- `OQ-2` is closed by `DEC-195`; `OQ-1` closed narrow by the owner.
- Carried into `/plan` (each stated in its decision, gathered here):
  the kernel-unit `leaf` classification as an exit criterion (`DEC-197`);
  `capsule-check` every phase and `capsule-verify` as a phase exit criterion for
  payload-touching and arguable phases (`DEC-199`); the committed key translation
  table in the `RowId` phase and the pre-split characterisation test (`DEC-199`);
  the pre-split test-band reorganisation with ~67 tests needing triage
  (`DEC-200`); the REV as a phase carrying four payloads (`DEC-201`).
- Added by the review pass, also for `/plan`: the `today` → `observed_at` rename
  is **source-side only**, the rendered `date=` key is unchanged (`D9`); and
  `just capsule-check` needs `bwrap` too, so neither instrument runs on a host
  without it until `IMP-427` lands.

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
