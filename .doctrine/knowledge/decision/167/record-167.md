# DEC-167: Cutover installs before the plugin is disabled

## Decision

The prescribed cutover order is:

1. Run the install that writes the settings hooks.
2. Disable the doctrine plugin by hand.

Between the two acts every hook fires twice. That is the intended cost.

## Why this order

The reverse order leaves the repo with **no activation at all** for the window,
and absence is the more dangerous state. A silently inert `WorktreeCreate` hook
does not merely degrade dispatch — `isolation: worktree` teardown is
*conditional* on that hook firing, so its absence "changes dispatch's semantics
without saying so" (SL-250 § The dispatch exposure).

A double-fire, by contrast, means memory-sync runs twice and boot emits twice:
wasteful, visible if you look, harmless if you do not.

**Order toward the degraded state, never the absent one.** `R3` already settled
that the human hand-repairs their own activation across the cutover; this only
says which way to walk through the window.

## What the two cutover notes now say

The two risks came apart once [[DEC-164]] landed.

**`R10` — reduced to a description.** It used to read *if you switch scope,
delete the old entry from the other settings file*. Doctrine now sweeps the
abandoned scope and reports what it removed. The note shrinks to a line
explaining the eviction, so it is not mistaken for doctrine eating a hand-placed
hook.

**`R8` — stays pure documentation, and cannot be otherwise.** The plugin's
entries load through `enabledPlugins` plus per-user marketplace registration.
The ownership sweep cannot reach them — not because it would be hard, but
because this slice reads and writes no per-user harness state by Non-Goal
(migration is IMP-400's remaining leg). So the note stands as written: *when you
take the new activation, disable the doctrine plugin, or every hook fires twice.*

The asymmetry is worth stating in the design: `R10` is engineered because both
settings files are doctrine's to write; `R8` is documented because the plugin's
activation state is not.

## Not part of the flip

`A3`'s `.gitignore` edit (`.gitignore:4` ignores `/.claude` wholly) affects
whether the activation is **reviewable in git**, not whether it **works**. Claude
Code reads `.claude/settings.json` regardless of tracking. It is a
benefit-realisation step, and conflating it with activation would make the
cutover look more fragile than it is.

Recorded from design run `dr-019fd692` checkpoint `cp-7` disposing `inq-9`.
