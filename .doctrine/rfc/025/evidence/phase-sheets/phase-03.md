# PHASE-03: The four-stage pipeline and the rig's own red/green

Disposable phase sheet — runtime scratch under `.doctrine/state/`, gitignored
and `rm -rf`-able. Expands the plan's phase entry into working detail. Durable
risks / decisions / findings are harvested into the slice audit at close-out;
until then, lift anything that must survive into the slice's `notes.md`.

## Objective

Implement the closed four-stage pipeline against a per-run quarantine
repository, with the closed refusal-token vocabulary and the outcome-conditional
`assert_outcome`, then prove the rig does not lie. Until the happy-path
self-test lands green, every 'refused' is indistinguishable from 'rig broken'
(R4) — so no hostile row may claim a kill before it does.

## Reading list

- `plan.toml` PHASE-03 — EN-1/2, EX-1..11, VT-1..4, VA-1..3. Authoritative;
  `slice phase` is a writer, read criteria from the TOML.
- `design.md` §5.1 four stages + closed vocabulary (:176, :228) · §5.2 harvest /
  conform legs / provenance (:297) · §5.4 doorbell + matrix harness (:501) ·
  §5.5 I1–I6 (:557).
- `notes.md` §R2 (settled — leg 3 is load-bearing) · §PHASE-01/02 findings.
- `fixtures.md` F1 (light, BUILT) · §"Not fixtures" (quarantine is per-run).
- Rig: `lib/common.sh` · `capsule/{sandbox,provision,verify,worker-stub}.sh` ·
  `control/probe-capsule.sh`.
- Memory: `mem.pattern.doctrine.path-policy-shell-hardening` (EX-5's invocation
  form — copy, do not re-derive) · `mem.pattern.tdd.wire-before-guard` (VA-3
  wants a *wrong-admission* red, not a payload red) ·
  `mem.pattern.harness.grep-negative-needs-positive-control`.

## Entrance criteria — CONFIRMED

- **EN-1** — PHASE-02 complete; stub worker runs confined and rings. Re-observed
  this session: `probe-capsule.sh all` all-green, including the live join
  (worker rings, waiter hears).
- **EN-2** — R2 settled in PHASE-01 (`notes.md` §R2). Conform leg 2 has a
  known-good invocation; leg 3 is separately load-bearing.
- Research advisory reports drift against an **absent** `research.md` — the
  pre-design round was never run, deliberately not restamped (`notes.md` §Open,
  plan.md §Notes). Not a blocker; not re-litigated here.

## Entry audit — three findings carried in from PHASE-02

Run before planning, per the handover's "audit my assertions" brief. Evidence in
this session's transcript; lift to `notes.md` as F-P03-1..3 during execution.

1. **Two of five ABSENT legs in `probe-capsule.sh` are vacuous.** `~/.ssh` does
   not exist on this host, and no `credential.helper` is configured, so
   `test ! -e ~/.ssh` and `[ -z "$(git config --get credential.helper)" ]` both
   pass **unconfined**. They are F-P02-1 unfixed: a must-fail assertion whose
   subject was never present. (`~/.gitconfig`, the canonical repo and the capsule
   root are genuine — subjects exist on the host and do not resolve inside.)
2. **The disk bound has two legs and only one reports.** `SANDBOX_DISK_BLOCK`
   is correct (8 MiB cap truncates at exactly 8388608 B). But
   `disk: an oversized write is REFUSED (status 4)` passes on `du` reporting
   8392704 vs a cap of 8388608 — **one 4096-byte block of accounting slop**, not
   the 8× overshoot the comment claims (a 64 KiB overshoot is byte-identical).
   Underneath: `ulimit -f` fires as **SIGXFSZ**, which reaches the parent as a
   raw status `sandbox.sh` never classifies. Observed: a sparse oversize gives
   inner 153 and **outer status 0** — the bound bit and the sandbox reported
   success. `du`'s cumulative leg (D-P02-7) is genuine (40×512 KiB → 20975616 B,
   status 4).
3. **`--print-mounts` equality is sound** — it has resolution (a `--no-net`
   difference is visible), and no kind token appears in the 52 emitted lines, so
   a kind-dependent mount would red it. Narrow gap: both invocations omit
   `--source`, so kind-dependence in that branch is untested.

