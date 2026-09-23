SL-244's `design_prompts_have_no_consumer_outside_the_design_run`
(`tests/e2e_claude_install.rs`) collects every file containing the literal
string `design-prompts` and asserts set-equality with a small allowlist.
It scans `install/` and `plugins/` too, so a **prose pointer** in a shipped doc
or skill (e.g. "owned by `design-prompts/reviewing.md`") turns the gate red.

Grepping for the fragment's path (`reviewing\.md`) will not find this pin — the
test keys on the *store name*. Before naming a fragment by address anywhere
outside the design run, grep `tests/` for `design-prompts`.

Remedies: add the file to `store_allowlist` with a comment stating it is a
pointer and consumes nothing (what SL-260 did for `install/review-ledger.md`),
or name the fragment without its store path. The separate `name@digest`
wire allowlist guards the receipt protocol and should not be widened for a
prose mention.
