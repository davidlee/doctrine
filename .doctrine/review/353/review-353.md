# Review RV-353 — design of RFC-025

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

> **HERESIS URITOR; DOCTRINA MANET**

An **inquisition**, convened at the User's command, against the whole capsule
programme — its governance (`RFC-025`, `ADR-020`, `SPEC-030`, `REV-046`) and the
implementation raised under it (`SL-241`, `SL-248`, `SL-252`,
`crates/doctrine-control`). Facet is `design`: the aspect arraigned is the
**architecture**, not the workmanship. The code of `SL-248` is, by every local
measure, competently built; that is precisely what makes this a design trial and
not a code review. A heresy executed with skill is still a heresy.

`review prime` was refused — it takes a slice target only, and this tribunal sits
on an RFC (`mem_019f25e0d93175a2b47c5cc73bd8e263`). The evidence below was
gathered by hand and is cited by path, id and measurement.

### The standard the accused is held to

The founding claim, confessed in `RFC-025`'s own opening line: *"the current
investigation into **simplifying** Doctrine's dispatch and confinement model."*
`ADR-020` restates it — the incumbent arms' "composition is brittle", and the
capsule is offered as the smaller authority model.

Alongside it, the project's own sanctioned rule of proportion, from `CLAUDE.md`
§ Guiding Principles: *"Everything has a denominator: complexity, token cost,
human attention. Work with disciplined laziness: spend as little as possible on
each axis to obtain a useful outcome."* And `POL-002` — platform independence
from host-project conventions.

### Lines of interrogation

1. **The simplification ledger.** What has the programme deleted? Against what
   it has added? If the answer is "nothing yet, deletion is the last slice",
   the claim of simplification is not a design property — it is a promissory
   note, and the tribunal must price it.
2. **The manufacture heresy.** The User's charge, in their own words: *"Doctrine
   should not discover how to manufacture an execution environment from
   arbitrary host paths. It should select/provision an environment using a
   self-contained, preferably existing mechanism, then confine it."* Does the
   design derive environments, or select them?
3. **Concern tangling.** Provisioning, confinement, control-plane transport,
   admission, verification, retention, capacity, host probing — are these
   separable, and did the design separate them?
4. **The measurement prototype.** What proportion of the shipped artefact is the
   thing, and what proportion is the apparatus that measures the thing? And is
   the measurement sound?
5. **Governance provenance and order.** `ADR-020` (accepted) and `SPEC-030`
   (active) descend from an `open` RFC that self-describes as a *"living
   investigation document"* "dropped in largely as-is" from an external web-agent
   conversation. Is the governance load-bearing on evidence, or on prose?
6. **Blast radius.** What has the programme done to the host project's gate,
   release posture, and open-debt surface?
7. **Salvage.** The User's actual question. Of everything built, what survives
   the fire?

## Synthesis

> **Revised 2026-08-11** after external architectural review. Findings `F-11`,
> `F-12` and `F-13` narrow the first pass; this synthesis carries the narrowed
> holding and governs where it conflicts with `F-1`, `F-3`, `F-6`, `F-7` or
> `F-10`. The ledger is append-only, so those findings stand as raised — read
> them with their corrections. The inquisitorial register of the first pass has
> been retired here: this section is intended to read as an architectural record
> six months from now, not as catharsis.

### Holding

**The programme is stopped.** `RFC-025`'s capsule programme failed its
simplification objective because it conflated environment construction,
confinement, and dispatch-transaction concerns. The decisive error was
attempting to derive a portable executable environment from arbitrary host state
rather than selecting or provisioning one through an existing substrate. That
error introduced host-sensitive semantics whose defects propagated across
otherwise separable concerns, made local fixes programme-level changes, and
required a conformance apparatus whose strongest claims were invalidated by the
environment in which it was developed. The programme should stop before further
investment; reusable confinement mechanics and empirical findings should be
harvested, while the successor is redesigned around separately shippable
provisioning, confinement, and control-plane concerns.

### The case

The load-bearing argument is **`F-2` → `F-5` → `F-8`**, and it stands
independent of every other finding:

- **`F-2` — the architectural inversion.** The system derives an execution
  environment from arbitrary host state instead of selecting one and confining
  it. This single diagnosis explains why canonicalization, `$PATH`, closure
  resolution, multicall aliases, shebangs, provisioning bootstrap and host
  topology all became entangled with one another.
