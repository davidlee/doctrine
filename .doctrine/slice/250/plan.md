# Implementation Plan SL-250: Retire the Claude plugin delivery channel

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See glossary.md § reference forms. -->

## Overview

Six phases, ordered along one spine: **build the mechanism, then flip
activation onto it, then remove what it replaces.** The design's `sec-1`
already names the fault line the phasing rides — the hook leg and the skills
leg "share no code and no file", and "the hook leg can land, be verified live,
and be used before the skills leg starts". So the hook leg is four phases, the
skills leg is one, and the retirement plus its documentation is the last.

| phase | leg | what changes for a user |
|---|---|---|
| `PHASE-01` | merge core | nothing — a strict generalisation with N=1 as the existing case |
| `PHASE-02` | scope | doctrine writes `.claude/settings.json` instead of `.claude/settings.local.json`, in the portable command form |
| `PHASE-03` | sweep | the abandoned file is cleaned and the act is announced on both install paths |
| `PHASE-04` | registry | activation flips: eleven entries across five events |
| `PHASE-05` | skills | `.doctrine/skills/` + `.claude/skills/` symlinks return |
| `PHASE-06` | retirement | the installer stops touching plugin machinery; the docs land |

## Sizing: on the evidence, not the line count

The scope reasons a phase band from prior slices' design line counts. That input
is unreliable here for the reason `mem.pattern.doctrine.size-phases-on-evidence-not-lines`
records: `design.md` runs 1674 lines, but a large fraction of them are **review
archaeology** rather than instruction — the RV-348 remediation is narrated
inline ("the earlier text claiming it did was wrong in the one direction this
slice cannot afford"), findings are cited by bare id throughout, and three
sections carry a paragraph explaining what an earlier draft got wrong. Those
lines measure how contested the design was, not how much work it describes.

Sized on the design's own evidence instead — `sec-7` enumerates roughly 35 new
test cases and 12 knowingly-rewritten sites — the six phases carry about 11 / 16
/ 11 / 5 / 6 / 3 tests. Two of those look light and are light for stated
reasons: `PHASE-04` is small in lines and large in blast radius (it is the
moment activation flips, and isolating it makes a bisect mean something), and
`PHASE-06` is mostly deletion, where the proof is a green gate with nine tests
removed and none replaced.

## Sequencing & Rationale

### Why the merge core is first, and alone

`PHASE-01` is the only phase whose correctness argument is *by construction*.
The matcher set is a strict generalisation with N=1 as the existing case, so
both harness arms keep emitting byte-identical JSON — and that is exactly the
claim the design leans on in place of testing the generalisation directly.
That claim is only worth anything while the suite is otherwise quiet. The
moment a large expected red is baked into `src/boot.rs`'s tests for an unrelated
reason, "any red here means the generalisation narrowed something" stops being
true. So the generalisation lands with **no file under `tests/` edited** and no
assertion rewritten, and `VA-1` says so as a criterion rather than a hope.

The four new specs are deliberately *not* in this phase. The workspace denies
`dead_code`, so shipping four constructors with no caller would buy an
`expect(dead_code)` allowance in the exact registry `PHASE-04` then has to clean
up. Multi-matcher coverage instead rides a test-local `HookSpec` — which is the
better test anyway, since it keeps the core's own suite independent of what the
registry happens to contain.

### Why scope and sweep are two phases and not one

They are one mechanism in the design (`sec-3` opens by saying so) and two
phases here, for a testing reason rather than an architectural one. `PHASE-02`
carries the **class (ii)** rewrite — the eleven-odd assertions the default-scope
flip knowingly invalidates. `PHASE-03` adds only new behaviour and must touch
none of them. Keeping them apart is what makes the class (i) / class (ii)
discipline `sec-7` insists on legible in the history: after `PHASE-02` lands,
every rewritten assertion is on `EX-7`'s list, and a red outside that list is a
real narrowing.

`EX-9` on `PHASE-02` is the other half of that discipline. The design goes out
of its way to say `:3926`, `:3948` and `:5303` are *not* class (ii) — an
earlier draft cited them and inverted the table's purpose. The plan repeats the
exclusion because a table that exists so an uncited red reads as class (i) is
worthless if it is loose.

`EX-10` **completes** that enumeration rather than repeating it. `sec-7`'s
table is scoped to `src/boot.rs`, so it never reaches
`tests/e2e_memory_sync.rs`, where `sync_install_wires_a_session_hook_no_boot_hook_settings_wired`
reads `.claude/settings.local.json` and asserts the sync hook landed there with
an `expect`. Once `run_sync_install` inherits the project scope, that test does
not fail — it **panics**. Since `sec-7` makes "the enumeration still matches
what actually changed" a closure criterion, an unlisted panic in a phase whose
whole discipline is *expected red vs real red* is worth more than its one line.
Found by re-grepping `tests/` for both settings paths; `tests/e2e_worktree_import.rs`
only seeds settings files as fixtures and is unaffected.

The order within the pair is forced: the sweep is defined over the *sibling of
the scope being written*, so it cannot precede the scope. That leaves one
intermediate state — a flipped default with no sweep yet — and it is cheap,
because until `PHASE-04` the only Claude hook write is `memory sync install`'s
single `sync` spec. One stale entry, in a dev tree, for one phase.

### Why activation flips in its own phase

`PHASE-04` adds four constructors, a seven-element vector and a `for` loop. It
is small. It is also the only phase in the slice where a user-visible
capability changes state, where every golden that asserts the opposite inverts,
and where the `VH` leg's preconditions become satisfiable. Folding it into
`PHASE-03` would bury the flip inside a phase whose diff is dominated by
eviction machinery.

`EX-4` states the count as **eleven**, not seven, and the reason is in the
design's own ledger: seven is the spec count, and a criterion written against
seven passes a four-entries-short install. Four numbers describe this change at
four altitudes and only one of them is what a human counts in `/hooks`.

### Why skills come before the retirement

The design does not order these two, and the default reading of `sec-4` →
`sec-5` puts the retirement first. The plan inverts it, on `DEC-167`'s own
argument applied one level up.

Today the plugin serves *both* hooks and skills. Retiring it before `PHASE-05`
would leave an interval with **no skills channel at all**; retiring it after
leaves an interval where skills arrive twice. `DEC-167` settles exactly this
shape for the human cutover — "order toward the degraded state, never the
absent one" — and the same reasoning holds at phase granularity, for the same
reason: a doubled channel is visible and annoying, an absent one is silent.

### Why the documentation shares the retirement's phase

`sec-6` is not commentary on `sec-4`; it is the other half of it. The escape
hatch `R9` documents is *literally the two commands* `PHASE-06` deletes from the
installer — `claude plugin marketplace add` and `claude plugin install` stop
being an automated default and become the named remedy for a
`strictPluginOnlyCustomization` environment. Splitting the deletion from the
text that redirects it would ship one without the other. The same holds for the
command-form note: `PHASE-02` is what makes a reader open `.claude/settings.json`
and find a variable where they expected a path, and `PHASE-06` is where they are
told why.

## The one departure from design `sec-2`

`sec-2` places `ClaudeSettingsScope::command_form()` in `src/install_config.rs`.
That is not implementable, and the design supplies its own correction one
section later.

`.doctrine/adr/001/layering.toml` classifies `install_config = "leaf"` with
**out=0**, and `boot = "command"`. A method on `ClaudeSettingsScope` returning
`CommandForm` would make a pure leaf name a type from the command tier — a tier
inversion, and also a genuine **cycle**, since `boot` already reaches
`install_config` through `dtoml`. `tests/architecture_layering.rs` enforces
this.

The resolution is forced and is the design's own rule: `sec-3` already puts
`settings_rel(scope)` in `boot.rs` rather than `install_config`, on the ground
that "putting the path in `install_config` would give a pure leaf domain
knowledge it does not otherwise have (ADR-001, and the module's own doc)". The
identical argument covers the form mapping, so `command_form(scope)` lives in
`boot.rs` beside it. `ClaudeSettingsScope` keeps `sibling()`, which is pure
vocabulary.

Nothing else moves: `CommandForm` still lives in `boot.rs` with the core that
consumes it, and it is still deliberately **not** `ClaudeSettingsScope` — the
Codex arm answers `Baked` on its own file's gitignored status and never sees a
Claude-settings type. `PHASE-02` `EX-2` records the placement and `VA-2` makes
the layering gate a criterion rather than an accident.

## Design premises, re-checked

Every grep-pable reference in `design.md` was resolved against the tree on
2026-08-08 before this plan was authored. All symbols, constants, paths, test
sites, documentation citations and the recovery commit resolve. Two notes for
an implementer:

- **Line numbers drift by ±1–3** in several `sec-2` citations. The design's
  `fallback_for` at `:1587` is its *call site* inside `annotate_fallback`; the
  definition is at `:1224`. Trust the symbol names, not the line numbers.
- **The research staleness advisory fires and is a false positive.** It reports
  that `design.md` was *added* since the research baseline was stamped, which is
  normal lifecycle progression rather than drift. Already captured as friction
  under SL-248.

## Notes

**What this plan does not decide.** `QUE-209` — whether the REV widens
`REQ-186` or adds new requirements for the newly-governed hook set and the
scope key — stays deferred to reconciliation, where the REV is authored and
requirement granularity is the natural call. No phase settles it, and no phase
authors the REV.

**Backlog disposition is closure work, not phase work.** IMP-400 reduced to its
migration leg and left open on two counts; IMP-407 and IMP-406 confirmed still
open and still sequenced after; CHR-045 resolved or explicitly retained — it is
no longer moot, since the plugin manifest survives as `R9`'s escape hatch;
IMP-234 and CHR-037 assessed for overlap. These belong to `/close`.

**This repo is the first place SL-195's invariant covers hooks.** `.gitignore:4`
already carries `!/.claude/settings.json`, so the eleven entries `PHASE-04`
writes will be committed here. On the first install after this slice, `git diff`
is the check: eleven `${DOCTRINE_BIN:-doctrine}` commands and no `/home/…`
anywhere. Note that `.claude/settings.json` is currently modified in the working
tree by other work — coordinate before `PHASE-02` writes to it.
