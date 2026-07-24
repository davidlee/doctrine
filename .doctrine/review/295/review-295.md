# Review RV-295 — reconciliation of SL-208

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Reviewed surface.** SL-208 is a dispatched slice. Audit ran against the
candidate interaction branch `refs/heads/candidate/208/review-001`
(cand-208-review-001, tip c7108b0), a `review_surface`/`impl_bundle` projection
of the immutable impl-bundle evidence ref `review/208` (b325bc3a) onto
`refs/heads/main`. The raw `review/*` + `phase/*` refs are evidence (R2), not the
review target. Built and exercised in the provisioned candidate worktree.

**Lines of attack.**

1. **D2 info-contract fidelity.** The whole slice exists to replace clap's ragged
   flat help with a cozy-table. The RFC-011 case notes (`SL-208 sess-b`) flagged
   three PHASE-01 deviations from design D2 that no VT keyword catches (VTs assert
   structural tokens, not clap-contract fidelity): usage dropped the `doctrine
   <path>` prefix; the global `--color` vanished from every subcommand's Options
   (clone-of-unbuilt drops ancestor globals); local args dropped
   `[default:]`/`[possible values:]`. Probe the *rendered output*, not just the
   green VTs, to confirm the folded-into-PHASE-02 fixes (G1/G2/G3) hold.
2. **Conformance algebra.** `slice conformance` undeclared/undelivered cells vs the
   design-target selector set (`src/main.rs`, `src/commands/cli.rs`; `src/listing.rs`
   scope-relevant). Every undeclared path is a lead: designed-but-unselected, or
   genuine creep.
3. **Behaviour preservation.** Top-level `--help` unchanged; `MissingSubcommand`
   still error-exits (a scriptless `doctrine worktree` must not silently succeed);
   the `write_class_tests` (dispatch identity) stay green.
4. **Plan/design truth.** The plan.toml VT control-char hazard (case notes) and any
   internal design-artifact contradictions.

**Invariants held.** `--color never` ⇒ zero ANSI (box-drawing `│` preserved);
leaf commands skip the Commands table but keep borderless Options; the renderer
walks the clap *tree* (`find_subcommand`), never raw argv (so `--color never`'s
`never` is not mistaken for a subcommand).

## Synthesis

SL-208 lands the subcommand-level cozy-table help cleanly. The delivered
impl-bundle realizes design D1–D4 in full, and the audit's central risk — that
the mechanical VT gate is a *presence floor*, not a contract-fidelity check
(RFC-011 case notes) — was retired by an empirical render probe against the built
candidate binary rather than by trusting green VTs. The three D2 info-contract
deviations that PHASE-01 shipped (usage-path prefix, global `--color`, arg
annotations) were caught by the orchestrator and folded into PHASE-02; the probe
confirms G1/G2/G3 all hold in the delivered artifact. `doctrine worktree --help`
now renders about → full-path usage → `│`-separated Commands table → borderless
Options carrying `[default:]`/`[possible values:]`; the global `--color` is
present; a leaf (`onboard --help`) drops the Commands table and keeps Options;
top-level `--help` is byte-for-byte unchanged; and `doctrine worktree` (no verb)
still error-exits (`MissingSubcommand` preserved). Full gate green on the
candidate: clippy `--bin` zero warnings, fmt clean, 3779 unit + 8 e2e green, 14/14
VTs PASS, `write_class_tests` (dispatch identity) untouched.

**Closure story.** No blockers. Four findings, all terminal: two `aligned`
(F-3 the SL-150 `related` edge is the *correct* semantics vs F-5's imprecise
"supersedes"; F-4 D2 fidelity confirmed), and two `verified` doc/registry-truth
gaps routed to reconcile (F-1 selector registry under-enumerates designed paths;
F-2 a contradictory `textwrap`-transitivity claim in design F-3). Both reconcile
items are per-slice truth edits, not code — the code is conformant.

**Standing risks / tradeoffs consciously accepted.** (1) The conformance signal
is a *where-to-look* aid, not a pass/fail: its undeclared cell here was pure false
positive (designed-but-unselected paths), which is exactly why F-1 routes to a
registry edit, not a code change. (2) The class of defect the case notes name —
a renderer satisfying every VT keyword while silently dropping clap-contract
semantics — is a general gap in VT-as-verification, captured for RFC-011; this
slice closes it by hand-probe, but the systemic affordance is out of scope here.

## Reconciliation Brief

### Per-slice (direct edit)

- **F-1 — selector registry (load-bearing) + design §5 mirror.** Conformance
  reports `Cargo.toml`, `Cargo.lock`, `tests/e2e_subcommand_help.rs` as undeclared
  though all three are designed (§5 declares the `textwrap` direct dep; §6 declares
  the e2e suite). Fix at the registry, which `slice conformance` reads:
  `doctrine slice selector add SL-208 Cargo.toml --intent design-target`,
  `doctrine slice selector add SL-208 tests/e2e_subcommand_help.rs --intent design-target`
  (and either add `Cargo.lock` or note it as the lockfile consequence of the
  Cargo.toml selector). Then mirror the added targets in design.md §5 Code Impact
  prose (the human mirror — prose alone leaves conformance red). `slice-208.toml`
  needs no selector (the slice's own authored file; benign churn).
- **F-2 — design.md §7 F-3 truth edit.** Correct the F-3 parenthetical from
  "`textwrap` is already in the dependency tree (transitive via other crates)" to
  state textwrap is added as an explicit **direct** dependency (matching D2, §5,
  and the delivered `Cargo.toml`). Doc-truth only.

### Governance/spec (REV)

None. No ADR/policy/standard/spec/requirement drift surfaced — SL-208 is a
UI-formatting slice with no governance targets (plan `[specs]`/`[requirements]`
empty).

### No-action (recorded for closure)

- **F-3 (aligned)** — the `related` SL-150 edge is correct; do **not** "fix" it to
  `supersedes`.
- **F-4 (aligned)** — D2 fidelity confirmed; no change.

## Reconciliation Outcome

### Direct edits applied
- **Selector registry (F-1, load-bearing):** `doctrine slice selector add SL-208
  Cargo.toml Cargo.lock tests/e2e_subcommand_help.rs --intent design-target`.
  `slice conformance SL-208` now reports undelivered=0, conformant=5, and the only
  undeclared residual is `slice-208.toml` — the slice's own authored file, which is
  self-referentially churned by recording the selectors and therefore can never be
  conformant (the expected benign residual F-1 identified).
- **design.md §5 (F-1 mirror):** added a "Selector manifest (design-target)"
  subsection recording the selector set so the prose mirrors the registry.
- **design.md §7 F-3 (F-2):** corrected the contradictory parenthetical —
  `textwrap` is now stated as an explicit **direct** dependency (not "already
  transitive"), matching D2/§5 and the delivered `Cargo.toml`.

### REVs completed
None — no governance/spec items in the brief.

### Withdrawn / tolerated
None. F-3 and F-4 were `aligned` (no-action, recorded above); F-1 and F-2 were
`verified` and are now reconciled by the direct edits above.

Reconcile pass complete — handoff to /close.
