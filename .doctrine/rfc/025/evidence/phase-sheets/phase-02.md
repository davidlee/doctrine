# PHASE-02: Capsule and sandbox

Disposable phase sheet — runtime scratch under `.doctrine/state/`, gitignored
and `rm -rf`-able. Expands the plan's phase entry into working detail. Durable
risks / decisions / findings are harvested into the slice audit at close-out;
until then, lift anything that must survive into the slice's `notes.md`.

## Objective

Build the capsule side — the single bwrap profile both capsule kinds share, the
provisioner, the stub worker, the verify runner, and the doorbell — with the
I4a read-only bind posture and the resource bounds wired in from the start.
Confirm A1 and A2 on day one, because a credential failure here means the
capsule model needs a credential-proxy design and that is worth learning
before anything is built on top of it (R3).

## Reading list

Authoritative, in order. `plan.toml` PHASE-02 is the criteria source — `slice
phase` is a writer, there is no read-back verb.

- `.doctrine/slice/241/plan.toml` — PHASE-02 EN-1, EX-1..8, VT-1, VA-1..4.
- `design.md:501-521` — § 5.4 the doorbell (F-1/H14) and the sandbox resource
  bounds (F-11). The doorbell's four properties are the EX-5 checklist verbatim.
- `design.md:601-623` — I4 (the verdict is the runner's exit status) and **I4a**
  (runner provenance: ro-bind in, never copied). EX-4's whole content.
- `design.md:395-441` — § 5.2 declaration + provenance invariant. Context for
  why `capsule/` provenance matters; the F-5 probe itself is PHASE-05's.
- `design.md:933-962` — § 9.1 rig tree. `capsule/{sandbox,provision,worker-stub,
  verify}.sh` are this phase's files.
- `notes.md` § Harvest / § PHASE-01 findings / § R2 — fresh at `bd4dee1b`; cite
  ids, do not re-survey.
- `fixtures.md` F1 (light, BUILT) / F4 (heavy, BUILT). F2/F3 are PHASE-05's and
  this phase does not build them.
- `scripts/spike-capsule/lib/common.sh` — `rig_enter` (line 85), the assertion
  helpers (101-131), `rig_resolve`/`rig_capsule_root` (25-34). Ride these.
- `scripts/spike-capsule/rig:144` — the `smoke` arm, currently degrading with
  "PHASE-02 provides it". T7 is what it is waiting for.
- `scripts/spike-capsule/control/fixture-light.sh:22-56` — the control-script
  idiom (shellcheck source pragmas, `rig_enter` as a statement, `rig_die`).
- `scripts/pi-spawn-confined.sh:99-115` — the bwrap **seed**. Nesting mechanics
  and the fail-closed empty-PREFIX guard transfer; its **floor does not** (R-2).
- `mem.pattern.dispatch.nested-bwrap-userns-confines-worker-at-os-floor` —
  A1's precedent, and the load-bearing detail that **binds apply in order**, so
  a later ro-bind overlays an earlier rw bind.
- `mem.pattern.shell.guard-exit-swallowed-by-command-substitution` — F-P01-1;
  this phase adds entry points and `exit`-based refusals, which is where it
  recurs.

Research advisory: `slice research 241` reports drift against an artefact that
does not exist (no research round was run). Deliberately not restamped —
plan.md § Notes. Not a blocker for this phase.

## Assumptions & STOP conditions

**EN-1 met.** PHASE-01 `completed`: both fixtures provision and assert clean at
`~/capsules/fixtures/{heavy,light}`, and the I6 guard was observed refusing in
both directions (`rig selftest`, PHASE-01 VA-1).

Environment pinned at planning time: `bwrap` 0.11.2, `timeout`, `node` 26.5,
`npm`, `tsc` 5.9.3, `claude`, `jq`, `git`, `realpath`, `truncate`/`dd`,
`shellcheck` 0.11.0. **ABSENT: `nix`, `direnv`, `shfmt`** (EX-8). `$HOME`
writable; credential at `~/.claude/.credentials.json`, plus `~/.claude.json` at
`$HOME`.

**STOP → `/consult`, never improvise past:**

1. **A1 false** — nested bwrap refuses under an allowlist floor. Do **not**
   fall back to `--ro-bind / /` to make it work: that grants ro *visibility* of
   the canonical repo and breaks EX-1's ABSENT clause (R-2). A1 failing is a
   result about the model, not a rig bug to route around.
2. **`claude -p` needs a writable home** (R-1). Do not copy credentials into
   the capsule and do not widen the rw bind to `$HOME`. Consult; the fallback
   worth proposing is a tmpfs home with the credential file ro-bound over it.
3. **No disk-cap mechanism bites.** Do not record VA-4 as coded-but-unobserved.
   A mechanism never seen to fire is not known to work (§ 9, F-P01-3).
4. **`src/` change looks necessary.** Out of scope (design § 9.1, R2 settled).
5. **A criterion looks unsatisfiable as written.** Raise it; do not weaken the
   VT keyword set until the gate passes (plan.md § *On the VT mandates'
   brittleness*).

Also carried: **do not widen the conformance selectors** to silence the three
`undeclared` paths (`fixtures.md`, `notes.md`, `flake.nix`). Dispose at audit —
`notes.md` § PHASE-01 boundary. `edge` is shared; expect foreign commits inside
this phase's range too.

## Tasks

<!-- [ ] todo · [WIP] · [x] done · [blocked]. Graduates to TOML rows in the
     tracking file when a consumer needs queryable per-task status (D5/Q5). -->

- [x] **T1 — A1: nested bwrap under an allowlist floor** (EX-6)
  Near-free, and everything downstream stands on it, so it runs first. Assert
  a nested `bwrap` exits 0 in-jail *with an explicit ro-bind allowlist*, not
  the seed's `--ro-bind / /`. Record the observation (uid mapping, exit status).
  Touches: throwaway probe → folded into T2 once green.

- [x] **T2 — `capsule/sandbox.sh`: the P-C2 v0 profile** (EX-1, EX-2, EX-4,
  EX-8 · VT-1)
  One profile, both kinds. Kind selects the *command and the bounds*, never a
  second profile — EX-2 is P-C2's uniform-confinement claim.
  - rw: `--bind <capsule-dir> <capsule-dir>` — **only**.
  - ro: `/nix/store`, the toolchain paths, `/proc` `/dev` `/tmp` as tmpfs, and
    the agent home carrying the API credential.
  - **ABSENT (not ro)**: the canonical repo, other capsules, `~/.ssh`,
    `~/.gitconfig`, credential helpers. Absent is achieved by *not binding*,
    which is only possible under an allowlist floor (D-P02-1).
  - **I4a**: `sandbox.sh` / `provision.sh` / `verify.sh` ro-bound at a mount
    target **outside** the rw root (D-P02-2), so "not copied in" is structural
    rather than asserted.
  - `--clearenv` + explicit `--setenv` (PATH from the ro-bound store), and the
    seed's fail-closed empty-argv guard (`pi-spawn-confined.sh:115`).
  - `rig_enter` as a **statement**. New entry point ⇒ F-P01-1's exact trap.
  VT-1 keywords must be genuinely present, not decoratively: `bwrap`,
  `--ro-bind`, `--bind`, `timeout`.

- [x] **T3 — resource bounds** (EX-3 · VA-4 half 1)
  Both kinds, both dimensions, enforced **trusted-side** so the capsule cannot
  remove them (D-P02-4):
  - wall clock — `timeout -k` wrapping the `bwrap` exec;
  - disk — `ulimit -f` set before the exec so it is inherited, plus a post-run
    `du -s` cap on the capsule dir (per-file limit alone misses many-small-files).
  Emit **distinguishable exit statuses**, distinct from `RIG_EXIT_USAGE`/
  `RIG_EXIT_GUARD`. **No tokens here** — `verify-timeout` / `resource-cap` are
  trusted-side-computed in PHASE-03's pipeline (I5; plan.md § Notes item 2).

- [x] **T4 — `capsule/provision.sh`, `worker-stub.sh`, `verify.sh`** (EX-8)
  Provision = clone the fixture at the pinned OID into the capsule's rw root;
  toolchain arrives by ro-bind + PATH. **No `direnv allow`, no `nix`** — record
  the divergence from probe-specs § P-C1 step 2 as an environment fact (EX-8);
  do not edit RFC-025 prose (slice non-goal). `worker-stub.sh` makes a trivial
  commit and rings. `verify.sh` runs the declaration's `verify:` command; its
  exit status **is** the verdict (I4) — it authors no verdict file.

- [x] **T5 — the doorbell** (EX-5)
  Worker touches `result-ready` in the capsule rw root. Waiter is trusted-side,
  as `rig_wait_doorbell <capsule> <deadline> <interval>` in `lib/common.sh`
  (D-P02-3 — a placement choice, PHASE-03's pipeline is its consumer). Four
  properties, each asserted, not merely commented:
  content never read · identity from the caller's argument · loss degrades to
  polling with a wall-clock deadline · duplicate ring is a no-op (I2).

- [x] **T6 — `control/audit-i4a.sh` with its positive control** (VA-2)
  Audits that no control-plane runner exists inside the capsule's *writable*
  root. Compare by content, not name alone. **The audit must be shown to FAIL**
  when a `cp` of a runner is deliberately planted — a negative grep without a
  positive control proves only that grep ran
  (`mem_019fa18161f4…`, § 9). Both directions recorded.

- [x] **T7 — `control/probe-smoke.sh`: A2 as TWO assertions** (EX-7 · VA-1)
  Run *separately*, both outcomes recorded separately: (1) unauthenticated
  network reachability from inside the sandbox; (2) authenticated
  `claude -p 'print OK'`. Credential availability and network egress are
  distinct failure modes and one test conflates them (A8). Wires the `rig
  smoke` arm that currently degrades (`rig:144`).

- [x] **T8 — absent-not-ro, asserted on observables** (VA-3)
  Inside the sandbox, assert each ABSENT path **does not resolve** — never on
  absence of error output (DQ-3). Paired with a **positive control**: a path
  that must resolve (`/nix/store`) is observed present, so a probe that
  resolves nothing cannot score green.

- [x] **T9 — the bounds observed biting** (VA-4)
  A deliberately hung run is killed at the wall-clock bound; a deliberately
  oversized write hits the disk cap. Both **observed and recorded**, run inside
  the sandbox as `bash -c` (DQ-2 — a bound "held" by a polite worker is void).

- [x] **T10 — gate, record, commit**
  `shellcheck -x -S style` clean on every new file (the whole rig is clean at
  that level — keep it there). `doctrine validate` clean. No Rust expected; if
  any appears, `doctrine check gate`. Lift T1/T6/T7/T8/T9 observations and the
  D-P02-* decisions into `notes.md` **before** this sheet is `rm -rf`'d.
  Path-limited conventional commits scoped `(SL-241)`.

## Risks

- **R-1 — the agent home may not survive a read-only bind.** `claude` writes
  `~/.claude.json` (at `$HOME`, *outside* `~/.claude`) and likely state under
  `~/.claude`. EX-1 says the credential home enters **ro**. If A2 fails on
  writability rather than on credentials, that is R3's "credential-proxy
  design" signal in miniature — STOP-2, not a quiet widening.
- **R-2 — the seed's floor is the wrong floor.** `pi-spawn-confined.sh` uses
  `--ro-bind / /`, which makes *everything* readable and only writes deny.
  EX-1 requires the canonical repo to be **ABSENT, not ro**. Copying the seed's
  floor would pass a casual read of "confined" while silently failing EX-1 and
  making VA-3 unassertable. The seed's transferable parts are the nesting
  mechanics and the empty-argv fail-closed guard.
- **R-3 — the `cp` reflex undoes RT-1** (I4a). The rw bind would happily permit
  it, T6's audit is the only thing that would catch it, and the programme's
  only blocker is what is lost.
- **R-4 — F-P01-1 recurrence.** Every new entry point is a fresh chance to call
  a refusing guard inside `$( … )`. The unit probe stays green while the entry
  point is wide open; only an entry-point observation catches it.
- **R-5 — `--unshare-all` drops the network**, which EX-7's reachability
  assertion and the agent both need. `--share-net` is required; note it as a
  P-C2 property (the verify kind may want it off — a bounds parameter of the
  same profile, never a second profile).
- **R-6 — boundary pollution.** `edge` is shared and the operator commits to it
  mid-session; expect foreign interior commits in this phase's recorded range
  as in PHASE-01. Flag at audit, do not re-scope mid-phase.

## Decisions

Proposed at plan time from the criteria and the pinned environment; each is
confirmed or revised in execution and then lifted to `notes.md`.

- **D-P02-1 — allowlist floor, not `--ro-bind / /`.** EX-1's ABSENT clause is a
  *visibility* claim; the seed's floor is a *writability* posture. Absent is
  achieved by not binding, and only an allowlist can express that.
- **D-P02-2 — runners mount outside the rw root.** I4a becomes structural: no
  path the capsule can write is on the runner's mount path at all, which also
  gives T6's audit a crisp subject.
- **D-P02-3 — the doorbell waiter is a `lib/common.sh` function**, not a
  control script. PHASE-03's pipeline is its only consumer and the design does
  not name a file. Same class of placement choice as `control/selftest.sh`
  (plan.md § *One placement decision beyond the design's tree*).
- **D-P02-4 — bounds are enforced trusted-side**: `timeout` outside the `bwrap`
  exec, `ulimit -f` inherited into it, `du` after it. Nothing the capsule can
  unset. Tokens stay PHASE-03's.
- **D-P02-5 — one profile, parameterised.** Capsule kind selects command,
  bounds, and network; it never selects a profile. EX-2 is a claim about the
  mechanism being singular.

## Findings

<!-- F-P02-n — durable; lift to notes.md before this sheet is discarded -->
