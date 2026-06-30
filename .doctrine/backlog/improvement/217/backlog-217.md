# IMP-217: Retiring a local-capture memory to a shipped master needs a first-class verb — supersede can't name a shipped successor

## Problem

Promoting a project-local capture (`items/`) to a shipped global-orientation
master (`memory/`) has no clean retire path for the old capture:

- `memory status superseded --by <new-uid>` resolves the successor against
  `items/` only (`src/memory.rs` resolve_uid_prefix; "items/ wins any uid
  collision"). A shipped master lives under `memory/`→`shipped/`, never `items/`,
  so it **cannot be named as a supersession successor** → "memory not found".
- Key→uid resolution ignores status: `memory show <key>` keeps returning the
  archived/superseded `items/` capture via the `items/mem.<key>` alias symlink
  (status filters `find` only). So retiring **requires physically `git rm`-ing the
  alias symlink** — a status change alone is silently insufficient.

The working dance (SL-178 PHASE-01, via `/consult`): scrub body → `memory record
--global` → `git rm` the items/ key-alias symlink → `memory status archived` the
capture. Three manual steps with a sharp footgun (forget the symlink rm and the
key dangles at the dead capture).

## Why it matters now

IMP-216 migrates ~46 project-local operational memories to shipped reference
knowledge. Every one repeats this exact manual dance and footgun. A first-class
`memory promote <uid>` (or `supersede --by` that accepts a shipped successor +
re-points the alias) would make IMP-216 ergonomic instead of death-by-papercut.

## Durable facts

[[mem.pattern.doctrine.key-resolution-ignores-status]],
[[mem.pattern.doctrine.shipped-master-body-scrub]].

Surfaced by RV-196 (SL-178 reconciliation audit), F-1/F-2.
