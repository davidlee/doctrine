# Review RV-350 — reconciliation of SL-250

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed.** The solo `edge` branch at `ccd458d13`, not a dispatch
candidate — SL-250 ran `/execute`, not `/dispatch`, so there is no
`review/*` evidence ref and no candidate interaction branch. Working tree
clean of SL-250 content (the three untracked paths belong to CHR-059 /
IMP-410 and another agent's `scripts/find-empty-reqs.sh`).

**Mechanical evidence.** `doctrine check gate` exit 0 (clippy workspace zero
warnings, full suite green). `doctrine slice conformance 250`: 27 undeclared,
1 undelivered, 12 conformant — read with the pollution caveat in `F-5`.

### Lines of attack

1. **Did the design's own unresolved probes get resolved?** RV-348 round 2
   closed with five named probes the *design* could not answer. Probes 1
   (`SweepReport`'s three fields against the seven-spec loop, and `dry_run`),
   2 (the `CommandForm` wire end to end), 3 (`install_baseref` reading a
   malformed sibling) and 4 (the `F-12` guard under `--dry-run`) are
   implementation questions. Each is re-run here against landed code.
2. **Is the reporting mechanism honest at file granularity?** The slice spends
   `sec-3` arguing that a destructive write must never be silent and that no
   fold state may be absorbing. The inverse obligation is that no rider line may
   overclaim. Probe every reachable combination of `SweepReport`'s three fields
   for a pair that contradicts.
3. **Do the eleven entries and the five events hold on a real install, and is
   `VH-1`'s interactive-only evidence re-derivable by an auditor?** The slice's
   whole thesis is activation that does not fail silently; a criterion whose
   evidence lives in one agent session would reproduce the defect at the
   governance layer (`mem_019fd1d862887d42b7a1f88c28fd28a7`).
4. **Is the conformance signal trustworthy?** The audit skill treats the
   undeclared cell as the highest-signal lead. Establish whether each entry is
   this slice's before treating any of them as scope creep.
5. **What does the slice leave asserting something false?** Three classes:
   the governing spec (`REQ-186`), the durable memory corpus, and `design.md`'s
   own prose where implementation forced a departure. The slice's thesis is that
   a silent falsehood is worse than a loud gap, so this is the lens it invites.
6. **Did the retirement's transitive closure actually close, and did the
   deliberately-surviving artefacts survive?** `plugins/`,
   `plugins/doctrine/hooks/hooks.json`, `.claude-plugin/marketplace.json`,
   `SpawnSeamSymmetry`.

### Invariants the slice is pinned to

- **SL-195 `INV-1`** — no absolute host path in any tracked file. Now covering
  hooks as well as `.mcp.json`; `baked ⟺ gitignored` in both directions.
- **Never-clobber** — including on the new destructive path. Eviction is gated
  by exactly the predicate that protects foreign entries during a merge.
- **ADR-001 layering** — `install_config` stays a pure leaf at out=0.
- **POL-002** — no host-project convention in the engine; no per-machine value
  in a tracked file.
- **Behaviour preservation, class (i)** — the Codex arm byte-identical,
  `check_spawn_seam_symmetry_passes_the_shipped_hooks_config` unmodified.
- **`DEC-167`'s ordering** — order toward the degraded state, never the absent
  one. Absence is the dangerous state because `isolation: worktree` teardown is
  conditional on `WorktreeCreate` firing.

## Synthesis

### The closure story

SL-250 set out to move Claude activation off a substrate that fails silently at
three independent trust layers and onto one the project can inspect. It did
that, and the evidence is stronger than the criteria demanded: eleven settings
entries across five events, every one labelled `[Project]`, on a cold install in
a scratch project where the operator had **deregistered the doctrine marketplace
from their host config first** — so the plugin path was not merely unused, it
was unavailable, and the eleven entries cannot have come from it. All three
`VH-1` observed effects landed, including the two that mattered most:
`WorktreeCreate`, because `isolation: worktree` teardown is conditional on it,
and `PreToolUse`, which took three probes and a doctrine-free control to
confirm because two obvious probes were each blind to it in a different way.

The gate is green (`doctrine check gate` exit 0, clippy workspace zero
warnings, full suite). The behaviour-preservation split held: class (i) — the
Codex arm byte-identical, `check_spawn_seam_symmetry_passes_the_shipped_hooks_config`
unmodified, `plugins/doctrine/hooks/hooks.json` untouched — and class (ii)
confined to three test files (`e2e_claude_install.rs`, `e2e_memory_sync.rs`,
`e2e_skills_symlink.rs`), each named by a phase criterion. The deletion closure
closed: no `MarketplaceSource`, `RegisteredSource`, `MarketplaceAction`,
`DOCTRINE_MARKETPLACE` or `--dev` survives in `src/`, and the workspace's
`dead_code` denial — not a hand-derived list — is what proved it.

Eleven findings, all terminal, none contested, no blocker. Four were fixed
inside the audit; four routed to owned backlog items; three go to
`/reconcile` as brief items.

### What the audit actually bought

Three of the four fixed findings would not have surfaced from reading the
notes, and one of them is the audit's real yield.

**`F-3` closed the last of RV-348's five design-time probes.** Round 2 ended
with the raiser naming five things the design could not answer and only an
implementer could. Probes 2, 3 and 4 were discharged in flight. Probe 1 —
*"probe for a state where two lines print and contradict each other"* — was
not, and the contradiction was there: `removed` and `skipped` are **both**
per-spec facts folded across seven specs, but only the `unreadable` line was
worded partially. One wrongly-typed event array in the target file, and the
operator reads *"evicted 1 stale hook entry from `settings.local.json`"*
immediately above *"did not sweep `settings.local.json`"*. The mechanism was
right and the sentence attached to it overclaimed by one degree — which is
verbatim the pattern the RV-348 raiser named across both rounds, surviving into
the one place neither round could reach.

**`F-2` found a second falsified memory by following the first one's link.**
The notes flagged `mem.pattern.distribution.skill-refresh-command`. Its sibling
`mem.pattern.distribution.skills-source-vs-installed` held the stronger claim —
a July correction declaring the `.doctrine/skills/` mechanism *"dead"* and the
directory nonexistent — and would have been the more damaging of the two to
leave standing, because it is scoped to the same file and reads as
authoritative precisely for having been corrected once already.

**`F-5` is the one that should change how the next audit is run.** The
conformance report's undeclared cell — the highest-signal lead the audit stage
has — was **20/27 foreign**. Concurrent agents' commits on the shared `edge`
branch land inside this slice's phase deltas, and the capture even absorbed a
release-bump edit to `.claude-plugin/marketplace.json` into a range `PHASE-06`
`EX-5` requires untouched. `EX-5` holds; the report cannot say so. At that
ratio the cell does not merely widen, it **inverts**: an auditor who treats it
as instructed raises findings against other slices' work.

### Standing risks

**The host repo has not cut over** (`F-8`). `.claude/settings.json` still
carries `enabledPlugins` and no `hooks` key. `VH-1` is fully discharged — it
demands a *scratch* project by its own preconditions — but `sec-6`'s separate
claim, that the first post-slice install here is where SL-195's `INV-1` gets
exercised on hooks *in git* for the first time, is untested. This is a two-act
operator cutover under `DEC-167` and **the order is load-bearing**: install
first, then disable the plugin. The reverse leaves an interval with no
activation, and absence is the dangerous state.

**`REQ-186` is live and false** (`F-1`) until the REV lands. A governing
requirement that describes a `<exec> boot` hook in `settings.local.json` is not
inert documentation — it is what a future agent will read as the contract.

**Un-policed drift between `claude_hook_specs` and the published
`hooks.json`.** Already live, `R6` withdrawn on it, and IMP-407's Leg 2. The
registry exists as a single function specifically to make that check cheap; it
has not been built.

**`install_baseref` on a malformed sibling is silent.** `plan_baseref` returns
`PrintedFallback`, so `stranded` is `None` and no advisory prints. Acceptable
as it stands — the hook leg's `unreadable` rider fires on the same file in the
same run — but the coupling is incidental rather than designed, and it would
break if the two legs ever diverged on which file they read.

### Tradeoffs consciously accepted

- **Seven `doctrine.toml` reads per install.** `DEC-163` resolves the scope
  inside `install_claude_hook` rather than passing it, because the second caller
  is `memory sync install` and a parameter it could forget is the defect. The
  cost is real and correctly priced.
- **A mid-loop `?` leaves a half-activated install straddling both files.** The
  `with_context` names the spec and the count, and the operator's only
  obligation is to re-run. A partial `RefreshReport` would buy a type for a
  condition with an unconditional remedy.
- **`worktree.baseRef` is read but never swept.** Extending a destructive path
  to non-hook keys is the wrong trade; no-clobber survives and the directional
  advisory is the whole signal.
- **Windows is out of scope.** `${VAR:-default}` is POSIX parameter expansion
  and `PreToolUse` hooks fail open, so it would degrade silently there. Stated
  as a boundary, not engineered around.
- **`strictPluginOnlyCustomization` is acquired, not shed.** Direct-write drops
  two plugin-only failure modes and gains one. The shipped doc says *fewer
  failure modes, not a subset* — the honest framing, and it survived into
  `install/claude-activation.md`.

## Reconciliation Brief

Three surfaces. `/reconcile` writes the first two; the third is a close act
with no write.

### Governance/spec (REV)

- **`REQ-186` (SPEC-011, `FR-006`, status `active`) → REV.** Current text:
  *"`boot install` merges a `<exec> boot` SessionStart hook into Claude
  settings.local.json, refreshing a stale owned copy and preserving every
  foreign hook and key."* False on four axes against shipped code:
  1. **Not one hook** — seven specs emitting eleven entries across five events
     (`claude_hook_specs`, `src/boot.rs:1255`; distribution asserted by
     `install_wires_eleven_hook_entries_across_five_events`,
     `tests/e2e_claude_install.rs:285`).
  2. **Not `settings.local.json`** — a scope-selected file defaulting to
     `.claude/settings.json` (`settings_rel`, `:558`; `ClaudeSettingsScope`
     defaults `Project`).
  3. **Not `<exec>`** — the committed scope writes `PORTABLE_EXEC`
     (`command_form`, `:574`), on SL-195's `baked ⟺ gitignored`.
  4. **A new obligation the requirement does not mention** — the
     abandoned-scope sweep, gated on the target write landing
     (`evict_hook_from_file`, `:2210`; `install_claude_hook`, `:2076`).
  `QUE-209` — widen `REQ-186`, or add new requirements for the hook set and the
  scope key — is **deferred to the REV author** by `design.md` `sec-7`
  Governance and is not settled here.
- **SPEC-010 does NOT enter the REV.** `DEC-171` made it a conformance claim.
  `PHASE-05` `VA-1` discharged responsibilities 3–6 and this audit re-derived
  each by reading: canonical tree `skills_canonical_dir` (`src/install.rs:1994`)
  → `install_skills_direct` (`:2064`); relative symlink via `claude_skills_dir`
  (`:1999`) → `reconcile_link` (`:1677`), classified by value equality;
  `.tmp-<id>` staging with remove-then-rename and `symlink_metadata`-not-`exists`
  in `materialise_canonical` (`:2013`); gitignore self-enforced by
  `install/manifest.toml:46` with no `ensure_gitignored` call, as `EX-7`
  requires. Observed live: 35 symlinks in the `VH-1` capture.
- **RFC-018** takes the empirical harness findings — the
  `strictPluginOnlyCustomization` asymmetry, and the two memories `VH-1`
  produced (`mem.fact.claude.native-worktree-isolation-is-tool-layer-only`,
  `mem.fact.worktree.gitdir-pointer-unresolvable-in-sandboxed-subagent`).

### Per-slice (direct edit)

All of these edit `design.md` only. **None edits `plan.toml`** — `PHASE-NN` and
`EN-/EX-/VT-` ids are immutable-append, so where a criterion mis-cites, the
amendment states the rule in prose rather than rewriting the criterion.

1. **`sec-2` — `command_form` placement.** `sec-2` sites
   `ClaudeSettingsScope::command_form()` in `install_config`. It ships as
   `boot::command_form(scope)` (`src/boot.rs:574`). ADR-001 classifies
   `install_config` leaf at out=0 and `boot` as command tier, so the method
   would be a tier inversion *and* a cycle (`boot` already reaches
   `install_config` through `dtoml`). This is `sec-3`'s own argument for siting
   `settings_rel` in `boot.rs`, applied to the sibling mapping. `PHASE-02`
   `VA-2` made the layering gate a criterion. Nothing else in `sec-2` moves.
   Amend the sentence to match; the shipped doc-comment already carries the
   reasoning and a *do not tidy this back* warning.
2. **`sec-3` — the stranded-`baseRef` message is directional.** The design's
   line asserts *"local settings override project settings, so it still
   governs"*, which is true only when the abandoned file is the **local** one.
   Under scope `Local` precedence runs the other way and the stranded value is
   inert. `stranded_baseref_line` (`:1860`) renders both. Amend to show the
   pair, or state a `Project`-only scope.
3. **`sec-3` *Reporting shape* omits `BaseRefWrite`.** `install_baseref`
   (`:1805`) returns `BaseRefWrite { outcome, stranded }` and
   `RefreshReport.baseref` takes it; the design shows a bare `BaseRefOutcome`.
   Bundling was chosen over a separate inspector so the second fact is
   unforgettable at the call site — the same reasoning `DEC-163` used to
   resolve the scope inside the installer.
4. **`sec-3` — `parse_settings` has a third consumer.** `EX-1` shared it
   between `plan_hook` and `plan_evict`; `plan_baseref` carried the
   byte-identical prologue and now rides it too, with five `PrintedFallback`
   literals collapsed into `baseref_fallback`.
5. **`boot::claude_scope` renamed `configured_scope`** (`:549`), to stop
   colliding with `RefreshReport.claude_scope`, which means a different thing.
6. **`sec-2` — `fallback_for`'s output shape changed.** `EX-4` makes the repair
   snippet render the whole matcher set, so it is now a JSON **array** for N=1
   as well as N>1. User-visible on both install paths and asserted nowhere (the
   sole assertion is `!snippet.is_empty()`). Say so.
7. **`sec-3` — the sweep rider's `skipped` line was reworded at this audit**
   (`F-3`). It now reads *"left some doctrine hooks in `<abandoned>` — they
   could not be written to `<target>`, so there is nothing to replace them
   with"*. Both flags are per-spec facts folded across seven specs, so both
   lines are worded partially. `design.md` `sec-3` *The announcement* shows the
   old file-scoped text; update the specimen. `EX-7`'s intent is unchanged.
8. **`sec-7` — the class-(ii) enumeration was incomplete three times.**
   `PHASE-04` `F1` (`wire_adds_import_and_hook_then_is_idempotent` carried the
   `EX-5` assertion twice, unenumerated); `PHASE-05` `F1`
   (`tests/e2e_claude_install.rs` carried the same now-false assertion as
   `e2e_skills_symlink.rs`, named in neither `sec-5` nor `EX-8`); `PHASE-06`
   `F1` (`EX-4` over-broad by three and short by one — the
   `install_config.rs` `repo` **field** doc, distinct from the module doc).
   Plus `PHASE-03`: every `src/boot.rs` line citation in its authored criteria
   was stale after `PHASE-01`'s renames, with every cited **fact** re-verified
   and holding.
   **Add two general rules**, not four more list entries:
   - a criterion's **line numbers are advisory; its named symbols are binding**;
   - an **enumerated inventory is a starting set** — close it by running the
     full suite and grepping the *class* of assertion, never the cited paths.
   The audit adds a fifth data point in the same direction: `F-6`'s two
   undeclared surfaces are named by criteria in the *last* phase and were still
   missed, because the `review.selectors` pass runs once at design time and
   cannot see the surfaces later phases acquire.
9. **`sec-1` *Code impact* — the `src/install.rs` row is short one change.**
   It describes `reconcile_link`, the restored skills channel and the plugin-step
   removal. `PHASE-04` also fixed `install_harnesses` / `boot_harness_names`
   (`src/install.rs:271`, `:298`): `run_forward_steps` and
   `print_forward_summary` both resolved the boot leg with
   `resolve_harnesses(&[], root)`, discarding the operator's `--agent`, so
   `doctrine install --agent claude` in a fresh project wired no `@`-import, no
   hooks and no `.mcp.json` and reported it as routine detection output. This is
   an **addition** to the row, not an instance of it — and it is the slice's own
   thesis one layer up. Consider whether IMP-407's diagnostic leg has a sibling
   here (the install verb should be able to say *why* it wired nothing).
10. **Constant-list partitioning across phases** (`D1`). `Cargo.toml:175` sets
    `warnings = "deny"`, so a constant or field with no consumer is a hard
    error. `PHASE-01` `EX-8`, `PHASE-02` `EX-2` and `PHASE-04` `EX-1` partition
    the constant list; `PHASE-02` → `PHASE-03` partitions
    `ClaudeSettingsScope::sibling()` and `RefreshReport.claude_scope` the same
    way. Net result: the slice carries **zero** outstanding `dead_code`
    allowances. Worth stating as the general phasing rule it is.
11. **`PHASE-02` `EX-7` enumerates tests, not assertions.** At three of its
    eight sites the *command literals* flip too, and the create-fork test has a
    fourth such assertion no cited line reaches. The enumeration held only
    because it was read by name.
12. **`VT-2`'s two-spec fold is asserted through `WorktreeCreate`,** not the
    `SubagentStart` the criterion's example names — `HookSpec::nominate` does
    not exist until `PHASE-04`. The behavioural claim is asserted exactly; only
    the event name differs.
13. **New, design-silent:** `wire`'s Claude arm announces the scope even when
    zero specs merged (`PHASE-03` wired `write_scope_report`; `PHASE-04`
    supplied the specs). Honest rather than vacuous — `install_baseref` writes
    to the very file the line names — and `EX-7` makes the announcement a
    criterion. Recorded so it is not read as a leak.

### Closed inside the audit (no reconcile action)

- **`F-2`** — `mem.pattern.distribution.skill-refresh-command`
  (`mem_019eae55811f7412b11559068fe8a279`) and
  `mem.pattern.distribution.skills-source-vs-installed`
  (`mem_019eacd4b2af7a7099792cc3e1671cc5`) corrected in place. Both asserted
  `.doctrine/skills/` does not exist and nothing relinks `.claude/skills/<id>`;
  both now record the SL-250 restoration and split their verification advice by
  channel (claude direct vs the release route).
- **`F-3` / `F-4`** — `write_scope_report` (`src/boot.rs:2162`): the `skipped`
  line reworded to per-spec scope, and `{tag}` added to the two lines that
  omitted it. New test `a_skipped_sweep_does_not_contradict_a_successful_eviction`;
  `the_skipped_line_names_what_it_did_not_replace` updated for the new phrase.
- **`F-6`** — `publication/manifest.toml` and `tests/e2e_skills_symlink.rs`
  added as `design-target` selectors. Conformance: undeclared 27 → 25,
  conformant 12 → 14.

### Routed to owned backlog (no reconcile action)

- **`F-5` → ISS-307**, with IMP-175 and IMP-282 named as the other two faces of
  one problem. Evidence appended: the 20/27 attribution, the absorbed
  `marketplace.json` release bump, and the argument that the cell *inverts*
  rather than widens.
- **`F-7` → IMP-406.** `install_skills_direct` prints `agent claude (direct):`
  from a body `EX-3` argues is not Claude-specific — shared by `design.md`
  `sec-5`'s own code block, latent at one link dir, wrong at two.
- **`F-8` → IMP-400 `OQ-4`.** This repo's own cutover, as a two-act operator
  action in the `DEC-167` order.
- **`F-9`** — carried to close: **CHR-045** (resolve or explicitly retain;
  `just sync-plugin-versions` inside `just release` already derives all five
  manifests from the Cargo version, so retain-as-fixed is the likely call) and
  **IMP-234 / CHR-037** (assessed for overlap — no assessment recorded).

### Undelivered, and correctly so

`slice conformance` reports `.doctrine/spec/tech/011/**` **undelivered**. That
is the REV target above: it is reconcile's write, not a phase's. It should
clear when the REV lands.
