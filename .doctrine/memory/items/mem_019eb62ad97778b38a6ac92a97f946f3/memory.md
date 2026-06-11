# Resolve every ref operand in the shell before a pure equality guard

A safety-guard verb must resolve every ref operand in the impure shell before the
pure equality compare; verbatim trust on one side silently defeats the guard.

ISS-002 / SL-041: `worktree::run_branch_point_check` resolved `--head` to a sha
but passed `--base` (and a passed `--head`) verbatim into `matches` (raw `base ==
head`). `--base HEAD --head HEAD` string-matched as "stationary" against a base
the verb never resolved — the unsafe direction, the guard passing when it should
refuse.

Fix shape: a `resolve_commit(root, ref)` shell helper (`git rev-parse --verify
<ref>^{commit}`) peels *both* operands to a commit sha before the compare; an
unresolvable ref bails (the safe failure direction). The pure leaf (`matches`)
stays ref-equality only — resolution lives in the impure shell (ADR-001).

Tells: (1) the bug hides between two individually-sound parts — a correct ref
resolver and a correct equality leaf, wired so one operand skips the resolver
(see [[mem.pattern.review.interaction-bugs-hide-between-sound-parts]]); (2) peel
with `^{commit}`, not plain `rev-parse`, or an annotated tag resolves to the tag
object, not the commit. Don't fix only the operand named in the bug title — the
verbatim-trust defect is usually symmetric across all operands.
