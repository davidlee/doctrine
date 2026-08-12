# Notes SL-253: Conformance verdict kernel

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-12 · design/exploring · 9b5a0fae8

### Produced

- `EVD-021` — the vacuous-admission finding.
- `DEC-195` — verdict shape: reduced floor, published profile, named admission axis.
- Design run `dr-019ff3ff-b227-7030-a75b-65981efd87ca`, revision 5, stage
  `exploring`, eight inquiry nodes, `changes 0 since the declared baseline`.
- Research artefact at `.doctrine/slice/253/research/` (runtime tier, gitignored;
  `raw/governance.md`, `raw/codemap.md`, `research.md`).
- Friction observation `019ff42b-2655-7aa3-b042-0a83272a328c` — `design apply`
  silently absorbs unknown payload keys.

### Learned

- The change surface is far smaller than 14,252 lines implies: one reducer, one
  production call site, two production consumers.
- Row titles mislead. `TrustedTerminationObservation` reads epistemic and is a
  file-size resource bound. Read the `Row`'s `delta`, not its name.

### Open

- `inq-1`..`inq-8` in the design run. `inq-1`/`inq-2` are decided by `DEC-195`
  but **not yet dispositioned in the run** — see § Design-run state below.
- `OQ-2` is closed by `DEC-195`; `OQ-1` closed narrow by the owner.

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

## Design-run state, and one thing that is stuck

Run `dr-019ff3ff-b227-7030-a75b-65981efd87ca`, revision 5, stage `exploring`,
`resolved=0`.

`inq-1` and `inq-2` are **decided** (`DEC-195`) but **not dispositioned in the
run**, and this is not an oversight. The `design show` envelope advertises only
the `declare` and `traversal` payload keys. Two attempts to disposition a node
(`{"resolve":[{"subject":"inq-1"}]}`, then the same with an unknown field) were
**silently accepted** — each bumped the revision and wrote a receipt, emitted no
events, and changed no node state. An unknown top-level key and an unknown inner
field were both absorbed without complaint.

Per `/design`'s degradation rule — *detect and surface, do not self-heal, do not
improvise a workflow of your own* — guessing was stopped rather than continued.
`design-prompts/exploring.toml` and `design-prompts/inquiry.md` were read from
the library; they carry the stage's obligations and craft but not the mutation
schema. Friction recorded as an observation (`019ff42b-2655-7aa3-b042-0a83272a328c`).

**The next agent should find the disposition payload shape before applying
anything else** — likely candidates are a different verb, a stage advance from
`exploring` to `inquiring`, or a payload key the envelope does not advertise.
Revisions 4 and 5 are no-op receipts from the two probes; nothing is corrupt.
