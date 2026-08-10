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
