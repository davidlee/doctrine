# IMP-221: memory edit body and verify friction

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

During RFC-012, a stale spec reference was surfaced in 
`mem.signpost.project.orientation`. Editing it required:

1. Directly editing the `memory.md` file by hand (no CLI verb for body edits)
2. The `doctrine memory edit` MCP tool only touches metadata fields
3. Re-verifying required a fully clean working tree — the unrelated dirty
   `src/worktree/jail.rs` had to be stashed, verified, then popped

## Scope

Three sub-improvements:

### A. Extend `doctrine memory edit` for body prose

- Add `--replace-body` (overwrite entire body from stdin or file)
- Add `--append-body` (append to existing body from stdin or file)
- Possibly add `--edit-body` (open $EDITOR with current body, write back on
  save — lower priority, nice-to-have)

### B. Extend the `doctrine_memory_edit` MCP tool

- Add `body` field accepting the replacement text
- Add `body_mode` field: `replace` | `append`
- Keep the current metadata-only edit path as the default (backward compat)

### C. Relax the verify dirty-tree gate

- `doctrine memory verify <key>` currently refuses with a dirty working tree.
  This is unnecessary friction when editing a single memory — the commit/verify
  cycle should be loosened to:
  - Allow verify on a dirty tree when only the target memory's files are
    modified (the rest of the tree can be dirty)
  - Or: verify should not require a commit at all — the attestation is about
    the memory content, not the tree state
  
  See also IDE-008 (executable phase gates / verify contract) and IMP-212
  (plan-time re-grep) for adjacent verify-pattern thinking.

## Delivery status

**A and B shipped in SL-230** (closed 2026-07-27, `memory body-write seam`).
The delivered shape differs from the sketch above and the sketch is superseded,
not unmet: one `--body` flag with a `--body-mode replace|append` companion,
sourced inline or from stdin (`--body -`), on both `memory record` and `memory
edit` — no `--replace-body` / `--append-body` pair, and no `--body-file` (stdin
already covers a file). `--edit-body` was not built; it was the item's own
"lower priority, nice-to-have" limb and nothing depends on it. The MCP surface
gained `body` on `memory_record` and `body` + `body_mode` on `memory_edit`,
metadata-only staying the default path (SL-230 design D1/D2).

**C is not delivered.** The verify dirty-tree gate proved to be a substantially
larger problem than the body-write seam, and the two halves stopped converging
at the same rate; **DEC-027** split it out at RV-307 round 8 into **SL-232**
(`Corpus-aware memory verify gate`), which inherits the design that survived
eight adversarial rounds. SL-008 D6 — the gate itself — is reviewed canon and
bounds how far the relaxation may go, which is why C is a slice and not a patch.

**This item therefore stays `open` at SL-230's close**, against the general rule
that a closed slice resolves its origin. Resolving it `fixed` now would assert C
delivered. Provenance is intact and traversable — `IMP-221 → SL-230
(originates_from) → SL-232 (originates_from)`. **SL-232's close resolves this
item.**

## Related

- Surfaced during RFC-012 corrective edit
- `doctrine memory edit` MCP tool: metadata fields only (title, status, scopes,
  trust, severity, lifespan, key, review_by)
- Shipped memory workflow also involves `cargo build` + `doctrine memory sync`
  + `doctrine claude install` — this isn't about that path

## Tags

area:memory, area:cli, area:mcp, quality
