---
name: notes
description: Use whenever you complete a unit of work, a task, or a phase — record implementation notes so progress and findings are durable, not stranded in conversation context.
---

# Notes

Record implementation notes as you go.

During execution, notes belong in the **active runtime phase sheet**
(`state/.../phases/phase-NN.md`) — disposable working context. At phase or slice
wrap-up, durable items are harvested into `notes.md`
(`doctrine slice notes <ID>` scaffolds it on demand). Honour the
storage rule: live progress lives in the state tree, never in authored files.

If you don't know which slice owns the work, find it with `doctrine slice list`.

Be concise, but record:

- what's done
- any:
  - surprises encountered or adaptations required
  - potential rough edges, omissions, or refactorings for later
  - follow-up actions advisable
  - open questions relating to completed or upcoming work
  - durable facts, patterns, or gotchas that should become a memory
  - relevant commit hash(es), or: uncommitted work
  - whether `.doctrine` changes were committed promptly per repo doctrine, or
    are still pending and why
  - if committed, whether they went out with code or separately when that
    matters for the next agent
- whether the verification gate (`doctrine check gate`) has run successfully since code
  was last modified, or: outstanding errors

If the note identifies a reusable fact, pattern, or gotcha that would save a
future agent meaningful time, run `/record-memory` before you treat the task,
phase, or slice as wrapped.

If instead it identifies a unit of follow-up **work** — an issue, risk, chore, or
idea worth doing later — **capture** it with `backlog new` so the intent is not
stranded in this context. Work, knowledge, or a decision? The boundary that
arbitrates is in `using-doctrine.md`.
