# CHR-067: design skill Recovery block omits the positional SLICE

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

The `/design` skill's **Recovery** block writes `design resume` and `design show`
without the required positional `<SLICE>`, and invents a `--slice` flag the binary
does not accept. Both forms are refused.

## Why it matters

Recovery is, by definition, read in a cold or confused context — the one moment an
agent is least able to diagnose a wrong command shape and most likely to conclude
the run itself is broken. A doc-level typo costs a probe cycle here that it would
not cost anywhere else in the skill.

## Fix

Correct both invocations to the positional form. While in there, check the rest of
the skill's command shapes against `doctrine design --help` — the same class of
drift is likely elsewhere, and the CLI is the source of truth.

## Evidence

- `019fd72c-4593` — `/design` skill Recovery omits the positional SLICE

## References

- `IMP-412` — locked design run has no handover exit (the adjacent recovery gap)
- `IMP-321` — verify the advice surface: refusals and prescriptions against the machine