- **`F-5` — the second-order error.** Environment construction was welded to
  confinement, transaction handling, admission, transport, retention and
  capacity, so a local defect becomes a programme-level design event. `F-12`
  adds a further instance: the tangle reached into the root package's *target
  layout* (a `lib` target, a curated leaf-only export set, the `EXPORTED`
  assertion, the `sec-6`/`sec-9` invariants) to serve one capsule-only module.
- **`F-8` — the empirical proof that the abstraction is wrong.** Fixing one
  supposedly local fixture defect (`SL-252`/`ISS-341`) reached backwards into
  production semantics, an earlier phase's exit criterion, `PATH` construction
  and provisioning. `DEC-188`: *"That is a slice, not a phase."*

Three findings, sufficient on their own to stop the programme.

**Supporting, in narrowed form.** `F-3`'s soundness limb is the substantive
addition: the conformance apparatus emitted strong claims while its fixture
silently granted capabilities far broader than those claims implied — 691 store
paths and an executable toolchain inside a capsule reading `Proven` for
`BoundedInputSet`. The 14,252-line / 61% / 2.2× figures are *denominator
evidence* of low evidentiary return per unit complexity, not proof of anything
by themselves (`F-11`). `F-1` contributes the narrower fact that after the first
foundational slice the replacement already exceeds the incumbent's apparent
complexity budget while the benefit remains hypothetical, and that the deferral
is **self-sealing** — `REV-046` may not be approved until slice 5, and slices
2–5 stay unminted until their predecessors are designed, so a cutover plan is
not permitted to exist until the cutover. `F-6` contributes that accepted
governance hardened assumptions untested across the environmental variation
those assumptions depended upon. `F-4` is the operational finding and is
separable from all of the above. `F-10` is an appendix.

**What is not charged.** `SL-248`'s workmanship. Constants are named, refusals
are distinguishable, layering respects `ADR-001`, the ledger is honest, and the
authors caught their own errors in time — `DEC-186` was withdrawn as refuted by
the hand that wrote it, `DEC-188` states the failure more plainly than this
tribunal did, and `RSK-231` was filed by the accused against their own
programme. This is a well-built wrong thing, and the review is against the
design precisely because the execution cannot be blamed.

### The three kinds of complexity (`F-13`)

The single most important thing in this record for the successor's benefit.
Every mechanism the successor considers must be assigned to one tier:

| Tier | What it is | Under abandonment |
|---|---|---|
| **1. Intrinsic** | Namespace and mount semantics, descriptor inheritance and closure, process teardown and signals, credential/capability posture, `no_new_privs`, resource limits | **Survives.** Salvage it. |
| **2. Incumbent transaction** | Deterministic workspace creation, agent lifecycle, control-plane communication, result transport, teardown, recovery, concurrency, non-Nix platforms, stronger isolation on demand | **Returns unsolved.** Predates the programme; abandoning it dissolves nothing. |
| **3. Self-inflicted** | Environment inference from `$PATH`, the `closure-resolver` extension point, `expand_closure_root`/`closure_members`, inner-`PATH` synthesis, host-shaped conformance fixtures, and the apparatus measuring all of it | **Disappears.** The only tier the verdict may call waste. |

The question no artefact currently answers, and the post-mortem must: *which
complexity disappears when the architecture changes, and which merely moves back
to the still-unsolved orchestration problem?* Without it, the successor cannot
tell its own legitimate hard requirements from evidence that it is repeating the
failure — and `F-8`'s triage has no principled basis for sorting dissolved from
carried. **Tier 3 → dissolved. Tiers 1 and 2 → carried.**

### Salvage manifest

**1. The five-minute workflow is the reference baseline — not the finished
architecture.** Clone, existing flake, bwrap, run the agent; add the one flake
line for network isolation. It proves the essential confinement workflow can be
radically simpler than the programme assumed. It does not solve tier 2. The rule
it yields: **any successor must justify every mechanism it adds relative to that
working composition.**

**2. `backend/bubblewrap.rs` — the argv assembly, in part (2,635 lines).** The
knowledge of *how to invoke bwrap correctly* is dearly bought and hard to
re-derive: namespace flags, `no_new_privs`, the `RLIMIT_FSIZE` `pre_exec`, the
`/proc/self/fd` descriptor sweep, standard-stream ownership, the two `unsafe`
sites documented in `ADR-021`. Keep the mechanism. Discard everything that
*computes what to bind* — `system_readable_roots`, `closure-resolver`,
`expand_closure_root`, `closure_members`, the inner-`PATH` derivation. Under
selection the bind list is a manifest the provisioner already holds.

