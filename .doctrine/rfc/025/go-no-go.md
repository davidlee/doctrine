# RFC-025 · the capsule model — scoped go/no-go

Produced by **SL-241 PHASE-06** (EX-5 … EX-8), 2026-08-03. The verdict on the
capsule spike.

**Read `evidence/README.md`'s seven limits and `evidence/measurements.md`'s
closing section first.** They bound everything below, and this document is
written to be *harder* to over-claim from than the evidence it summarises — that
is its job, not a stylistic preference. Where this document and
`evidence/results-c3.tsv` disagree, the TSV wins.

---

## The verdict

**GO — on Linux/bwrap, for a client project of this build shape, for the
ingestion and confinement halves of the model. Not a go for the parts named
below as out of evidence.**

The scope is not a hedge attached to the verdict. **The scope is the verdict.**
A reader who takes "go" without it has taken something this spike did not
measure.

---

## 1 · Scope — what "go" covers (EX-5)

### Where it was measured

Every probe in this spike ran **in-jail on Linux, under bwrap**, on a NixOS host,
with git 2.54.0. Two fixtures: `light` (a small TypeScript project) and `heavy`
(a ~169 MB clone of this repository).

### What is portable, and what "model-level" actually means

The altitude vocabulary (design § 5.4 / A-3) grades a row by **how many fixtures
it held on**, not by how many operating systems. `model-level` means *the row
held on both fixtures* — it is a claim about **client-project shape**, not about
**host platform**.

> **FOR RECONCILIATION.** Design § 9's closure bullet reads *"model-level rows
> proven portable"*, which reads as OS-portability and is not what the altitude
> measures. `evidence/README.md` limit 5 and `measurements.md`'s closing section
> both state the narrower meaning. `/reconcile` owns the § 9 wording; this
> document uses the narrow reading throughout.

On that reading the rows split in two:

| class | portable? | why |
|---|---|---|
| **admission-boundary rows** — ancestry, selector conformance, forbidden paths, gitlinks, `.gitmodules`, OID identity, CAS | **portable in reasoning, unmeasured off Linux** | every scored refusal token is computed trusted-side from object identity by the mechanism-*independent* downstream pipeline (EVD-007). Nothing in that computation is a kernel feature |
| **environment-conditional rows** — the whole of P-C2's confinement set, `harvest/resource-cap`, H9's `verify`-stage containment leg, H12's env-file surfaces | **outstanding for macOS** | the boundary is bwrap: mount namespaces, `--unshare-all`, `ulimit`. macOS has no bwrap; an equivalent would be `sandbox-exec` or a VM, and it is a different mechanism with a different failure surface |

**Nothing here is portable to macOS without re-measurement.** The
environment-conditional rows are outstanding, not assumed to fail and not assumed
to hold.

**One row is below model-level even on Linux.** H12 (env-file execution surfaces
— `.envrc`, `flake.nix`) is `unproven-beyond-rust`: the light fixture had nothing
to plant, so only one fixture exercised it.

### What client shape "this build shape" means

Both fixtures build and test with a toolchain reachable inside the capsule.
**QUE-204 is open** — how a capsule obtains build inputs git cannot carry. Today
`heavy`'s web assets are built on site per cell, every stage-3 cell reaching
`registry.npmjs.org`; an outage lands as `verify/suite-failed` with nothing to
distinguish it from a real verdict. A client whose build needs inputs the capsule
cannot fetch is outside the measured shape.

---

## 2 · What was proven, and exactly how much (EX-6)

### The count, stated honestly

> **Sixteen hazard rows with a capsule-model boundary or a recorded dissolution,
> plus two regression legs.**

**Never "16/16".** Three things that phrasing would hide:

1. **The two regression legs count toward nothing.** H10 and H16's
   candidate-layer legs are the conflict sub-probe: **scaffolded
   incumbent-layer regression checks**, run against the *existing* dispatch
   implementation, not against the capsule model. They are excluded from every
   coverage number in this document (F-9, `evidence/guards.md`). They inform
   QUE-202; they evidence nothing about the model.
2. **Altitude is not uniform.** Fifteen rows are `model-level`; **H12 is
   `unproven-beyond-rust`**. A flat count erases that column.
