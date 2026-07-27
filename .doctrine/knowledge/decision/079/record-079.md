# DEC-079: Live managed-design evaluation follows SL-233 closure

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

SL-233 authors the deterministic fixtures, moderator protocol, scoring rubric,
and evidence surfaces for managed-design evaluation, but does not gate closure
on a live agent comparison.

The live human-in-the-loop measurement runs immediately after closure under
CHR-049, once the changed skill and prompt assets are genuinely available to
the harness. This avoids a false pre-close evaluation from a dispatch worktree:
Doctrine skills are installed through the published GitHub/plugin path rather
than loaded from candidate worktree bytes.

The post-close exercise remains an explicit deliverable, not indefinite
dogfooding. It measures knowledge-checkpoint adoption, map steering, refresh
and recovery, prompt cost, human interview usefulness, and resulting design
quality. Findings inform the next process/prompt revision and any RFC-021
authority fast-follow.
