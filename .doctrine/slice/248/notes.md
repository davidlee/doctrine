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
fresh-as-of: 2026-08-06 · pre-design (research round complete) · 17860ef3

### Produced

- `DEC-153` (accepted) — answers `QUE-207`; two binaries split at canonical mutation.
- `IMP-404` — `SL-112`'s deferred engine/leaf crate extraction, captured.
- `ISS-319` — fresh-id allocation fails open when the trunk ref is unreachable.
- `QUE-208` — capsule-side entity id allocation; parked (see Open).
- `QUE-207` → answered.
- RFC-025 § State of play next-action 2 — rewritten to carry the five-slice
  decomposition, the unminted-until-predecessor rule, and the cross-cutting
  requirement warning.
- Research artefact — `research/research.md` + three `research/raw/*.md`
  (runtime tier, gitignored).

### Learned

- `POL-001` bans "load-bearing" plus tired physical metaphors — *plumbing, wiring,
  scaffolding, lever, seam, substrate, cut, sharp, spine* — across all English,
  including code comments and entity names. Three occurrences were corrected in
  authored records at `b33f2ce6`. It constrains this slice's module, error, and
  comment naming more than most.
- `dtoml.rs` is leaf-tier (`.doctrine/adr/001/layering.toml:32`) and deliberately
  tolerant. `reserve.rs:78-97` is the precedent for a validated typed projection
  over the shared file reader without an upward layering edge.
- `jail.rs:302-393` already implements REQ-449's refusal shape for jail policy.
- `jail.rs:806` — bwrap network is default-open; the capsule posture must invert it.
- REQ-461 is net-new: no free-space probe in `src/`.
- Governance sweep found **no revision candidates** for this slice.

### Open

- `QUE-208` — capsule-side entity id allocation. **Parked deliberately** 2026-08-06;
  does not block this slice. Live for the ingestion slice, unavoidable by the
  recovery slice. Its own first settling condition — whether v0 permits capsules to
  mint entities at all — is upstream of every option in it.
- `ISS-319` — separable defect; fixable independently of `QUE-208`.
- `SL-248` `OQ-3` — the shape of the cutover point (flag day / dual-run / atomic
  release). Belongs to the cutover slice, not this one.
- `SL-248` `OQ-4` — what replaces `review/*` and `phase/*` refs; REV-046 § ADR-012
  leaves it a target-design question.
- REQ-448, REQ-450, REQ-460 close in no single slice. This slice's share: REQ-448's
  denial half (proven by REQ-459's suite) and REQ-450 criterion 1.
- `IMP-397` / `QUE-204` — egress allowlist and non-Git build inputs, out of scope
  but adjacent to REQ-459's network-posture control.