**3. `SL-241`'s hostile-ingestion findings, as knowledge — not as code.** The
16-row hazard matrix taught real lessons about untrusted result transport. They
belong in `EVD`/`DEC` records and, when a successor needs a result transport,
as production acceptance tests. The rig itself was correctly scoped disposable
by `SL-241` and stays disposable.

**4. The measured host facts, as memories.** `--ro-bind` is read-only but **not
`noexec`**. Identity-mapped roots leak 691 store paths and an executable
toolchain. `canonicalize` destroys a declared bind name (`ISS-344`). A
`path_exists("/bin/sh")` guard passes on the host that cannot exec it, so
capability probes must be **structural, not existential** (`ISS-339`). These
cost real off-jail runs to learn and must not be re-purchased.

**5. The epistemic discipline itself.** The ledgers, the withdrawn decision, the
honest `DEC-188`, `RSK-231` filed by the accused against their own programme,
and the 180 friction observations. The governance machinery worked: it recorded
the failure faithfully and in time for the abort lever to be worth pulling. That
is the system functioning, and it is why this cost 60k lines instead of 600k.

**Discard (tier 3):** `conformance.rs` (14,252), `provision.rs`'s thirteen-step
protocol with its publish-or-adopt and creation-token machinery (1,922),
`capacity.rs` (391), `SPEC-030`'s fourteen pending requirements, and `ADR-020`'s
authority-boundary decision.

**Re-examine, do not discard (`F-12`):** `src/interpretation.rs` (1,624) and the
`[interpretation]` policy. The *mechanism* is disproportionate on its own
evidence — a required-with-no-defaults block obliging every adopting project to
exhaustively enumerate its interpreters is a security surface whose omissions
are silent. But the *requirement* — who may interpret worker-controlled content
— is orthogonal to how the environment is obtained, and any successor retaining
a trusted/untrusted execution split inherits it. Answer the retention question
before deleting anything, and decide the fate of the root package's `lib` target
separately; it does not automatically follow.

### Penance, in order

1. **Immediately, before anything is deliberated (`F-4`).** Remove
   `crates/doctrine-control` from `default-members` (`ISS-343`) and put the
   live-bwrap conformance tests behind explicit opt-in (`ISS-342`). The project's
   gate must be green and its release unblocked while the successor is designed.
   *Verification:* `just gate` green off-jail; `doctrine check gate` clean.

2. **Write the complexity partition (`F-13`).** Assign every mechanism in
   `crates/doctrine-control` and `SPEC-030` to tier 1, 2 or 3. This blocks step
   3, which cannot sort dissolved from carried without it. *Verification:* every
   mechanism tiered; the tier-2 list is recorded as returned-unsolved work.

3. **Vacate the governance (`F-6`).** A `REV` against present-tense governance:
   `ADR-020` superseded (not amended — the central choice is being reversed);
   `SPEC-030` withdrawn or reduced to what a successor actually specifies;
   `REV-046` closed unapplied. `RFC-025` **stays** as the historical record and
   receives the post-mortem. *Verification:* `doctrine spec validate` clean; no
   `active` spec with zero shipped consumers.

4. **Triage the register, do not work it (`F-8`).** Sort `cluster:capsule` by
   the tier rule from step 2: tier 3 → dissolved, tiers 1–2 → carried, `F-4`'s
   items → live. `SL-252` is dissolved. *Verification:* no `open`
   `cluster:capsule` item lacking a triage disposition.

5. **Harvest before deleting (`F-3`, salvage 3–4).** Land salvage items 4 and 3
   as memories and `EVD` records *before* any code is removed. Knowledge deleted
   with its carrier is knowledge re-purchased at full price.

6. **Design the successor (`F-5`, `F-11`).** Three separately-shippable
   concerns — provisioning (select + realise), confinement (apply the platform
   sandbox), control-plane transport. Two provisioning options as the Owner
   framed them: (a) nix+bwrap / seatbelt; (b) microVM or `systemd-nspawn` into a
   configured base image. The delivery criterion is **incremental independently
   valuable delivery** — *not* the first pass's withdrawn "retire incumbent
   mechanism in the same slice", which would force unsafe partial cutovers
   (`F-11`). Every added mechanism must be justified against the five-minute
   baseline, and the design must answer `F-12`'s retention question — does a
   trusted/untrusted execution distinction survive? — before any deletion in the
   `[interpretation]` area.

### Standing risks

- **`RSK-231` remains open and is now evidenced**, not merely suspected. It
  should govern the successor's design gate.
