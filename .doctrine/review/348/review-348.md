# Review RV-348 — design of SL-250

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Subject: `.doctrine/slice/250/design.md` at design run rev 39, stage
`reviewing` — seven sections, all `review=outstanding`. Reviewed as a **design**
artefact, but held against the code it makes claims about: a design that argues
*by construction* has to be checked *against the construction*.

**Fenced off — not probed, by instruction.** DEC-161, DEC-162, DEC-163, DEC-164,
DEC-166, DEC-167, DEC-171 are settled premises. `QUE-209` (REV widens `REQ-186`
vs adds requirements) is deferred to reconciliation. Migration of existing
`enabledPlugins` / marketplace registrations is a Non-Goal (IMP-400 carries it).
The doctor leg is IMP-407's (with `R2`, `R7`, `OQ-5`). A finding that a decision
is *wrong* is out of scope; a finding that the draft **misrepresents** one, does
not follow from one, or contradicts itself section-to-section is in scope.

### Invariants the draft is pinned to

1. **Never-clobber.** Only doctrine-owned hooks are ever dropped — on the write
   path and, newly, on the destructive sweep.
2. **No silent partial activation.** The slice exists because the plugin channel
   fails silently at every layer. Any state the draft creates that is
   indistinguishable from success is a defect in its own terms.
3. **Behaviour preservation by construction.** The matcher set is claimed to be a
   strict generalisation with N=1 as the existing case, so the existing suites
   are the proof.
4. **Governance applied, not cited.** ADR-001 (leaf ← engine ← command), STD-001
   (named constants), ADR-019 (projection needs owner + mutation policy),
   POL-002 (no host-project / per-user harness state).

### Lines of attack, as taken

1. **The `sec-2` generalisation claim** — the load-bearing, untested one. Read
   `plan_hook` (`src/boot.rs:1153-1220`), `owned_positions` (`:1039`),
   `entry_is_canonical` (`:1060`), `hook_is_sole` (`:1078`) and checked the
   positional-zip rewrite for a shape where a current no-write becomes a rewrite,
   or worse, a current rewrite becomes a no-write.
2. **`sec-4`'s six-spec loop and its `?`** — which specs go missing on a mid-loop
   abort, what fires, and whether the resulting state is distinguishable from
   success.
3. **`sec-3` resolving scope inside `install_claude_hook`** — the cost side the
   draft waves through, and the second caller (`run_sync_install`,
   `src/corpus.rs:506`) it claims needs no change.
4. **The abandoned-scope sweep** — `plan_evict`, the one destructive path. Checked
   never-clobber against `owned_positions` / `drop_owned_hooks` (`:1108`) for a
   mixed owned+foreign entry and a wrongly-typed `hooks` object. Fail-soft on the
   file being *left* is a different risk from fail-soft on the file being written.
5. **Deletion scope in `sec-4`** — every named item verified orphaned with a
   positive control on its callers, and the two the draft says survive
   (`DOCTRINE_MARKETPLACE`, `[install] repo`) checked both ways.
6. **`sec-5`'s "rename and a hoist"** — diffed against the real
   `install_for_claude` at `git show 347197e8^:src/skills.rs`, including whether
   anything else consumed the dropped up-front `claude_links` vector.
7. **Verification honesty (`sec-7`)** — is the `VH` evidence re-derivable at
   audit, or does it die with the session
   (`mem_019fd1d862887d42b7a1f88c28fd28a7`)?
8. **Counting.** The spec inventory is stated four times at four altitudes
   (plugin entries, specs, merges, printed lines, `/hooks` entries). Cross-footed
   against `plugins/doctrine/hooks/hooks.json` and the draft's own
   `claude_hook_specs` registry.

### Where the bodies were likely buried, and were

The draft is strongest exactly where it is most technical (the merge-core
generalisation survives attack 1 intact) and weakest at its **seams and its
arithmetic** — the second caller it declares out of scope, the sweep's silent
failure mode, the orphan set its deletion inventory does not name, and an
acceptance criterion whose entry count is short of what the design itself emits.

## Synthesis (raiser's pass — findings open, no dispositions taken)

Eleven findings: 2 blocker, 4 major, 3 minor, 2 nit. This is the raiser's side
only; dispositions are the responder's and are not taken here.

**Overall: revision-required** — on two blockers that are both cheap to fix and
both in the permissive direction, which is the wrong direction for this slice
specifically.

### What survived attack, and it is not a short list

These were probed hard and are clean. Recording them because seven sections
carry `review=outstanding` and a section that holds up is a result.

- **`sec-2`'s generalisation claim is sound (attack 1).** The positional-zip
  no-write short-circuit reduces to the incumbent at N=1: `owned.len() == 1`
  plus one `entry_is_canonical` plus `hook_is_sole` is exactly the current
  `if let [(ei, hi)] = owned[..]` guard (`src/boot.rs:1183-1186`). I looked
  specifically for a shape where a current no-write becomes a rewrite, and for
  the more dangerous inverse — a current rewrite becoming a no-write — and
  neither exists. The `hook_is_sole` conjunct is what closes the interesting
  hole: because it forces each owned entry to carry exactly one hook, N owned
  positions with all-sole entries necessarily span N *distinct* entries, so a
  single entry stuffed with four identical owned hooks cannot masquerade as a
  canonical four-matcher set. The claim that the gate holds *by construction* is
  earned on the merge core itself. F-3 is about the gate's diagnostic value being
  degraded from elsewhere, not about this.
