# Capsule provisioning and Linux backend

## Context

The first implementation slice of RFC-025's capsule programme. It builds the
trusted side of *starting* a capsule transaction: resolving its contract,
provisioning fresh confined state, and proving the backend's properties.

The target contract is settled and this slice does not relitigate it —
**ADR-020** (accepted) makes the execution capsule the dispatch authority
boundary; **SPEC-030** (active, under SPEC-003, from PRD-015) specifies the
container across REQ-448–REQ-461; **SL-241**'s spike supplies the Linux/bwrap
evidence; **DEC-134** and **DEC-136** settle persistent control-plane
orchestration and the interpretation-policy home. **REV-046** is the governance
cutover Revision and stays `proposed` — it is not this slice's to apply.

### Why this slice exists at this size

SL-248 originally scoped all fourteen SPEC-030 requirements as one cutover.
Measured against this project's own history that was a repeat of two known
failure modes at once: SL-233 ran 16 phases on 781 design lines (~49 per phase)
and left holes that CHR-049 and SL-244 had to patch; SL-244 ran 8 phases on
3459 design lines (~430 per phase) and was uncomfortably inefficient. The
comfortable band here is ~1000 design lines over 6–9 phases. Fourteen
requirements extrapolated past both ends of it.

The programme is therefore decomposed into roughly five slices (provisionally;
see OQ-1). **Shipping is not cutting over** — every slice but the last lands as
tested machinery sitting beside the incumbent worktree arms, unused, with no
flag day. Only the final slice flips the switch. This is that first slice.

### Why REQ-449 is here and not a slice of its own

The decomposition first proposed the `[interpretation]` policy surface as a
standalone precursor, on the theory that it is purely additive config work.
Reading REQ-449's acceptance criteria kills that: its first criterion is
*"**Capsule provisioning** refuses a missing block, missing required key,
unknown key, unsupported schema version, invalid normalized value, or empty
verification sequence"*, and its fourth is the phase-contract
monotonic-restriction algebra — both of which are provisioning-time behaviour,
not parser behaviour. SPEC-030 § Transaction authority says the same thing
structurally: the control plane creates a transaction *from* base + resolved
policy + work contract + capsule identity + resource choices.

Only REQ-449's second criterion (typed parse plus canonical hash) is
independently landable. Splitting there would either ship a parser with no
consumer and retrofit the refusals later, or put the schema design in the
provisioning slice and its implementation in the config slice — backwards. So
contract resolution and provisioning land together.

## Scope & Objectives

**REQ-449 — contract and interpretation provenance.** The required v1
`[interpretation]` block in `.doctrine/doctrine.toml`: typed parse; validation
of `trusted_side_forbidden_executables` (normalized basenames, no slash /
whitespace / empty / `.` / `..`), `interpreted_paths` (normalized
repo-relative gitignore-style patterns; absolute, backslash, NUL, and lexical
`..` refused), and `[[interpretation.verification]]` rows (non-empty `argv` of
non-empty UTF-8); duplicate rejection, then byte-sorted set-valued lists with
verification-row and argument order preserved; one canonical hash over the
typed value. Resolution **once** from the contracted base, bound into the work
contract, never re-resolved from a capsule checkout. Phase-contract refinement
that may add forbidden entries and append verification rows but may not remove,
reorder, widen, or replace — subset validation over normalized typed values,
never source text. Missing block, missing key, unknown key, unknown schema
version, or empty verification sequence refuses provisioning. Keeps the block in
`.doctrine/doctrine.toml` (DEC-136) and does not fork a second config document;
the *reader* is a typed projection beside the shared one, because
`read_doctrine_toml_text` reads disk at `root` while this resolves a blob at the
contracted base OID, and the shared reader is deliberately tolerant where this
must be strict. DEC-136's handoff note expecting a direct extension of the
existing loader is corrected accordingly.

**REQ-450 (partial) — fresh mutable state.** Provisioning from the exact
accepted commit and only explicit immutable inputs, with fresh mutable phase
state. This slice discharges criterion 1 — two phase transactions share no
mutable checkout, repository, runtime, process, or temporary state — and builds
the mechanism criteria 2 and 3 later assert against. See OQ-3. The immutable
input is a **per-base bare export** (DEC-157), built trusted-side once per base,
shared read-only, and cloned *inside* the capsule with `--no-hardlinks`; the
canonical repository is outside the mount set under every arm, so REQ-448's
denial of shared object storage is structural rather than resting on a read-only
mount. The transaction binds only what has a consumer now (DEC-159): the
accepted base OID, the canonical policy hash, the refined policy or none, the
capsule identity, the resource choices, and phase identity as a durable
reference — never `plan.toml` or the phase sheets, which is where RFC-027's
churn would land.

