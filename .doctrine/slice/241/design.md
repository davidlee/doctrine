# Design SL-241: Capsule spike rig

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

RFC-025 proposes replacing the shared-worktree dispatch trust model with
**execution capsules**: a worker runs in a disposable clone at a pinned base
inside an OS sandbox, and the trusted control plane provisions work, harvests
results, verifies conformance, and performs canonical mutations through a
narrow channel.

This slice builds and runs the probe rig that decides whether that model
survives contact. **The deliverable is evidence, not product** — EVD records
against QUE-200 (ingestion mechanism), the measurement table from
`probe-specs.md` § Measurements, and a scoped go/no-go. The rig is disposable
scaffolding; nothing in it migrates into dispatch machinery.

### 1.1 Scope honesty — read this before reviewing (CON-005)

The claim under test is narrow:

> **The capsule model bounds what a worker process can *do*. It does not bound
> what a worker can *say*, nor make its outputs trustworthy.**

Bounded by construction: the worker process's filesystem and network authority;
trusted-side *interpretation* of capsule content; git-level ingestion hazards;
atomicity of landing.

**Explicitly not bounded, and not pursued here:** a lying test suite (a worker
authoring its own tests passes them in any sandbox — RT-1, RFC-022, RFC-023);
prompt injection into the control-plane agent, which is an LLM reading
capsule-authorable refusal text, commit messages, file names and diffs; the
orchestrator's own authority (RT-8); semantically malicious code that passes
review; supply chain inside the capsule.

Operator ruling (2026-08-01): address what is achievable by construction —
especially where it falls out of design elements already being arranged — and do
not spend review rounds attempting to prove watertightness against a fully
compromised in-capsule agent. Reviewers should treat the narrow claim above as
the thing under review.

## 2. Current State

Today's dispatch model couples workers to the canonical repository through
shared worktrees and a shared object database. The coordination branch is a
**mutable staging area**: advance coord, project refs via `sync
--prepare-review`, write journal rows, mint a candidate branch, pin it, replay
integrate.

That apparatus is the source of the model's recovery affordances —
`sync --record-integration` (split-lineage close recovery), `candidate ingest`
(adopt an operator's hand-resolved merge), supersede-on-moved-trunk,
`--allow-corpus-clobber`, `worktree marker --clear` ("the self-brick cure"),
`gc --force`, the heal-forward import ladder, `dispatch next`'s triage
prescriptions. Each is individually reasonable; collectively they are a census
of the model's wedge states.

The high-judgment groundwork is banked in `.doctrine/rfc/025/`:
`mechanism-census.md` (13 DELETE / 12 TRANSFORM / 14 KEEP / 2 SCOPED),
`red-team.md` (RT-1 blocker, RT-2/RT-3 binding), `probe-specs.md` (P-C1/P-C2/P-C3
designs, the 16-row hostile matrix).

### 2.1 What the code actually says

Investigated during design; these findings shaped the pipeline.

**F1 — the import belt is reusable, its shell is not.** `worktree import --fork`
carries the belt this design wants (`doctrine-touch`, `claude-touch`,
`undeclared-scope`) but implements it as `git apply --3way --index` into a live
coordination worktree, requiring `HEAD == B` and a clean tree. Those are
worktree preconditions, not admission logic. The existing code says so: the MCP
arm calls the same pure belt with two of its five inputs hardcoded true, because
"the compose is working-tree-free onto the coord tip, so neither a coord HEAD
position nor a dirty coord tree is a precondition" (`src/mcp_server/dispatch.rs`
§ `import_plan`).

**F2 — the better-isolated import arm is gated on deleted machinery.** The MCP
`dispatch_import` is object-db only (`git::merge_tree`, no working tree, no
execution) but refuses anything without a durable **fork binding**
(`unprovable-fork`). A capsule is not a fork. So the stronger arm is gated on
exactly the identity choreography the census marks DELETE.

