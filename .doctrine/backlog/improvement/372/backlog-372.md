# IMP-372: Asset customization declares an override that no code resolves

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`publication/manifest.toml` declares a `customization` per asset —
`customizable` or `fixed` — and ADR-019 mandates that every projected asset
declare one. The field is parsed, stored, and displayed. **Nothing resolves it.**

Its only production consumer outside `src/publication.rs` is the library view
(`src/commands/library.rs:111`), which prints the value. Assets are fetched from
the embed: `install::asset_text` → `asset_source::read_text`
(`src/commands/design.rs:1633-1638`). So an asset marked `customizable` is
delivered by exactly the same path as one marked `fixed`, and a project that
edits its copy gets nothing.

The declaration is therefore a statement of *intent* — "this one would be yours
to change" — rather than a live policy. That is defensible while nothing depends
on it, and it is what ADR-019 literally requires. It stops being defensible the
moment a decision reads the flag as delivering an override.

## Why it surfaced now

SL-233's DEC-102 originally claimed that shipping an obligation runbook with
`customization = "customizable"` would give v1 "exactly one override seam". An
external adversarial review of the PHASE-16 design established that it would not.
The owner's ruling was to **defer the seam rather than claim it**: the v1 runbook
ships embedded, and DEC-102 is corrected to identify-and-defer. This item is the
deferral, recorded so it is distinguishable from an oversight.

## What the work is

Not the flag — the resolution behind it:

- **Project-path lookup.** Where does a project put its override? A per-asset
  path under `.doctrine/`, mirroring the address? The hymn cascade already
  resolves from `.doctrine/hymns` plus the embed, so there is a precedent to ride
  rather than a new mechanism to invent.
- **Framework fallback and precedence.** Project copy wins; missing project copy
  falls back to the embed; a `fixed` asset ignores a project copy (or refuses,
  loudly — silently ignoring an override a user authored is its own defect).
- **Fingerprint scope.** Which bytes are digested for receipts and drift when the
  resolved bytes may now come from either tier. The design-run fragment receipt
  (`Fragment::parse_receipt`) currently digests embedded bytes and would need to
  digest *resolved* bytes.
- **What `doctrine prompt check` and `doctrine library` then report** — an
  override that exists, an override of a `fixed` asset, an override that has
  drifted from the framework version it was forked from.

## Notes

- This is a **capability gap**, not a bug: no shipped behaviour is currently
  wrong, because nothing currently promises an override. It becomes a bug the
  first time something does.
- Scope check before starting: does any *other* record already read the flag as
  live? DEC-102 did and is corrected; worth a sweep for others.