3. **`n/a` is a structural absence, not a pass.** H13 is `n/a` on M-A because M-A
   reads no trusted-side artifact at all — there is nothing to refuse. H12 is
   `n/a` on `light` for the same structural reason. Counting `n/a` as coverage
   would count the absence of a hazard as a defence against it (README limit 3).

Beyond the sixteen: **four `fail` rows in `results-c3.tsv` are four successes** —
the falsification round's mutants, stamped `MUTATED=m32…m35` in the preceding
preamble. Anything counting outcomes must respect that stamp (README limit 7).

### Confinement — P-C2, seven rows, all pass

Final scored run 2026-08-03T04:36:53Z, after the D-P06-5 profile change, all
seven rows `pass` with a positive control each: `write-floor`, `canonical`,
`git-creds`, `api-cred`, `env`, `escape-git`, `resource`.

**`api-cred` is a re-take, not a caveat.** The row previously asserted on a write
to a *different file in the credential's directory* — read-only-**directory**
standing in for read-only-**credential**, indistinguishable only while the whole
of `~/.claude` was ro-bound. The tmpfs pulled the proxy apart from the claim and
the row went red (F-P06-8). It was realigned onto the credential itself — refused
on append, truncate **and unlink**, with the credential shown readable first and
a positive control writing successfully beside it — and the realigned row is
**strictly stronger** than the one it replaced (D-P06-8).

### The phase that actually ran — P-C1b, n = 1

A real agent executed a real red→green phase inside a capsule and **committed its
own work**: `9872a712`, `src/split.ts`, +25, the agent's own commit message.
`agent-committed=yes tree-dirty=no`; the worker's residue sweep did not fire
because the tree was clean. All four pipeline stages `pass`, exactly one canonical
ref changed, accepted ref at the pinned OID (F-P06-9).

**This is n = 1 by decision, not by accident** (D-P06-2). One scored attempt; the
prior attempt is disclosed with its usage in `measurements.md`, never discarded.
It supports *"a phase reaches green in a capsule at roughly this cost"*. It
**cannot** support a comparison.

---

## 3 · What is NOT proven

### 3.1 · Conflict/staleness **resolution** — out of evidence (EX-7, QUE-202)

**This is the largest gap the spike surfaces, and it outlives the slice.**

What *is* evidenced: the capsule model's **refusal** of a second result from one
base is **safe and total**. Stage 4's compare-and-swap, 8/8 cells at
`model-level`.

What is **not** evidenced: **admission**. How the second result is *resolved* —
three-way merge, a `Conflicted` state, supersede guidance — is not designed, and
the spike measures none of it.

Two properties of the refusal bound what any admission design must supply:

- **The refusal is content-blind.** Stage 4 compares two OIDs and never reads a
  tree. A genuine conflicting pair and a trunk advance a three-way merge would
  take without a murmur produce **the same token** (`advance/stale-base`). Any
  admission design must supply that discrimination itself.
- **The incumbent's recovery path discriminates and disagrees with itself on
  signalling.** It classifies a genuine conflicting pair as `Conflicted` and
  parks it for hand-resolution; but conflict exits **zero** with the verdict in
  `candidates.toml` while staleness exits **non-zero** (F-P05-40, ISS-305). A
  scripted caller reading exit status alone cannot see a conflict at all. A design
  reusing conflict semantics per DEC-110 inherits that unless it is named.

**Admission is out of evidence, not disproven.** Nothing here says it cannot be
built; the spike simply never designed or measured it. **QUE-202 stays open and
settles in the post-spike REV, not here.**

### 3.2 · QUE-200 does not settle on this evidence (EX-9, D-P06-9)

The ingestion-mechanism question — fetch-into-quarantine (M-A) versus bundle
(M-B) — **stays `open`**. PHASE-06 T8 ruled the EVD insufficient rather than
forcing a settlement, which EX-9 explicitly permits.

Candidate 3 (tree materialization) **is** excluded, on the question's own
grounds: it forfeits the forensic-history invariant by construction (EVD-008), and
neither rigged mechanism failed anything that would reopen it.

Candidates 1 and 2 cannot be separated, because **each carries exactly one
trusted-side surface the other lacks and only one of the two was measured**:

