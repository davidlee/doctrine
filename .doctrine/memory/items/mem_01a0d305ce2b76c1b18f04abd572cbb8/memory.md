`design apply` payload keys are homed per subject prefix. On an `inq-` subject,
`resolution`, `dispose` and `blocking` are refused as **inert** ("honoured for
`fnd-` / `cp-` subjects"). Neither `design contract` nor the inquiry fragment
says where they belong.

Lawful shapes (verified SL-262, 2026-09-24):

- Resolve an inquiry into a knowledge record:
  `{"subject":"cp-1","disposes":"inq-1","dispose":{"form":"create","kind":"decision","title":…,"facet":{…},"acceptance":{"basis":…}}}`
  → `checkpoint_disposed … record=DEC-NNN`. Other forms: `adopt` (existing
  record), `unresolved`, `non-durable` (`note`).
- Declare the blocking set: not a node flag — the agent act
  `"agent_declaration":{"act":{"blocking-set-declared":{"blocking":["inq-1",…]}},"basis":…}`,
  recorded in the same submission as the user's `graph-reviewed` checkpoint act.

Related: [[mem_019ff439bede7fb29c7c09b7fd76d893]] (payload vocabulary undiscoverable).