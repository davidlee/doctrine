# ISS-247: No verb removes a needs edge

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`doctrine needs <SOURCE> <TARGET>` appends a hard prerequisite. Nothing removes
one:

- `doctrine unlink SL-230 needs REV-034` → *"`needs` is not a relation label
  authorable by SL via `link`. Legal labels: references, references, references,
  supersedes, governed_by, related, fulfils"*
- `doctrine needs --help` offers no `--remove` / `--clear`; its own summary is
  "Append a hard prerequisite".

So a `needs` edge is **write-once via the CLI**. The only way to retract one is to
hand-edit `[relationships] needs` in the entity TOML.

## Why it matters

`needs` is not decorative — ADR-017 makes an inbound `needs` on an unsettled
record an **actionability gate**. A stale prerequisite therefore blocks work that
is no longer blocked, and the blockage is invisible in the sense that the edge
looks deliberate.

The retraction case is not exotic. It arises whenever scope moves between slices —
exactly what DEC-027 did: SL-230 needed REV-034, the slice split, and the
amendment went with the departing half to SL-232. Without a removal verb SL-230
would have stayed gated on a revision it no longer has any reason to await.

## Observed

Hit while applying DEC-027. Resolved by hand-editing
`.doctrine/slice/230/slice-230.toml` to `needs = []`, which
`install/using-doctrine.md` sanctions — *"hand-edit the TOML for fields no verb
yet owns (cite the CLI gap if so)"* — with this item cited inline. That is the
documented escape hatch working as intended, but it is an escape hatch: it
bypasses whatever validation and reciprocity derivation the verb path performs,
and it is the raw-file write the guardrails otherwise tell agents not to do.

## The asymmetry worth noting

`link` / `unlink` are a matched pair for tier-1 relations. `needs` and `after` are
appended by their own verbs and have no inverse. Whatever the reason for routing
them separately, the *removal* half was not carried across — this looks like an
omission rather than a decision, since nothing in the docs argues that a
prerequisite should be irrevocable.

## Suggested shape

Either `doctrine unneeds <SOURCE> <TARGET>` (mirrors `needs`, discoverable next to
it), or teach `unlink` the `needs` / `after` labels so the removal surface stays
uniform even though the authoring surface is split. The second is likely better:
one removal verb, and it makes the `link` / `unlink` asymmetry visible at the
point someone would trip over it.

Same question applies to `after` — untested here, but it has the same
append-only shape.
