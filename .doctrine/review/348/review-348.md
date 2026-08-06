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

## Synthesis — round 1 (raiser's pass, at run revision 39)

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

## Round 2 — the remediation, at run revision 41

`F-1` … `F-11` all **verified** (terminal). Each was re-derived against revision 41
before accepting, not accepted on the responder's report. The remediation is
genuinely good: the count ledger in `sec-1` is now the single authority and every
restatement cites it; `EvictOutcome` gained the third state with a printed
diagnostic; the deletion closure is tabulated with per-orphan sole-reader
citations and `DOCTRINE_MARKETPLACE` resolved rather than deferred; the
class-(i)/class-(ii) split restored the gate's diagnostic meaning; `F-8`'s fix
went past what was asked, adding a test written against parsed TOML text plus a
companion asserting the underscore spelling does *not* bind.

Two claims I checked hardest and which hold exactly as written:

- **`MCP_COMMAND` really is `${DOCTRINE_BIN:-doctrine}`** (`src/boot.rs:549`), its
  three named consumers are exactly the three in its doc-comment, and the new
  `is_doctrine_program` arm mirrors `is_doctrine_mcp_entry`'s two-form posture
  (`:1475-1488`) precisely rather than by analogy. The rename claim is sound.
- **The `is_doctrine_program` widening is strict.**
  `Path::new("${DOCTRINE_BIN:-doctrine}").file_name()` yields the whole token, so
  the two arms cannot collide and no foreign command becomes ours. The
  `Local`→`Project`→`Local` round trip heals in both directions: the abandoned
  file's entries are owned and evicted, the target file's wrong-form entries are
  owned-but-not-canonical and rewritten. Attacked and clean.
- **`baked ⟺ gitignored` survives on the Codex arm** — `.codex/hooks.json` is
  gitignored (`.gitignore:9`, `/.codex`; the file is untracked), so the invariant
  is not breached there. `SpawnSeamSymmetry`'s input really is the surviving
  plugin manifest (`HOOKS_CONFIG_PATH`, `src/doctor_checks.rs:622`), so "needs no
  change" is correct.

### But the two unreviewed rulings both carry defects

The remediation went past the ledger on two human rulings, and neither had been
reviewed by anyone. Both were the *right* rulings; both have a flaw in the
argument attached to them.

- **`F-14`** — `install_baseref` following the scope is correct. The carve-out
  declining to sweep the key rests on the value being invariant, and
  `BaseRefOutcome::Conflict` (`src/boot.rs:1341-1343`) exists precisely to let it
  not be. A stranded operator override lands in a known footgun
  (`mem.signpost.doctrine.dispatch-claude-arm-wrong-base`).
- **`F-16`** — the command form is right, and SL-195's invariant is the right one
  to keep. But shell-form expansion is a property of the *shell*, and the cited
  paragraph names two shells where `${VAR:-default}` does not expand.

### And one new blocker, from the interaction of two fixes

**`F-12`.** `sec-3` argues write-before-evict guarantees "activation lands before
removal". It does not, because `PrintedFallback` is an `Ok` value: a malformed
*target* file yields no write, the `?` does not short-circuit, and the sweep then
deletes the working entries from the *sibling*. Zero activation, from a design
that orders its two acts specifically to avoid that state.

This is not `F-4` re-opened — `F-4` was the sibling unreadable, and
`EvictOutcome::Unreadable` fixed it. Here the sibling is perfectly readable and
the sweep *succeeds*, which is what makes it destructive. The likely trigger is a
consequence of this slice's own choices: `sec-3` makes `settings.json` the
committed, hand-editable default, and `sec-6` notes this repo already tracks it.

`F-13` and `F-15` are the other two: the scope is specified as an input to
`plan_hook` and threaded nowhere, with no answer for what the Codex arm passes to
a Claude-only enum; and `EvictOutcome::merge`'s absorbing `Unreadable` discards a
sibling `Removed(n)`, so a partly-successful destructive sweep reports only the
failure — re-breaching the DEC-164 clause `F-4` was raised on, in the narrow
mixed case its new test cannot reach.

**Overall: revision-required**, on one blocker. The pattern across both rounds is
consistent and worth naming plainly: the mechanism is right nearly every time,
and the *argument attached to the mechanism* overclaims — "nine entries", "no
change to `corpus.rs`", "a compiler-checked rename", "equivalent on every input",
"cannot disagree with the live one", "this works because hooks are shell form".
Each is a sentence asserting a property one degree stronger than the code
supports. That is a cheap defect to fix and an expensive one to inherit, because
an implementer reads those sentences as discharged analysis.

### Haiku

Write, then sweep — unless
the write only *said* it wrote.
Now nothing will fire.

