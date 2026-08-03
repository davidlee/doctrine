# QUE-200: Safe capsule ingestion mechanism

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Question

What is the minimal safe parent-side mechanism for ingesting a phase result
from a potentially hostile capsule git repository?

Lifted from RFC-025 (next design question 1) and sharpened during discussion.
It was initially framed for empirical selection by the capsule spike's C3
ingestion probe. `SL-241` narrowed the choice but did not settle it; `DEC-128`
then established that the remaining issue was architectural rather than a call
for more sampling. The answer is recorded below and in `DEC-135`.

## Why it is trust-bearing

C3 is the control-plane half of the capsule authority boundary. Confinement
(C2) bounds what the worker can *touch*; ingestion bounds what the control
plane will *believe*. It is needed even with a perfect sandbox — the worker is
fallible as well as potentially hostile — and it is the defense-in-depth if
the sandbox leaks. It replaces `worker_commit`-style gating: instead of gating
the worker's commit, the control plane stops treating worker commits as
canonical at all.

The core hazard: after the doorbell, the result must be extracted from a git
repository whose entire contents — including `.git` itself — were writable by
an untrusted process. A git repo is not just data; its config and hooks can
execute code. If the control plane runs git commands *inside* the capsule's
repo directory, the capsule's `.git/config` is attacker-controlled, and
several keys are command-execution vectors (`core.fsmonitor`,
`credential.helper`, pager/ssh commands — the CVE-2022-24765 class).
Ingestion must treat the capsule repo as untrusted input, never as a working
context.

## Candidate mechanisms

