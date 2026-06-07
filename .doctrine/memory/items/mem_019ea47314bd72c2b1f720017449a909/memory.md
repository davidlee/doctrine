# Usage guidance must ship to clients, not hide in the build-repo CLAUDE.md

This repo's `CLAUDE.md` serves people **building** doctrine. Clients who
**install** doctrine into their own repos get the shipped surfaces — the boot
snapshot (`install/routing-process.md` + `governance.md` + memory/ADR sections),
the skills (`plugins/doctrine/skills/`), `install/rules/AGENTS.md`, templates —
but **never this repo's `CLAUDE.md`**. So any guidance about *using* doctrine
that lives only in `CLAUDE.md` is invisible to every client.

**The split that governs placement:**
- *Building doctrine* guidance (the Rust layout, the impure seams, the dev
  build target, contributor lint gotchas) → stays in `CLAUDE.md`.
- *Using doctrine* guidance (how to read entities, the storage tiers, reference
  forms, lifecycle, the change loop) → must ship on a client-facing surface.

**Push vs pull when you ship it:**
- *Push* (always in context): `install/routing-process.md` rides the boot
  snapshot into every session's cached prefix. Use for rules that must fire
  without being invoked.
- *Pull* (read on demand): a skill (`/canon`, etc.). A pull-only home does NOT
  prevent an error made *before* the skill is invoked — a new "how to use it"
  skill repeats the failure mode and duplicates `/canon`.

**Worked failure (the origin):** an agent judged a requirement "hollow" by
reading its `.md` prose tier (empty by design); the statement + acceptance
criteria were in the sibling `.toml`. The storage rule stated the *writing*
split but not the *reading* consequence, and lived only in `CLAUDE.md`. Fixed in
commit 8206b67 by landing the tier-aware read-rule on the boot asset + `/canon`
+ `/inquisition`. See [[mem.pattern.embed.rustembed-recompile-and-symlinks]] —
the boot asset is embedded, so an edit needs a full crate rebuild (`cargo clean
-p doctrine`) + `doctrine boot` to reach the snapshot.

**Why:** doctrine's value is correct agent behaviour in *client* repos; guidance
that never reaches the client cannot shape it.

**How to apply:** before writing usage guidance into `CLAUDE.md`, ask "does a
client need this to use doctrine correctly?" If yes, put it on a shipped surface
(boot asset for push, skill for pull) and let `CLAUDE.md` keep only the
build-doctrine half. Open follow-up: audit `CLAUDE.md` end-to-end and rehome
every usage-guidance fragment (the storage rule itself included).
