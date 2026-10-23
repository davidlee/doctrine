# REV REV-075 — reconcile SL-285

Revision — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

<!-- Why this revision: what authored truth needs to change and why, the scope of
     the staged delta, and (for ADR/POL/STD/prose rows) the before/after excerpts
     the structured payload only labels. Seeded at `revision new`. -->

`REQ-189` (SPEC-012) described provisioning as copy-only. SL-285 made
`run_provision` copy the allowlisted files and then regenerate the derived
runtime tier (`install::regenerate`). Design sec-6 named this a reconcile-time
revision; the RV-426 brief omitted it and the user chose to land it here
(2026-10-23, option a).

### Before — REQ-189 title

> Provision a fork only through `worktree provision`, copying solely
> allowlisted (`.worktreeinclude`) gitignored files; `select_copies` drops every
> coordination/runtime-tier path even under a broad `**` glob.

### After — REQ-189 title

> Provision a fork only through `worktree provision`: copy solely allowlisted
> (`.worktreeinclude`) gitignored files — `select_copies` drops every
> coordination/runtime-tier path even under a broad `**` glob — then regenerate
> the derived runtime tier in place from the running binary.

### SPEC-012 § Fork provisioning — appended paragraph

> After the copy, provision regenerates the derived runtime tier that `doctrine
> install` projects — shipped memory, harness config and hooks, skills, agent
> defs and harness link dirs — in place, from the running binary
> (`install::regenerate`, SL-285). Every creation caller regenerates; a
> conflicted dispatch candidate copies only and prints the repair command
> (`doctrine worktree provision <dir>`). A source with no tracked
> `installed-version` is copied only, with a disclosure. Regeneration refuses
> before any write on an unreadable stamp, a downgrade (unless
> `--allow-downgrade`, repair only) or stale tracked wiring, and fails on any
> tracked drift its legs cause. Creation compensates a failure; repair never
> deletes.

## Reconcile narrative

- [RV-426 brief / design sec-6]: REQ-189's copy-only wording no longer covers
  provisioning. Title and SPEC-012 prose amended to the copy-then-regenerate
  split. The copy-path guarantee (`select_copies`) is unchanged.