1. **Fetch into quarantine** (RFC-025's leading hypothesis).
   `git fetch file:///capsule/repo <ref>:refs/quarantine/<id>/…` with
   `fetch.fsckObjects=true`. Preserves worker commit history as first-class
   objects (the forensic evidence RFC-025 commits to retaining). The subtlety
   the probe must test: local fetch spawns `git upload-pack` in the capsule
   repo's context, which reads that repo's config. Git deliberately ignores
   the known-lethal key at repo level (`uploadpack.packObjectsHook`), but
   "git's protected-config rules cover everything a hostile repo config can do
   to upload-pack" is a trust-bearing claim to test, not assume. Cheap
   hardening: two-hop — fetch into a throwaway quarantine *repository*, fsck
   there, then fetch canonical ← quarantine, so hostile objects never enter
   the canonical object db unvetted.

2. **Bundle.** The worker runs `git bundle create result.bundle base..HEAD`
   before ringing the doorbell. A bundle is a single flat file rather than a
   Git repository context, so the control-plane boundary is a bounded snapshot
   followed by `git bundle verify` and an integrity-checked import in a fresh
   quarantine repository. The bytes remain hostile and the file boundary has
   its own refusal obligations. The bundle may also serve as the forensic
   archive artifact. Worker dishonesty in creating it (wrong base or omitted
   commits) is caught downstream by trusted ancestry and conformance checks.

3. **Tree materialization.** Ignore worker git entirely: copy the working
   tree out, diff against the contracted base, control plane constructs the
   commit itself. Essentially what the pi arm does today (working-tree diff
   import) — the incumbent, and the weakest on evidence: worker commit history
   is lost, and it trades git-object hygiene for filesystem hygiene (symlink
   escapes during copy).

## Mechanism-independent downstream pipeline

Whichever mechanism wins, the admission pipeline is the same:

- **Ancestry validation** — candidate tip descends from the contracted base,
  first-parent, no merges (configurable). Catches wrong-base and
  rewritten-history in one check.
- **Actual-path computation** — tree-delta(base, tip) from object identity,
  never from worker assertion.
- **Selector conformance** — the declared/changed algebra over the actual
  set; forbidden-path rejection (`.doctrine/`, `.claude/`, workflow config)
  as admission policy, the second layer behind the sandbox.
- **Verification against the exact tree** — check the candidate out inside a
  separate verification capsule and run the declared verification there. The
  runner is supplied read-only from outside mutable capsule state, and the
  sandbox process's exit status is the verdict.
- **Normalization** — the control plane authors the canonical single phase
  commit itself (own committer identity, message convention, exact tree).
  Worker commits become archived evidence, never ancestry.

## Answer criteria

The probe answers this question by running the hostile-probe subset against
each candidate mechanism: wrong base, rewritten history, merge commits,
undeclared paths, `.doctrine/` writes (all must die at admission); malicious
`.git` config/hooks, oversized blobs, gitlink/submodule insertion, symlink
games (all must die at the ingestion boundary without executing anything).

Probe recommendation from discussion: rig fetch-into-quarantine and bundle
side by side (they share ~90% of the rig; the difference is a few lines at
the boundary) and let probe results pick. Skip materialization unless both
fail something, since it forfeits the forensic-history invariant.

## Evidence inputs (SL-241 PHASE-05)

The probe ran. Five verdict inputs are recorded as EVD records, each linked to
this question — four `supports`, one `disputes`. Candidate 3 (tree
materialization) was not rigged, on this question's own recommendation, and
neither mechanism failed anything that would reopen it.

| input | record | what it says |
|---|---|---|
| per-mechanism H6/H7 behaviour | **EVD-006** `supports` | hostile `.git/config` + hooks (H6) and oversized objects (H7) score identically on both mechanisms — 8 cells, all pass at `model-level`. Neither hazard discriminates the candidates |
| trust-bearing lines at the boundary | **EVD-007** `supports` | every scored refusal token is computed trusted-side by the *mechanism-independent* downstream pipeline. The mechanism decides the transfer, not the admission boundary |
| forensic completeness | **EVD-008** `supports` | both mechanisms carry the worker's commit into quarantine as a first-class object. The forensic invariant does not discriminate candidates 1 and 2 |
| operational friction | **EVD-009** `supports` | cost is equal (~2.5s a heavy cell either way). The asymmetry is fragility: M-B's larger transfer trips git's auto-maintenance in the quarantine, which M-A did not |
| M-B's trusted-side file-ingestion boundary | **EVD-010** `disputes` | H13 exists on M-B only — four refusal legs for a capsule-authored file. On M-A the row is structurally `n/a`. The bundle **moves** the surface rather than removing it, which is counter-evidence to candidate 2's "cleanest trust story" above |

**The shape of the result.** On the axes that were expected to separate the
candidates — config/hooks, resource bounds, forensic retention, cost — they are
equal. Three asymmetries were measured and all three sit on the M-B side:
EVD-010's four-leg trusted-side file boundary, EVD-007's defective two-site
`harvest/fsck-failed` token, and EVD-009's quiesce obligation. **All three are
costs — an extra boundary to write, a token to split, a provisioning step to
run.**

## The prior EX-9 ruling: the evidence did NOT suffice (SL-241 PHASE-06 T8, D-P06-9)

At `SL-241` closure the question stayed **`open`** for the post-spike decision.

**Candidate 3 is excluded** on grounds this question already accepts — EVD-008:
tree materialization forfeits the forensic-history invariant by construction, and
neither rigged mechanism failed anything that would reopen it.

**Candidates 1 and 2 cannot be separated on the evidence, because each carries
exactly one trusted-side surface the other lacks and only one of the two was
measured:**

| | M-A (fetch) | M-B (bundle) |
|---|---|---|
| the surface it alone carries | **run git inside a capsule-authored repository** — three touches in `harvest-fetch.sh` (`rev-parse` at `:57`/`:66`, the `fetch` at `:73`, which spawns `upload-pack` there) | **parse a capsule-authored file** trusted-side |
| how well it is measured | **sampled at two config keys**, and the sample's own result shows the surface is not uniformly defended — F-P06-11 | **enumerated and tested** — EVD-010's four legs, all `pass` on both fixtures |

EVD-010 proposed the right comparison — *"the two arms should be compared on
total trusted-side surface"* — and that is precisely the axis where the
measurement is uneven. **The three asymmetries favouring M-A are cost
asymmetries; the one favouring M-B is a safety asymmetry**, and this question
asks for the minimal **safe** mechanism. Settling on M-A now would adopt the
candidate whose own named hazard is the less-measured one, on the strength of
evidence that mostly shows the *other* candidate carries friction. *"M-B has
costs"* is not *"M-A is safe."*

**What would close it** — bound or eliminate M-A's trusted-side git surface in
the capsule repo, rather than sample it further. The residual is a universal over
git's config space (*"protected-config rules cover everything a hostile repo
config can do to upload-pack"*) which no probe can exhaustively discharge; the
tractable form of the question is whether the parent needs to run git in that
repository **at all**. Carried into the go/no-go's outstanding work.

**Not a gap the spike failed to fill.** F-P06-11 makes the residual smaller and
more precise than PHASE-05 recorded it — the vector *was* entered, and git's
documented defence *was* observed holding for `uploadpack.packObjectsHook`. What
changed is that the second planted key, `core.fsmonitor`, turns out to be
honoured from repo config and to have stayed silent for a contingent reason.

Scored data: `probes/c3/results.tsv`; committed summaries under
`.doctrine/rfc/025/evidence/` (SL-241 PHASE-05 T8).

## Answer and implementation handoff (2026-08-03)

The v0 mechanism is **Git bundle ingestion**, governed by a structural rule:
trusted control-plane code never runs Git with a capsule-authored repository as
its repository or working context. This answers the tractable architectural
question identified above instead of claiming that further config-key samples
could prove fetch safe.

The worker creates the bundle at a fixed control-plane-selected location and
rings the doorbell only after publication. The parent treats the artifact as
hostile bytes, snapshots it once into parent-owned storage under no-symlink and
resource bounds, and lets Git read only that snapshot from a fresh disposable
quarantine repository. The downstream pipeline then:

1. verifies and imports the bundle, runs object-integrity checks and pins the
   result object identity;
2. checks contracted-base ancestry, merge policy, actual paths, declared scope,
   forbidden paths and tree modes against quarantine objects;
3. runs declared verification against the pinned result in a separate
   verification capsule;
4. rechecks that the accepted ref is still at the contracted base, transfers
   the pinned objects and performs one expected-old-object compare-and-swap;
5. writes the durable admission journal and disposes quarantine, while the
   bundle follows `DEC-133`'s separately-owned forensic-exhibit lifecycle.

This is an implementation path, not merely a transport preference. Existing
Git ancestry, compare-and-swap, strict slice-conformance, candidate identity and
admission seams are to be reused. `QUE-202` still owns decoupling conflict and
staleness recovery from the incumbent dispatch journal; it blocks complete
cutover design, not bundle ingestion itself. `SL-241`'s hostile matrix and stage
assertions should become production acceptance tests, while its disposable
shell rig remains evidence rather than migrated product code.

`DEC-135` records the decision and its consequences. It is an input to
`REV-046`, which will promote this scoped answer into binding architecture and
specification.
