# ISS-528: A foreign codex MCP entry holds boot install incomplete, unexplained

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

`doctrine install` ends every run with:

```
install incomplete: boot wiring; version not recorded
```

The maintainer saw this for weeks and read it as a bug. The cause is printed
mid-output (the codex MCP leg's "has a doctrine entry doctrine did not write …
left untouched" plus a snippet), far from the closing summary, which names only
the leg.

## Cause

`.codex/config.toml` held a hand-written, working entry:

```toml
[mcp_servers.doctrine]
command = "doctrine"
args = ["serve", "--mcp"]
```

REQ-484 (SPEC-011) owns only an emitted shape, so this entry is foreign and is
correctly left untouched. But the foreign outcome joins `refused_wiring`, so the
boot leg is never stamped complete and the install version is never recorded —
permanently, until the user edits a file doctrine told them about once, in the
middle of a long report.

## Options

1. **Name the cause in the closing summary** — e.g. "install incomplete: boot
   wiring (codex MCP: `.codex/config.toml` has a foreign doctrine entry)". Cheap,
   and fixes the "looks bugged" read regardless of option 2. STD-003 favours it:
   a degraded outcome should be disclosed where it is read.
2. **Reclassify a foreign entry as reported, not incomplete.** A foreign entry is
   the user's choice and may work; holding the leg incomplete forever punishes a
   working setup. Counter: a foreign entry may be *broken* (wrong binary, missing
   `DOCTRINE_BIN` forwarding), and "incomplete" is the only nudge. Distinguish
   *uninterpretable* (keep incomplete) from *interpretable but foreign* (report).

Recommend 1 now; decide 2 alongside IMP-497 (re-verify the codex MCP seam
against a live codex), since the same question — what counts as a working
entry — sits under both.

Same shape likely applies to the Claude `.mcp.json` arm and the pi settings
fallback: check every `refused_wiring` contributor's summary wording.

Found during SL-284 close (VH-1 `doctrine install`).

## Update — SL-286 (2026-10-24)

Partially resolved by DEC-406. An absolute-path project override is now reported
and no longer holds install incomplete. The case above (bare `command =
"doctrine"`) still holds install incomplete, by design: it is PATH-dependent, and
interactive codex's frozen-PATH daemon breaks it (EVD-032). It is now named
`path shadow` in the leg report with the reason, and the closing line names the
leg (`codex user MCP`). Option 1 (cause in the closing summary itself) remains
open, as does the same check for the Claude and pi arms.