- **Sunk-cost relapse.** 23,201 lines of competent code will argue for its own
  survival, most persuasively through `bubblewrap.rs`, which genuinely has
  salvage value. The line between "keep the argv assembly" and "keep the crate"
  must be held explicitly, or the successor becomes a refactor and the whole
  edifice returns wearing a new name.
- **`POL-002` misreading recurs.** The policy was applied correctly in letter
  and catastrophically in spirit. Until the successor records an explicit
  reading — that `POL-002` forbids assuming a host's conventions, not naming a
  confinement mechanism — the same inversion will be re-derived by the next
  agent that reads it.
- **Confinement strength on non-NixOS hosts** is genuinely weaker under option
  (a), as the Owner already notes. That is a known, bounded, statable tradeoff.
  It is not a reason to rebuild the capsule.
- **Tier-2 relapse (`F-13`).** The successor will meet deterministic workspace
  creation, agent lifecycle, control-plane communication, result transport,
  teardown, recovery and concurrency, because abandoning the capsule returns
  those problems rather than solving them. If that difficulty is misread as
  recurrence of this failure, work that is going well will be aborted. The tier
  partition exists to prevent exactly this.
- **Over-reading this verdict.** The holding is narrow: the *derivation* of
  environments from host state was wrong, and the concerns were welded. It is
  not a finding that isolation is unnecessary, that rigour is waste, or that
  large test apparatus is inherently suspect. `F-11` records where the first
  pass overstated; a reader who takes the first pass at face value will
  over-correct.

### Tolerated

Nothing. Every finding is terminal. The one finding disposed `fix-now` at the
first pass (`F-4`) is triage, not tolerance; `F-11` and `F-12` withdraw
overstated reasoning without disturbing the holding.

> *Heresis uritor; doctrina manet* — and the doctrine that remains is narrower
> than the fire that found it.

## Complexity partition (`F-13`)

> Written 2026-08-13 as **penance step 2**. Step 3 (vacate the governance) and
> step 4 (triage `cluster:capsule`) take their sort order from here. This
> section is a classification, not a work plan: nothing below is an instruction
> to delete, keep, or build anything.

### The test, applied literally

`F-13`'s three tiers are defined against **the requirement**, never against the
code that serves it:

| Tier | The obligation | Disposition |
|---|---|---|
| **1 — intrinsic** | Survives any successor that confines anything at all | Salvage |
| **2 — incumbent transaction** | Predates the programme and **returns unsolved** when the capsule is abandoned | Carry on the register |
| **3 — self-inflicted** | Exists *only because* environments are derived from arbitrary host state (`F-2`), and dies with that error | The only tier that may be called waste |

So the question asked of every entry below is `F-12`'s: **does this mechanism's
requirement die with the architectural error?** — never the easier question,
*is this mechanism a consequence of the architectural error?* Almost everything
in `crates/doctrine-control` is a consequence of `F-2`; that is what `F-2`
means. Very little of it has a requirement that dies with `F-2`. The first pass
conflated the two, `F-12` caught it, and the difference is the whole content of
this section. Where the argument would not close either way, the entry is tier
2, which is the conservative call: tier 2 says *still a real problem, still
unsolved*.

**Two things this partition does not decide.**

1. **Tier is not code disposition.** A tier-2 obligation whose implementation is
   deleted is still tier 2 — the problem returns to the register, not to the
   bin. The Salvage manifest's *Discard* list answers "what code goes"; this
   section answers "what problem stays". They diverge in three places, recorded
   below, and both are correct about their own question.
2. **Tier is not salvage value.** Tier-3 code can carry a fragment worth
   keeping, exactly as salvage item 3 keeps `SL-241`'s findings as knowledge
   while its rig stays disposable. Where that applies it is noted at the entry.

### Scope

`F-13`'s wording names `crates/doctrine-control` and `SPEC-030`. Tier 2 is
defined by what the abandonment *hands back*, and much of that was never inside
either — it is owned today by `src/dispatch.rs`, `src/worktree/` and
`src/interpretation.rs`. A partition confined to the crate would under-populate
tier 2 and make the successor look cheaper than it is. So the crate and the
spec supply the inventory, and tier 2 additionally pulls in mechanisms from
wherever they live, naming their home.

### Tier 1 — intrinsic

Survives any successor. This is the salvage list at mechanism granularity.