**REQ-459 — platform backend contract.** The shared property-conformance suite,
**nine** properties over SPEC-030's eight clauses: fresh mutable state; a bound
input set no wider than what was declared; those declared inputs bound
*immutably*; no writable canonical repo / shared object store / control-plane
state / credentials; bounded host filesystem visibility; explicit network
posture; deterministic working directory; process-tree teardown; and trusted
observation of resource limits and termination. The ninth row is DEC-156's
second correction: boundedness and immutability are two claims of one clause and
they fail apart, so a backend binding exactly the declared inputs *writable*
passed every earlier row while handing capsules mutable shared host state. Each
property carries a one-property-removed control (DEC-156). Plus the
Linux/bubblewrap backend implemented against it, taking SL-241's rig profile as
its starting point. It is built self-contained and does not extend
`src/worktree/jail.rs` (DEC-155): the two profiles differ on every structural
axis, and `bwrap_core_argv` carries a byte-parity contract with
`scripts/pi-spawn-confined.sh` that forbids widening it in place. The suite is
the admission gate for any future backend.

**REQ-461 — advisory capacity.** Configurable expected capsule size, in a
`[capsule]` table of its own (DEC-158 — not `[interpretation]`, whose hash is
bound into the work contract, and not `[dispatch]`). Two tiers: below twice the
expectation, a conspicuous structured warning and provisioning proceeds; below
one expectation, provisioning refuses. Nothing is reserved. A probe that cannot
produce a usable answer yields a named *capacity-unknown* outcome rather than
silently reading as ample. Exhaustion halts for manual intervention and never
deletes a capsule or result; rolling back a directory the failing call itself
created, before any transaction exists in it, is not eviction and is permitted.

**Also in scope:** answering `QUE-207` as a DEC (see OQ-2) — provisioning is the
first trusted-side code written, so the control-plane topology question gets
decided on concrete ground here rather than in the abstract.

**The CLI surface (DEC-160).** Two verbs on `doctrine-control`, built and
reachable from the build tree only, never released here: `provision`, which
consumes contract resolution, the refinement algebra, capacity, layout and the
backend; and `backend verify`, which drives the conformance suite for an on-host
admission verdict, exiting nonzero with structured output when not admitted. The
suite lives in the product behind that verb with the integration test as a
second caller — a `tests/`-only suite would report a green skip on a host
without the backend, which is exactly what DEC-156 forbids.

## Non-Goals

Everything downstream of a provisioned capsule belongs to later slices and is
explicitly **not** here: result publication, snapshot, and quarantine ingestion
(REQ-451, REQ-452); trusted conformance over the pinned result (REQ-453);
verification-capsule construction and normalization (REQ-454); the admission
journal and CAS (REQ-455); the capsule-provenance candidate adapter (REQ-456);
freeze, repair, and cleanup discipline (REQ-457); the journal/exhibit retention
lifecycle (REQ-458); and the named cutover point with its skill and CLI collapse.

Inherited from SPEC-030 and REV-046, and out of the whole programme:

- **macOS / Seatbelt backend** — unselected until independently specified and
  measured against the REQ-459 suite. No cross-platform parity claim.
- **Egress allowlisting and non-Git build-input provisioning** — `IMP-397` and
  `QUE-204` own it.
- **Capacity reservation, backpressure, eviction, rescue archive** (D7).
- **Retention durations and quota hierarchy** beyond DEC-133/DEC-137.
- **Migrating solo `/execute` worktrees to capsules** — SPEC-012 keeps that
  mechanism and it survives the cutover.
- **Production optimisations** — overlays, snapshots, reflinks, shared caches,
  remote execution.
- **Retiring any incumbent dispatch mechanism.** Marker identity,
  `DOCTRINE_WORKER`, the SubagentStart stamp, `worker_commit`, worktree import,
  and coordination-worktree placement all keep working. This slice is additive.
- **Applying REV-046**, or rewriting RFC-025 beyond its § State of play entry.
- **Migrating the SL-241 rig** (`scripts/spike-capsule/`) into product — its
  hostile rows and stage assertions carry across as *behaviour*, as production
  acceptance tests.
- **The rest of the work contract** (DEC-159) — phase criteria, worker
  instructions, and everything launch and harvest require have no consumer here.
  Only REQ-449 criterion 4's restriction algebra and the bound set above are
  built.
- **Releasing `doctrine-control`** — the nix `srcWithDist` graft, the binstall
  asset name, `install.sh` and `release.yml` move together for whichever slice
  first ships the binary (R5). What this slice *does* change of the release
  arrangement is the opposite: keeping the new crate out of `cargo publish` and
  keeping `src/lib.rs` out of the published `doctrine` package (R7).

