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
fresh-as-of: 2026-08-06 · design drafting, sections 1–6 of 9 · 80f8e02b

### Produced

- Design run `dr-019fd432` — stage `drafting`, revision 44, materialised.
  Sections `sec-1` (governing context), `sec-2` (platform backend contract),
  `sec-3` (transaction and provisioning), `sec-4` (interpretation policy),
  `sec-5` (operational config: the `[capsule]` table, capsule root, capacity),
  `sec-6` (crate topology, export set, layering enforcement).
- `RV-346` — design facet, raiser codex / responder claude. Nine findings on
  `sec-2`/`sec-3`, seven blocking; **all upheld**, eight fixed in revision 35,
  the ninth (`F-2`) fixed in 37 via the `DEC-156` correction below. A second
  round covering `sec-5`, `sec-4`, and verification of all nine remediations
  is in flight.
- `DEC-156` **corrected** — the backend property count moves seven → eight,
  splitting process-tree teardown from trusted observation of resource limits
  and termination. Only the number moved; the record's arguments stand. Human
  authorised.
- `DEC-160` **corrected** — same count, swept consequentially. The record cited
  `DEC-156`'s arithmetic in two places and the first correction did not reach
  it; nothing in its own reasoning depends on the row count.
- `mem.fact.capsule.bwrap-ro-bind-dereferences-source` — `bwrap` resolves the
  *source* path of a `--ro-bind`, so a lexically validated allowlist does not
  confine.
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
- **`bwrap` dereferences a `--ro-bind` source path.** A declared root that is a
  symlink binds its target, so lexical path validation confines nothing. Resolve,
  validate the resolved path, bind the resolved path. Found by `RV-346` `F-1`
  executing a probe — reading the flags did not contradict the wrong claim.
  Recorded as a memory; the general form is that a confinement claim owes an
  executed probe, which the `SL-241` spike already learned twice
  (`F-P02-2`, `F-P05-17`).
- **A whole package store is not an explicit input set.** Binding `/nix/store`
  gives a capsule every package ever realised on the host. The design takes the
  *closure* instead: `[capsule] closure_roots` expanded through a declared
  `closure_resolver`, each returned path bound individually. Closure expansion
  also removes the link-escape class by construction, since a closure is
  transitively complete.
- **`trusted_side_forbidden_executables` has a consumer in this slice** — the
  closure resolver is the only external command provisioning runs outside a
  capsule, and its basename is checked against the bound policy *after* the
  phase refinement is applied.
- The design-run apply envelope is `#[serde(flatten)]`: unknown top-level keys
  are accepted silently and burn a revision as a no-op. Section bodies must begin
  with their own ATX heading — the title is derived, never declared
  (`src/design_run/section.rs`). Section document order is declaration order
  (`seq` claimed at declare time), not id order.
- **The root package has no lib target**, so nothing outside it can reach any of
  its 94 modules today. `crates/cordage` is the workspace-split precedent but
  points the other way (`doctrine → cordage`); `doctrine-control → doctrine` is
  new. Three consequences confirmed by execution on a minimal reproduction, not
  by reading: a package may declare the same module in both its bin and lib
  targets and builds cleanly; `pub use` of a `pub(crate)` item is `E0364`, so the
  visibility promotion is compiler-required; and the exported set must be closed
  over `#[cfg(test)]` imports too — `git.rs:2896` reaches `crate::kinds`, and
  omitting it fails the *library's own test build* with `E0432` while
  `cargo build` stays green.
- **`dtoml` cannot cross a crate boundary.** Seventeen `crate::` references
  (`conduct`, `verify`, `estimate`, `value`, `dispatch_config`,
  `install_config`) cascade into engine-tier `coverage`. Its two lowest-altitude
  items — `DOCTRINE_TOML` and `read_doctrine_toml_text`, used 35 times across 33
  files — move to an out-edge-free leaf module and are re-exported under their
  existing names, so no call site changes.
- **The layering gate has two distinct holes, not one.** It walks `src` alone
  (`tests/architecture_layering.rs:1148`, and `check`'s filter pins the same
  path at `:558`), *and* `CratePathCollector` (`:216`) only sees paths beginning
  `crate::`, so a `doctrine::…` import from a second crate produces no edge even
  if the tree were walked. Only the first is fixable by parameterisation.
- **Capacity has no precedent to ride.** Nothing in `src/` and nothing in the
  spike probes free space; `capsule_disk` (`lib/measure.sh:58`) is a post-hoc
  `du`. The only sizing evidence is two spike measurements — 256 MiB/300s for a
  Node fixture, a 4.4 GiB/352s peak for a Rust workspace re-bounded to 8 GiB/900s
  (`lib/fixtures.sh:70-92`) — at `R1` altitude, one host.
- **`[capsule]` is kebab-case.** The repo's operational tables are kebab
  (`dispatch_config.rs:51`, `reserve.rs:63`); `[interpretation]`'s snake_case is
  fixed verbatim by `SPEC-030` and is not a house style. `sec-2` and `sec-3` were
  amended from their first-draft snake spelling.
- **`src/clock.rs` is the single home for wall-clock reads**, under the
  pure/imperative rule that the value is passed in. `HostFacts` therefore carries
  no clock, correcting `sec-3`'s first signature comment; provisioning needs no
  timestamp.

### Open

- **Sections `sec-7`–`sec-9` are unwritten**: the eight-property conformance
  suite; code impact and verification alignment; risks and open questions. The
  runbook step `draft.selectors` discharges off the code-impact section, so it
  stays outstanding until `sec-8`. `sec-7` inherits more from `sec-2` than any
  other section and should not start before the second codex pass on the
  remediated `sec-2` returns.
- **Residual for `sec-9`: the resolution-time race.** Declared paths are resolved
  at provisioning, and a declared path could be re-pointed afterwards. Not
  capsule-reachable — no declared path is writable by a capsule — so it is a
  control-plane-side race on operator-owned config. Closing it needs the bind
  performed against an open file descriptor rather than a path. Recorded, not
  solved.
- **`RV-346` is `await=raiser`** with all nine findings answered. A second codex
  pass is expected over `sec-4` and the remediated `sec-2`/`sec-3`; the ledger is
  the rolling one for this design, not a one-shot.
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
- **Correction to land at reconcile: `DEC-136` handoff item 1.** It expects "a direct
  implementation seam in the existing `doctrine.toml` loader rather than a new
  configuration subsystem". Not available: `read_doctrine_toml_text` reads disk at
  `root`, `REQ-449` reads a blob at the contracted base OID, and the shared reader is
  deliberately tolerant where `REQ-449` must be strict. A separate typed projection
  (the `reserve.rs` pattern) is required either way. The decision itself — the
  `[interpretation]` block stays in `.doctrine/doctrine.toml` — stands; only that
  supporting note is wrong, so this is a record correction, not a Revision.
- **Follow-Up at close: the `doctrine-control` distribution contract.** The workspace
  member is cheap in this slice because nothing new ships (`DEC-153` § The cheap path).
  Whichever slice first ships it as a released binary owes the nix `srcWithDist` graft,
  the binstall asset name, and `install.sh` / `release.yml` together — see `R5`. Also
  an `RFC-025` § State-of-play note.