| Mechanism | Home | Why it survives |
|---|---|---|
| Namespace and mount semantics — `--unshare-all`, `--new-session`, `--die-with-parent`, `--clearenv`, the bind flags, and their ordering rules | `backend/bubblewrap.rs` § flag tokens, `confinement_argv` | How to invoke the mechanism correctly is dearly bought and mechanism-shaped, not architecture-shaped (salvage 2) |
| Wall bound and per-file size cap applied **outside** the namespace — `timeout -k`, `RLIMIT_FSIZE` via `pre_exec` | `bubblewrap.rs` `T10`, `wall_bounded_argv` | "A bound a capsule can reach is not a bound" holds under any provisioning model |
| Descriptor closure above the standard streams — the `/proc/self/fd` sweep, marked in the parent before the fork | `bubblewrap.rs` `T9` | An already-open descriptor is not a namespace, a mount or an environment entry; no provisioning choice changes that |
| Standard-stream ownership — parent-owned endpoints, no inbound channel | `bubblewrap.rs` `T8`, row 12 | Intrinsic to running anything untrusted |
| Process teardown, and termination classification as a closed table | `classify_termination`, the measured termination table (`D12`) | Signals and exit shapes are the platform's, not the architecture's |
| Credential posture — a declared, **non-configurable** uid/gid, and capability confinement | `CAPSULE_UID`/`CAPSULE_GID`, rows 13 and 14 | An operator-chosen identity is a second way to weaken a capsule under any design |
| `no_new_privs` | `conformance.rs` `Unrowed` (`EX-12`, `sec-9` `R8`) | Intrinsic property. Recorded as an unrowed *observation* because bubblewrap sets it unconditionally, so no differential removal exists — a fact about the mechanism, not a gap in the design |
| The inner layout as **reserved** destinations — `/source`, `/capsule`, `/agent`, `/proc`, `/dev`, `/tmp` | `backend.rs` `INNER_*` | Any confined workspace has an inner layout, and shadowing the input is a substitution attack regardless of where the input came from |
| The rule that **the mount set is the confinement**, so an unvalidated placement is an unconfined capsule with a confined shape | `backend.rs` `CapsulePlacement` doc | Intrinsic. Note: the *apparatus* enforcing it is tier 3 — see there — because its input is what `F-2` creates |
| A confinement contract that is **property-shaped, never flag-shaped** | `backend.rs` `CapsuleBackend` | A flag-shaped vocabulary is one mechanism's and cannot be asked of a second |
| The fourteen properties of Table A | `conformance.rs` `Property` | These are claims about a running confined process. They are what a successor must still prove; only their *fixture* is architecture-shaped |
| Differential probe/control, and `Unproven ≠ Violated` | `RowVerdict` | The distinction between "the property held" and "removing it changed nothing" is method, not architecture, and it is the single most transferable idea in the suite |
| Liveness first, observation second | `conformance.rs` § classification | A payload that never ran and a payload that was denied are opposite facts; conflating them is the default failure mode of any probe suite |
| A removal named by the **property**, never by the flag; every removal states what it does *not* change | `PropertyRemoval` | A removal that moves two things names no mechanism when its row fails |
| Weakenings **recorded rather than merely absent** | `Unrowed`, `the_identity_is_exactly` | An unrecorded weakening reads as an oversight; this is the discipline that made `F-3`'s soundness limb findable at all |
| The suite's self-guard: no `#[ignore]` in the crate stands in for a claim unless reasoned `"instrument: …"` | `conformance.rs:11163` | Mechanism-independent, and it is the guard that keeps a suite honest as it ages |
| Capability probes must be **structural, not existential** | `ISS-339`'s measured lesson | A `path_exists("/bin/sh")` guard passes on a host that cannot exec it — true of every host, under every architecture |
| Unavailability names what is missing **and the remedy** | `Availability`, `SHELL_REMEDY`, `REV-051`'s `setsid`/`socat` contract | `POL-002` facet 3: a red row for a reason that is not a defect must say so |

### Tier 2 — incumbent transaction

**Returns unsolved.** Every entry here is a problem the successor will meet
because abandoning the capsule dissolves nothing about it. Several are visibly
hard; that difficulty is *not* evidence of relapse into this failure. Read this
list as the standing debt the programme did not create and did not discharge.

#### Owned inside the crate or the spec today

