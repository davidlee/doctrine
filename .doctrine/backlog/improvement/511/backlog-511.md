# IMP-511: Mutation-beat helper for worker negative controls

## What

Workers prove a test has teeth with a mutation beat: mutate code or fixture,
expect red, restore. Every worker hand-rolls the driver, and the same failures
recur (EVD-031; ~16 mutation-related observations):

- the mutation silently did not apply (sed/perl non-match) and the green is read
  as a verdict;
- the mutation cannot compile under `-D unused` / `dead_code`, and the compile
  error is miscredited as the test convicting;
- a grep-filtered or tail-piped run hides the real outcome;
- a formatter pass mid-battery invalidates mutation anchors.

## Direction

A small helper: apply a mutation → assert it applied → build → run the named
test(s) → classify the red (compile error vs assertion failure vs pass) → restore,
always. RFC-023 Direction F (durable home for the negative control).
`mem.pattern.testing.mutation-beat-asserts-application` documents the
discipline the helper would enforce.

## Placement

Probably a repo script / `just` recipe before it is a doctrine verb (POL-002:
cargo and test layout stay out of the engine). Decide in design.
