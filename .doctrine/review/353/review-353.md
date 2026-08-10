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

### Judgement

**Guilty.** The capsule programme is heresy against its own founding claim, and
the heresy is architectural, not clerical.

Let the record show what is *not* charged. `SL-248`'s code is careful. Its
constants are named, its refusals are distinguishable, its layering respects
`ADR-001`, its ledger is honest, and its authors caught their own errors and
recorded them — `DEC-186` was withdrawn as refuted by the very hand that wrote
it, and `DEC-188` states the programme's failure more plainly than this tribunal
could. This is a well-built wrong thing. The inquisition is convened against
the design precisely because the workmanship cannot be blamed.

The root heresy is **F-2**, and everything else descends from it. Doctrine set
out to *manufacture* an execution environment by discovering it from arbitrary
host paths, when it should have *selected* one from a self-contained mechanism
and confined that. From that single inversion follows the whole edifice: because
the environment is discovered, its contents must be excavated from `$PATH`
(`system_readable_roots`); because excavation is host-shaped, a generic escape
hatch is invented (`closure-resolver`, an operator-supplied executable **inside
the trust boundary**); because the escape hatch cannot know a project's
toolchain, every operator must hand-declare it (`[interpretation]`, no
defaults); because a hand-declaration is a security surface, 1,624 lines of
validation algebra are purchased to police it; and because nothing about any of
this is provable by construction, 14,252 lines of apparatus are built to measure
it — apparatus which then measured the jail it was born in and returned nineteen
green rows for a property its own fixture falsifies.

Sixty thousand inserted lines. Zero deleted. One binary with two verbs,
`provision` and `backend verify`, wired to nothing. Fourteen requirements, all
`pending`. Twenty-four open follow-ups, one of them a release-blocker. The host
project's own commit gate held permanently red by an "additive and unused"
subsystem. And a plan under which the first deletion — the entire justification
for the work — is the last thing anyone will write.

The Owner's five-minute baseline is the indictment's final exhibit: *clone the
repo, use the existing flake, bwrap it, run the agent.* Working confinement, in
five minutes, with an existing mechanism. Against that, sixteen thousand lines
of probes and ten phases of transaction lifecycle are not engineering rigour.
**Burn it.**

> *Igne natura renovatur integra.* Let the pyre be built high, and let the
> fourteen thousand lines feed it first.

### What survives the fire — the salvage manifest

The fire is not indiscriminate. An inquisitor who burns the true with the false
is merely an arsonist. These survive:

**1. The Owner's own baseline — the actual architecture.** Clone, flake, bwrap,
five minutes. It is not a prototype of the answer; it *is* the answer, and it
needs automating and a control plane, not replacing. Add the one flake line for
network isolation. This is salvage rank one and everything else is subordinate
to it.

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

**Ash, and to be treated as such:** `conformance.rs` (14,252), `provision.rs`'s
thirteen-step protocol with its publish-or-adopt and creation-token machinery
(1,922), `src/interpretation.rs` (1,624), `capacity.rs` (391), `SPEC-030`'s
fourteen pending requirements, and `ADR-020`'s authority-boundary decision.

### Penance, in order

1. **Immediately, before anything is deliberated (`F-4`).** Remove
   `crates/doctrine-control` from `default-members` (`ISS-343`) and put the
   live-bwrap conformance tests behind explicit opt-in (`ISS-342`). The project's
   gate must be green and its release unblocked while the successor is designed.
   *Verification:* `just gate` green off-jail; `doctrine check gate` clean.

2. **Vacate the governance (`F-6`).** A `REV` against present-tense governance:
   `ADR-020` superseded (not amended — the central choice is being reversed);
   `SPEC-030` withdrawn or reduced to what a successor actually specifies;
   `REV-046` closed unapplied. `RFC-025` **stays** as the historical record and
   receives the post-mortem. *Verification:* `doctrine spec validate` clean; no
   `active` spec with zero shipped consumers.

3. **Triage the register, do not work it (`F-8`).** Sort `cluster:capsule` into
   dissolved / carried / live per `F-8`'s disposition. `SL-252` is dissolved.
   *Verification:* no `open` `cluster:capsule` item lacking a triage disposition.

4. **Harvest before the fire (`F-3`, salvage 3–4).** Land salvage items 4 and 3
   as memories and `EVD` records *before* any code is deleted. Knowledge burnt
   with its carrier is knowledge re-purchased at full price.

5. **Design the successor under an inverted rule (`F-1`, `F-5`).** Three
   separately-shippable concerns — provisioning (select + realise), confinement
   (apply the platform sandbox), control-plane transport. Two provisioning
   options as the Owner framed them: (a) nix+bwrap / seatbelt; (b) microVM or
   `systemd-nspawn` into a configured base image. **No slice lands that does not
   retire incumbent mechanism in the same slice.** First increment must be
   statable against the five-minute baseline or it is refused at design gate.
   `src/interpretation.rs`'s deletion is the correct opening move.

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

### Tolerated

Nothing. Every charge is upheld, and the one finding disposed `fix-now` is
triage, not tolerance.

> **HERESIS URITOR; DOCTRINA MANET**
