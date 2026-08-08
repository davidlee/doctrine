# REV REV-049 — reconcile SL-250

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-250 (*Retire the Claude plugin delivery channel*) moved doctrine's Claude hook
activation off the plugin channel and onto a direct write into Claude's settings
JSON. SPEC-011 (*Boot snapshot*) governs `boot install`, and it still describes the
hook leg as SL-152 left it: one hook, one file, one command form, no sweep. RV-350
`F-1` established that this is false on four axes against shipped code, and the
audit's reconciliation brief routes it here.

This REV is the whole of that write. `slice conformance SL-250` reports
`.doctrine/spec/tech/011/**` **undelivered** for exactly this reason — it is
reconcile's write, not a phase's, and it clears when these rows land.

### The four axes `REQ-186` gets wrong

`REQ-186` (SPEC-011's `FR-006`) reads:

> `boot install` merges a `<exec> boot` SessionStart hook into Claude
> settings.local.json, refreshing a stale owned copy and preserving every foreign
> hook and key.

1. **Not one hook.** Seven `HookSpec`s emitting **eleven entries across five
   events** — `SessionStart`, `WorktreeCreate`, `SubagentStart`, `SubagentStop`,
   and six `PreToolUse` matchers (`claude_hook_specs`, `src/boot.rs:1255`). The
   distribution is asserted by `install_wires_eleven_hook_entries_across_five_events`
   (`tests/e2e_claude_install.rs:285`).
2. **Not `settings.local.json`.** A scope-selected file defaulting to
   `.claude/settings.json` (`settings_rel`, `src/boot.rs:558`;
   `ClaudeSettingsScope` defaults `Project`).
3. **Not `<exec>`.** The committed scope writes the portable `PORTABLE_EXEC`
   literal `${DOCTRINE_BIN:-doctrine}` (`command_form`, `src/boot.rs:574`), on
   SL-195's `baked ⟺ gitignored` invariant.
4. **A new obligation the requirement does not mention** — the abandoned-scope
   sweep, gated on the target write landing (`evict_hook_from_file`,
   `src/boot.rs:2210`; `install_claude_hook`, `src/boot.rs:2076`).

### `QUE-209` settled: three requirements, not one widened one

SL-250's `design.md` `sec-7` *Governance* deferred requirement granularity to this
REV. Settled by the user (2026-08-08): **split**.

SPEC-011's other seven requirements are each one sentence covering one verifiable
behaviour. Widening `REQ-186` to swallow all four axes would produce a run-on out
of line with its siblings, and — the load-bearing objection — would bury the sweep
inside a requirement about a merge. The sweep is doctrine's one *destructive* write
in this area, with its own ownership predicate, its own gate, and its own reporting
contract. A requirement that governs it only by implication is the kind of coverage
that reads as present and verifies as nothing.

So: `REQ-186` keeps its identity (the owner-locked merge, generalised over the set),
and the two obligations it never described become requirements in their own right.

## Change rows

### `[[change]]` 1 — `modify REQ-186` (primary)

**Before**

> `boot install` merges a `<exec> boot` SessionStart hook into Claude
> settings.local.json, refreshing a stale owned copy and preserving every foreign
> hook and key.

**After**

> `boot install` merges doctrine's owned Claude hook set — every spec's entries,
> in matcher order, across every event it declares — into the selected settings
> file, refreshing a stale owned copy and preserving every foreign hook and key.

Axis 1 is absorbed by *set*; axes 2 and 3 move to `FR-008` and axis 4 to `FR-009`,
so this statement stops naming a file and a command form it no longer fixes. The
never-clobber half is unchanged, because it did not change: the ownership predicate
is still `command`-only (`DEC-161`), and entry identity simply became the set.

### `[[change]]` 2 — `modify SPEC-011` (prose + responsibilities)

The same falsehood is carried in the spec body and in the spec's own responsibility
list, so the requirement edit alone would leave SPEC-011 self-contradicting.

- **`spec-011.toml`, responsibility 6** — "merge a `<exec> boot` `SessionStart`
  hook into `.claude/settings.local.json` preserving foreign hooks" → the hook set,
  the selected scope, the command form, and the sweep.
- **`spec-011.md` § *`boot install` — import wiring and hook merge*** — the
  "Second, for Claude it merges a `<exec> boot` `SessionStart` hook (matcher
  `startup|clear`) into `.claude/settings.local.json` …" passage, rewritten for the
  set, the scope key and the sweep. Fail-soft, narrow-path mutation and
  foreign-preservation all survive verbatim — they are the parts that held.
- **`spec-011.md` Overview and § *Concerns*** — the two incidental "`SessionStart`
  hook" / "the hook merge" references, widened to the set.

### `[[change]]` 3 — `introduce FR-008` (scope selection and command form)

Covers axes 2 and 3. The two are one requirement rather than two because they are
one decision: the scope key selects the file, and the file's tracked-ness *dictates*
the command form on `baked ⟺ gitignored`. Splitting them would invite an
implementation that honours the key and writes a host abspath into a committed
file — the POL-002 breach the invariant exists to prevent.

### `[[change]]` 4 — `introduce FR-009` (the abandoned-scope sweep)

Covers axis 4. Three obligations in one behaviour: evict by the same ownership
predicate (so no foreign entry is ever at risk), gate on the target write landing
(so a failed write never strands the operator with *no* working copy — `DEC-164`
and the `F-12` guard), and report all three outcomes (removed / unreadable /
not-attempted). The reporting clause is not decoration: the failure this requirement
exists to prevent — every hook firing twice, permanently — "presents as mild
slowness and nothing else".

## Reconcile narrative (SL-250)

- **`RV-350` `F-1`** → rows 1–4. `REQ-186` false on four axes against shipped code;
  `QUE-209` settled as a three-requirement split.
- **SPEC-010 (*Skills distribution*) does not enter this REV.** `DEC-171` made it a
  **conformance claim**, which is the stronger of the two — an amendment edits text
  to fit what was built, a conformance claim checks the built thing against text
  written before it. `PHASE-05` `VA-1` discharged responsibilities 3–6 and RV-350
  re-derived each by reading: canonical tree `skills_canonical_dir`
  (`src/install.rs:1994`) → `install_skills_direct` (`:2064`); relative symlink via
  `claude_skills_dir` (`:1999`) → `reconcile_link` (`:1677`), classified by value
  equality; `.tmp-<id>` staging with remove-then-rename and
  `symlink_metadata`-not-`exists` in `materialise_canonical` (`:2013`); gitignore
  self-enforced by `install/manifest.toml:46` with no `ensure_gitignored` call, as
  `EX-7` requires. Observed live: 35 symlinks in the `VH-1` capture.
- **RFC-018 (*Claude harness field notes*) takes the empirical residue**, not this
  REV: the `strictPluginOnlyCustomization` asymmetry, and the two memories `VH-1`
  produced (`mem.fact.claude.native-worktree-isolation-is-tool-layer-only`,
  `mem.fact.worktree.gitdir-pointer-unresolvable-in-sandboxed-subagent`). An RFC is
  not authored governance truth, so it is a `/harvest` sink, not a change row.
