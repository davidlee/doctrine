# Review RV-231 — reconciliation of SL-185

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation reconciliation audit of SL-185 (subprocess-arm Seatbelt
confinement, macOS jail parity). All 4 phases implemented and landed on trunk;
code at 1fdbf4fa (PHASE-04 tip), main promoted there, edge at 5eeb5be0. Reviewed
surface: the **parent tree** authored artefacts + the landed trunk code (solo
delivery per phase; P01–03 arrived via dispatch merge 31d97083, PHASE-04 solo).

**Lines of attack / invariants held:**

1. **Conformance algebra** — does what git touched match the design-target
   selectors? (undeclared = scope creep / missed design update; undelivered =
   dropped work / stale design.) Boundaries had to be re-recorded first (P01–03
   rows were absent — dispatch-on-sleipnir + gitignored runtime registry).
2. **Behaviour-preservation gate** (ADR-008 / the shared jail.rs machinery) — the
   existing resolve_inputs + pretooluse suites must stay green UNCHANGED after the
   acquire_policy ∘ resolve_with_policy split (design D3/§3, XR-3 one-Topology).
3. **XR-1** — inline `--extra-rw` is raw shell input; jail-prefix must realpath it
   BEFORE the lexical validate_policy, else a `..`/symlink grant widens the sandbox.
4. **RISK-1 / VH-3 (the go/no-go)** — does a long-lived `pi --mode rpc` fifo host +
   child spawn actually function under the SL-183 Seatbelt write-floor? Cleared for
   PHASE-04 via a stand-in; this audit escalates to **real pi 0.80.3**.
5. **OQ-b** — does real pi write `~/.pi` at runtime with the session redirected
   under `$D`? (Determines whether the mac arm needs an `~/.pi` extra_rw grant.)
6. **Fail-closed posture (AR-1)** — any resolve/materialize/write error ⇒ abort the
   spawn, never fall through to an unconfined pi.
7. **Design/plan truth** — does design.md still describe where the code lives and
   what the launcher actually runs (the bash-3.2 `mapfile` deviation)?

**Evidence gathered:** `doctrine check gate` exit 0; bin unit suite 2925 passed / 0
failed (incl. the full resolve_inputs behaviour-preservation set + the three
resolve_with_policy dangerous-extra_rw rejections + resolve_inputs_is_a_recomposition_one_topology_probe);
e2e_worktree_jail_prefix 10/10; `cargo check --target aarch64-apple-darwin` clean
(native on this arm mac — stronger than the off-mac tripwire); a full **real-pi
VH-3 run on this macOS host** (see Synthesis).

## Synthesis

**Closure story.** SL-185 delivers what it scoped: a macOS Seatbelt write-floor
for the subprocess (pi) spawn arm, at policy parity with the SL-183 claude arm,
reusing SL-183's pure builders unchanged. The change is exactly the launcher seam
the design committed to — a new `jail-prefix` command-tier consumer plus a Darwin
branch in `pi-spawn-confined.sh` — with the claude arm behaviour-preserved by
recomposition (D3). The behaviour-preservation gate is met: 2925/0 bin unit tests
green, including the full `resolve_inputs_*` suite unchanged and the
`resolve_inputs_is_a_recomposition_one_topology_probe` proof (XR-3, one Topology).
XR-1 is covered three ways (`resolve_with_policy_rejects_dangerous_inline_extra_rw_{root,ancestor,dotgit}`)
plus the e2e reject; the AR-1 fail-closed `--out` contract and the mapfile-reader
empty-guard are exercised on Linux in the 10/10 e2e suite. `doctrine check gate`
is green (fmt + clippy + build + suites), and `cargo check --target
aarch64-apple-darwin` type-checks the mac branch — natively, since this host is an
arm mac, which is stronger than the off-mac cfg-rot tripwire the design specified.

**The RISK-1 / VH-3 go/no-go, now on real pi.** The design gated all mac-branch
wiring behind RISK-1 (does a long-lived `pi --mode rpc` fifo host survive
`sandbox-exec`?) and cleared it for PHASE-04 via a faithful stand-in + a live
`~/.pi` observation, deferring the full real-pi event stream (VH-3). This audit
closed that residual with **real pi 0.80.3** on this macOS host (Darwin 25.4.0,
arm64), driving the **real shipped launcher path** end-to-end:

- `doctrine worktree jail-prefix --dir <real worker fork> --main-root <root>
  --extra-rw ~/.pi --out …` resolved a real worker topology, materialized the real
  floor `jail.sb` (byte-identical to `seatbelt_profile`), and emitted a working
  16-token NUL-delimited sandbox-exec prefix — **EX-1/EX-2 confirmed live on a mac**,
  beyond the type-check.
- **VH-3 (rpc lifecycle):** real `pi --mode rpc` booted under `sandbox-exec`,
  **stayed alive the whole fifo window** (rc=124 timeout-kill, not early exit), and
  **emitted its JSON-RPC event stream** (`{"id":1,"type":"response",…}` round-tripped
  a frame back). The long-lived fifo-reader + child-spawn lifecycle — the exact
  thing SL-183's short-lived-bash probe never covered — holds against the real
  binary. (The event stream flowing does not depend on a working provider API key:
  the response was pi rejecting a minimal test frame, which is itself proof the rpc
  channel round-trips under confinement. The user notes pi is crashy outside the
  jail pending ENV API keys — orthogonal to these sandbox-mechanics claims.)
