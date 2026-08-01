# QUE-200: Safe capsule ingestion mechanism

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Question

What is the minimal safe parent-side mechanism for ingesting a phase result
from a potentially hostile capsule git repository?

Lifted from RFC-025 (next design question 1) and sharpened during discussion.
To be answered empirically by the C3 ingestion probe of the capsule spike —
not by argument.

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
   before ringing the doorbell. A bundle is a single flat file — pure data, no
   config, no hooks — so the control-plane boundary is `git bundle verify` +
   fsck'd fetch *from the file*. Cleanest trust story, and the bundle doubles
   verbatim as the forensic archive artifact. Worker dishonesty in creating it
   (wrong base, omitted commits) is caught downstream by the ancestry check,
   so trusting the worker to run the command is acceptable.

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
- **Verification against the exact tree** — check the candidate out into a
  clean control-plane workspace and run VTs there. Checkout of a hostile tree
  is itself a small surface (in-tree symlinks, `.gitattributes`), but
  smudge/clean filters only fire if *config* defines drivers, which a fresh
  workspace does not; symlinks are inert until followed.
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
