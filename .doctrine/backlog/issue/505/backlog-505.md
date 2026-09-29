# ISS-505: Skills contradict their owners: worktree worker mode, elicit kinds, lifespan filter

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found by SL-273's audit re-pass (RV-414 F-5; rows in
`.doctrine/slice/273/inventory-audit.toml`). Each skill restates something
its owner owns, and the copy has drifted. All three predate SL-273.

- **A-071 — worktree skill, worker mode.** `plugins/doctrine/skills/worktree/SKILL.md`
  (~43-48, ~255-258, ~268-269) says a worker self-arms `DOCTRINE_WORKER=1`
  and commits one delta `S`. Since SL-254 the spawn sets the marker and a
  worker cannot commit; `lib:reference/dispatch-mechanics.md` says so.
- **A-027 — elicit skill, entry kinds.** `plugins/doctrine/skills/elicit/SKILL.md`
  (~22-30) says there are two kinds; `doctrine compare elicit --help` lists
  four (`comparison`, `anchor-review`, `sizing-probe`, `claim-reprobe`).
- **A-054 — retrieve-memory skill, `--lifespan`.** `plugins/doctrine/skills/retrieve-memory/SKILL.md`
  (~51-57) describes an at-or-above threshold; `memory retrieve --help` says
  "Hard filter by lifespan". Confirm the real semantics, then fix whichever
  is wrong.

Fix by cutting to the owner (`lib:` / `--help` pointer) per DEC-345, not by
re-syncing the copy. Related: IMP-505.
