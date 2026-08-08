# Notes SL-248: Capsule provisioning and Linux backend

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage
<!-- `explore.triage`, design run dr-019fd432. Recorded once at the exploring gate;
     superseded by design.md once the run materialises. -->

### Constraining governance (all read this session)

`ADR-020` (target architecture; items 1–2 of its decision are this slice)
· `SPEC-030` § Transaction authority / Contract and interpretation provenance /
Platform backend contract · `REQ-449`, `REQ-450` (criterion 1 only), `REQ-459`,
`REQ-461`, plus `REQ-448`'s denial half · `DEC-153` (two binaries, split at
canonical mutation) · `DEC-136` (one reader, resolve once from base) · `DEC-134`
(headless worker) · `DEC-133`/`DEC-137` (a harvested capsule is live work —
capacity handling may never delete one) · `ADR-001` (a new top-level module needs a
`layering.toml` classification) · `POL-001` (naming; see Learned) · `POL-002` (bwrap
and any free-space probe are declared host capabilities) · `STD-001` (every
`[interpretation]` key, the schema version, the capacity default — named constants).

### Shaping decisions already taken

- `DEC-153` fixes the topology: new transaction verbs go to a `doctrine-control`
  workspace member over a lib target on the root package. Nothing migrates out of
  the agent-facing binary.
- `DEC-136` fixes provenance: resolve once, from the contracted base, bound into the
  work contract; never re-resolved from a capsule checkout.
- Research fixes three implementation postures: the `[interpretation]` projection
  follows `reserve.rs`'s out-of-band pattern rather than joining `DoctrineToml`; the
  refusal machinery copies `JailPolicy`'s fail-closed shape; the Linux backend reuses
  the existing bwrap vocabulary rather than adding a second builder.

### Open questions carried into the inquiry

1. Does this slice create the `doctrine-control` workspace member and the root lib
   target, or land provisioning inside the existing binary and split later?
2. What exactly is shared with `dtoml` when the source is a git object rather than a
   file on disk — `read_doctrine_toml_text` is `root`-relative, so the shared part
   can only be the pure parse.
3. How the capsule confinement profile relates to `bwrap_core_argv`, which carries a
   byte-parity contract with `scripts/pi-spawn-confined.sh` (VT-7) and therefore
   cannot simply be widened.
4. The form of `REQ-459`'s property suite — what makes it a backend-admission gate
   rather than a Linux test, and how denial is asserted positively.
5. The provisioning mechanism and capsule storage layout, and what proves `REQ-450`
   criterion 1.
6. `REQ-461`'s free-space probe: mechanism, config placement, `POL-002` declaration.
7. How much of the work contract exists here, given `REQ-449` criterion 4 needs the
   phase-contract restriction algebra but the rest of the transaction is out of scope.
8. What `doctrine-control` exposes as a CLI in this slice, and how provisioning is
   verified end-to-end with no launch, harvest, or admission to follow it.

### Risks

Scope `R1` (evidence altitude), `R2` (incumbent suites green unchanged — this slice
touches `jail.rs`), `R3` (a weakly written property suite certifies nothing) stand as
authored. Added here:

- **R4 — the parity contract is a second behaviour-preservation obligation.**
  `bwrap_core_argv` is asserted byte-equivalent to the pi-spawn script
  (`jail.rs:1215`). Any change to it fails a test whose purpose is to notice.
- **R5 — `doctrine-control` inherits every embed root under the cheap path**
  (`DEC-153` § The cheap path). The nix `srcWithDist` graft and the binstall asset
  contract move together or the binary ships hollow with no compile error.

### Assumptions

Scope `A1`–`A3` stand. Added: **A4** — `git::read_path_at` reads a blob at an
explicit OID with no working tree, and is the whole impure surface `REQ-449`'s
resolution needs. To be verified at point of use, not assumed from the code map.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-08 · plan · stage 2 complete (all ten phases carry criteria) · 721f1fa77

### Produced

- Stage 2 complete — `PHASE-01` and `PHASE-03`…`PHASE-10` criteria expanded
  (`41f9abdc2`…`721f1fa77`, one commit per phase). Ten phases now carry 31 `EN`,
  160 `EX`, 55 `VT`, 35 `VA`, 1 `VH`.
- `PHASE-07` `EX-14` amended mid-stage — `Property`'s variants arrive with their
  rows, not all at `PHASE-07`; `PropertyRemoval` is the enum that must be
  complete early, for a different reason.
- Twelve `# NOTE for the integration pass` comments recorded in `plan.toml` —
  stage 3's concrete worklist, in the file stage 3 reads.
- `DEC-180` § *Where it lands* amended — the cost arrives at `PHASE-08`, not
  `PHASE-10`; the decision itself does not move.
- No code touched, so no `doctrine check gate` applies; `doctrine validate`
  clean and `verify-vt` reports **0 `UNCHECKABLE`**. All `.doctrine` changes
  committed path-limited — another agent's `SL-249`/`SL-250` and
  `.doctrine/rfc/027/` left untouched. `flake.lock` and `.claude/settings.json`
  were dirty on arrival and are left alone.

### Learned

- `mem.pattern.doctrine.assign-vt-by-code-owner-not-provenance-block` — a design's
  `Verification alignment` groups titles by section, not by phase; five titles
  moved phase this session.
- `mem.fact.bubblewrap.unshare-user-is-a-no-op-unprivileged`.
- `EVD-014` — the measured arms carry table A rows 13 and 14.
- `ISS-320` — no verb emits the `adopt_authored` section map; validate the
  recomputation against the unedited document before relying on it.
- `ISS-271` / `ISS-226` — at plan time `verify-vt` FAILs a `test_file` that is
  the phase's own output and mis-words `UNATTRIBUTABLE`; expected tooling
  behaviour. The signal that *is* trustworthy at plan time is the
  `UNCHECKABLE` count.

### Open

- **Next is stage 3, the integration pass** — `plan.md` § *Authoring stages*
  carries the checklist; the twelve `# NOTE for the integration pass` comments
  in `plan.toml` are the concrete worklist.
- `plan.md` § *File ownership* disagrees with the criteria on three files
  (`src/lib.rs`, `tests/architecture_layering.rs`, root `src/main.rs`) —
  stage-3 prose correction.
- `plan.md` § *What each phase changes about the tree* attributes the flag day
  to `PHASE-10`; the first executed capsule test is `PHASE-08` — stage-3 prose
  correction, `DEC-180` carries the durable version.
- `doctrine slice phases 248` deliberately **not** run — belongs after stage 3.
- `ISS-323` — design-text correction owed at `PHASE-01` and again at reconcile,
  because `sec-9`'s corrections list cannot be edited without a recovery cycle.
- `DEC-180` — settles the local-host case only; the CI ruling stays owed by
  whichever slice first runs this suite in CI.
- `QUE-208` — capsule-side entity id allocation; parked, does not block.
- `ISS-319` — separable defect, fixable independently of `QUE-208`.
- `SL-248` `OQ-1` (five-slice decomposition, provisional), `OQ-3` (three
  cross-cutting requirements), `OQ-4` (what replaces `review/*` and `phase/*`).
- `IMP-397` / `QUE-204` — egress allowlist and non-Git build inputs, out of
  scope, adjacent to `REQ-459`'s network row.
- `REQ-448`, `REQ-450`, `REQ-460` close in no single slice; this slice's share is
  `REQ-448`'s denial half and `REQ-450` criterion 1.
- Corrections owed at reconcile and follow-ups at close are carried in
  `design.md` `sec-9`, plus `ISS-323`.