**F3 — no belted multi-commit admission path exists.** `import --fork` requires
exactly one non-merge commit with `S^ == B`; the multi-commit verb `worktree
land` is *explicitly* beltless (`src/worktree/land.rs`: "land's beltless
`--no-ff` merge is a different verb from import's belted apply"). Reported as a
finding against the current model. It does not constrain the target model —
see D2.

**F4 — the candidate provenance gate presumes staging.** `candidate create`'s
`check_provenance` (`src/dispatch.rs`) accepts a source ref only when it is a
journaled `review/<N>` or `phase/<N>-NN` carrying a `Verified` row on the
coordination branch. This is slice risk R1, sharpened: the gate does not
validate the *result*, it validates that a staging ritual completed.

## 3. Forces & Constraints

From `probe-specs.md` § Rig ground rules and `red-team.md`:

- **DQ-1 / RT-2** — the admission path must call existing verbs; hand-rolled
  merge, admission, or CAS logic is disqualified as evidence even when green.
  Narrowed by D1 below.
- **DQ-2** — confinement is the real mechanism (bwrap), not a simulated denial.
  Hostile probes are scripted `bash -c`, never prompted: a probe "contained" by
  a worker politely declining is void.
- **DQ-3** — every hostile probe asserts on an *observable* — a sentinel that
  must not exist, a ref that must not move, a byte-identical tree — never on
  absence of error output.
- **DQ-4 / RT-1** — the trusted side never executes or evaluates capsule
  content. Anything requiring candidate execution runs in a verify capsule.
- **RT-3** — v0 topology is control-plane-persistent: interactive session in the
  primary tree, headless fresh-context workers per phase. No in-session subagent
  workers.
- **RT-5** — harvest pins the result OID and operates on objects thereafter.
- **RT-7** — capsule reuse weakens the evidence chain; provisioning mode must be
  recorded.
- **POL-002** — the *rig* is a client-local artifact and may use this project's
  conventions freely. The *evidence* may not: anything the eventual product
  enforces must rest on contracts doctrine owns.
- A failed probe row is a **finding**, never a quiet rig edit — `/consult`
  before adapting (`probe-specs.md` § Order and gating).

## 4. Guiding Principles

1. **Measure the target model, not the current one.** A rig built on the
   existing choreography measures the existing choreography.
2. **Simplicity is the deliverable's substance.** If the capsule model is not
   markedly simpler, it has not earned the migration.
3. **Zero archaeology after landing** (CON-004) — stated as a testable
   invariant, not an aspiration.
4. **Evidence over assertion.** Where a property can be measured rather than
   claimed, measure it — including the portability classification itself.
5. **Reuse the hard semantics; question the transport.**

## 5. Proposed Design

### 5.1 System Model

Three zones. One mutation.

```
┌─ TRUSTED SIDE ──────────────────────────┐    ┌─ CAPSULE (bwrap) ───────────┐
│ accepted ref  ──── one CAS advance ───▶ │    │ clone at pinned base B      │
│ quarantine dir (disposable, fsck on)    │    │ worker commits FREELY,      │
│                                          │    │ amends, rebases, iterates   │
│ git plumbing + candidate verbs only      │    │ survives a refusal (CON-004)│
│ interprets NO capsule content (CPT-001)  │    └─────────────────────────────┘
└──────────────────────────────────────────┘    ┌─ VERIFY CAPSULE (bwrap) ────┐
   no coordination worktree                     │ clone at pinned OID         │
   no coordination branch                       │ runs the suite (RT-1)       │
   no journal                                   └─────────────────────────────┘
```

Both capsule kinds use the **same** sandbox profile — P-C2's uniform-confinement
claim and RT-1's verify capsule satisfied by one mechanism, not two.

#### The five stages

| # | stage | what it does | reuse |
|---|---|---|---|
| 1 | `harvest` | M-A fetch / M-B bundle → quarantine, fsck, **pin OID** | plumbing |
| 2 | `conform` | ancestry (descends from B, no merges) + belt predicate (forbidden paths, declared scope) | belt as **pure logic** |
| 3 | `verify` | verify capsule at the pinned OID; verdict is the sandbox's exit status | rig |
| 4 | `admit` | 3-way, `Conflicted`, OID pin — conflict/staleness semantics | **existing verbs** |
| 5 | `advance` | one CAS ref move, or refuse | **existing verb** |

Each stage emits `stage=<name> verdict=pass|refuse token=<t>`. The runner records
the **first refusing stage**.

Absent by construction, relative to the current model: coordination-branch
staging, `prepare-review` projection, journal rows, the fork-binding gate, and
any derive-to-single-commit step.

### 5.2 Interfaces & Contracts

**Work contract** — pins an OID, never a branch name (RT-11: "accepted canonical
commit" needs naming per project; an OID is the POL-002-clean form).

```toml
base   = "<sha>"        # contracted base OID
slice  = 241            # selector source for the conform stage
label  = "p01"          # capsule identity
mode   = "fresh"        # fresh | reused-from:<capsule,phase>   (RT-7)
verify = "npm test"     # from the fixture's declaration
```

**Harvest** — two mechanisms, one signature, so the matrix loops without
branching:

```
harvest_fetch  <capsule> <quarantine>  → pinned OID on stdout
harvest_bundle <capsule> <quarantine>  → pinned OID on stdout
```

*M-A two-hop fetch*: capsule → throwaway quarantine with
`fetch.fsckObjects=true`; `git fsck`; quarantine → canonical quarantine
namespace. *M-B bundle*: worker `git bundle create` pre-doorbell; control plane
`git bundle verify` then fsck'd fetch from the file.

Both read the result ref **exactly once** and pin the OID (RT-5).

**Interpretation-surface declaration** — default-deny, the dual of
`.worktreeinclude` (DEC-099):

```
# fixtures/heavy/interpretation-surface.txt
exec:      cargo nix direnv just rustc
interpret: build.rs flake.nix .envrc rust-toolchain.toml **/build.rs
verify:    cargo test
```

- `exec:` drives the DQ-4 audit — grep `control/**`; these tokens must be absent.
- `interpret:` drives the P-C3 **hostile instantiations** — where a class-2/3
  payload is planted per fixture.
- `verify:` is the one place project execution is wanted, and it runs inside the
  verify capsule.

Note the asymmetry: **class-2/3 enforcement is structural, not
pattern-matched.** The trusted side never materialises a harvested tree — it
handles objects only — so there is nothing for a glob to guard. The `interpret:`
list tells probes where to plant payloads; it does not police the control plane.
Universal classes (git-level auto-load, path-shaped data, resource shape) are
doctrine-owned, hardcoded, and not declarable.

### 5.3 Data, State & Ownership

**Fixtures.** Two (DEC-101), with a per-row column recording which produced each
cell.

*Heavy* — `git clone --no-hardlinks` of this repo; remotes stripped, no
`credential.helper`. Doctrine-installed by construction. Carries P-C1's cost
baseline, the verify-capsule leg, and the H11/H12 instantiations.

*Light* — a small TypeScript project the rig builds, deliberately unlike this
repo: `package.json` scripts (`build`/`clean`/`test`/`lint`/`format`), one
red→green test, trunk branch `mainline`, commit style `[add] …`. Doctrine-
installed with one scratch slice carrying `design-target` selectors over
`src/**`.

A fixture needs a git repo, doctrine install, one slice with selectors, a
declaration, and something that builds. It needs **no plan and no phases** —
that requirement came solely from `prepare-review`'s phase-completion gate,
which is out of the pipeline.

**Evidence storage (OQ-1).** Committed text summaries and the generated
measurement table under `.doctrine/rfc/025/evidence/`; raw logs gitignored. The
summaries are the evidence; the logs are the exhibit. RT-9's archive-tier
question at small scale.

### 5.4 Lifecycle, Operations & Dynamics

**Order and gating** (DEC-103, amending `probe-specs.md` § Order and gating):

1. **P-C1a** — deterministic: clone, provision, nix, build, test, harvest cost.
   Stub worker. Banks every measurement except tokens.
2. **A2 smoke** — trivial `claude -p 'print OK'` inside the sandbox. Near-free,
   run early, purely to prove the jail's `~/.claude` credential arrangement
   survives nested bwrap. A failure here means the capsule model needs a
   credential-proxy design — worth learning on day one.
3. **P-C2** — the confinement matrix on the same rig.
4. **P-C3** — the hostile matrix, M-A and M-B side by side.
5. **P-C1b** — the real agent executing a real red→green phase: the token
   measurement, and "does a phase actually reach green in a capsule".

The rig defaults to `--stub`; `--agent` is explicit opt-in. Everything in P-C2
and P-C3 is scripted by DQ-2 mandate, so P-C1 is the only probe needing an LLM.

**The retry loop.** On a refusal the capsule is still alive with its work
intact: the control plane hands back a refusal, the worker fixes it in place,
rings again, and harvest re-runs at a **new pinned OID**. Teardown is
conditional on a *successful landing*, not on the worker exiting.

**Matrix harness.** One loop over `matrix.tsv`; per `(row, fixture, mechanism)`:

```
provision fixture  →  Hnn.mutate (fixture's instantiation)  →  pipeline
                   →  record first refusing stage
                   →  Hnn.assert            (row-specific observable)
                   →  assert_pristine       (universal, CON-004)
                   →  emit result row
```

### 5.5 Invariants, Assumptions & Edge Cases

**I1 — landed state is append-only (CON-004), in three clauses.**
*(a)* Inside a capsule before landing: total freedom — commit, amend, rebase,
iterate. *(b)* At the boundary: all-or-nothing. Either the accepted ref advances
or it does not move; what is forbidden is a partial landing needing
reconstruction. This does **not** mean redoing the phase. *(c)* After landing:
append only. A subsequent fix is a new commit from a new capsule at the new
base — never an amend, rebase, or force-push of landed history. Incremental
fixes across phases are expected and cheap.

**I2 — idempotency is content-addressed.** RT-6 requires idempotent harvest and
keys it on the journal. With no journal there is nothing to key on except the
**pinned OID**: harvesting the same OID twice is a no-op by content-addressing;
a new OID is a fresh attempt. Simpler than bookkept idempotency and it cannot
drift out of sync with reality.

**I3 — stage ordering is load-bearing.** Ancestry precedes everything that
normalises the result. A merge commit `S` with parents `(B, X)` satisfies a
naive single-commit predicate; only an ancestry check that rejects merges kills
H3.

**I4 — the verify verdict is the runner's, not the candidate's.** The verdict is
the sandbox process's exit status as observed by the parent, never a file the
capsule wrote. The capsule may write logs; those are untrusted text, size-capped,
never parsed for the verdict. The clone itself runs inside the sandbox, so no
candidate tree is materialised trusted-side.

**I5 — refusals report trusted-side-computed tokens.** Stage, refusal token, and
paths derived from git — never capsule-authored prose (CON-005; extends RT-4's
"artifact content is never authoritative" with "nor relayed verbatim unmarked").

**Assumptions carried:** A1 nested bwrap works inside the jail (ADR-008 D-B3
precedent; `pi-spawn-confined.sh` is the seed). A2 headless `claude -p`
authenticates inside the capsule sandbox — tested early and explicitly. A3
worker token cost is external to the orchestrating session, so probe *execution*
is context-cheap for the driver; log volume is the context cost to manage.
ASM-007 — the five interpretation-trigger classes are exhaustive; the TypeScript
fixture is its falsification vehicle.

## 6. Open Questions & Unknowns

- **OQ-1** — long-term home for probe evidence logs (RT-9 archive tier at small
  scale). v0 ruling in § 5.3; revisit if raw logs exceed the gitignored dir's
  usefulness.
- **QUE-200** — ingestion mechanism (M-A vs M-B). The rig's whole point; settles
  only on probe evidence.
- **QUE-201** — where the interpretation-surface declaration lives in shipped
  form (`doctrine.toml` block / dedicated manifest / work-contract field).
  Post-spike REV.
- **OQ-2** — whether the conform stage can reach the belt predicate from shell.
  If it needs a read verb that does not exist, that is a `/consult`, not an
  improvised `src/` change (§ 8 R2).

## 7. Decisions, Rationale & Alternatives

**D1 — RT-2's reuse mandate binds conflict semantics, not transport
(DEC-104).** Operator ruling. *Reuse, mandatory:* `candidate create`'s 3-way and
`Conflicted` refusal, `admit`'s OID pin, integrate's CAS. RT-2's worked example
— result #1 lands, result #2 goes stale — is genuinely subtle and already
modelled exactly; re-deriving it is the disqualifying redo. *Reuse as pure
logic:* the belt predicate. *Do not rebuild:* coordination staging,
`prepare-review`, journal-row-as-precondition, fork binding.

*Alternative rejected:* stand up journal machinery so `candidate create`'s gate
is satisfied. This was the first draft of this design and it is a local
maximum — it measures the old choreography and guarantees we never discover the
gate is the wrong shape.

**D2 — the provenance gate is a finding, not scaffolding (DEC-104).** The
capsule model does not lack provenance; it carries a *different* proof — pinned
OID + verify-capsule attestation + ancestry from a contracted base — where the
journal row proves only that a staging ritual completed. Re-grounding REQ-316 on
that proof is post-spike REV work. Dropping the verb also dissolves F3's
single-commit constraint: conformance is `diff B..S --name-only`, which does not
care how many commits lie between. The worker commits freely and nothing
squashes it. That the simplification *dissolved* a finding rather than working
around it is the signal the direction is right.

**D3 — interpretation surface: danger is a triple, not a tool (CPT-001).**
`cargo test` inside a capsule is the deliverable's build; `cargo test` on the
trusted side over harvested content is an exploit. Same binary. What differs is
**(operation × content provenance × side)**. A forbidden-tool list conflates the
tool with the trigger and can only encode the tools of the project that authored
it. Five trigger classes: explicit execution · build-system evaluation ·
toolchain auto-load · git-level auto-load · path-shaped data · resource shape.

**D4 — ownership splits on the universal/language-bound line (DEC-099).**
Doctrine owns the taxonomy and enforces the universal classes; the client
declares its language-bound instances; **absence is refused, never defaulted**.
A shipped default list is coupled to whichever project authored it and passes
*silently* for a project whose triggers it does not know — POL-002's
invisible-until-the-second-client failure exactly.

**D5 — two fixtures, and the second is a portability control (DEC-101).** A
*convention-free* fixture would only prove no dependency on this repo's habits,
and can pass **vacuously** (no build system ⇒ verify trivially skips ⇒ green
means nothing). A *differently-conventioned* fixture proves correct
parameterisation. Any stage passing heavy and failing light has exposed a
host-convention dependency — the cheapest POL-002 audit available. Cost is the
secondary benefit: 16 rows × 2 mechanisms = 32 cells, of which only H11/H12 need
a real build.

**D6 — abstract rows, per-fixture instantiation (DEC-102).** H11 is "hostile
build-time code writes outside the workspace" — `build.rs` on heavy,
`postinstall` on light. Same row, same expected boundary, two instantiations.
This makes **altitude a measurement instead of an assertion**: holds under both
⇒ `model-level`; one only ⇒ `client-local`, and the divergence is a finding.
Hand-authoring an altitude column would put judgement where the spike is
supposed to put evidence.

**D7 — stub-first worker (DEC-103).** More than one full run will be needed, and
most of what the rig settles does not need a slow, expensive, non-deterministic
agent.

## 8. Risks & Mitigations

- **R1 (was the headline risk) — resolved into findings.** The candidate
  provenance gate presumes dispatch ledger state. Traced during design to F1–F4;
  addressed by D1/D2. No longer a blocker; it is evidence.
- **R2 — the conform stage may not reach the belt predicate from shell.** The
  belt is Rust (`classify_import` / `conformance::undeclared_paths`). If no read
  verb exposes it, the choice is a `/consult`, not an improvised `src/` change
  (slice § Non-Goals). Mitigation: identified before implementation; first thing
  the rig skeleton probes.
- **R3 — A2 (credentials in nested bwrap) fails.** Mitigated by testing it on
  day one as a standalone near-free smoke, decoupled from P-C1b.
- **R4 — the rig lies.** A hostile row "passing" because the rig is broken is
  indistinguishable from a real kill. Mitigated by the happy-path self-test
  (§ 9) and positive controls on both audits.
- **R5 — log volume swamps the driving session's context** (A3). Mitigated by
  machine-readable `results.tsv` as the only thing the driver reads; raw logs
  stay on disk.
- **R6 — scope creep into product.** The rig is scaffolding. Non-goals fence
  dispatch machinery, new CLI verbs, census migration, FR-007, RFC-025 prose
  edits, and sandbox-profile productization.

## 9. Quality Engineering & Validation

The evidence is the deliverable, but the rig itself must not lie:

- **Happy-path self-test on the light fixture** — a clean run that must land
  green *before* any hostile row claims a kill. Without it every "refused" is
  indistinguishable from "rig broken". This is the rig's own red/green.
- **Positive controls on both audits** — `audit-dq4` and `audit-nohooks` each
  grep for a token known to be present and fail if that grep returns empty. A
  negative grep without a positive control proves only that grep ran
  (`mem_019fa18161f4…`).
- **Kill-boundary attribution tested directly** — the pass/partial distinction
  requires knowing *which* stage refused, so stage emission is asserted, not
  inferred from exit codes.
- **`assert_pristine` on every row** — after a refusal at any stage the trusted
  side is byte-identical to its pre-run state, modulo the disposable quarantine
  dir (CON-004). Today only H5 asserts byte-identity; here it is universal.
- **H15 sharpens** — kill at *each* stage in turn, not one interruption point.

**Measurements** (`probe-specs.md` § Measurements, plus one): trust-bearing
lifecycle states; mutable refs written; security-significant hooks (target 0);
role-detection rules (target 0); git operations between worker-done and
candidate-create; tokens per accepted phase; human interventions; wall-clock and
disk; **recovery affordances reachable (target 0)** — counted against both
models for the before/after column.

**Closure.** Every P-C1/P-C2 row and every P-C3 matrix row has a recorded
pass/partial/fail for both M-A and M-B (or a consulted deviation); EVD records
exist and are linked to QUE-200; the measurement table is filled; a **scoped**
go/no-go lands in `.doctrine/rfc/025/`. Scoped means: go on Linux/bwrap, for a
client of this build shape, with model-level rows proven portable and
env-conditional rows outstanding for macOS. Writing the scope in is what stops
the REV over-claiming.

### 9.1 Code impact

| path | change |
|---|---|
| `scripts/spike-capsule/**` | new — the entire rig |
| `.doctrine/rfc/025/evidence/**` | new — committed probe summaries + measurement table |
| `.doctrine/knowledge/**` | EVD records (CPT-001, DEC-099/101/102/103/104, ASM-007, QUE-201, CON-004/005 already landed) |
| `.doctrine/slice/241/**` | design, plan, notes |
| `.gitignore` | one entry for the raw-log dir |

No `src/` changes (see R2).

```
scripts/spike-capsule/
  rig                        entry: rig <c1a|smoke|c2|c3|c1b> [rows…] [--stub|--agent]
  lib/{common,emit}.sh
  control/                   TRUSTED — audited against the active declaration
    fixture-{heavy,light}.sh
    harvest-{fetch,bundle}.sh        M-A / M-B behind one interface
    pipeline.sh                      the five stages
    audit-{dq4,nohooks}.sh           with positive controls
  capsule/                   runs inside the sandbox only
    sandbox.sh               the bwrap profile (P-C2's subject)
    provision.sh
    worker-{stub,agent,hostile}.sh
    verify.sh
  fixtures/
    heavy/interpretation-surface.txt
    light/                   the TypeScript project + its declaration
  probes/c3/
    matrix.tsv               row | fixture | vector-class | instantiation | expected-stage | altitude*
    H01.{mutate,assert} … H16.{mutate,assert}
```

`*altitude` is computed from results, never authored.

## 10. Review Notes

Internal adversarial pass pending. Reviewers: § 1.1 sets the claim under review;
please do not open findings that require watertightness against a compromised
in-capsule agent (CON-005, operator ruling).

Forward compatibility: RFC-023 (executable plan gates, adversarial TDD) will
substantially revise plan machinery. Operator ruling — adopt current machinery
as-is; the five-stage pipeline does not depend on plan-gate mechanics, so those
revisions should land orthogonally (`notes.md` § Forward compatibility).
