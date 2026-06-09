# Skill description is the auto-trigger surface

A skill's `description:` frontmatter is the one chance to make the harness invoke
the skill at the correct moment — it is matched against the situation to decide
auto-execution. It is NOT a catalogue blurb.

**Why:** if the description reads as a static summary ("Lifecycle for X — does A,
B, C"), the skill won't fire when it should. The well-tuned prior art
(`superpowers:*`) writes descriptions in trigger form: "Use when <situation> —
<what it does>." That phrasing is load-bearing, not stylistic.

**How to apply:** write every skill description as a trigger. Lead with the
*when* (the situation/intent that should reach for it), then the *what* (concise).
Even for a sub-skill not in the `/route` table (e.g. `/worktree`), the description
still governs auto-invocation, so name the invoking situation ("when /execute
must run a phase in isolation"). Confirmed by user during SL-029 PHASE-02.

Related: [[mem.pattern.authoring.reuse-tuned-prior-art-verbatim]],
[[mem.pattern.distribution.skills-source-vs-installed]].