| | M-A (fetch) | M-B (bundle) |
|---|---|---|
| the surface it alone carries | **runs git inside a capsule-authored repository** — three touches in `harvest-fetch.sh`: `rev-parse` (`:57`, `:66`) and the `fetch` (`:73`), which spawns `upload-pack` there | **parses a capsule-authored file** trusted-side |
| how well measured | **sampled at two config keys** — and the sample shows the surface is not uniformly defended (F-P06-11) | **enumerated and tested** — EVD-010's four refusal legs, all `pass` on both fixtures |

The three measured asymmetries all sit on M-B and are all **costs** (an extra
boundary to write, EVD-007's defective two-site `harvest/fsck-failed` token,
EVD-009's quiesce obligation). The one asymmetry on M-A is a **safety**
asymmetry — and QUE-200 asks for the minimal *safe* mechanism. *"M-B has costs"*
is not *"M-A is safe."*

**Do not read the matrix as "fetch is proven safe against hostile config."**
`upload-pack` *was* exercised in the capsule repo's context, on every M-A cell,
and git's protected-config defence of `uploadpack.packObjectsHook` *was* observed
holding — the corpus previously said this vector was never exercised and that was
wrong (F-P06-11). But the surface was **sampled, not cleared**: QUE-200's claim is
a universal over git's whole config space, two keys do not discharge it, the
result is bound to git 2.54.0, and the second key — `core.fsmonitor` — **is**
honoured from repo-level config and stayed silent only because nothing in the M-A
harvest path refreshes an index.

### 3.3 · The assumption line — ASM-007 and ASM-008 (EX-8, per D-P06-1)

**EX-8 is discharged by its intent, not its letter, on an operator ruling.** Its
literal wording asks that *"ASM-007 is recorded strengthened, not discharged"*.
Writing that would plant a false claim in the one document whose whole purpose is
preventing over-claim, so it is not written.

The truth:

- **ASM-007 (*interpretation classes exhaustive*) was FALSIFIED and is
  `invalidated`** — not strengthened, not discharged. PHASE-04 step 0 ran the
  independent enumeration ASM-007's own validation plan specified: 96
  npm/TypeScript triggers enumerated with CPT-001 unread (`beb4b665`), classified
  append-only afterwards (`61ea9f08`, 0 deletions). The residue R1–R4 is the
  falsifier — terminal escape sequences and prototype pollution alone suffice.
  **The claim's shape failed, not merely its truth value**: adding a sixth class
  and re-asserting exhaustiveness would re-run the same error.
- **ASM-008 replaced it** — the universal / language-bound responsibility split —
  and **ASM-008 is STRENGTHENED, not discharged.** Empty residue over two
  ecosystems strengthens; it never discharges.

> **FOR RECONCILIATION.** Design § 9's closure bullet — *"ASM-007 is recorded
> strengthened, not discharged, whatever step 0 returns"* — was already false when
> PHASE-06 opened. `/reconcile` owns it. The divergence is recorded here rather
> than smoothed away because `VA-3` checks that `doctrine knowledge show ASM-007`
> and this document agree, and they do: both say `invalidated`.

### 3.4 · Everything in `evidence/README.md`'s seven limits

They apply unchanged and are not restated here. Limit 1 was **corrected** in
PHASE-06 (§ 3.2 above); limits 2–7 stand as written.

---

## 4 · Stated properties of the model

Not implementation details — properties a reader of this verdict is entitled to,
and which a downstream design would inherit.

**The OS boundary IS the boundary** (D-P06-6). The worker runs
`claude -p --dangerously-skip-permissions` inside the capsule, and that is
**endorsed**, not tolerated. A harness-level permission flag inside an
already-confined capsule is not a change to the bwrap boundary. The capsule
model's confinement claim is therefore explicitly that confinement is the
sandbox's job and a harness inside it need not re-litigate it per tool. A design
that weakened the sandbox to compensate for a harness setting would be inverting
this.

**The credential refusal is `EROFS` from the mount flag, not `EACCES` from mode
bits.** The capsule runs as `uid=1000` and *owns* `.credentials.json`
(`-rw-------`), so permission bits would not stop it; only the read-only mount
does, and `--unshare-all` denies the `CAP_SYS_ADMIN` needed to remount. **Any
future narrowing that reintroduced the secret on a writable mount would be a real
weakening however tight its mode bits looked.**

**Nothing the capsule says decides anything** (I5). Token usage is the single
measurement with no trusted-side source — `claude` counts tokens and `claude` runs
inside the capsule, which is precisely what EX-3 requires. Two mitigations, both
taken, neither a fix: the JSON is streamed over the worker's stdout so it never
becomes a file the capsule could revise, and it is **recorded, never asserted** —
no stage, token, outcome or altitude reads it. The column header is
`capsule-reported` and that is the honest one (F-P06-5).

**A degraded agent is not a cheap one — it cost roughly double.** The prior
attempt, which executed nothing, burned **2.4× the output tokens and 2.0× the
cache reads** of the run that did the work (F-P06-9). Failure has a cost curve of
its own and it points against the intuition that a blocked agent gives up early.

**Refusals need no operator.** All 18 (stage, token) pairs terminate the run with
the accepted ref unmoved and the capsule discarded — no partial state to repair,
no prompt to answer. Observed across every P-C1a, P-C1b and P-C3 run. The honest
limit: the set is closed but **not fully exercised** — `cas-lost` is reachable and
owned by no matrix row.

---

## 5 · Outstanding work

Lodged rather than left in prose (EX-10 lodges the backlog items; these are the
verdict's own list).

1. **Settle QUE-200** — bound or eliminate M-A's trusted-side git surface in the
   capsule repo rather than sample it further. The residual is a universal over
   git's config space that no probe can exhaustively discharge; the tractable
   form is whether the parent needs to run git in that repository **at all**.
2. **Design conflict/staleness admission — QUE-202.** § 3.1. The largest gap.
3. **Re-measure the environment-conditional rows on macOS**, or scope the model
   to Linux hosts explicitly. § 1.
4. **The agent home's write scope is coarser than it wants to be.** What shipped
   is a blanket `--tmpfs /agent/.claude` with the credential and `~/.claude.json`
   ro-bound in. Three directions, none showstopping:
   (a) **scope the writes** to the session folder and `~/.claude.json` rather
   than the whole directory — noting that `session-env` is an *undocumented
   harness internal that will move*, so this needs a stable seam or a
   harness-version canary, not a hardcoded path (D-P06-8 rejected pinning to it
   for exactly that reason);
   (b) **populate from fixtures in a known state** — `~/.claude.json` is today
   the control plane's live 90 KB file;
   (c) **salvage the tmpfs for forensics**, attached to the run — it currently
   dies with the capsule, discarding exactly the harness session state that would
   have diagnosed F-P06-7 in seconds.
   **Any design here must keep the `EROFS`-not-`EACCES` constraint in § 4.**
5. **Split `harvest/fsck-failed`.** One token stands for two unlike causes — *the
   ingested objects are bad* (security) and *this quarantine's derived cache is
   stale* (operational) — emitted at two sites with git's stderr discarded at
   both. The harvester also fscks the **whole quarantine**, not the range it just
   ingested, so damage from any source is attributed to the capsule (F-P05-28).
6. **Reconcile ISS-305's signalling disagreement** — conflict exits zero,
   staleness exits non-zero. Inherited by any design reusing DEC-110's semantics.
7. **The allowlist / QUE-204 follow-on slice** (D-P05-17) — egress must be
   allowlisted per capsule rather than on/off, and how a capsule obtains build
   inputs git cannot carry shares the same lever. Already placed as a follow-on
   slice; the largest piece of forward work PHASE-05 created.
8. **`probe-smoke.sh`'s capability leg should be permanent**, not a PHASE-06
   one-off. It is what catches a harness version that starts needing to persist
   to a read-only path — a failure that presents as *partial*, not as an error.

---

## 6 · What this document does not establish

- **It is not a comparison with the incumbent.** No row in `measurements.md` is a
  before/after measurement of the same real phase, which is what
  `probe-specs.md` § Measurements literally asks for. Rows 1–5 compare *models by
  static inspection*; rows 7 and 8 have no incumbent source at all.
- **It is not a production readiness assessment.** It is a verdict on whether the
  capsule model's ingestion and confinement halves survive adversarial probing at
  the altitudes recorded. Operability, migration, and the incumbent's replacement
  path are not in scope.
- **It does not establish that a second agent, a second client project, or a
  second host would behave the same.** n = 1 on the agent axis; two fixtures on
  the project axis; one OS.
- **It settles no open question.** QUE-200, QUE-202 and QUE-204 all remain
  `open` and settle in the post-spike REV. This document records what the
  evidence does and does not support for each; it is not their answer.
