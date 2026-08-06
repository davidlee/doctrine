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
fresh-as-of: 2026-08-06 · design reviewing, RV-346 round 4 remediated · dbe96d22

### Produced

- Design run `dr-019fd432` — stage `drafting`, revision 56, materialised.
  Sections `sec-1` (governing context), `sec-2` (platform backend contract),
  `sec-3` (transaction and provisioning), `sec-4` (interpretation policy),
  `sec-5` (operational config: the `[capsule]` table, capsule root, capacity),
  `sec-6` (crate topology, export set, layering enforcement), `sec-7` (the
  conformance suite: the properties, controls, admission — the count lives in
  its Table A and nowhere else, which is `F-28`'s class fix).
- Sections `sec-8` (code impact and verification alignment) and `sec-9` (risks,
  residuals, open questions) — drafted at `d08468bc`, amended by round 3.
  Drafting is complete: 9 of 9.
- `RV-346` — design facet, raiser codex / responder claude. **28 findings, 27
  upheld, one rejected.** Round 1: nine on `sec-2`/`sec-3`. Round 2: five
  remediations verified, four contested, eight new (`F-10`…`F-17`). Round 3:
  seven on `sec-8`/`sec-9` (`F-18`…`F-24`), four blockers. Round 4: four on
  `sec-2`/`sec-6`/`sec-7` (`F-25`…`F-28`), three blockers. `await=raiser`, all 28
  answered; the rolling ledger for this design, not a one-shot. Its `## Brief`
  carries rounds 3 and 4's lines of attack (`de73c958`, `6e454608d`).
- **Round 4's remediation** — the largest of the four. `sec-2` gains the
  **channel ledger** (the closed enumeration of ways authority crosses
  `execute`, each channel naming its row), invariants 12–13, a typed
  `SourceExport` admission rule and the positive-control discipline; `sec-7`
  gains the **payload-strength rule**, table A rows 10 and 11, and restated
  payloads for rows 1, 5 and 7; `sec-6`, `sec-8`, `sec-9` swept. `DEC-156` and
  `DEC-160` corrected a third time. Run adopted at revision 68, materialised at
  69, watermark `dbe96d22`.
- `DEC-156` **corrected a third time** — nine → eleven. Not arithmetic this
  time: the rows are now derived from **authority channels**, not from
  `SPEC-030` clauses. A clause names what a capsule must not *have*; a row must
  name the mechanism by which it *gets* it, and clause 2 reaches four mechanisms
  while naming none. The tenth row was human-authorised; the eleventh is what
  the mandated class fix produced — the first blank cell the ledger showed.
- Round 3's remediation (`79d06645`) — amended `sec-2`, `sec-6`, `sec-7`,
  `sec-8`, `sec-9`; corrected `DEC-156` and `DEC-160`; added `Cargo.lock` and
  `justfile` to the design-target selectors.
- `DEC-156` **corrected a second time** — eight → nine, human-authorised. Its
  one-to-one-with-the-clauses sentence is **withdrawn**: nine rows over eight
  `SPEC-030` clauses, because clause 2 carries two claims. `DEC-160` swept
  consequentially again (three citations) and gained a caveat on its
  nested-bubblewrap claim.
- `EVD-013` — the teardown 2×2 (see Learned). Linked `concerns` → `SL-248`,
  `DEC-156`.
- `sec-3` amended — its freshness control named a provision its own refusal test
  forbids; corrected to the placement-level control `sec-7` runs.
- `sec-6` amended — `conformance`'s layering out-edges, and the no-lib-target
  ruling for `doctrine-control`.
- Friction observation `019fd675` — `design apply` silently accepts unknown
  top-level keys; schema probing cost three no-op revisions.
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
- **Capsule teardown needs the pid namespace *and* `--die-with-parent`** —
  `EVD-013`, the full 2×2. Neither is redundant, because bubblewrap runs its own
  init as pid 1 (payload reports `pid=2`), so the namespace never collapses when
  the command exits. The plausible inference — *a pid namespace reaps its
  members, so `--die-with-parent` is belt-and-braces* — is false, and believing
  it would have baked a control that cannot fire into the design. Also: there is
  **no `--share-pid`**; `--share-net` is bubblewrap's only re-share flag.
- **A control must remove the mechanism *unique* to its property.** Where two
  rows share a mechanism, removing it cannot establish which guard produced the
  result — `RV-346` `F-2`'s objection one level down. Corollary: a property with
  no unique mechanism cannot be controlled independently and the rows must be
  re-cut.
- **`tests/` cannot link a bin-only crate** (`E0433`), and adding a lib target
  does not rescue a `pub(crate)` item (`E0603`) — only `pub` compiles, which
  would have forced `sec-7`'s weakening vocabulary public. `#[cfg(test)]` units
  in a bin crate reach `pub(crate)` and do run under `cargo test`. All verified
  on minimal reproductions, per this slice's standing habit of executing build
  claims rather than reading them.
- **`Failed` must not be defined as "not `Held`".** In a probe/control suite a
  payload that breaks *on the control arm only* then reads as `Failed`, which is
  exactly what a row needs to report proven — a false green through the hole the
  token rule exists to close. Liveness must be established before any
  observation is read; absence is indeterminate, never negative.
- **The new crate is outside `just check`** (root-package only), so its tests
  would be green by never running. The phase landing `sec-7` owes the checked-set
  change.
- `git cat-file --batch-all-objects` is fatal without a batch mode
  (`--batch-check='%(objectname)'`).
- Loopback is denied under `--unshare-all` and reached under `--share-net`
  (measured), so a network-posture row holds offline and in CI.
- **A property suite's gap is invisible from inside it.** `R3` has now been
  realised twice, both times by the external pass: `F-2` (two clauses merged
  into one row) and `F-19` (one clause carrying two claims, only one with a
  row). Neither read as incomplete. The only reliable detector is an adversary
  *constructing the backend the suite would wrongly pass* — `F-19` executed
  `bwrap --ro-bind / / --bind HOST HOST` and changed the host marker at exit 0.
- **`default-members` is a package-selection default, not a build-command one.**
  It also selects for `cargo package` / `cargo publish`. And `publish = false`
  does not make a bare `cargo publish` *skip* a member — it fails the whole
  command — so keeping a workspace member unreleased needs the manifest key
  **and** a `-p` on the recipe. Both measured.
- **`cargo fmt` ignores `default-members`** and walks every workspace member.
  Only `lint`, `build` and `test` take the default set.
- **Cargo `include` accepts gitignore-style negation** (`!/src/lib.rs`), so
  excluding one file from a published tarball costs one line, not an
  enumeration — measured; `cargo package` completes with a warning.
- **A sandboxed reviewer measures its own cage.** `F-20`'s netlink failure was
  codex's `workspace-write` seccomp filter, not this jail. A confinement claim
  from an agent needs a positive control run outside that agent's sandbox
  before it is believed — the same discipline `F-1` established, pointed the
  other way.
- **A host-namespace pid is the right value to hand a capsule** in a
  process-isolation probe: under the probe arm the number names nothing in the
  observer's own namespace, under the shared-namespace control it resolves. A
  namespace-local pid would make the probe hold for an arithmetic reason.

### Open

- **The stage has moved to `reviewing`** (revision 64, 2026-08-06). All 9
  sections exist and are materialised. `sections_outstanding_review` is 9 — the
  run's own per-section attestation state, distinct from `RV-346` — and clearing
  it, disposing the review pass, and the user's `design-accepted` are the three
  remaining conditions on the lock.

- **Round 4 ran and its four findings are remediated** (`await=raiser`). The
  narrowing was right: three blockers, all in the sections named below, none in
  the four sections the brief told it to skip. What follows is the round-3
  statement kept for provenance — its lines 1, 2 and 4 are now closed by
  `F-25`…`F-28`, and line 3 is what actually delivered.

- **What a further review pass would probe** (`review.passes`, written after
  `RV-346` round 3). A round 4 is worth running, and it is not a re-read of what
  has already been read three times. Its targets, in priority order:

  1. **`sec-7` and `sec-2` as amended.** Both changed materially in round 3 —
     `ConformanceBackend::execute_observed` and its `HostPid` callback, table A
     row 9 with its `InputsWritable` control, invariant 11 — and neither has been
     read since. Round 3 was scoped to `sec-8`/`sec-9` and still found two
     blockers in `sec-7`, which is the empirical case for this being first.
  2. **`sec-6`, which has never had a clean external pass at all.** The crate
     topology, the five-item export set, and the two-tree layering gate are
     load-bearing for every other section's file placement, and `R6`/`R7` — the
     double compilation and the package/build divergence — were both created
     there.
  3. **The nine-row table against a tenth mutant.** `R3`'s standing form is that
     the gap in a property suite is invisible from inside it; `F-2` and `F-19`
     were both found by an adversary constructing the backend the suite would
     wrongly pass, not by re-reading the rows. The probe is *build the next such
     backend*, not *check the nine rows are well-written*.
  4. **Table C's split capacity claim**, added in round 3 and unread: whether the
     unconditional leg really discriminates, given `F-24` showed the previous
     arrangement did not.

  What a pass would **not** buy: another read of `sec-1`, `sec-3`, `sec-4` or
  `sec-5`, which have been through rounds 1 and 2 and were not amended by round
  3.
- **Five sections are now unreviewed as amended: `sec-2`, `sec-6`, `sec-7`,
  `sec-8`, `sec-9`.** Round 4 read `sec-2`, `sec-6` and `sec-7` — closing the
  gap that `sec-6` had never had an external pass — and the remediation then
  amended all five. `sections_outstanding_review` is 9 and
  `review_pass` is **STALE**: the round-4 pass concluded against fingerprints
  that no longer exist, which is the run correctly refusing to let a concluded
  pass vouch for text written after it.
- **The round-4 remediation added design surface no one has read**, and it is
  larger than round 3's: `sec-2`'s channel ledger and invariants 12–13, the
  typed `SourceExport` admission rule and its four tests, `sec-7`'s
  payload-strength rule with rows 1, 5 and 7 restated under it, table A rows 10
  and 11 with their removals and hazards, and `sec-9` residual 5.
- **`sec-9` carries five residuals**, as amended by round 4:
  1. *The resolution-time race.* Declared paths are resolved at provisioning and
     could be re-pointed afterwards. Not capsule-reachable — and that dismissal
     now rests on `sec-2` invariant 11 proven by row 9, **not** on invariant 4,
     which never covered it (`F-19`). Closing it needs the bind performed
     against an open file descriptor rather than a path. Recorded, not solved.
  2. *Out-of-crate backends.* `sec-7`'s sealing rests on `pub(crate)`, which
     holds only while every backend lives in `doctrine-control`. A backend
     shipping from outside makes the weakening vocabulary public API and it
     needs a newtype over a private enum. `ConformanceBackend` carries two
     methods and `PropertyRemoval` now seven variants — the surface grows every
     round, so the cost of deferring the seal rises rather than staying fixed.
  3. *Any host that cannot run the backend* — no longer macOS-only (`F-20`).
     The conformance test asserts `Admitted` unconditionally, so `cargo test`
     fails wherever the backend is unavailable, **including under any
     confinement layer that denies network-namespace setup** (an agent sandbox,
     a seccomp-filtered CI runner). What CI should do instead is explicitly
     open and owed by whichever slice first runs the suite there.
  4. *The `just check` cost* — the executed suite joins the fast inner loop;
     measurement is a phase obligation, and the lawful adjustment is a `--skip`
     on `test:` alone, never `#[ignore]`. Round 4 added two arms and two
     controls.
  5. *The channel ledger cannot prove its own completeness* — new in round 4,
     and created by round 4's own fix. It turns an unthought-of channel into a
     readable blank cell and does nothing more. It names three channels it lists
     but does not row: `argv` (closed by construction, not by probe), process
     reachability (split across row 7 and `B5`), and **process credentials**
     (uid/gid mapping, supplementary groups, capabilities, no-new-privs), which
     `SPEC-030` states no clause for. That last one is where the next reviewer
     should start.
- **`RV-346` is `await=raiser`** with all 28 findings answered. Round 5 has not
  been requested.
- **Three phase obligations were created by round 4's remediation**, each a
  place the design reasons where it previously measured, and each recorded in
  the design rather than left in this file:
  1. `EVD-013`'s teardown 2×2 was measured against row 7's *old* payload. The
     escaping payload should still be reaped by the pid namespace — `setsid`
     changes session, not namespace — but that is reasoning from a measurement
     made against different code, which `R1` forbids. If `--die-with-parent`
     turns out not to be required, row 7's control is wrong.
  2. Row 5's abstract-unix-socket leg is reasoned, not executed. If it is denied
     by a different mechanism than the TCP leg, it is a twelfth row rather than
     a second assertion in row 5.
  3. The `DescriptorsClosed` and `EnvCleared` bubblewrap deltas are the only two
     in that table not covered by `EVD-013`.
- `QUE-208` — capsule-side entity id allocation. **Parked deliberately** 2026-08-06;
  does not block this slice. Live for the ingestion slice, unavoidable by the
  recovery slice. Its own first settling condition — whether v0 permits capsules to
  mint entities at all — is upstream of every option in it.
- `ISS-319` — separable defect; fixable independently of `QUE-208`.
- The shape of the cutover point (flag day / dual-run / atomic release). Belongs
  to the cutover slice, not this one. Not a scope `OQ` on `SL-248` — an earlier
  note labelled it `OQ-3`, which is taken: the scope's `OQ-3` is the three
  cross-cutting requirements.
- `SL-248` `OQ-4` — what replaces `review/*` and `phase/*` refs; REV-046 § ADR-012
  leaves it a target-design question. Now carried in the scope (`review.scope`,
  2026-08-06), which previously did not state it while `design.md` `sec-9` cited it.
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
