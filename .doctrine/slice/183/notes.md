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

### Probe hygiene notes

- Every `(param "X")` the profile references MUST have a `-D X` bound or
  `sandbox-exec` refuses to load (`invalid data type of path filter; expected
  pattern, got boolean` — misleading text; it's an undefined-param fail-CLOSED).
- `-D DUTMP` MUST be the realpath (`/var/folders/$USER/T` → `/private/var/folders/…`);
  `subpath` matches the resolved path (INV-M2).