- **`ins + k` is in bounds.** `ins <= len` before the loop and the array has
  grown by exactly `k` at the `k`-th insert. The draft's proof is correct as
  written.
- **Never-clobber holds on the destructive path (attack 4).** `plan_evict` gates
  on the same `is_ours` predicate as the merge, and `drop_owned_hooks`
  (`:1108-1120`) rebuilds each `hooks` array retaining non-owned hooks and drops
  only entries whose array empties — so an entry carrying both an owned and a
  foreign hook survives with the foreign hook and its entry-level keys intact.
  The wrongly-typed-`hooks` case is refused by `hook_array_mut` returning `None`
  (`:1127-1137`) rather than clobbered. Also checked and clean: `hook_array_mut`
  *does* mutate its input by `or_insert_with`-ing an empty `hooks` / event key,
  but `plan_evict` discards the value whenever `count == 0`, so a sweep of an
  absent or hook-less sibling writes nothing and creates no key. F-4 is about
  reporting, not safety.
- **`sec-5`'s double-classify collapse is correct.** Read against the real
  `install_for_claude` at `git show 347197e8^:src/skills.rs:765-820`. The up-front
  `claude_links` vector had two uses — carrying the `(id, dest, target)` triple
  into the loop, and an early `continue` on a plan-time `KeepForeign` — and both
  are subsumed: the triple is recomputed inline from `link_dir` / `canon_dir` /
  `entry.id`, and the plan-time `KeepForeign` arm printed the identical string the
  mutation-time arm prints. One classify at mutation time is observably the same
  output. Nothing else consumed it, including the print path. The draft is right.
- **`sec-1`'s separability claim is true.** The hook leg is JSON-entry surgery in
  `boot.rs` over the settings file; the skills leg is a canonical tree plus
  symlinks in `install.rs`. No shared code, no shared file. Phasing them
  independently is sound.
- **`DEC-166`'s byte-identical duplication is real and exactly as cited** —
  `src/install.rs:2179-2191` and `:2279-2291`, verified line by line.
- **`[install] repo` survives, as claimed.** Live independent consumer at
  `src/install.rs:423` feeding `delegate_argv` (`:1989`, called `:2058`).
- **ADR-001 layering on `sec-3` is applied, not cited.** `ClaudeSettingsScope` in
  `install_config` (config vocabulary; the module is a pure serde leaf) with
  `settings_rel`'s path mapping in `boot.rs` (which owns the path constants) is
  the correct side of the line, and the draft's reason — a pure leaf should not
  acquire path knowledge — matches that module's own doc.
- **STD-001 is satisfied** by the constant set at `sec-2` lines 248-264, including
  naming the five event literals that are inline today.
- **POL-002 is not contradicted.** After the cut, `claude_cmd_stdout` — the only
  reader of per-user harness state, shelling `claude plugin marketplace list` at
  `src/install.rs:456` — is gone. Hook writes go to the project root regardless of
  `--global`: `install_refresh` receives `wire`'s `root` and never consults
  `install_base`, so the `--global` surprise I went looking for in attack 3 is not
  there. `--global` reaches only the skills leg, via the same `install_base` seam
  the agents and workflows legs already use.
- **`sec-4`'s named deletion set is accurate** as far as it goes; every item I
  checked is genuinely orphaned. F-5 is that it stops early, not that it is wrong.

### One observation, deliberately not raised

`sec-3` moves `load_doctrine_toml(root)?` inside `install_claude_hook`. That adds
a failure mode the section does not mention while costing out "one small TOML
read per spec": `load_doctrine_toml` errors on malformed TOML
(`src/dtoml.rs:101-107`, `with_context("Failed to parse …")`), so a broken
`.doctrine/doctrine.toml` now aborts a hook write that is config-independent
today — and via the `sec-4` loop's `?`, the whole Claude arm. I am not raising it:
failing loudly on unparseable config is defensible, arguably right, and the
draft's factoring argument for resolving inside rather than passing a parameter
is the stronger consideration. Noted so it is a choice on the record rather than
an oversight.

### Standing risks if the blockers are dismissed rather than fixed

- F-1 leaves the slice's only human verification gate satisfiable by a
  four-entries-short install.
- F-2 leaves the highest-frequency install path (`memory sync install`, the
  documented ritual after any shipped-memory edit) performing the slice's one
  destructive write with no output.

Both are silent-in-the-permissive-direction, which is the failure class the slice
was scoped to eliminate. That is why they are blockers rather than majors: not
because the mechanism is wrong — it is not — but because the design would ship
its own thesis unenforced.

### Haiku

Nine, six, seven, ten —
the merge core counts honestly.
Only the prose lies.