## Assumptions & STOP conditions

- **A-1** — the pipeline's "canonical" is a **per-run disposable clone of the
  fixture**, not the fixture itself. See D-P03-1; if the design is read to mean
  the fixture *is* canonical, stage 4 mutates the pristine base and
  `assert_outcome` loses its subject. Settled as a placement decision, not a
  design change.
- **A-2** — the quarantine is cloned from canonical (worktree at **B**, the
  control-plane-pinned base) and then fetches S. This gives conform leg 2 a
  `.doctrine/` registry to run `-p` against while S stays an object-only
  presence. B is not the candidate, so I4's "no candidate tree materialised
  trusted-side" holds. See D-P03-2.
- **A-3** — the declaration is read from canonical's **sibling** file (the
  control-plane-pinned copy), which is "read from B" in F-5's sense. F2
  (PHASE-05) manufactures the in-repo case; the read must be structured so it
  becomes `git show B:<path>` there and never `S:<path>`.
- **STOP-1** — a genuine `--strict`-vs-belt divergence → `/consult`, never an
  improvised `src/` change. (R2 says there is none; this stands for the run.)
- **STOP-2** — **do not mint a token outside §5.1's closed set.** Two mapping
  gaps are already known (see §Open questions); they are recorded as findings,
  not filled by invention.
- **STOP-3** — no `src/` changes in scope.

## The status → token mapping (PHASE-03 owns it; D-P02-4, handover item 4)

PHASE-02 deliberately emits statuses and no tokens (I5). Re-deriving this
incompatibly would look like a pipeline bug, so it is pinned here:

| sandbox status | kind | stage | token |
|---|---|---|---|
| 0 | either | — | pass |
| `RIG_EXIT_DISK` (4) | worker | harvest | `harvest/resource-cap` |
| `RIG_EXIT_SANDBOX` (5) | verify | verify | `verify/sandbox-failed` |
| `RIG_EXIT_TIMEOUT` (124) | verify | verify | `verify/verify-timeout` |
| 153 (SIGXFSZ) | either | — | **T1 folds this into 4** — it *is* the disk bound |
| other nonzero | verify | verify | `verify/suite-failed` |

## Tasks

- [ ] **T1 — close the SIGXFSZ hole** (entry finding 2). `capsule/sandbox.sh`:
  map 153 → `RIG_EXIT_DISK` alongside the existing 127 → `RIG_EXIT_SANDBOX`.
  Sound because *the rig itself sets* `ulimit -f` to the cap, so SIGXFSZ inside
  the namespace is by construction that bound firing. Red first: the sparse-file
  case currently exits 0/153. Touches `capsule/sandbox.sh`.
- [ ] **T2 — de-vacuum the two ABSENT legs** (entry finding 1). Require the
  subject to resolve **on the host** before asserting it does not resolve inside
  — the `runner_present_and_read_only` shape, applied where it was skipped. For
  the credential helper: assert `git` RUNS inside first (else the substitution is
  empty for the wrong reason), and plant a helper in the probe's own host-side
  config subject so the leg has something to hide. Where a subject genuinely
  cannot be made to exist, record the leg `n/a` with its reason rather than
  leaving a green that means nothing. Touches `control/probe-capsule.sh`.
- [ ] **T3 — pipeline scaffolding + the token vocabulary** (EX-9, VT-1).
  `control/pipeline.sh`: `stage_emit <stage> <verdict> [token]` printing
  `stage=<n> verdict=<v> token=<t>`; the closed token set as named constants
  (STD-001); the runner recording the **first** refusing stage; an
  outside-the-set token treated as a **rig defect**, not a result. `cas-lost`
  declared legal, owned by no row, recorded reachable-but-unexercised.
- [ ] **T4 — per-run repos** (EX-2, D-P03-1/2). `pipeline_setup`: canonical =
  `clone --no-hardlinks` of the fixture; quarantine = `clone --no-hardlinks` of
  canonical with `fetch.fsckObjects=true`; both under a per-run dir, both
  `rm -rf`'d at teardown. `guard_not_real_repo` on each before creation.
