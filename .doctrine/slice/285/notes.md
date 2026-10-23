# Notes SL-285: Fork regenerates derived runtime

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Further review pass (after RV-425 integration, rev 28)

A further pass would probe what RV-425's repairs introduced, not what it
already covered: whether `plan_wire`'s dry-run arms report every path the live
arms write (the one claim the design defers to the implementer), whether the
`installed-version = "999.0.0"` lever reaches `regenerate` without an earlier
refusal, and whether the post-legs `git status` drift check false-reds on any
legitimately tracked write. All three are settled faster by the first phase's
tests than by another design pass, so none is recommended.

## Reconcile carry

Design changes decided during planning, deferred to reconcile by the user
(2026-10-23: "let's note it for integration in the design during
reconciliation"):

- **Uninstalled source → copy only, disclose.** Source with no tracked
  `installed-version`: regenerate writes nothing, prints `derived tier not
  regenerated — doctrine not installed in <source>`, exits 0 — creation and
  repair alike. Integrate into sec-1 (not-regenerated list), sec-3 and sec-5;
  record as a decision (or amend DEC-399). Plan: PHASE-01 EX-6.
- **Absent destination stamp is not a downgrade** (installed source, older
  destination base). Integrate into sec-3's downgrade bullet. Plan: PHASE-01
  EX-3.
- **Regeneration progress goes to stderr** for provisioning callers; machine-read
  stdout of fork / coordinate / candidate create is unchanged. Integrate into
  sec-3 (disclosure channel). Plan: PHASE-01 EX-7, PHASE-02 EX-7.
- **Unreadable stamp refuses** (source or destination): regeneration cannot
  tell installed from not, or verify no downgrade. Integrate into sec-3.
- **Drift check compares bytes, not just paths** (PHASE-01 review F2): a leg
  that rewrites an already-dirty tracked file, or reverts it to clean, is drift
  too — otherwise repair in a dirty tree silently loses an uncommitted edit.
  Integrate into sec-3's tracked-drift bullet.
- **A `Refused` leg outcome halts regeneration like `Failed`.** Integrate into
  sec-3's fatal-legs paragraph.
- **Layering re-tier (departs from sec-4's "no tier edit").** PHASE-02 re-tiers
  `worktree::provision` and `worktree::fork` engine → command in
  `layering.toml`: provision now composes `install::regenerate` and fork calls
  provision — the same upward-edge shape `worktree::coordinate` is already
  command-tier for. All their importers are command tier. Measured: the gate
  passes at `command = 76`, so no tangle bump (sec-4's predicted +1 did not
  materialise). User confirmed 2026-10-23 (QUE-229 answered: keep); integrate into sec-4 and sec-6.
- **Provisioning progress reaches `out` for the boot-wiring leg too**
  (`boot::apply_wire_into`); PHASE-01 left it on stdout (PHASE-02 review F1).
- **Clean-fork fixture ignores `/.codex` itself** (user ruling 2026-10-23, PHASE-03 R4 (b)): the shipped managed `.gitignore` block lacks `.codex/` (and `.doctrine/slice/*/research/`) for new projects — ISS-531, tagged `next`, out of SL-285 scope. EX-2 is proven relative to a clean source.

## F-7 observation (PHASE-03 EX-6, RV-425 F-7)

Observed 2026-10-23: one `--worker` fork minted from the primary tree by the
SL-285 binary (`DOCTRINE_BIN=<SL-285>/target/debug/doctrine worktree fork …
--worker`), probed under the real jail prefix (`doctrine worktree jail-prefix`
→ `bwrap --ro-bind / / … --bind <fork> <fork> --unshare-net --`).

- Ignored `.codex/hooks.json` bakes `<SL-285 tree>/target/debug/doctrine …` (absolute).
  Inside the jail: `test -x` OK; `<baked> prompt resolve --role orchestrator` exit 0.
- Tracked `.claude/settings.json` and `.mcp.json` carry the portable
  `${DOCTRINE_BIN:-doctrine}`; inside the jail it resolves via inherited PATH to
  `~/.local/share/doctrine/bin/doctrine` (0.46.10), exit 0.
- Fork porcelain clean (this repo ignores `/.codex`).

Verdict: resolves — because the Linux jail read-only binds the whole host root,
so every host absolute path is visible at the same location. No backlog item.
Residual (already design sec-6 risk): the baked hook pins the *minting* binary;
rebuilding or deleting that tree changes or breaks the fork's codex hooks, and a
jail that stops binding `/` (or the macOS sandbox-exec arm, not probed) would
need re-checking. VH-1 confirmed by the user 2026-10-23 ("VH-1: confirmed").

## Harvest
- audit: RV-426 (7 findings, all verified; brief in review-426.md)
- follow-ups: ISS-531 (managed gitignore block omits .codex + slice research; next), ISS-532 (boot_harnesses silent skip)
- residual: tests/common `fresh_inputs` leaves a registered git worktree after its TempDir drops (harmless today)
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-10-23 · PHASE-02 · fork sl-285 5727bb85b

### Produced
- design locked rev 31 (6fe624a0a, 3b21595c6); plan + phases (78ac815f9)
- PHASE-01 done on branch sl-285 (.worktrees/SL-285): d7af02edf, 7a17cbce4, a107c0ff2 — gate green
- PHASE-02 done on sl-285: 5727bb85b — provision seam, six callers, compensation, layering re-tier; gate green
- minted: ISS-529 — fork rollback misses binding on non-dispatch branch; ISS-530 — clean candidate leaks on ledger-write failure
- RV-425 concluded; F-1/F-6 routed control (transcribed PHASE-01 EX-4, PHASE-02 EX-3)
- design deltas for reconcile: § Reconcile carry above

### Learned
- mem_01a16847bf3c7d93b0aeb61d27afd338 — pi writable delegates: worktree below root, no toolchain, orchestrator verifies
- mem_01a1680243ea78e383f785c81fff6e07 — apply_wire dry-run reports no touched paths (now superseded by plan_wire)
- mem_01a1680244397e23b4c63de9a0223fad — linked worktrees read main's install marker; ignored hook files bake the absolute exec

### Open
- QUE-229 — answered: keep the re-tier (2026-10-23)
- DEC-402 — regeneration legs + tracked-file guard
- DEC-396 — ProvisionMode single entry
- DEC-397 — creation compensates, repair preserves
- DEC-393 — harness set from source
- DEC-399 — downgrade refusal
- DEC-401 — fatal legs
