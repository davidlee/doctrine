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
fresh-as-of: 2026-08-07 · design reviewing rev 75, RV-346 round 6 briefed and held · ed8800414

### Produced

- Round 5 remediated and answered — `RV-346` `F-29`…`F-33`, 33/33 findings
  answered, `await=raiser` (`3271af47d`).
- **Ten drift defects found by a pre-brief sweep and fixed** (`48bc59a44`,
  `9ffe8a1d6`, `b504edc95`). Eight are `F-28`'s class; two — the `EVD-013`
  over-attribution and row 13's dead control — are older than round 5. Three came
  from cheap scout agents, one from executing a probe, none from re-reading.
- `RV-346` round 6 briefed, six lines of attack, **deliberately not released**
  (`69faf77ba`, `02427ef5c`, `ed8800414`).
- Credential spike run unjailed on the real host — `spike-credentials.{md,sh}`,
  `spike-credentials-output.txt` under `slice/248/`.
- `sec-9` `R1` withdrawn as a statement of fact and restated as an obligation;
  `sec-7`'s delta split corrected to one measured, nine reasoned. **The phase
  obligation to measure the deltas is now nine, not four.**
- Both row 13 entries marked ⚠ in place rather than re-cut — codex must see what
  was specified in order to rule on it.
- No code touched; no gate applicable. All `.doctrine` changes committed
  path-limited (`src/review.rs` and `observations/records/43/` belong to another
  agent's SL-250 work — left untouched).

### Learned

- `EVD-014` — bubblewrap credential confinement: `--unshare-user` is a no-op for
  non-setuid bwrap; `--cap-add ALL` fires; the signal lives in `CapBnd`/`CapInh`,
  never `CapPrm`/`CapEff`.
- `mem.fact.bubblewrap.unshare-user-is-a-no-op-unprivileged` — the same, as
  reusable agent guidance.
- `EVD-013`'s positive-control rule recurred twice in one spike session and
  caught both times; carried into `EVD-014`'s method.

### Open

- `QUE-210` — whether a design this size can hold parallel enumerations without a
  machine guard; put to round 6 as line 6.
- **`RV-346` round 6 must rule on row 13's mechanism** — the candidate is
  `Widened`-shaped where every other delta is `PropertyRemoval`-shaped.
- Review-pass disposal is **blocked**: the `Conducted` arm is unreachable until
  `IMP-392`. The design run still names the empty `RV-347`.
- `sections_outstanding_review` is 9 of 9 — the largest remaining gate, untouched.
- `QUE-208` — capsule-side entity id allocation; parked, does not block.
- `ISS-319` — separable defect, fixable independently of `QUE-208`.
- `SL-248` `OQ-1` (five-slice decomposition, provisional), `OQ-3` (three
  cross-cutting requirements), `OQ-4` (what replaces `review/*` and `phase/*`).
- `IMP-397` / `QUE-204` — egress allowlist and non-Git build inputs, adjacent to
  `REQ-459`'s network row, out of scope.
- `REQ-448`, `REQ-450`, `REQ-460` close in no single slice; this slice's share is
  `REQ-448`'s denial half and `REQ-450` criterion 1.
- Corrections owed at reconcile and follow-ups at close are carried in `design.md`
  `sec-9`, not restated here.
