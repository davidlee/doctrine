# ISS-226: verify-vt UNATTRIBUTABLE says keyword present when absent

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed

`doctrine slice verify-vt SL-214` run pre-implementation (plan authored, no
work landed) reports for every mandated file that exists but is untouched:

```
≈ UNATTRIBUTABLE VT-2 — keyword present but `plugins/doctrine/skills/design/SKILL.md` not modified by this slice
```

But the keyword is verifiably absent:

```
$ grep -F "/knowledge" plugins/doctrine/skills/design/SKILL.md; echo $?
1
```

Same for every UNATTRIBUTABLE row that run (PHASE-01 VT-2..4, PHASE-02
VT-1..2; keywords `/knowledge`, `unsettled`).

## Expected

The message should not assert keyword presence it hasn't established (or the
keyword check should run and be reported truthfully). Pre-work, an honest
status is something like "file not modified by this slice — mandate pending",
distinct from "keyword present". As written, an agent must re-grep every
mandated file to disprove the tool before trusting its own keyword floors.

## Guess at mechanism

UNATTRIBUTABLE looks like a short-circuit on the modified-by-slice check with
a canned message that hard-codes "keyword present" regardless of whether the
substring scan ran.

Surfaced while planning SL-214 (RFC-011 case-noted).
