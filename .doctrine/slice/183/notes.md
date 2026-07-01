# SL-183 — implementation notes

Durable findings that must survive to `/audit` → `/reconcile` → `/close`.
Phase-local working detail lives in the (gitignored) runtime phase sheets; only
cross-phase / design-affecting facts land here.

## PHASE-01 (confirmation probe) — RV-203 F-2/F-3 discharged

Full evidence: `.doctrine/backlog/risk/014/probe-h2-seatbelt/results.md` §"Pass 3".

### Reconcile at close — design §5.1 profile line correction (F-P3-A)

The design's illustrative §5.1 profile shows the xcrun_db re-allow as a BARE regex:
`(allow file-write* (regex #"/xcrun_db[^/]*$"))`. **Probed bare, it LEAKS** — it
allowed `/private/tmp/xcrun_db-*`, a write entirely OUTSIDE the per-user temp. The
design's own prose already required scoping ("under a DUTMP subpath scope"; "not the
substring `DUTMP/xcrun_db`"), so this is an illustrative-line error, not a decision
change. **Proven-correct shipped form:**

```scheme
(allow file-write* (require-all (subpath (param "DUTMP")) (regex #"/xcrun_db[^/]*$")))
```

- PHASE-02 encodes `require-all` in `seatbelt_profile` + the `XCRUN_DB_REGEX` const.
- At `/reconcile`: update design §5.1's profile line illustrative→proven (bare →
  `require-all`). This is a per-slice-artefact direct edit (design.md), not a REV.

### Over-match to carry into the Rust const comment (F-P3-3)

`xcrun_db[^/]*$` (scoped to DUTMP) allows *any basename beginning `xcrun_db`, at any
depth under DUTMP* — over-matches `xcrun_db_x`, `xcrun_dbEVIL`, nested `…/xcrun_db`.
xcrun writes only at DUTMP top level so it's safe in practice; documented, not
tightened (a literal would break the `xcrun_db-<hash>` atomic-temp family). The
committed cache file is plain `xcrun_db`; the atomic temps are `xcrun_db-<hash>` —
the regex's empty-tail match covers both.

### Cross-slice: SL-182 seam still matches (F-P01-5)

SL-182 moved to `started` with post-lock design commits (`6f97b50e` seam upstream,
`a7707b48` RV-202 `select_jailer` capability-as-data). Confirmed no conflict: SL-182
owns the seam (`Backend::{Bwrap|Seatbelt|Deny{reason}}`, pure `select_jailer`) and
defers the macOS profile body to SL-183. **Re-check SL-182 design at PHASE-02 entry**
— it is still in flight. New constraint to honour: per-arming profile granularity
(serial ⇒ per-worker, parallel ⇒ one shared profile; RV-200 F-1 / RV-202).

### Conformance-boundary note — PHASE-01 source-delta binding absent (accepted)

At `completed`, doctrine warned `record_source_delta: code_start 38ca3a76 is not an
ancestor of code_end c321254c (not a forward delta)` and **skipped** the binding:
`phase-01.toml` keeps `code_start_oid = 38ca3a76` and has **no `code_end_oid`**.

- Cause: `code_start` (38ca3a76 "mem(SL-183): network-field-is-bool") was stamped on
  a lineage later discarded when the `f3539349`/`133880a2` "doctrine" auto-commits +
  parallel SL-182 landings restructured history. 38ca3a76 is now orphaned (in no
  branch, not an ancestor of HEAD). HEAD `c321254c` (edge tip) is forward-intact and
  the probe evidence (`results.md`, the rig) is fully reachable/committed.
- Decision (consulted, David — Option 1): **accept the absent binding**. PHASE-01
  ships NO Rust — its conformance value is the evidence in `results.md`, not a source
  delta. History-repair is forbidden (doctrine tracks oids as the boundary; AGENTS.md
  / handover). PHASE-02+ stamp `code_start` fresh from HEAD, so the anomaly does not
  propagate. At `/audit`: note PHASE-01 has no git-range delta by design of a code-free
  probe phase; rely on evidence-conformance, not delta-conformance, for it.

## PHASE-02 (pure builders) — implemented, gate blocked on SL-182

`seatbelt_profile` + `sandbox_exec_argv` implemented TDD behind SL-182's `Seatbelt`
seam; `Seatbelt::wrap_argv` wired to the builder. **41 jail unit tests green**
(31 SL-182 behaviour-preserved + 10 new SL-183). Clippy clean.