## Affected surface

`QUE-207` is answered by `DEC-153`: the trusted side is a `doctrine-control`
workspace member over a lib target on the root package, and nothing migrates out
of the agent-facing binary. The design has now fixed the touch-set; `design.md`
`sec-8` is the authority on it, and what follows is that table at scope
altitude, recorded as design-target selectors on this slice.

| Area | Paths |
|---|---|
| Capsule contract, provisioning, backend, conformance suite | `crates/doctrine-control/**` (new, bin-only — no lib target) |
| Interpretation policy parse/normalize/hash | `src/interpretation.rs` (new, leaf, beside `dtoml`), `.doctrine/doctrine.toml` |
| `DOCTRINE_TOML` + `read_doctrine_toml_text` relocated so they can cross a crate boundary | `src/config_file.rs` (new, leaf), `src/dtoml.rs` (re-exports under the existing names), `src/main.rs` (one `mod` line) |
| Root lib target + leaf-only export set | `src/lib.rs` (new), `src/git.rs` (two items `pub(crate)` → `pub`), `.doctrine/adr/001/layering.toml` |
| Capacity + capsule root config | `.doctrine/doctrine.toml` `[capsule]` |
| Workspace membership and the checked set | `Cargo.toml` (`members`, `default-members`, `include` gains `!/src/lib.rs`), `Cargo.lock`, `justfile` (`publish` gains `-p doctrine`; `pkg-check` asserts `src/lib.rs` is absent from the package) |
| Layering / export fitness gates | `tests/architecture_layering.rs` |

**`tests/**` is not where the acceptance tests live.** A bin-only package cannot
be linked from `tests/` (`E0433`), so every `doctrine-control` test — including
the executed conformance tables — is a `#[cfg(test)]` module inside the unit it
tests. Only the export-set and two-tree gate assertions land in `tests/`. The
scope previously said otherwise; this is `sec-9`'s correction 4.

**`src/worktree/` is unedited**, not merely un-imported (DEC-155), which is what
retires R4 and changes R2's subject.

## Risks / Assumptions / Open questions

**OQ-1 — the decomposition is provisional.** The working shape is: (1) this
slice; (2) ingestion and conformance (REQ-451–453); (3) verification and
admission (REQ-454, 455); (4) recovery — candidate provenance, freeze/repair,
retention (REQ-456–458); (5) cutover. Later slices are deliberately **unminted**
— scoping slice 4 before slice 2 is designed is SL-233's failure at a coarser
grain. RFC-025 § State of play carries this as provisional, not settled.

**OQ-2 — closed.** `QUE-207` (binary and crate topology for the control plane)
is answered by `DEC-153`: option B, a workspace with `doctrine` +
`doctrine-control` over a lib target on the root package, split at canonical
mutation authority. Provisioning is `doctrine-control`'s verb. The residual —
the distribution contract owed by whichever slice first *releases* that binary —
is a close-time Follow-Up, not this slice's (see Risks `R5`).

**OQ-3 — three requirements are cross-cutting and close in no single slice.**
REQ-448 (control plane as sole canonical mutation authority), REQ-450
(freshness, whose criteria 2 and 3 need the candidate identity and harvest that
slices 3 and 4 build), and REQ-460 (the non-destructive failure envelope, whose
adversarial matrix spans stale base, candidate conflict, ref movement, and crash
replay). Coverage records per (slice, requirement, **change**), so each can
carry multiple contributing changes and close at the end — but an invariant
owned by every slice is owned by none unless each slice's closure intent names
its obligation explicitly. This slice's obligations: REQ-448's *denial* half
(the backend proves a capsule cannot reach canonical refs, shared object
storage, control-plane state, or credentials — REQ-459's suite is where that is
proven) and REQ-450 criterion 1.

**OQ-4 — what replaces the `review/*` and `phase/*` refs.** REV-046 § ADR-012
leaves it a target-design question. Nothing here depends on the answer — this
slice provisions and does not integrate — so it belongs to the cutover slice.

**R1 — evidence altitude.** SL-241 is Linux/bwrap, one client shape, n = 1 on
the real-agent leg. Feasibility evidence, not performance, portability, or
production-readiness evidence. No design or plan claim may exceed it. The
"16/16" summary is forbidden: fifteen rows reached model level, the env-file row
is unproven beyond the Rust fixture, structural `n/a` cells are not omissions,
and four `fail` rows are successful mutant detections.

