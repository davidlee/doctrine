# CHR-049: Run post-SL-233 managed-design measurement exercise

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Intent

Immediately after SL-233 closes and its skill/prompt changes are available to a
real harness, run the live human-in-the-loop measurement exercise defined by
the slice. Use the findings to choose prompt granularity, enforcement,
authority priming, and the next RFC-021 descent rather than treating closure as
proof of behavioural adoption.

## Entry checks

- Confirm the installed `/design` skill bytes and embedded prompt assets contain
  the SL-233 changes; authored or dispatch-worktree presence is insufficient.
- Preserve or identify the pre-SL-233 skill baseline if the protocol includes a
  paired comparison.
- Use the normal best-available model family and real human moderator.

## Primary observations

- Unprompted creation and slice-linking of appropriate `DEC`, `QUE`, and `ASM`
  records during the interview, including checkpoint latency.
- Correct map maintenance and user-directed traversal.
- Refresh and exact resume across a deliberate context break.
- Prompt/tool/token overhead and repeated-fragment elision.
- Human-rated interview usefulness and adversarial review of the resulting
  design.
- Accuracy of user-acceptance attestations: sample whether each concise
  `basis` faithfully describes its cited turn (when available), whether a
  reviewer or resumed agent can detect a mismatch, and whether the field earns
  its interaction/token cost.

If adoption is weak despite successful activation, run the small
`.doctrine/governance.md` authority-primer arm described by RSK-229 before
expanding prompt volume.

## Boundary

SL-233 must leave behind the fixture, moderator protocol, scoring rubric, and
mechanically collectible evidence surfaces. This chore owns the live run and
interpretation after the changed skill is genuinely installed.
