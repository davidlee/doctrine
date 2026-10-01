# ISS-509: check gate and check commit outrun the default agent command timeout

## What

`doctrine check gate` and `doctrine check commit` take 2–3 minutes; the agent harness's
default foreground command timeout is 2 minutes. The run is killed mid-gate, and agents
then background it and poll, or pipe through `tail` (which masks the exit code — see
IMP-396).

## Fix sketch

Options, cheapest first: document the timeout to pass in the skills that mandate the
gate; make `check` emit a progress heartbeat plus a final verdict line; cut gate time
(IMP-502 duplicate suite runs, IMP-503 slow e2e tests).

## Evidence

`01a0ba7f` (2026-09-19, check commit), `01a0dda0` (2026-09-26, check gate). Related tail/exit-code records (IMP-396 territory): `019fc6de` (`/close`, red gate read as exit 0), `01a0ba42` (capsule-driver, background tail lost transcript and status).
