# IMP-516: Install explains itself in plain English, then hands over a commit

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

Interactive `doctrine install` (without `-y`) is hard to follow (observed
2026-10-16, fresh `/tmp` project):

1. **Its questions mean nothing to a user.** "Materialize shipped memory
   corpus?", "Project exposed hymn starters?" use internal vocabulary.
2. **Some aren't real choices.** Doctrine doesn't work properly if you answer N
   to any of them, so the prompt is a trap, not a decision.
3. **No overview first.** The first screen is a raw step list: `create dir`
   repeated for the same path, eleven `gitignore +` lines. Forward steps are
   disclosed one prompt at a time, after the base install has already run.

## Direction

- Open with a plain-English summary of what will happen and why, e.g.:
  - "We'll add a managed block to `.gitignore` so generated files stay out of
    git."
  - "We'll create `.doctrine/`; everything in it that isn't meant to be
    customised is ignored."
  - "We'll add a short section to the top of `CLAUDE.md` / `AGENTS.md` so your
    agent loads doctrine's context."
  - Offer: "Want more detail?" (shows the step list).
- Ask one question: proceed or not. Required steps aren't optional prompts.
  Keep separate choices only for real options (which harnesses; a skills
  subset).
- Finish with the next step, offered or printed:
  `git add .doctrine CLAUDE.md AGENTS.md .gitignore && git commit -m "…"`.
- `-y` behaviour unchanged.

## Related

- SL-281 — install artefact ownership; its managed gitignore block and ownership
  split are what the summary describes. Natural next slice after it.
- IMP-513 — once hymn starters leave install, their prompt goes away.
- IMP-514 — missing/stale install guidance; shares the "say what to do next"
  surface.
