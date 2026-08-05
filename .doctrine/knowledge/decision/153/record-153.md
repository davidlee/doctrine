# DEC-153: Two binaries split at canonical mutation

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The decision

`QUE-207` asked where the capsule programme's authority boundary gets expressed —
in the environment, the crate graph, binary absence, or a protocol. It offered
three frames: **A** one `doctrine` binary with privilege from environment and
filesystem access; **B** one workspace, separate binaries over shared crates; **C**
a separate control-plane system.

**Answer: B, and the split line is *canonical mutation*, not orchestrator-versus-worker.**

- **`doctrine`** — today's agent-facing binary, unchanged. Every verb that exists
  now: slice, design, plan, knowledge, memory, backlog, spec, adr, review, search,
  library, prompt/hymns, status, next. It runs on the control host **and inside the
  capsule**, as the same binary, with no modes and no per-verb refusals.
- **`doctrine-control`** — new code only: the SPEC-030 transaction verbs (resolve,
  provision, launch, snapshot, harvest/freeze, conform, orchestrate verification,
  normalize, journal, admit/integrate, close, cleanup). Present only on the control
  host; never mounted into a capsule.

The verb sets are disjoint. Nothing migrates out of the existing binary — the split
line falls *between* existing and new code rather than through it.

## What this does NOT decide

**Whether orchestrator and worker differ in authorization.** They may not. A capsule
inhabitant is not assumed to be a narrow implementation worker: it may phase-plan,
navigate entities, research, mint decisions, resolve hymns, search, and read and
write memories. Those are ordinary workflow operations against the capsule's own
checkout, and per SPEC-030 (via REV-046 § PRD-015) capsule writes are **non-canonical
until trusted-side admission**. That question stays open and is not closed by
implication here.

The only division this record relies on is the one `REQ-448` already settles: the
trusted control plane is the sole **canonical mutation** authority. Everything else
— however much workflow authority the capsule inhabitant turns out to hold — sits on
the `doctrine` side.

## Rationale

**1. Option A re-creates the failure the programme exists to kill.** RFC-025
§ Bounded authority: *"Authority should be absent by construction where possible,
rather than inferred from fragile process identity."* A's privilege-from-environment
is inference from process context — the same shape as worktree marker identity,
`DOCTRINE_WORKER`, and the SubagentStart hook stamp, all three on REV-046's retire
list precisely because inferred authority is fragile. Adopting A would ship the
replacement architecture with the retired pattern at its centre.

**2. B is what makes REQ-459's property suite tractable.** Under B the assertion is
"no control-plane binary is reachable in the capsule's mount set" — one check,
backend-agnostic. Under A it becomes "the present binary refuses every privileged
path", a combinatorial enumeration that grows with every verb added and rots when
one is forgotten. That suite is the admission gate every future backend must pass;
built on refusal-enumeration it silently stops meaning anything. This argument gets
*stronger* the larger the legitimate capsule-side surface turns out to be.

**3. The one-way dependency rule is mechanically enforced from day one.** Cargo
refuses package-level cycles, so with `doctrine-control` depending on the root
package, the root can never depend back on control. `QUE-207`'s
`agent-facing tools -X► control-plane implementation` arrow becomes a compiler
error rather than a convention. Control-depends-on-shared is the direction the frame
explicitly permits.

**4. Migration asymmetry favours deciding A-vs-B now.** A→B is a refactor of a tree
that already has a workspace (`crates/cordage`) and a stated one-way rule; B→C is the
extraction `QUE-207` leaves open; A→C is the expensive path. Deciding B keeps the
cheap side of that asymmetry.

## The cheap path, and its price

There is no `src/lib.rs` today — the root package is bin-only, ~90 modules with
ADR-001 layering but no crate boundaries. **v0 does not extract crates.** Add a lib
target to the root package and give `doctrine-control` a workspace member that
depends on it.

Honest costs, none of them hidden:

- `doctrine-control` links the root crate and therefore **inherits every embed root**
  (`install/`, `plugins/`, `memory/`, `publication/`, `web/map/dist/`,
  `templates/mcp.ts`). Binary bloat, not a correctness defect, and trimmable by the
  extraction below. An earlier reading of this record predicted zero embeds for the
  control binary; that is wrong under the cheap path.
- The three-place embed discipline doubles — `Cargo.toml` `include`, `srcWithDist`
  in `flake.nix`, and the release/binstall asset path — and its failure mode is a
  silently hollow binary with no compile error.
- The binstall asset contract is a three-way change (`Cargo.toml`, `install.sh`,
  `release.yml`) that must move together.
- `just check`'s per-crate carve-out (currently skipping `cordage`) gains a second
  case and needs to stay comprehensible.

## The improvement path

`IMP-404` — execute `SL-112`'s deferred engine/leaf crate extraction. SL-112 faced
this exact fork, shipped the `syn` dependency-fitness gate
(`tests/architecture_layering.rs`, `LAYER_MAP` + frozen `ACCEPTED_VIOLATIONS` +
intra-tier cycle ratchet, under `just gate`) and deferred the crate split on the
reasoning that its layer map de-risks the later extraction. Extracting along those tiers is
where `doctrine-control` stops inheriting embeds, because the embed roots hang off
command-tier and engine modules the control plane does not need.

Deferring is safe: extraction is behaviour-preserving, so the decision does not get
harder by waiting.

## Residuals

- **`doctrine serve --mcp` placement** is a provisioning-manifest question, not a
  crate question. If a capsule worker uses MCP, the server launches *inside* the
  capsule from the capsule-side binary with capsule-local authority. `IMP-401` is the
  evidence that today's arrangement — server outside every sandbox, resolving against
  the primary repo root — is a live confinement hole. `map serve` is read-only and
  human-facing; it stays with `doctrine` and never enters a capsule.
- **Reading trusted-side state from inside a capsule.** An agent phase-planning in a
  capsule may want the funnel position or the admission journal. Writes are cleanly
  out; reads are not obviously. Expected resolution is pre-distillation into the work
  contract at provisioning (fits DEC-134); a read protocol would be C-creep arriving
  through the back door. Not settled here.
- **Sequential id allocation is the real cross-capsule merge hazard**, not memories.
  Memories are uuid-addressed, so parallel capsules minting them produce disjoint
  files and the merge is a union. `RV-NNN` is the worst case — allocation collides
  *and* the ledger is turn-based with mutable dispositions. `DEC-NNN` / `QUE-NNN` /
  `SL-NNN` have the allocation half without the ledger half. ADR-006 already carries
  trunk-side ID allocation as a preserved invariant; under capsules "trunk-side"
  becomes control-plane-side. Uuid-addressed-until-admission generalises the memory
  trick; pre-allocated id blocks are the alternative. Slice 2–4 design work.
- **Egress.** Most capsule-side research is local code and doc inspection and needs
  no network; some is web. `REQ-459` owns the posture knob, `IMP-397` / `QUE-204` own
  the allowlist policy.
- **The daemon axis stays deferred.** Nothing in v0 requires a persistent service;
  keep process-model separate from binary-count so C's costs are not imported by
  accident.

## Settling conditions

`QUE-207` named four. Their disposition:

1. *Enumerate the capsule-side verb set.* Reframed and dissolved. The record's test
   was whether the set is "small **and disjoint** from provisioning, harvest, and
   admission". It is **not** small — a capsule inhabitant may run most of the
   workflow — but it **is** disjoint, because none of it touches canonical state. A
   large disjoint set keeps B cheap and makes A more expensive.
2. *Price the distribution multiplier.* Priced above, and worse than first predicted.
   Accepted with `IMP-404` as the mitigation.
3. *Settle jointly with the provisioning-manifest question.* Resolves in B's favour
   rather than blocking: under B the manifest omits `doctrine-control` by default and
   would have to deliberately add it. B fails safe; A fails open.
4. *Name where `doctrine serve --mcp` and `map serve` land.* Named under Residuals.