| Obligation (successor-neutral wording) | Where it sits now | Note |
|---|---|---|
| Provision a **fresh, deterministic workspace from an immutable base**, and prove two units of work share no writable state | `provision.rs` steps 8–13; `conformance.rs` Table B's five freshness axes | The capsule framing dies; freshness does not. The incumbent solves a weaker version with worktrees |
| Publish or adopt a **per-base immutable input**, once, under concurrency | `provision.rs` `T4` publish-or-adopt | This is deterministic workspace creation plus concurrency, both on the returned list |
| **Own a workspace exclusively**, and roll back completely on any later failure | `provision.rs` `create_exclusively`, the creation token, `roll_back` | "A refused provision leaves no transaction and removes nothing it did not create" is an orchestration invariant, not a confinement one |
| Bind what one unit of work is **about** — base, phase identity, bounds, mechanism, policy identity | `transaction.rs` | The incumbent's version is `src/worktree/dispatch_record.rs` |
| Decide **where isolated work lives** on disk, with a platform-sensible default | `config.rs` capsule root, XDG resolution | Returns immediately; the incumbent answers it in-repo |
| **Advisory capacity**: warn before starting, halt on exhaustion, never evict or auto-delete work | `capacity.rs`, `REQ-461` | Any successor provisioning fresh workspaces meets this. See divergence 1 |
| **Property equivalence across platforms**, and stronger isolation on demand | `backend.rs` `CapsuleBackend`, `REQ-459`'s equivalence clause | Explicitly on the returned list ("non-Nix platforms") |
| **Interpretation authority** — who may execute worker-controlled content — and monotonic phase restriction of it | `src/interpretation.rs` (1,624), `provision.rs` steps 4–6, `REQ-449` | **Conditional (`F-12`).** Carries iff the successor retains a trusted/untrusted execution split. Answer that question before deleting anything here; the *mechanism* is separately judged disproportionate on its own evidence, which is a different finding |

#### Owned outside both — the scope ruling's additions

| Obligation | Home today |
|---|---|
| Deterministic workspace creation and provisioning | `src/worktree/create.rs`, `fork.rs`, `provision.rs`, `mod.rs` |
| Agent lifecycle and harness-agnostic spawn | `src/worktree/subagent.rs`, `src/dispatch.rs`, `scripts/pi-spawn-confined.sh`, `ADR-011` |
| Control-plane communication and the per-phase funnel | `src/dispatch.rs` (12,463), the `worker_commit` / `observation_record` MCP seam |
| Result transport and hostile ingestion | `src/worktree/import.rs`, `dispatch import`, the `worker_commit` gate; `SL-241`'s 16-row hazard matrix as evidence |
| Teardown and reaping | `src/worktree/gc.rs`, `claim_lock.rs`, `dispatch reap` |
| Recovery, staleness, and repair | `src/worktree/dispatch_record.rs`, `inventory.rs`, `land.rs`; the candidate engine (`SPEC-022`) |
| Concurrency — claims, markers, prefix collision, allowlists | `src/worktree/claim_lock.rs`, `marker.rs`, `jail_prefix.rs`, `allowlist.rs` |
| Confinement on non-Nix hosts | `src/worktree/jail.rs` (2,218), `pretooluse.rs`, `flake.nix`; `IMP-426`'s microVM spike |
| Admission: journal-before-mutation, expected-tip compare-and-swap, idempotent replay | `SPEC-022` substrate, partly shipped |

### Tier 3 — self-inflicted

**Dissolves with `F-2`.** Each entry carries the argument that its *requirement*
— not merely its code — dies when environments are selected rather than derived.
This is the only tier the verdict may call waste.

