A self-paced loop that finds everything quiet has three candidate explanations.
They present identically and route differently, so name the one you are in
before acting. (Silence is never proof of *completion* — only a notification is.)

## 1. The worker died — resume it, do not re-spawn it

`SendMessage` to the dead agent's id resumes it **from its saved transcript**,
with everything it learned still in context. A fresh spawn starts from the sheet
and re-derives all of it. On a phase that measured host behaviour the design got
wrong — the kind that fills a findings ledger — that context is the expensive
thing in the tree, not the code.

The revive message states **disk state, not instructions**: the tip sha and what
it contains, which boxes are ticked, exactly what is dirty and how large, and
"nobody has touched it since". Then re-state the contract in one paragraph — a
resumed agent's oldest context is its briefing, and that is what degrades first.
Two things worth saying every time:

- **assess the uncommitted work before building on it** — it was in flight when
  the process died and may not compile;
- **if finishing it costs more than redoing it, redo it** — sunk cost is not
  evidence, and a worker resumed mid-edit is prone to defending its own draft.

**Check `TaskList` before planning on resume.** The transcript is reachable only
while the agent registry is. An empty `TaskList` where a worker should be means
resume is unavailable at any price: re-spawn, and treat the tree as your only
witness. Read the tip, the ticked boxes and the findings shard before writing
the brief — a worker that got most of the way through usually left its evidence
in the shard even though its context is gone.

## 2. The harness process died — nothing will notice but a human

It takes the worker *and* the loop's own `ScheduleWakeup` with it, so the loop
**stops firing entirely** and only restarts when a human re-invokes it. No guard
can catch this, because the guard runs only when the loop runs.

The tell: every liveness leg cold *at once*, by hours, with a clean recent
commit and a plausible dirty file. That looks like a stalled worker; it is a
stopped loop.

## 3. The sandbox was rebuilt — same symptom, worse blast radius

The agent registry goes too, so the transcript is unreachable and resume is off
the table. One command distinguishes it: `ps -o pid,etime,comm -p 1`. A pid 1
younger than the work you are looking at means the whole jail was torn down and
restarted, not merely a process killed inside it.

## What follows for every brief

**Commit each task as it goes green**, not at the phase's end. The uncommitted
window is the entire exposure to all three deaths, and it is the one variable
the orchestrator controls from outside the worker. This is the same argument as
harvest-as-you-go, applied to the tree instead of to the knowledge.