- [ ] **T5 — stage 1 M-A** (EX-1, VT-2). `control/harvest-fetch.sh
  <capsule> <quarantine>` → pinned OID on stdout. Reads the result ref
  **exactly once** (RT-5), fetches into quarantine, `git fsck`. No second hop
  into canonical (F-3 — the deleted hop).
- [ ] **T6 — stage 1 M-B** (EX-3, VT-3). `control/harvest-bundle.sh
  <capsule> <quarantine>` → pinned OID, same signature so the matrix loops
  without branching. Four legs in order: control-plane-chosen fixed path (the
  capsule never names it); resolve with symlinks refused → `bundle-unsafe-path`;
  size-cap **before** `git bundle verify` reads it → `resource-cap`;
  absent/truncated → `bundle-absent`/`bundle-invalid`, no silent wait-and-retry.
  Requires extending `worker-stub.sh` to emit a bundle pre-doorbell at a path
  passed in over the sandbox env (same single-sourcing discipline as
  `RIG_DOORBELL`, F-P02-6).
- [ ] **T7 — stage 2 conform, four legs** (EX-4/5/6). (1) ancestry —
  `merge-base --is-ancestor B S` + `rev-list --min-parents=2 B..S` empty;
  (2) scope — `doctrine slice conformance <id> -p <quarantine> --against
  B..<oid> --strict`; (3) forbidden paths — **the belt's own form**, copied not
  re-derived: `git -c core.quotePath=false diff --name-only --no-renames -z
  B..S`; (4) tree mode — `ls-tree -r` refusing `160000` and `.gitmodules`,
  symlinks (`120000`) stay legal. Leg 3 is **not** folded into leg 2 (R2b).
- [ ] **T8 — stage 3 verify + the provenance invariant** (EX-7). Verify capsule
  at the pinned OID; verdict is the sandbox's exit status (I4). The `verify:`
  command is read from **B** (A-3) and passed in; `capsule/verify.sh` never
  reads a declaration. Map status→token per the table above.
- [ ] **T9 — stage 4 advance, ordered** (EX-8). precondition (accepted ref still
  at B? else `advance/stale-base` **having transferred nothing**) → transfer →
  CAS expecting old value B (`advance/cas-lost` on a genuine race). Getting this
  backwards reds `assert_outcome` on H10/H16.
- [ ] **T10 — `assert_outcome`, outcome-conditional and token-keyed** (EX-10,
  I1). Keys off the **token**, never the stage, so `cas-lost`'s refs-only clause
  cannot absorb `stale-base`'s full clause. Three arms: refused at
  harvest/conform/verify/`stale-base` ⇒ canonical byte-identical **including
  object count**; refused at `cas-lost` ⇒ refs unchanged, orphan count
  **recorded not asserted**; passed ⇒ exactly one canonical ref changed.
- [ ] **T11 — happy-path self-test** (EX-11, VT-4, VA-1). `control/selftest.sh`
  on F1, both mechanisms. `rig selftest` auto-dispatches the moment the file
  exists (D-P01-3) — **do not rewire the arm**. Runs `guard_not_real_repo` first.
  Observed green and **recorded** before any hostile row.
- [ ] **T12 — VA-2: stage emission ASSERTED, not inferred.** The direct
  descendant of F-P02-2. A run must prove *which* stage refused from the emitted
  `stage=` line, not from an exit code — pass/partial attribution depends on it.
- [ ] **T13 — VA-3: `assert_outcome`'s object-count clause shown falsifiable.**
  Per `mem.pattern.tdd.wire-before-guard` this wants a **wrong-admission** red,
  not a payload red: land the pipeline, then run a deliberately
  canonical-writing harvest (the deleted second hop, F-3) and observe the
  object-count clause **RED**. Use `rig_assert_fails` — never `rig_assert '…' !
  cmd` (F-P02-3).
- [ ] **T14 — lint, harvest, boundary.** `LC_ALL=C.UTF-8 shellcheck -x -S style`
  on every rig file (F-P02-5 — without the locale it aborts on em-dashes with no
  line and no rule id). `doctrine validate`. Lift F-P03-1..3 and D-P03-* into
  `notes.md`; `verify-vt` VT-1..4; check the recorded range.

