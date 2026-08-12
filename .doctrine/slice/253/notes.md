# Notes SL-253: Conformance verdict kernel

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-12 · design/exploring · 718b97ee6

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
- Design run `dr-019ff3ff-b227-7030-a75b-65981efd87ca`, revision 18, stage
  `exploring`, **all eight inquiry nodes resolved**.
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

### Open

- **The inquiry axis is closed; the stage gate is not.** All eight nodes are
  `resolved`, but `exploring`'s two contracts are undischarged and its runbook
  still reads `obligation 1/5 explore.scope`. Needed before drafting: the agent
  performs `blocking-set-declared`; the user performs `graph-reviewed` naming it,
  and `governance-confirmed`. None of this is decision work — the decisions are
  made and banked.
- `OQ-2` is closed by `DEC-195`; `OQ-1` closed narrow by the owner.
- Carried into `/plan` (each stated in its decision, gathered here):
  the kernel-unit `leaf` classification as an exit criterion (`DEC-197`);
  `capsule-check` every phase and `capsule-verify` as a phase exit criterion for
  payload-touching and arguable phases (`DEC-199`); the committed key translation
  table in the `RowId` phase and the pre-split characterisation test (`DEC-199`);
  the pre-split test-band reorganisation with ~67 tests needing triage
  (`DEC-200`); the REV as a phase carrying four payloads (`DEC-201`).

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
  empty fronts.
- **Admission is named in the taxonomy and carries no outcome field**, on the
  `Unrowed`/`Reading` precedent — equal footing without a provable claim.
- **The seam cuts at row identity** (`DEC-196`). Kernel keeps identity and
  judgement; `Row`/`Delta`/`ArmShape`/`Under`/`Arm`/`PropertyRemoval`/
  `AuthorityGrant`/`ConformanceBackend` are construction and go to the payload.
- **The kernel classifies as a leaf** (`DEC-197`) — imports std, `backend` and
  `host` only. **Carry to `/plan` as a required exit criterion**: the
  architecture gate classifies the kernel unit `leaf` and passes.
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
