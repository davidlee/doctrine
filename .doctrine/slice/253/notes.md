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

- **Stage is `reviewing` (rev 42); all ten sections drafted and materialised.**
  Outstanding to lock: every section attested, the review pass dispositioned
  (`conducted` naming an RV, or `waived` with a reason), the owner's
  `design-accepted`, and runbook step `review.scope`. Eight `RF-` findings from
  the agent hostile pass are recorded above and **undispositioned**.
- `OQ-2` is closed by `DEC-195`; `OQ-1` closed narrow by the owner.
- Carried into `/plan` (each stated in its decision, gathered here):
  the kernel-unit `leaf` classification as an exit criterion (`DEC-197`);
  `capsule-check` every phase and `capsule-verify` as a phase exit criterion for
  payload-touching and arguable phases (`DEC-199`); the committed key translation
  table in the `RowId` phase and the pre-split characterisation test (`DEC-199`);
  the pre-split test-band reorganisation with ~67 tests needing triage
  (`DEC-200`); the REV as a phase carrying four payloads (`DEC-201`).

## Agent hostile pass over the drafted design (stage `reviewing`, rev 42)

Conducted against `design.md` at `499c2ebbc`, with every code claim re-read in
the working tree rather than recalled. Findings are `RF-` numbered doc-locally
and are **undispositioned** until the owner rules.

### Contradictions inside the design

- **`RF-1` — `FloorStanding::NotEstablished` is unreachable as typed.** § 5.2.2
  makes `Floor` total (one field per member) and `standing()` an exhaustive match
  over `RowVerdict` returning only `Held` / `Breached`; `NotEstablished` appears
  in no arm. But § 5.4's flow routes `Floor::from_rows` → `Err(missing)` →
  `Ran { floor incomplete }` → `NotEstablished`, and § 9.6 pins it with
  `a_missing_floor_row_is_not_established_not_breached`. `Qualification::Ran`
  holds a `Floor`, and an incomplete `Floor` is exactly what § 5.2.2 makes
  unconstructible. `D3` names the consequence in prose and § 5.2.3 does not carry
  it into the type. Needs a shape ruling: `Ran { floor: Result<Floor, FloorProperty> }`,
  or `Ran { standing: FloorStanding, … }`, or a third `Qualification` variant.
- **`RF-2` — `D5` produces a difference § 9.1 does not license.**
  `main.rs:render_verdict` emits `date={}` on the header line. Renaming the field
  to `observed_at` changes rendered output, and § 9.1's permitted list is closed
  at three (outcome line, exit constants, row key spellings). By the design's own
  rule that is a regression. Either `D5` scopes to the Rust identifier and the
  rendered key stays `date=`, or the licence gains a fourth entry — which § 9.1
  says takes a decision record.

### Claims that did not survive checking

- **`RF-3` — the `SHELL` precondition is a fourth backward reference.**
  `verify_over:5299` checks `host.path_exists(Path::new(SHELL))` under the comment
  *"Every payload runs under `/bin/sh -c`"* — an explicitly payload-shaped fact,
  with `SHELL`/`SHELL_REMEDY` defined in `conformance.rs:100,106`. § 5.4 keeps
  that step on the kernel's path; § 5.1's table enumerates exactly three
  crossings and this is not among them. **`R1`'s mitigation does not catch it**:
  a kernel that re-declares `const SHELL: &str = "/bin/sh"` imports nothing, so
  the `leaf` gate stays green. This is precisely the test § 10.1 invited — *what
  would have caught the fourth?* — and the answer the design gives fails on it.
  The seam leaks by duplicated constant, not by import.
- **`RF-4` — § 9.4's "`capsule-check` needs `bwrap`? no" is false.**
  `cargo test -p doctrine-control` runs `#[cfg(test)]` tests that assert
  `backend.availability() == Availability::Available` and provision real capsules
  (`conformance.rs:6915`, `:7244`, `:7453`, …). `EX-14` forbids skipping, so on a
  host without `bwrap` they fail. This contradicts § 9.5 / `IMP-427`'s own
  rationale and understates `R2`'s residual, which covers only `capsule-verify`.

### Gaps

- **`RF-5` — fronts have no home in any type.** § 5.4 and scope objective 2 make
  front-labelled rendering a `CPT-002` obligation, but no kernel type, no payload
  contract function (§ 5.2.4) and no verdict field carries a row→front map.
  A required rendering behaviour with nothing to compute it from.
- **`RF-6` — re-keying makes an unknown id representable, with nowhere to report
  it.** `run_row: &dyn Fn(&RowId) -> RowVerdict` is total, so a payload handed an
  id it cannot construct must fabricate a verdict. Today's `&[Row]` shape makes
  that state impossible. Related: `row_for(id) -> Option<&'static Row>` implies a
  static table; `tables():4289` builds an owned `Vec<Row>` whose `Delta`s carry fn
  pointers.

### Constructive

- **`RF-7` — the kernel may not need the `CapsuleBackend` trait at all.**
  `qualify_over` calls exactly `id()` and `availability()`. Passing `BackendId`
  and `Availability` as *values* drops a trait from the kernel's vocabulary
  entirely — strictly more `P1`/`P2` than narrowing `ConformanceBackend` to its
  supertrait, and it removes a dyn dispatch a reader must otherwise resolve.

### Provenance nits (§ 10.3 invited these)

- **`RF-8`** — § 5.2.1 cites `backend.rs:805` for `BackendId`'s shape; the struct
  is at `:811` (§ 2.6 has it right). § 2.2 cites `:2688` for `AdmissionVerdict`;
  the struct is at `:2689`.

### Checked and found sound — stated so the pass is not read as only negative

- `Claim` and `Unrowed` are `&'static str` `section`/`name` pairs
  (`:2726`, `:2770`), not closed mechanism enums, so § 5.2.3's placement of them
  in the kernel is safe.
- Leaf→leaf edges already exist in the control tree (`backend → config`,
  `capacity → config, host`, `layering.toml:259-261`), so the kernel's `leaf`
  classification introduces no new edge class — `DEC-197` holds.
- Every `justfile` and `layering.toml` cite in § 5.4's rename radius is exact
  (`:108`, `:114`, `:129`, `:143`, `:145`; `layering.toml:257`), as are
  `row_verdict:3177`, `admission:5166`, `verify_over:5270`, `RowId:2580`,
  `Delta::Widened:2548`, `ConformanceBackend:638`.
- § 10.1's own doubt — whether `axes` belongs beside `assurance` — resolves in
  the design's favour: `B1`–`B5` are transaction properties and are **not escape
  fronts**, so folding them into `assurance` would misrender under `CPT-002`.
- `D2` holds under the challenge § 10.1 set for it: a `BTreeMap` gives neither
  `DEC-195` guarantee, and `const ALL: [_; N]` gives only the compile-break, and
  only if someone remembers to extend it.

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
