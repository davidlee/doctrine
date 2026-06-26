# prepare-review gate couples projection to phase-completion

The SL-154 PHASE-05 completeness gate at `dispatch::prepare_review` (before the ref
projection) calls `state::registry_completeness(&primary, &primary, slice)`, which
reads `completed_phase_ids(primary)` — i.e. each phase's runtime tracking file
`.doctrine/state/slice/NNN/phases/phase-NN.toml` with `status == "completed"`.

After the PHASE-05 **derive** the registry mirrors the committed boundaries ledger,
so a committed/dispatched phase that is **not** marked `completed` in the primary
tree reads as an `Extra` gap → the gate halts. **prepare-review is therefore a
post-completion beat** — the *pre-audit conclude beat* (design §5.2, lines 119/159),
not a mid-drive projection. This is intended, not a bug.

## The trap it sprang (and how to avoid it)

Wiring the gate broke **27 existing `e2e_dispatch_sync` fixtures**: they seeded a
boundaries ledger + projected, but never marked phases `completed`. Two fixes,
both now baked into `build_fixture` / `build_fixture_uncommitted_ledger`:

- **Seed completion.** `seed_completed_phases(dir, slice, &["PHASE-01", ...])` writes
  `phase-NN.toml` = `status = "completed"` under the primary `.doctrine/state/`. Any
  dispatch test that drives `prepare-review` must seed the ledger phases completed.
- **Gitignore the runtime tier.** The derive now **writes the registry**
  (`.doctrine/state/slice/NNN/boundaries.toml`) during prepare-review; a fixture repo
  with no `.gitignore` shows it (and the seeded tracking) as untracked → dirties
  `git status --porcelain`, breaking the `integrate` tests' clean-tree asserts. Fixtures
  must `.gitignore .doctrine/state/` (mirroring production) at the base commit.

The **behaviour-preservation clause** (design line 154) names specific shared seams
(`set_phase_status` solo path, `worktree_for_ref` callers) — **not** these projection
fixtures, which are the direct subject of the change. So the fixture upgrade is
in-scope, not a preservation violation.

## Also

- The guard predicate's committed-set is a `BTreeSet<&str>`, not `HashSet` —
  `std::collections::HashSet` is clippy-disallowed in this repo (determinism).

Cousin of [[mem.pattern.dispatch.prepare-review-rerun-not-idempotent-until-gate]]
(the gate is what makes the re-run clean) and
[[mem.pattern.state.reopen-evict-degrades-self-heal]] (the binding-side registry
write degrades; the prepare-review derive/gate by contrast `?`-propagate — they are
the funnel conclude beat, allowed to hard-fail). Born SL-154 PHASE-05, anchor `d9892674`.
