# RFC-025 red-team — adversarial pass over the capsule authority model

Paper adversarial review of the capsule model as sharpened by discussion and
the mechanism census (`mechanism-census.md`): uniform subprocess workers
(`claude -p` / codex / pi) under OS sandbox in fresh clones at a pinned base;
control plane harvests → verifies → normalizes → admits (QUE-200 pipeline);
interactive session control-plane-side. Findings are things the spike's
happy-path and listed hostile probes would *not* surface on their own.

Severity: **blocker** (model unsound until addressed) · **design-change**
(model sound, a stated piece must change) · **probe** (becomes a probe
obligation) · **residual** (accepted, named) · **note**.

## RT-1 — Verification is arbitrary code execution on the trusted side — **blocker**

RFC-025 places "run verification against the exact candidate" in the trusted
control plane. But verification *executes the candidate*: `cargo test`,
`build.rs`, proc-macros, `flake.nix` evaluation, a direnv `.envrc` at
checkout — all worker-authored. A hostile phase does not need to escape the
worker sandbox at all; it plants its escape in the tree and waits for the
control plane to "verify" it unconfined. Checking out and building the
candidate on the trusted side collapses the entire authority boundary.

**Resolution:** verification runs in a capsule too — a fresh *verify capsule*
provisioned at the normalized candidate OID, same sandbox as a worker, results
reported out as evidence. The control plane trusts the *runner* (its own
infrastructure), never the code under test. Admission consumes the verify
capsule's report plus the OID identity; nothing from the candidate tree is
ever executed, evaluated (nix), or auto-loaded (direnv) outside a sandbox.
Cost: one more cold build per phase — C1's measurement (~3 min) says
tolerable; capsule reuse of the *worker* capsule for verify is unsound
(worker-controlled environment can fake outcomes), but note the worker
already ran the suite informally — the verify capsule is the attested run.

**Scope honesty:** capsules make verification *observe the exact candidate,
confined*. They do not make the verdict honest when the worker authors the
tests being run — a lying test suite passes in any sandbox. That gap is
RFC-022 (agent trust) / RFC-023 (adversarial TDD) territory; capsules must
not be claimed to close it.

**Probes added:** candidate whose `build.rs`/test writes outside the
workspace or phones home during verify (must be contained by the verify
capsule); candidate whose `.envrc`/`flake.nix` differs from base (must never
be evaluated trusted-side).

## RT-2 — Q8 admission must BE the candidate layer, not resemble it — **design-change**

Parallel results from one accepted base: result #1 admits and integrates;
result #2's base is now stale. The existing machinery already models this
exactly: `candidate create` (3-way merge of source onto base, `Conflicted`
rows refused for propagation, hand-resolve documented), `admit`-by-OID,
integrate refusing a moved trunk with supersede-candidate guidance
(REQ-316/317/320). The census keeps all of it (D5–D7).

The failure mode is building a parallel "admission" implementation in the
capsule pipeline that re-derives this — port/reuse is acceptable, a redo is
total failure (operator ruling, 2026-08-01). **Constraint:** QUE-200's
"normalize / admit" steps are *bindings to* `candidate create` / `candidate
admit` / journaled integrate — the capsule harvest produces a verified
source for the existing verbs, nothing more. The C3 rig must call the
existing verbs; a rig that hand-rolls merge/admit logic is disqualified as
evidence even if it passes.

**Probe added:** two capsules from one base with conflicting edits — the
pipeline must produce a `Conflicted` candidate row / supersede guidance and
halt; any auto-resolution is a probe failure.

## RT-3 — The token-efficiency invariant needs a topology decision — **design-change / open**

RFC-025 claims conversational context persists across phase transactions
while capsules are rebuilt. Two topologies satisfy the words:

1. **Control-plane-persistent (recommended v0):** the interactive session
   lives in the primary tree; workers are headless, fresh-context per phase.
   This is today's cost shape exactly (workers are already per-phase; the
   orchestrator's context already persists), so capsules cause **no token
   regression** — the invariant holds trivially.
2. **In-capsule-persistent:** an interactive session inside a capsule whose
   filesystem is torn down and re-provisioned under it between phases. No
   harness documents surviving a wholesale workspace swap (stale file-state
   caches, cwd invalidation — RFC-018's uneven-semantics territory). Unproven
   and not needed for v0.

Choosing (1) also settles RT-3's dependents: **escalation** (a headless
worker cannot take a mid-phase user turn; escalation = ring doorbell + halt,
control plane either answers via session-resume (`claude -p --resume`-class
mechanics, to be probed) or re-provisions with an amended contract) and
**C11's "whence interactivity"** (the interactive surface is the control
plane, full stop — a human who wants to pair *inside* a capsule is a later,
separate design). The RFC's "dispatch-configured interactive Claude session"
transitional step should be re-read under this ruling: it configures the
*control-plane* session, not a capsule-resident one.

## RT-4 — Coordination artifacts are parsed hostile input — **design-change**

The doorbell is payload-free, but `status.toml` / `result.toml` /
`escalation.toml` / `audit/` are capsule-written files the control plane
reads and parses. Attack surface: symlinks at artifact paths (control plane
reads through a link out of the capsule, or writes a harvest receipt through
one), path-valued fields (`../../…` traversal), oversized files, parser
bombs. The RFC's "inspect, don't trust" covers *semantics*; it does not yet
state *ingestion hygiene*. **Constraint:** artifact reads resolve strictly
inside the capsule root with symlinks refused (O_NOFOLLOW-class discipline),
size-capped, schema-validated; artifact content is never authoritative for
anything admission depends on (OIDs come from git ingestion, not from
`result.toml`). **Probes added:** symlinked artifact, traversal path in a
field, oversized artifact.