- **VH-1 (write-floor):** inside-`$WT` write OK; **outside write DENIED**
  (`Operation not permitted`); outside canary stayed `PRISTINE-OUT`; a spawned
  `/bin/bash` **child inherited** the floor (the denial came from the child).
- **VH-2 (fail-closed, proven by the converse):** the FIRST VH run — invoked WITHOUT
  the `~/.pi` grant — crashed pi on boot with `EPERM: operation not permitted, mkdir
  '~/.pi/agent/trust.json.lock'`. That is the write-floor doing its job, and it
  empirically demonstrates OQ-b.
- **OQ-b settled empirically:** real pi writes `~/.pi/agent/*.lock`
  (`proper-lockfile` mkdir) at boot even with `--session-dir` under `$D` — the
  session redirect covers session storage, NOT the `~/.pi/agent/` settings/trust
  locks. So the `~/.pi` extra_rw grant is **provably load-bearing** (no grant ⇒
  boot crash) and **provably sufficient** (with grant ⇒ clean boot, zero permission
  errors). The shipped `pi-spawn-confined.sh:80-81` Darwin arm **does** pass
  `--extra-rw "$HOME/.pi"`, matching the OQ-b resolution recorded in
  mem.pattern.dispatch.seatbelt-confines-longlived-fifo-worker. EX-3 confirmed.

**Standing risks / tradeoffs consciously accepted.**

- **Two design-truth corrections deferred to /reconcile (per-slice direct edits, F-1
  + F-5).** design.md §3 says `run_jail_prefix` lives in `mod.rs` but it was factored
  into a new `src/worktree/jail_prefix.rs` module (clean improvement); and §1/§3
  prescribe `mapfile -d ''`, a bash-4 builtin absent on the macOS bash-3.2 target —
  shipped code correctly uses a portable `read -d ''` loop that is in fact *stricter*
  than mapfile (it captures the final non-NUL env/TMPDIR token mapfile would drop).
  Both are code-correct / design-stale; neither is spec- or governance-scoped (the
  jail machinery is not spec-registered), so both route to a design.md direct edit,
  not a REV.
- **Conformance is clean after boundary repair:** 4 conformant / 0 undelivered / 6
  undeclared, every undeclared cell dispositioned (jail_prefix.rs → F-1 verified;
  guard.rs / e2e test / IMP-230 backlog → aligned). Zero undelivered is the strong
  signal — everything the design declared was delivered.
- **IMP-230** (pi-spawn reader de-dup vs its e2e-test copy) is open and correctly
  deferred — not a blocker.
- **Harvest scope (accepted):** the P01–03 runtime phase sheets live on the Linux
  dispatch host (sleipnir) and are gitignored runtime scratch that did not travel
  with the branch; the durable substance (RISK-1 verdict, OQ-b, the AR/XR log) is
  already reconstructable from the commit trail, design §8, and the existing RISK-1
  memory. Harvested from those sources per the user's decision; the sleipnir sheets
  are consulted-elsewhere, not lost.

No unresolved blocker. Ledger `done · awaiting=none`.

## Reconciliation Brief

Both non-aligned findings are **per-slice design.md corrections** (code is correct,
design is stale). Neither touches a spec or governance artefact — the seatbelt/jail
machinery is not spec-registered — so both are **direct edits at /reconcile**, no REV.

### Per-slice (direct edit)

- **design.md §3 (F-1) — module location.** The "### `src/worktree/mod.rs` — new
  command-tier consumer" subsection describes `run_jail_prefix()` living in
  `mod.rs`. Reality: `mod.rs` holds only the `WorktreeCommand::JailPrefix` variant +
  dispatch; `run_jail_prefix` and its helpers were factored into a new module
  `src/worktree/jail_prefix.rs`. Update the §3 prose to reflect this, and add
  `src/worktree/jail_prefix.rs` to the §3 **design-target selector list** (currently
  `jail.rs`, `mod.rs`, `pretooluse.rs`, `pi-spawn-confined.sh`) so future
  `slice conformance` runs show it conformant. Optional: also add
  `src/commands/guard.rs` (F-2, the arg-parse wiring) to the selector list for full
  conformance cleanliness.

- **design.md §1 (line 38) + §3 (line 179) (F-5) — reader idiom.** Both prescribe
  `mapfile -d '' PREFIX < …`, a bash-4 builtin ABSENT on macOS (stock bash 3.2.57).
  Correct to the shipped portable idiom
  `while IFS= read -r -d '' tok || [ -n "$tok" ]; do PREFIX+=("$tok"); done`, with a
  note that (a) macOS ships bash 3.2 so `mapfile` is unavailable, and (b) `--out`
  carries no trailing NUL (AR-1), so the `|| [ -n "$tok" ]` clause is required to
  capture the final `env TMPDIR=` token — a bare `mapfile` would silently drop it.
  This is the user-approved deviation; recording it in the design prevents a future
  "restore mapfile" edit from breaking macOS confinement. (Verified live: portable
  reader yields 16 tokens; mapfile would yield 15, losing TMPDIR.)

### Governance/spec (REV)

- None. No spec or governance finding surfaced.

### Also for /reconcile (lifecycle)

- The audit re-recorded the P01–03 source-delta boundaries (runtime state) and ran
  the real-pi VH — these are evidence, already landed in the ledger, no further
  write needed. `notes.md` harvest (durable findings) is the remaining
  reconcile/close-tail task.
