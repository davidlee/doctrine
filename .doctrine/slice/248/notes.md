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
fresh-as-of: 2026-08-08 · design LOCKED at rev 85 · next is /plan

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
- **`RV-346` is closed** — `done · await=none · concluded`, 38 findings over
  nine rounds (`ba07f8674`). Round 7 verified 7 of 8 and raised nothing under a
  scope-bound brief; rounds 8 and 9 closed the remainder.
- Round 7's credential ruling went **with** the design: supplementary groups and
  `no_new_privs` stay observed-but-unrowed. A permanent `Unproven` buys no
  discrimination where no control fires, and would make `Admitted` unreachable.
- `F-30` upheld (`3e0402b9b`) — the round-6 row 12 rewrite left `sec-7`'s test
  title and gloss projecting the *repudiated* property (non-delivery, and the
  bidirectionality reasoning). Renamed to name readability; row 12's containment
  hazard, a second instance codex did not spot, fixed with it.
- **Section review discharged in the adversarial lane.** Policy moved
  `human-only` → `adversarial-only` at rev 81 on the user's direction, the basis
  being truthfulness: this design was reviewed adversarially and not by a human,
  so nine human attestations would have claimed a reading nobody performed.
  `DEC-074` grants human-proxy standing. Codex then attested all nine; every
  section reads `review=current`, `sections_outstanding_review=0` (rev 82).
- **`review_pass` disposed `Waived`** at rev 83, the reason naming `RV-346` as
  where the review actually happened. `Conducted` is structurally unavailable —
  see `ISS-322`.
- **The design run is LOCKED at revision 85.** `design-accepted` recorded on the
  user's explicit acceptance; every act on `reviewing-locked` reads `current`.
  The engine stamps the lock honestly — *"locked on an auditable agent claim of
  user acceptance — not authenticated proof of a human act"* — because the agent
  recorded the act the user performed in conversation.
- `F-38` raised and remediated (`089b082e4`) — row 12's two legs shared one
  mutant that trips both, so the readability leg was not separately gating.
  `sec-7` gains `a_readable_descriptor_one_backend_fails_row_twelve` and states
  the rule; `sec-8`'s test-cost narrative gains round 8. Table A stays fourteen.

### Learned

- `EVD-014` — the measured arms now carry rows 13 and 14, not one bundled row.
- `mem.fact.bubblewrap.unshare-user-is-a-no-op-unprivileged`.
- Friction captured: `.doctrine/observations/records/d7/` — a contest's reasoning
  has no durable home.
- **A scope-bound review brief works, and the binding belongs on the ledger.**
  Round 7 was told the bar was blocker-or-regression and that a nothing-raised
  round was the successful outcome. It raised nothing out of scope and said so.
  Written into the brief rather than the driver's prompt, so it survived handover
  and codex read it directly.
- **Don't self-rule on a scope bar you authored.** The row 12 mutant-parity gap
  was found while remediating `F-30`; it was put to codex rather than fixed
  unilaterally, and codex ruled it a *blocker* (`F-38`). A self-issued finding on
  one's own remediation is worth little; the external ruling is the value.
- **`ISS-322`** — a design run mints its own pass `RV` on entry to `reviewing`
  and replaces it on re-entry; nothing binds an existing `RV` to the run. So an
  externally conducted review can never be named by `Conducted`, while the
  obligation itself offers *"a printed prompt for an external adversarial
  reviewer"* as a route. **Correction to the prior handover chain:** the
  `Conducted` arm being *reachable* (`IMP-392`) does not make it *usable* for an
  external `RV` — two different questions, only the first had been checked.
- `ISS-320` updated (`60a4f90ef`): the `adopt_authored` envelope is snake_case
  while sibling enums are kebab-case, and the refusal names one field rather than
  the convention. Also — the recomputation's cross-check is **not** luck: validate
  it against the *unedited* document (all nine digests must reproduce) before
  relying on it. Friction recorded at `observations/records/36/`.

### Open

- **Next is `/plan`** — `doctrine slice plan`. The design is locked; the slice
  is still at lifecycle status `design` and moves to `plan` next. Nothing in the
  design run remains open.
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
