# ISS-294: pi-review.sh reap belt reports survivors that are merely mid-kill

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The defect

`scripts/pi-review.sh` ends its reap with a belt that checks nothing outlived it:

```bash
kill -9 -"$PI" 2>/dev/null || kill -9 "$PI" 2>/dev/null
kill -9 -"$KEEP" 2>/dev/null || kill -9 "$KEEP" 2>/dev/null
rm -f "$PI_FIFO"

if ps -eo pid,args 2>/dev/null | grep -q "[-]-session-dir $SESSION_DIR"; then
  echo "[review] $LABEL WARNING: pi survived the reap for $SESSION_DIR" >&2
fi
```

`kill -9` is **asynchronous**. It queues the signal; it does not wait for the
target to die. The `ps` runs microseconds later, with no settle window and no
`wait`, so a process that is mid-kill — or a not-yet-reaped zombie child — still
appears. The belt then reports a survivor that is already gone.

## Observed, SL-233 review campaign S2, 2026-08-02

The `kindB-p16` raiser printed:

```
[review] kindB-p16 WARNING: pi survived the reap for .../.pi-session-kindB-p16
[review] kindB-p16 terminated reason=agent_complete after 395s of 3600s backstop
```

A `ps` moments later showed **no** process under that session dir. Nothing had
survived.

Honesty about the evidence: this was not caught in the act. The two candidate
explanations are (a) the race above, and (b) a process that genuinely outlived
the reap by a short interval and then exited on its own. Both are consistent with
what was seen. What is certain either way is that **the warning did not
correspond to a persistent orphan**, and that the script offers no settle window
that would let it distinguish the two.

## Why it matters

The belt exists to tell an operator to go hand-reap something. `ISS-293` has just
established that a genuine survivor is expensive and invisible — so this warning
is exactly the signal an operator is now trained to act on. A belt that cries
wolf on a clean run is worse than no belt: it costs a `ps` investigation every
time, and it teaches the operator to ignore the one case that matters.

## Fix

Give the kill time to land, and confirm rather than guess:

```bash
kill -9 -"$PI" 2>/dev/null || kill -9 "$PI" 2>/dev/null
kill -9 -"$KEEP" 2>/dev/null || kill -9 "$KEEP" 2>/dev/null
wait "$PI" "$KEEP" 2>/dev/null      # reap our own children; no zombie in ps
rm -f "$PI_FIFO"

for _ in 1 2 3 4 5; do              # settle: grandchildren are not our children
  ps -eo pid,args 2>/dev/null | grep -q -- "--session-dir $SESSION_DIR" || break
  sleep 0.2
done
```

`wait` is the important half: `$PI` and `$KEEP` are this shell's own children, so
without it they linger as zombies that `ps` will list. The bounded settle loop
covers the grandchildren (`timeout` → `bwrap` → pi wrapper → pi), which are not
this shell's children and cannot be waited on.

**Do not** "simplify" the reap itself while here. `setsid` is absent from this
jail and breaks the spawn outright; `set -m` + `kill -9 -"$PID"` is the working
form (`CHR-051`, and `ISS-293` for why `$KEEP` is now group-killed too).

## Why it was not fixed on the spot

Found mid-campaign while S2's second cheap pass was **executing this script**.
Bash reads a script incrementally rather than slurping it, so editing a running
script can corrupt the running instance. Deferred deliberately rather than
risking two live raisers.

## Links

- `scripts/pi-review.sh` — the reap and its belt.
- `ISS-293` — the two reap defects fixed immediately before this was noticed;
  that fix is what made the belt fire often enough to be worth auditing.
- `CHR-051` — the pi spawn-surface defect register; this belongs beside it.
