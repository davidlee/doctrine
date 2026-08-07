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
fresh-as-of: 2026-08-08 · design reviewing rev 76, RV-346 round 6 remediated and returned · 5ffdea138

### Produced

- `RV-346` round 6 released to codex, returned, and remediated in full
  (`b8a779cca`). 27 verified, 2 withdrawn, 6 contested, 2 raised; all 8 open
  items dispositioned, ledger back at `await=raiser`.
- **The credential row is split** — table A rows 13 (`MappedIdentity`) and 14
  (`Granted(AllCapabilities)`), on `EVD-014`'s measured arms. Two of the four
  bundled mechanisms have no control under bubblewrap and are recorded
  observed-but-unrowed. `Delta` gains a `Granted` variant; `PropertyRemoval`
  loses `CredentialsConfined`.
- **The bubblewrap profile now passes `--uid`/`--gid`** — a behaviour change,
  not bookkeeping: row 13 asserts a declared identity, so one must be declared.
- Row 10's holding condition repaired (`F-36`, reproduced); row 12 now tests
  descriptor-1 *readability* rather than delivery (`F-30`).
- `verify` takes `today: String` and `clock::today` becomes a third root-package
  export — removes the `time` edge rather than declaring it (`F-29`).
- Design re-adopted at revision 76 via `adopt_authored` after hand-edits; seven
  of nine section fingerprints moved.
- Minted: `ISS-320` — no verb emits the `adopt_authored` section map;
  `ISS-280` — second instance recorded, sharpening the fix's shape (`5ffdea138`).
- No code touched; no gate applicable. All `.doctrine` changes committed
  path-limited — another agent's `SL-249`/`SL-250` files left untouched.

### Learned

- `EVD-014` — the measured arms now carry rows 13 and 14, not one bundled row.
- `mem.fact.bubblewrap.unshare-user-is-a-no-op-unprivileged`.
- Friction captured: `.doctrine/observations/records/d7/` — a contest's reasoning
  has no durable home.

### Open

- **`RV-346` round 7 is the next move, and it should be scope-bound to
  verification** — 8 findings answered and awaiting the raiser; see
  `handover.md`.
- The split's one live disagreement: whether the two unrowed credential
  mechanisms should be rows carrying permanent `Unproven`. Put to codex in
  `F-32`'s response; the design argues not.
- `sections_outstanding_review` is 9 of 9 and every fingerprint moved at rev 76
   — the largest remaining gate, and not startable until the ledger closes.
- `review_pass` reads `STALE`; the `Conducted` disposal arm is now reachable
  (`IMP-392`'s blocking leg landed), so `RV-347` no longer forces `Waived`.
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
