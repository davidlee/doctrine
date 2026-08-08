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
fresh-as-of: 2026-08-08 · plan · authoring complete (all three stages) **and the
`/plan` critical pass run**; **the design is authorised to reopen and that work
is owing** · 18121962d

### Produced

- Stage 2 complete — `PHASE-01` and `PHASE-03`…`PHASE-10` criteria expanded
  (`41f9abdc2`…`721f1fa77`, one commit per phase). Ten phases carry 31 `EN`,
  160 `EX`, 56 `VT`, 35 `VA`, 1 `VH`.
- `PHASE-07` `EX-14` amended mid-stage — `Property`'s variants arrive with their
  rows, not all at `PHASE-07`; `PropertyRemoval` is the enum that must be
  complete early, for a different reason.
- `DEC-180` § *Where it lands* amended — the cost arrives at `PHASE-08`, not
  `PHASE-10`; the decision itself does not move.
- **Stage 3 complete — the integration pass.** All four checks run over the
  whole plan (plan.md § *What the four checks found*), and every
  `# NOTE for the integration pass` discharged into either a criterion, a
  ruling, or plan.md prose; what survives in `plan.toml` under `# Stage-3
  record` is the cross-phase fact an executing agent needs.
  - Check 1 (`EN` discharge) clean; `PHASE-09` `EN-2` tightened to cite
    `PHASE-07`'s `EX` ids alongside its `VT` ids.
  - Check 2 (`VT` duplication) — the `PHASE-01` `VT-1` / `PHASE-02` `VT-10`
    export-set overlap ruled **permitted** (one test at two export-set sizes,
    and `PHASE-02`'s third keyword makes the mandates non-interchangeable).
  - Check 3 (`sec-8`'s evidence table) — one gap, closed. `transaction.rs`
    carried no mandate: its only title sat in `PHASE-06` `VT-3` under
    `test_file = provision.rs`. Split out as **`PHASE-06` `VT-6`** (the one new
    criterion id this stage minted). Every title in all seven of the design's
    `Verification alignment` sections was matched to a claiming mandate.
  - Check 4 (section claiming) — the provenance table's out-of-scope sentence
    widened from `sec-1` + `sec-9`'s risks/residuals to name all four groups of
    unclaimed narrative; `sec-7`'s executed alignment blocks named on the
    `PHASE-08`/`09`/`10` rows that own them.
- plan.md corrections landed: `src/lib.rs` and `tests/architecture_layering.rs`
  moved to the shared append-only table (`01, 02`); root `src/main.rs` added to
  `PHASE-01`'s exclusive row; the flag day re-attributed to `PHASE-08`.
- § *Two corrections owed at execution* → § *Corrections owed*, extended to five
  — items 3–5 are stage-3 rulings on design text, owed to the reconciliation
  brief only.
- No code touched, so no `doctrine check gate` applies; `doctrine validate`
  clean and `verify-vt` reports **0 `UNCHECKABLE`**. All `.doctrine` changes
  committed path-limited — another agent's `SL-249`/`SL-250` and
  `.doctrine/rfc/027/` left untouched.
- **The `/plan` critical pass (steps 7–9) is run** (`18121962d`; plan.md
  § *What the critical pass found*). Five findings, four landed as plan edits:
  `PHASE-01` `EX-4` amended (`today` must go `pub`; `ISS-323`'s root cause in a
  second instance), `PHASE-07` `EX-18` appended (sixteen `VT` mandates pin one
  `conformance.rs`; a split would strand them behind `ISS-271`-shaped noise),
  `PHASE-10` `VA-1` amended (falsifying measurement, no negative path),
  `PHASE-06` `EX-12` corrected to `renameat_with`. Selectors tuned:
  `+src/clock.rs`, `-tests/**`.
- Checked and found sound, no change: `sec-9` obligation coverage (every
  uncovered item a deliberate deferral), measured-versus-reasoned delta ordering
  (no delta depended on before the phase that measures it), phase sizing
  (`PHASE-02` largest, split point already pre-identified).

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

- **The design reopen is OWING, and the user has authorised it.** The critical
  pass found `C-1`: `fetch_refspec` is `pub(crate)` (`src/git.rs:2718`) and
  absent from the export set, while `design.md:1222` says the per-base export
  build rides it and `:2597` says nothing else becomes `pub` — so `PHASE-06`
  `EN-3` reads as met while `EX-11` cannot compile. It is a design-text defect
  and the design is locked (`ISS-320`), so it cannot be settled plan-side.
  **Six further design corrections are to be folded into the same reopen**
  rather than left owed to the reconciliation brief — `handover.md` § *The
  design worklist* carries all seven as `D1`…`D7` with line numbers.
- **`PHASE-06` must not start until `C-1` is settled.** `plan.toml` records this
  at `PHASE-06` critical-pass note 5. `PHASE-01` is also exposed: if `C-1`
  resolves by widening the export set, `EX-1`/`EX-2`/`EX-4`, `PHASE-02` `EX-8`
  and two `VT` mandates all move, so the decision wants making before any phase
  lands.
- After the reopen: clear the design gates, then an **informal `codex` sanity
  pass over the diff** — the user's call, explicitly *not* a new `RV` round.
- `/phase-plan` for `PHASE-01` is the step after that, not instead of it.
- `ISS-323` — design-text correction owed at `PHASE-01` and again at reconcile,
  because `sec-9`'s corrections list cannot be edited without a recovery cycle.
  Three more of the same shape joined it at stage 3, all in plan.md
  § *Corrections owed* items 3–5: two title divergences in the design's
  `Verification alignment` sections (one spelling, one name shared by two
  tests), and `sec-9` residual 2's stale `PropertyRemoval` count (nine stated,
  ten landed). All are corrections to design *text*; no decision moves.
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