## Verification map

| criterion | how |
|---|---|
| VT-1 | `pipeline.sh` carries the four stages, the closed vocabulary incl. verify-stage tokens, `core.quotePath=false`, `--no-renames`, `160000`, `--strict` |
| VT-2 | `harvest-fetch.sh` — `fetch.fsckObjects`, `fsck` |
| VT-3 | `harvest-bundle.sh` — `bundle verify`, `bundle-unsafe-path`, `bundle-absent`, `bundle-invalid`, `resource-cap` |
| VT-4 | `selftest.sh` — `guard_not_real_repo`, `assert_outcome` |
| VA-1 | self-test observed green on F1 **before** any hostile row; run recorded |
| VA-2 | T12 — emission asserted |
| VA-3 | T13 — wrong-admission red on the object-count clause |

`doctrine slice verify-vt 241` after T14; VT-1..4 flip green (7 mandates
currently FAIL across PHASE-03..06, none UNCHECKABLE, none WAIVED).

## Risks

- **R-a — the quarantine's worktree.** Cloning canonical gives quarantine a
  checkout at B. If any leg is written against the *worktree* rather than
  against objects, EX-4's "no worktree/index/staging" is violated in substance
  while passing a casual read. Every leg is plumbing over `B..S`; the checkout
  exists only so `conformance -p` can find a registry.
- **R-b — leg 3 folded into leg 2.** The single most damaging available mistake
  (R2b, H5). A pipeline running only `--strict` passes a `.doctrine/` touch
  whenever a selector declares that path, and this slice's own selectors are
  that shape.
- **R-c — stage 4 ordering inverted.** Transferring before the precondition read
  reds `assert_outcome` on exactly the rows where it does the most work, for a
  reason belonging to git's object model rather than the model under test.
- **R-d — absence-shaped assertions.** The entry audit found two live ones. Every
  new must-fail assertion states what would have to break for it to red, and
  proves its subject reachable first.

## Decisions

- **D-P03-1 — the pipeline's canonical is a per-run disposable clone of the
  fixture.** Design §5.1 names canonical / quarantine / capsule as three zones;
  `fixtures.md` §"Not fixtures" makes only the quarantine per-run. But stage 4
  advances an accepted ref, and `assert_outcome` asserts canonical is
  byte-identical across a refused run — neither is expressible against the
  pristine base fixture, which PHASE-05 also reads. So canonical is instantiated
  per run from F1, the accepted ref is `refs/heads/mainline` (F1's trunk — the
  fixture is deliberately not `main`), and the fixture stays a template. Same
  class of placement choice as D-P02-3.
- **D-P03-2 — the quarantine is cloned from canonical, not `init`'d empty.** It
  must satisfy two things at once: hold S's objects for the conform legs, and
  give `slice conformance -p` a `.doctrine/` registry to read. A clone at B does
  both while keeping it a real, separate, per-run, disposable repository —
  which is all EX-2 requires. `--no-hardlinks` so a corrupt object cannot reach
  canonical through shared object files (the `provision.sh` reasoning).
- **D-P03-3 — SIGXFSZ is classified as the disk bound** (T1). The rig sets
  `ulimit -f` itself, so 153 inside the namespace has exactly one cause.

## Open questions — vocabulary gaps, recorded NOT filled (STOP-2)

Surfaced by pinning the status→token table. Neither blocks the happy path;
both want an operator ruling or a PHASE-05 `/consult` before a row needs them.

- **OQ-a — a verify capsule that overruns the DISK cap has no token.** EX-3
  reads bound→token (`timeout`→`verify-timeout`, `disk`→`resource-cap`), but
  §5.1's prefixes read stage→token, and the refusing stage there is `verify`.
  `verify/resource-cap` does not exist in the closed set and is not minted here.
- **OQ-b — a WORKER capsule that overruns the wall clock has no token.** The
  doorbell wait ends at its deadline and harvest finds no result ref; `harvest`
  has no timeout token. Likely PHASE-05 H15's business ("killed at each stage in
  turn").

## Findings

<!-- F-P03-N as execution surfaces them; lift to notes.md before rm -rf -->
