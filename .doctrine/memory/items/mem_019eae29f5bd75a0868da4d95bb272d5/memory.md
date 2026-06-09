# Reuse tuned prior art verbatim, attribute it

When a well-tuned source (e.g. `superpowers:*`) already solves part of the
problem, reuse its prose word-for-word where it fits. Do NOT reword for novelty's
sake — paraphrasing well-tuned instructions degrades them.

**Why:** skills like `superpowers:using-git-worktrees` are battle-tested; their
exact phrasing (directory-priority, `git check-ignore` safety, "fix broken things
immediately") carries the tuning. Rewriting loses it and adds risk for zero gain.
Write new prose only for the genuinely new substance with no prior-art equivalent.

**How to apply:**
- Copy the tuned passages; rewrite only what is doctrine-specific.
- `plugins/` is MIT — superpowers is MIT (Jesse Vincent, github.com/obra/superpowers).
  Word-for-word copy is license-compatible AND requires attribution: credit in the
  skill footer + a README `## Acknowledgements` entry.
- Confirmed by user during SL-029 PHASE-02 ("don't want novelty for the sake of
  novelty; word-for-word copy where it makes sense is good").

Related: [[mem.pattern.skill.description-is-the-trigger]].
