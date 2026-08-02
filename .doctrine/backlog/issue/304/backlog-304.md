# ISS-304: pi agent profiles are gitignored, so the runners are broken on a fresh clone

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed

`.gitignore:6` ignores `/.pi`. `scripts/pi-scout` and `scripts/pi-research` are
tracked, and both hard-exit unless `$PROJECT_ROOT/.pi/agents/{scout,researcher}.md`
exists:

```
ERROR: agent file not found: /workspace/doctrine/.pi/agents/scout.md
```

So the two runners `CLAUDE.md` tells agents to prefer over harness subagents
(*"DON'T use subagents … do use `./scripts/pi-scout` … or `./scripts/pi-research`"*)
do not work in a fresh clone, and nothing says why.

## Why it matters beyond the clone case

This is the **root cause of CHR-051 §3**. The profiles carry the model and tool
mapping the scripts load, they are unversioned, and so they drifted into
advertising the exact inverse of the documented tiers (`pi-scout` billed as
"quicker, cheaper" while resolving `deepseek-v4-pro`, `pi-research` billed as
"smarter" while resolving `deepseek-v4-flash`) with no diff, no review, and no
way to notice. CHR-051 is resolved, but its fix was applied to **untracked local
state** — it is per-machine, will not reach another clone or another agent, and
is one `rm -rf .pi` from gone. The same drift can silently recur tomorrow.

## Suggested fix

Un-ignore `.pi/agents/` while keeping the rest of `/.pi` ignored — it holds
session, auth, and cache state that must stay out of git:

```gitignore
/.pi
!/.pi/agents/
```

Then commit `scout.md` and `researcher.md` as they now stand (corrected models,
real researcher persona). Note `.pi/agents/dispatch-worker.md` is already a
symlink into tracked `.doctrine/agents/codex/`, so the directory is *partly*
version-controlled by intent already — this makes that consistent.

Worth checking as part of the same pass: whether `doctrine install` should seed
these profiles, which would remove the fresh-clone failure independently of the
`.gitignore` decision.

## Scope note

Raised rather than taken during the CHR-051 / ISS-266 / IMP-322 repair pass
(2026-08-02): editing `.gitignore` to begin tracking a directory is a repo-policy
call, not a chore fix.

## Links

- Root cause of **CHR-051** §3 (resolved; see its "Root cause of the drift"
  section).
- Siblings repaired in the same pass: **ISS-266**, **IMP-322**.
- Consumers: `CLAUDE.md` "Research", `.doctrine/governance.md` "Research agents".