| Mechanism | Home | Why the requirement dies |
|---|---|---|
| The `readable-roots` / `closure-roots` / `closure-resolver` configuration surface | `config.rs` `KEY_READABLE_ROOTS`, `KEY_CLOSURE_ROOTS`, `KEY_CLOSURE_RESOLVER` | It exists to let an operator declare **arbitrary host paths** from which an environment is assembled. Under selection the bind list is a manifest the provisioner already holds; there is nothing for an operator to declare, so the keys, their emptiness rules and their six refusals have no subject |
| Declared-entry resolution and probing — `resolve_or_refuse`, `resolved_readable_root`, `readable_paths`, `readable_set` | `bubblewrap.rs` `T5` | Resolution-before-validation exists because bwrap dereferences the source of a `--ro-bind` and a declared root may point elsewhere (`RV-346` `F-1`). A provisioner-emitted manifest names realised paths; the hazard has no input |
| Closure expansion of a **declared host root** — `resolved_closure_root`, `expand_closure_root`, `ClosureQuery`, `SpawnedClosureQuery` | `bubblewrap.rs` `T5` | The requirement is "given an arbitrary host path, discover what else must be bound for it to work". Selection replaces it with "realise the thing you chose". **Salvage note:** `closure_members`' *parse* of a resolver's output into paths is the reusable fragment (salvage 2's direction — take a nix closure); `expand_closure_root`'s *expansion of a declared root* is the part that dies |
| Resolver admission against the interpretation policy | `provision.rs` step 6, `admit_resolver` | A step whose whole subject is "may this project-supplied command be run to compute a bind list". No resolver, no admission question |
| Inner-`PATH` synthesis — `derived_inner_path`, `is_within`, `render_path_list` | `bubblewrap.rs` `T6` | Keeps host `$PATH` entries that lie beneath a bound path. `SL-252`'s `DEC-187` already killed it in favour of discovered reach. A selected environment carries its own `PATH` |
| Validation apparatus over **operator-declared** placement entries — the reserved-destination, inner-collision, bidirectional-overlap and forbidden-scope rules, and their mirror-defect guards | `backend.rs` `CapsulePlacement::try_new`, `ForbiddenScopes`, `PlacementRefusal` | The *invariant* (a placement must not expose canonical state, credentials, or a sibling transaction) is tier 1 and stated there. This apparatus is tier 3 because its **input** is what `F-2` creates: arbitrary declared paths that can name anything. Under selection the input is a trusted manifest and the check collapses to an assertion. `RV-346` `F-10` and `F-25` are both defects of the untrusted-input form |
| The capsule-input half of `HostFacts` — `canonicalize`, `path_exists`, and the environment read | `host.rs` | Three of the four impure inputs exist to resolve and probe declared host paths. `available_bytes` is tier 2 and stays |
| The conformance fixture's readable-set derivation — binding whole host top-level roots | `conformance.rs` fixture, `ISS-341` | This is `F-3`'s soundness limb in one line: the fixture derived its inputs from the host exactly as production did, so its capsules were more permissive than the ones it certified. It is the design's own error reproduced inside the instrument that was supposed to detect it |
| Row 2's per-root exec coverage set | `execs_only_what_is_bound`, `ISS-340` | Derives coverage from the inner `$PATH`, which is tier 3; it is unsatisfiable on a real host for that reason |
| The whole of `SL-252` — all eleven inquiry nodes, six still open | live design run `dr-019feb6b…`, `DEC-184`…`DEC-188` | Every node is a question about the *unit of a derived readable set*, the *inner `PATH` it empties*, or *who computes a closure of a host path*. Not one of them is a question a selection-based successor can be asked. `DEC-188`'s "that is a slice, not a phase" is `F-8` in the accused's own words |
| The root package's `lib` target, its curated leaf-only export set, the `EXPORTED` assertion, and the `sec-6`/`sec-9` invariants guarding them | `src/lib.rs`, `SL-248` design `sec-6` | `F-12`/`F-5`: the tangle reached into the root package's *target layout* to serve one capsule-only module. **Conditional twice over** — it dies with `crates/doctrine-control`'s consumer, but `IMP-404` (`SL-112`'s deferred engine/leaf extraction) is an independent motive for a library boundary that survives. Decide it separately, as `F-12` instructs |
| `doctrine-control`'s `provision` verb and its CLI shell | `main.rs` | The entry point of the dissolved transaction. `backend verify` is the exception: "report a mechanism's admission verdict on the host that will run it" is tier 1 and should outlive the binary |

### Where this diverges from the Salvage manifest's *Discard* list

The Discard list was written before this partition existed and answers a
different question (what code goes). Three of its parenthetical tier labels do
not survive the test above. Recorded rather than silently overridden.

1. **`capacity.rs` (391) is listed as tier 3; it is tier 2.** `REQ-461`'s
   obligation — warn against a configurable expectation, halt on exhaustion,
   never evict — is met by any successor that provisions fresh workspaces on
   disk. The code is discardable; the problem is not.
2. **`provision.rs`'s thirteen steps are listed as tier 3; the protocol splits.**
   Steps 1, 6 and 7 are tier 3 (config lists, resolver admission, readable-set
   expansion). Steps 8–13 — publish-or-adopt, exclusive creation, the creation
   token, rollback, workspace materialisation — are **deterministic workspace
   creation and concurrency**, which is the reviewer's own tier-2 list. Delete
   the file; keep the obligations on the register.
3. **`SPEC-030`'s fourteen requirements are listed as tier 3; none of them is.**
   See the roster below. This is the largest correction in this section, and it
   matters most: withdrawing the spec (step 3) must not be read as dissolving
   the obligations it names.

`conformance.rs` is not a divergence but needs its split stated: the file is
correctly on the Discard list, while its method (differential arms, the verdict
algebra, the self-guard, recorded weakenings) is tier 1 and its fixture is tier
3. `F-3`'s soundness limb condemns the fixture. `F-11` already withdrew the LOC
argument as proof of anything.

