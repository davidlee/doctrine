# ISS-293: pi-review.sh completion poll misses agent_end, so every raiser burns its full backstop

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The defect

`scripts/pi-review.sh` decides a reviewer is finished by polling the tail of the
raw rpc stream every 2s:

```bash
if tail -c 131072 "$OUT" 2>/dev/null | grep -qE '"agent_end"'; then
```

The comment above it states the load-bearing premise:

> `agent_end` is always at the end by construction.

**That premise is false in pi 0.83.0.** `agent_end` is not the last event —
`agent_settled` follows it. And because pi's rpc stream re-serializes the entire
accumulated conversation state on *every* event (the same fact the script's own
header documents as "50–150MB/turn is normal"), that one trailing event is
enormous by itself.

## Measured, SL-233 review campaign, 2026-08-02

`sheets-census.log`, one ordinary census turn:

| | bytes |
|---|---|
| log size | 769,753,626 |
| last `"agent_end"` offset | 769,068,858 |
| last `"agent_settled"` offset | 769,753,609 |
| **`agent_end` distance from EOF** | **684,768** |
| poll window (`tail -c`) | 131,072 |

`agent_end` sits **5.2× outside** the window the poll reads. Reproduce:

```bash
L=.doctrine/state/slice/233/campaign/s0/sheets-census.log
stat -c%s "$L"
grep -bo '"agent_end"'     "$L" | tail -1
grep -bo '"agent_settled"' "$L" | tail -1
```

Positive control — the poll's own grep does match when given a big enough tail,
so the pattern is right and only the window is wrong:

```bash
tail -c 131072 "$L" | grep -c '"agent_end"'   # 0
tail -c 5000000 "$L" | grep -c '"agent_end"'  # 1
```

## Consequence

`REASON` never becomes `agent_end`, so the loop runs to `BACKSTOP` and only then
`kill -9`s the group. Every raiser holds a live `pi` process and an open API
session for the full backstop regardless of when it actually finished.

Observed across all five raisers of the SL-233 campaign's S0 and S1 sessions.
The S0 pair wrote their findings files ~3 minutes in and were still live at
39:47, dying at the 2400s backstop — roughly **37 minutes of dead session each**,
five times over. Nothing warns: the findings file appears on time, so the run
looks successful, and the operator discovers it only by running `ps`.

This is also why the script's belt (`ps | grep -- --session-dir`) never fired —
nothing survived *the reap*; the reap simply never happened early.

## Why it was not caught before

It is invisible unless you look for it. `IMP-024` §6 records the adjacent
symptom from the `RV-324` run — "orphaned raiser processes block a `wait` on the
spawner, so completion reports arrive late and look like 'still running'" — and
that was treated as fixed by the poll/reap work (`a69274c8`, `299b6a55`). The
poll was indeed added; its window was just sized against an assumption about
event order that does not hold. `CHR-051` §3 exists because two header comments
named the wrong model — this is the same failure mode one level down: a comment
asserting a stream property nobody measured.

## Fix

Match the actual terminal event, and stop betting on a window size:

```bash
if tail -c 4194304 "$OUT" 2>/dev/null | grep -qE '"(agent_settled|agent_end)"'; then
```

`agent_settled` lands 17 bytes from EOF, so it is robust to any window; matching
either keeps the check working if pi's event order changes again. The larger
tail costs nothing at a 2s cadence — the script already deliberately polls the
tail rather than the whole file for exactly this reason, and 4 MiB is still four
orders of magnitude below the file size.

Do **not** "simplify" the reap to `setsid` while here — it is absent from this
jail and breaks the spawn outright. `set -m` + `kill -9 -"$PI"` is the working
form and is already in place (`CHR-051`).

Worth adding alongside: a one-line `echo` of elapsed-to-completion versus
backstop, so a silent full-backstop run is visible in the operator's terminal
instead of requiring `ps`.

## Links

- `scripts/pi-review.sh` — the poll loop and its premise comment.
- `IMP-024` §6 — the adjacent orphaned-raiser symptom from `RV-324`.
- `CHR-051` — the pi spawn-surface defect register; this belongs beside it.
- Live instances: `RV-341` (SL-233 Kind A) and the S0 census pair.