## RT-5 — Harvest must be OID-pinned against a live capsule — **probe**

Nothing guarantees the capsule is quiescent at harvest: the worker process
(or a stray child) may still be mutating the repo when the doorbell rings.
Fetch/bundle ingestion is safe *if* harvest first reads the result ref once,
pins the OID, and operates on objects thereafter — a mutating capsule can
only make the pinned tip stale, never corrupt the harvest. Tree
materialization has no such pin (copying a mutating tree is a torn read) —
a further strike against option 3 in QUE-200. **Probe:** harvest while a
background process is actively committing; result must equal the pinned OID
exactly.

## RT-6 — Doorbell: no authority, idempotent, pollable — **design-change (invariant statement)**

Three properties to state as invariants rather than leave implicit: a ring
carries **no authority** (a spoofed/duplicated ring causes at most a wasted
inspection); harvest is **idempotent** (journal-keyed, so duplicate rings
and re-runs are no-ops); liveness **must not depend on the doorbell** (the
control plane can poll capsule state cold — a lost ring is a delay, never
lost work). RFC-025 already implies all three; the spike should treat them
as pass/fail criteria for the notification design, not aspirations.

## RT-7 — Capsule reuse silently weakens the evidence chain — **residual, bounded**

The RFC permits operational capsule reuse across serial phases. A reused
capsule's starting state is not reconstructible from `accepted tree +
manifest` — it includes whatever phase N left behind, so phase N+1's
"contracted base" evidence is weaker than a fresh capsule's even when the git
base is identical. Acceptable transitionally, but it must be *visible*:
`contract.toml` records provisioning mode (`fresh` | `reused-from <capsule,
phase>`), and audit can weigh it. Best-effort teardown remains best-effort;
the recorded mode is what keeps the evidence honest.

## RT-8 — The orchestrator's authority is untouched — **residual, named**

Capsules bound *workers*. The control-plane orchestrator — itself an LLM
agent mediated by a human — retains full authority over canonical state,
exactly as today. The RFC's mediated→advisory migration is the long-term
answer; until then, claims for the capsule model must say "worker authority
bounded by construction", never "agent authority bounded". RFC-022 remains
open and is the governing artifact for the other half.

## RT-9 — Forensic archive needs a storage-tier ruling — **design-change (small)**

Capsule bundles + worker history + logs are binary, per-phase, and sizeable.
Committed under `.doctrine/` → repo bloat forever; runtime tier →
`rm -rf`-able, i.e. not evidence. Neither default is right. Needs an explicit
retention policy under the ADR-019 asset-policy lens (likely: a gitignored
archive dir with a configured retention window, referenced by OID from the
committed admission journal — the journal is the evidence, the archive is the
exhibit). Flag for the RFC's provisioning-manifest / evidence sections.

## RT-10 — In-capsule doctrine writes get silently stripped — **design-change (small)**

A worker running the in-capsule `doctrine` binary writes capsule-local
`.doctrine/` — which admission then rejects/strips as a forbidden path. For
observations this recreates the exact looks-like-success-loses-data hazard
the current fork rules exist to prevent (boot § Instrumentation). The
capsule-side capture path must therefore write to the *coordination artifact
area* (outside the repo tree, in the harvest manifest), or workers report
frictions in the structured hand-back for control-plane recording. Same
ruling covers any future in-capsule entity write: capsule `.doctrine/` is
scratch by definition; durable capture rides the harvest manifest only.

## RT-11 — Notes (minor, measured not argued)

- **Full clone, not shallow (v0):** worker-side merge-base/ancestry ops and
  honest history evidence want full history; local clones are cheap (C1).
- **Disk pressure:** N parallel capsules × in-tree `target/` (~GB each) with
  gc-at-slice-close; measure in the spike, add early teardown only if it
  bites.
- **Two-trunk accepted base:** doctrine's edge/main split means "accepted
  canonical commit" needs naming per project; the contract pins an OID so
  the ambiguity exists only at contract-authoring time (the existing
  promote-edge-to-main ritual survives unchanged).
- **Concurrent-same-slice guard** (census C11) relocates cleanly to the
  work-contract registry: one active contract per phase, CAS on the registry.

## Consolidated probe additions

Beyond RFC-025 § Investigation method and the census additions (conflicting
pair, credential boundary): verify-capsule containment (hostile
`build.rs`/test side effects; hostile `.envrc`/`flake.nix` never evaluated
trusted-side), harvest-under-mutation OID pinning, symlink/traversal/
oversize coordination artifacts, duplicate + lost + spoofed doorbell,
reused-capsule contamination visibility (`contract.toml` mode recorded).

## Standing for the spike

RT-1 is the one **blocker**: the v0 architectural sketch's "TRUSTED CONTROL
PLANE … verify exact result" box is unsound as drawn and must gain a verify
capsule before the sketch is built even as a rig. RT-2 and RT-3 are binding
constraints on rig shape (reuse candidate verbs; control-plane-persistent
topology). Everything else amends prose or adds probes without changing the
architecture.