**R2 — additive, so incumbent suites stay green unchanged. Subject changed by
the design.** As authored this named `src/worktree/jail.rs`. Under DEC-155 the
bubblewrap backend is self-contained and `src/worktree/` is unedited, so the
obligation now falls on what this slice actually changes: `dtoml`'s two
relocated items, behind re-exports under their existing names, and the layering
gate's directory parameter. Both take those shapes precisely so suites written
for something else are the proof (AGENTS.md).

**R3 — a property suite is only as good as its adversary.** REQ-459's suite is
the gate every future backend passes. Written weakly it certifies nothing.
SL-241's confinement matrix (P-C2) is the floor, not the ceiling. **Realised
twice during design, both times by the external pass rather than by the author**
— RV-346 `F-2` found two spec clauses merged into one row, and `F-19` found one
clause carrying two claims of which only one had a row. The standing form: the
gap in a property suite is invisible from inside it, and the only reliable
detector is an adversary constructing the backend the suite would wrongly pass.

**R4 — the `bwrap_core_argv` parity contract. Retired.** Added at pre-design
triage on the premise that this slice widens the shared bubblewrap builder.
DEC-155 gives the capsule backend its own flags and its own profile, so the
byte-parity test against `scripts/pi-spawn-confined.sh` is outside this slice's
diff and nothing here can fail it.

**R5 — the `doctrine-control` distribution contract. Stands, deferred, named as
a Follow-Up.** The binary is built and not released, so the nix `srcWithDist`
graft, the binstall asset name, `install.sh` and `release.yml` move together for
whichever slice first ships it — and as one change, since a missing embed graft
ships a hollow binary with no compile error. POL-002 is why it is named rather
than done here.

**R6, R7 — created by the design**, and `design.md` `sec-9` is the authority.
`R6`: `git`, `config_file` and `kinds` compile twice, which is safe only while
the binary names no `doctrine::` path — the mitigation is that rule, not a test.
`R7`: `src/lib.rs` is excluded from the published `doctrine` package, so the
published crate differs from the built one; `pkg-check` asserts the exclusion
rather than trusting it, the same shape as the crane embed-strip trap.

**A1** — SPEC-030 and ADR-020 are the authority; where this scope disagrees with
them, they win. **A2** — REV-046 stays proposed and unapplied throughout; this
slice retires nothing. **A3** — the existing `.doctrine/doctrine.toml` reader is
extended, not forked (DEC-136) — where *extended* means the typed projection
beside the shared reader described under REQ-449 above, which is what DEC-136's
intent admits and what its handoff note mis-describes. **A4 — discharged.**
`git::read_path_at` was to be verified as the whole impure surface REQ-449's
resolution needs; it was, at point of use, and `sec-6` exports it on that basis.

## Verification / closure intent

- REQ-449 and REQ-461 move `pending → satisfied` with recorded coverage
  (`doctrine coverage record`) naming the discharging test or agent evidence.
- **REQ-459 does not.** This scope originally expected it to; the design
  corrects that (`sec-8`). Criterion 1 is discharged by the nine-row table in
  full and criterion 3 structurally — one suite parameterised by backend — but
  criterion 2, bubblewrap becoming *the supported* backend, needs production
  acceptance evidence this slice does not produce (R1). So REQ-459 records a
  contributing `--change` and stays `pending`.
- REQ-450 records this slice as a contributing `--change` against criterion 1
  and stays `pending`; likewise REQ-448's denial half via the REQ-459 suite.
  All three are stated as partial in the reconciliation brief, in those words,
  not left for a reader to infer from a `pending` status.
- The REQ-459 property suite passes on Linux/bubblewrap and is structured so a
  second backend is admissible only by passing it independently — no
  Linux-specific assertion leaks into the shared contract. On a host where the
  backend is unavailable it reports **not admitted**, never a green skip
  (DEC-156); the cost of that ruling is residual 3 in `sec-9`.
- REQ-449's refusal cases are `VT` tests over the real parser: missing block,
  missing key, unknown key, unknown schema version, empty verification
  sequence, invalid normalized values, and each phase-contract widening attempt.
- A capsule-side rewrite of `.doctrine/doctrine.toml` demonstrably cannot change
  the bound policy (REQ-449 criterion 3).
- `QUE-207` is answered by an accepted DEC before the design gate clears.
- Existing dispatch, worktree, and confinement suites green **unchanged** (R2),
  and the 35 `DOCTRINE_TOML` / `read_doctrine_toml_text` call sites untouched.
- `doctrine check gate` green; clippy zero warnings — and `default-members`
  brings `crates/doctrine-control` into `lint`, `build` and `test`, so the
  executed conformance suite runs in the fast loop rather than only under
  `test-all`. Measuring its wall-clock cost is a phase obligation; the one
  lawful adjustment is a `--skip` on `test:` alone, never `#[ignore]`.

## Summary

## Follow-Ups
