Editing a skill master that appears under two plugin roots is not ambiguous, and
not a duplicate: the second path is a symlink.

    plugins/doctrine-memory/skills/record-memory   -> ../../doctrine/skills/record-memory
    plugins/doctrine-memory/skills/retrieve-memory -> ../../doctrine/skills/retrieve-memory
    plugins/doctrine-partner/skills/pair            -> ../../doctrine/skills/pair
    plugins/doctrine-partner/skills/walkthrough     -> ../../doctrine/skills/walkthrough

All four are tracked as `120000`, so `git ls-files` lists the *directory* name
and the master count looks larger than it is (`plugins/doctrine/skills` alone is
the source of truth — [[mem.signpost.doctrine.skill-masters]]).

Two consequences worth knowing:

- An edit through either path lands in the same inode, but `git status` reports
  the `plugins/doctrine/skills/...` path — expected, not a lost edit. Do not
  "fix" it by copying the file, which would de-link the pair.
- A cross-plugin skill (memory, partner) is *published* under its own plugin id
  while living in doctrine's tree, so a grep over one plugin root alone
  under-counts the corpus.

Why it matters: it cost a detour during the IMP-488 guidance sweep to work out
whether `plugins/doctrine-memory/skills/retrieve-memory/SKILL.md` was a second
master needing its own edit.