### Seam-gap closed: ResolvedMac fields (sanctioned by its doc comment)

SL-182 landed `ResolvedMac {}` EMPTY. PHASE-02 populated it: `wt`, `tmp`, `dutmp`,
`extra_rw`, `network: bool`, `profile_path` — all shell-canonicalized (PHASE-03's
`resolve_inputs` fills them; the pure builders consume them). Kept `#[derive(Default)]`
so SL-182's `ResolvedMac {}` test constructors compile unchanged (behaviour-preserved,
verified). No SL-182 signature/body change.

### D2 (TMPDIR) resolved seam-preservingly

Proven `seatbelt-jail.sh` exports TMPDIR *inside the wrapped body*; `opaque_wrap`
(shared, bwrap+seatbelt) must stay unchanged. So `sandbox_exec_argv` emits a trailing
`env TMPDIR=<tmp>` token after `--`; `opaque_wrap` appends `bash -c <body>` after that.
`opaque_wrap` untouched → PHASE-04 parity proof intact.

### F-P3-A encoded

`XCRUN_DB_REGEX` const + `seatbelt_profile` emit the `require-all (subpath (param
"DUTMP")) (regex …)` scoped form, NOT §5.1's bare regex. Over-match caveat is in the
const's doc comment. §5.1 reconcile-at-close debt (bare→require-all) still stands.

### BLOCKER — full gate red on a PRE-EXISTING SL-182 CHR-014 violation (ISS-204)

`doctrine check commit`'s full `test` recipe fails `e2e_no_baked_paths::no_baked_paths`
(CHR-014 / SL-162): SL-182's `pi_spawn_core_tokens` VT-7 helper bakes
`env!("CARGO_MANIFEST_DIR")` (introduced by SL-182 `b67b6299`, verified at clean
detached-HEAD — NOT an SL-183 artifact). Consulted (David): **SL-182 is being actively
worked in a parallel thread; the fix was handed to that thread.** SL-183 must NOT edit
jail.rs's SL-182 test surface (conflict + ownership). Captured as **ISS-204**
(`references SL-182 --role concerns`).

**Consequence for PHASE-02 close:** the pure builders are green in isolation, but
PHASE-02 must NOT flip `completed` until the full gate is green (else `code_end_oid`
binds to a red-gate state). **HOLD the completed-flip on ISS-204.** Commit the green
builder work now (durable); flip `completed` + re-run the gate once SL-182's thread
lands the fix. If SL-182's fix touches jail.rs concurrently, expect a rebase/merge on
this file — my additions are append-only (new consts block, new `ResolvedMac` fields,
two new fns, new tests), so conflicts should be localized.

### RESOLVED 2026-07-01 — ISS-204 fixed in SL-183 context (ownership deviation, sanctioned)

The SL-182-thread fix never reached the `edge` checkout (verified: no other worktree,
clean tree, `main` behind, gate re-run still red). On explicit direction (David, "try
now"), the ISS-204-mandated swap was applied HERE: `env!("CARGO_MANIFEST_DIR")` →
`crate::test_support::repo_root().join("scripts/pi-spawn-confined.sh")` in
`pi_spawn_core_tokens`. Test-only, behaviour-identical, no jail logic touched
(`8abcaae0`). Full `doctrine check commit` EXIT=0; `no_baked_paths` guard + 41 jail
unit tests green. **Deviation from the original "SL-182 owns this surface" note is
deliberate and user-sanctioned** — if the SL-182 thread also lands a fix, expect a
trivial dup/conflict on this one line (same target form). ISS-204 → resolved/fixed.

**PHASE-02 flipped `completed`** (`code_start 3a760f92`, forward-intact; conformance
confirms the PHASE-02 source-delta row registered — only PHASE-01 remains registry-gap,
accepted as a code-free probe phase per the boundary note above).

### Probe hygiene notes

- Every `(param "X")` the profile references MUST have a `-D X` bound or
  `sandbox-exec` refuses to load (`invalid data type of path filter; expected
  pattern, got boolean` — misleading text; it's an undefined-param fail-CLOSED).
- `-D DUTMP` MUST be the realpath (`/var/folders/$USER/T` → `/private/var/folders/…`);
  `subpath` matches the resolved path (INV-M2).