### `SPEC-030`'s roster, requirement by requirement

The declared mechanism set. Tier, then the obligation restated without capsule
vocabulary — which is what a successor actually inherits.

| Req | Label | Tier | The carried obligation |
|---|---|---|---|
| `REQ-448` | `FR-001` | 2 | One trusted writer owns canonical mutation; a worker's output, exit status and prose are evidence, never authority. The incumbent has this problem today (`ADR-006`) |
| `REQ-449` | `FR-002` | 2 (conditional) | Bind an immutable base to a unit of work, and let a phase only narrow the project's execution policy. The base half returns unsolved; the interpretation half is `F-12`'s open retention question |
| `REQ-450` | `FR-003` | 2 | Fresh mutable state per unit of work; no formal resumption of harvested state. Freshness is orthogonal to how the environment is obtained |
| `REQ-451` | `FR-004` | 2 | Get a result out of a less-trusted execution context as bounded, quiescent, parent-owned bytes. Bundle-as-transport is one mechanism; the requirement is result transport |
| `REQ-452` | `FR-005` | 2 | Never let trusted Git use a worker-authored repository as repository or working context. `SL-241`'s hazard matrix is the evidence and survives as knowledge |
| `REQ-453` | `FR-006` | 2 | Check the actual delta against the contract from objects, not from prose. The incumbent's `authored-divergence` probe is a weaker form of the same |
| `REQ-454` | `FR-007` | 2 | Verify the exact artefact that could land, not an approximation of it |
| `REQ-455` | `FR-008` | 2 | Journal intent before mutation; advance by expected-old compare-and-swap; replay idempotently. Partly shipped in `SPEC-022` |
| `REQ-456` | `FR-009` | 2 | Route stale work through object-only three-way classification with durable clean/conflicted state |
| `REQ-457` | `FR-010` | 2 | Repair as a new unit of work; cleanup requires mechanically recorded incorporation or explicit operator disposition |
| `REQ-458` | `FR-011` | 2 | Durable admission truth, small and permanent; unresolved work is never automatically destroyed; exhibits expire separately |
| `REQ-459` | `FR-012` | **1 / 2 / 3** | **Splits.** The fourteen properties are tier 1. Cross-platform property *equivalence*, and the refusal to claim an unmeasured backend, are tier 2. The fixture that derives its readable inputs from host roots is tier 3 (`ISS-341`), and so is row 2's coverage rule (`ISS-340`) |
| `REQ-460` | `NF-001` | 2 | No force, no silent resolution, no resumption of harvested state, no automated loss of unresolved work. Pure orchestration safety |
| `REQ-461` | `NF-002` | 2 | Warn early against a configurable expectation, halt visibly, reserve/evict/backpressure nothing |

**Zero requirements are tier 3 outright; one splits.** That is not a rescue of
`SPEC-030` — the spec is still `active` with no shipped consumer and step 3
should withdraw or reduce it. It is the statement that withdrawing the document
returns thirteen-and-a-half problems to the register unsolved, and that the
successor's design must answer for them rather than inherit a clean sheet.

### What step 4 does with this

The triage rule, stated once so `cluster:capsule` can be sorted mechanically:

- An item is **dissolved** iff the mechanism it is about appears in tier 3 above
  — and the item's own argument must name that entry. An item whose tier-3
  claim rests only on "the capsule is abandoned" is not dissolved; it is tier 2.
- An item is **carried** if its mechanism is tier 1 or tier 2. Carried items
  should be re-homed onto the obligation, not onto the capsule.
- `F-4`'s items are **live** and already discharged by step 1.
- Where the argument does not close, the item is **carried**. Under-dissolving
  costs a register entry; over-dissolving loses a real problem and hands the
  successor a clean sheet it has not earned.

`SL-252` is dissolved in full, by the eleven-node argument in the tier-3 table.

### The tier-2 register, as returned-unsolved work

Recorded here as `F-13`'s verification asks. The successor inherits, unsolved
and not by its own doing: **deterministic workspace creation** · **agent
lifecycle and spawn** · **control-plane communication** · **result transport
and hostile ingestion** · **conformance of a result to its contract** ·
**verification of the exact landable artefact** · **admission journalling and
compare-and-swap** · **staleness, conflict and repair** · **retention of
unresolved work** · **teardown** · **concurrency** · **capacity** ·
**confinement on non-Nix hosts** · **stronger isolation on demand** ·
**interpretation authority, conditionally**.

Meeting difficulty in any of the above is not relapse. It is the bill the
programme never paid and never could have, because none of it was the
programme's to solve.
