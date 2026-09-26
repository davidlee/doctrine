# IMP-492: Doctor checks install docs against the CLI surface

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

A `doctrine doctor` check that every `doctrine <verb>` and `--flag` a shipped
install doc or skill names exists in the clap surface, and every MCP tool/field
named exists in the server schema. Proposed by RFC-032 `decision-frontier.md`
D13; deferred from SL-268 by DEC-320, which runs a bounded manual guidance
review in-slice instead. Waits on IMP-491 (no spec governs doctor). Mind the
six-site cost of a new doctor category (`mem_019fe112c6af7d908afe714d41ddb718`).
