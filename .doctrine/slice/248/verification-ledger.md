# Verification ledger — SL-248

**The orchestrator's own gate runs.** Tracked and committed, because this is
evidence: it is what `/audit` reads to know that each task's claim was checked
by something other than the agent making it.

It lived in `handover.md` until 2026-08-10, which was wrong — that file is
gitignored and `rm -rf`-able, and this is the one record in it that existed
nowhere else. `LOOP.md`'s own rule ("never put anything load-bearing only
there") had been broken by the file that states it. Moved on the slice owner's
call; `handover.md` now references this file and holds no durable state.

## Why the pair, and why it is not optional

The phase sheet records each **worker's self-tally**. The rows below are
**independent runs** taken outside the worker's own process, which is what
`LOOP.md` beat 3 requires.

**Three times a worker's honest tally has been contradicted, and every time by
the first run taken outside its own process.** A strong hand-back tally is
exactly when the independent run is worth taking, not when it can be skipped.

Two reading conventions, both of which have bitten:

- **Anchor on the `doctrine_control-` binary header.** That crate's
  `test result:` line is **neither first nor last** in the gate log —
  `awk '/doctrine_control-/{f=1} f && /^test result:/{print; f=0}'`, never
  `head -1` / `tail -1`.
- **Run it twice.** The suite measures elapsed time, so a single green run is
  not evidence about a timing-sensitive change. Both arms must agree.

## The ledger

| task | row / subject | commits | orchestrator's own gate | suite |
|---|---|---|---|---|
| `T1` | measure four deltas | `4fcc9ab07` | — (measurement only) | — |
| `T2` | per-arm setup seam | `d1859c134` | contradicted: 1 of 3 red (`F-16`) | — |
| `T3` | row 9, mounts | `f1272ac25` | 6/6 PASS | 241 |
| `T4` | row 10, descriptors | `18047fc69`…`b81edd6e0` | contradicted: first run red | 248 |
| `T4/r` | the residual | `24155506e`, `22d2f34d4` | 2/2 | 254 |
| `T5` | row 11, environment | `8b31c073c` + 3 | 2/2, 92.01 / 92.03 s | 254 |
| `T6` | row 12, stdio | `4ed5e9c33` + 3 | 2/2, 92.23 / 92.33 s | 260 |
| `T7` | row 13, identity | `d0b75c49c` + 2 | 2/2, 92.31 / 92.34 s | 264 |
| `T8` | row 14, capabilities | `81fd5b20a`, `6ed8022b6` | 5/5 (worker's, read off raw logs) | 268 |
| `T9` | unrowed observations | `78ee8f8b6`, `2ee55afb7` | 2/2, 92.30 / 92.36 s | 269 |
| `T10` | `backend verify` | `42937f4db`, `57e24b542` | 2/2, 92.46 / 92.74 s | 274 |
| `T11` | the honesty pass | `80bedf1bf`, `876025a6e` | 2/2, 92.36 / 92.49 s | 277 |
| `T12` | `VA-3`, the suite's cost | `4e3882b72` | 2/2, wall **140 s**, 92.54 / 92.41 s | 277 |
| `T9`**/09** | `PHASE-09` row 7, **partial** | `c59f35ce2`…`9d48adbc8` | 2/2, wall 138 / 136 s, 92.56 / 92.13 s | 291 |
| `T9`**/09** | `PHASE-09` row 7, **complete** | `06ff02661`…`bcfcfec80` | 2/2, wall 136 / 136 s, 92.12 / 92.32 s | **296** |

`PHASE-09` **closed 2026-08-10** on that pair: 12/12, gate exit 0, `verify-vt`
`VT-1`…`VT-5` all PASS, and `doctrine-control backend verify` exits **0** with all
fourteen rows `Proven`, five axes `Proven`, every claim `Passed`. `UNWALKED` is
gone from the source. Suite 291 → 296 (**+5**), matching the worker's claim, and
**wall clock did not move** — the row-7 test fell 26.27 s → 6.26 s when the
escapee was re-sized, paying for its own additions. Warning lines back to **19**
from the partial's 18, both arms agreeing, 0 compiler diagnostics.

**The one contradiction that did not happen.** This worker's self-tally matched
the independent run exactly, and it had independently reached the same conclusion
as the orchestrator's mid-flight correction about which arm the cost lands on.
Recorded because the ledger would otherwise only ever preserve the failures, and
"three tallies contradicted" is a misleading base rate without its denominator.

`T1`–`T12` are `PHASE-10`'s. The last row is **`PHASE-09`'s `T9`**, not
`PHASE-10`'s — two different tasks share that number and the slice's prose cites
both, so always qualify it.

**Every suite delta has matched the tests the worker claimed to add.** Checking
that equality is the point of recording the suite column.

**Current baseline: 291 passed, 0 failed, 9 ignored.**

`PHASE-09` `T9`'s gate logs carry **18** `warning:` lines where every earlier row
carried 19. Both arms agree and a grep for compiler diagnostics returns **0**, so
this is doctrine's own test-output message count moving, not a warning
regression. Recorded because the next reader will otherwise re-investigate it.

## Delta boundaries — what `record-delta` must exclude at close

A phase's delta is **one contiguous commit range**, so a driver commit landing
between a worker's first and last cannot be excluded by any `--start`/`--end`.
It rides into the phase and surfaces in `slice conformance`'s undeclared cell.
Every phase so far has needed `record-delta` for this; `PHASE-06` had one that
could not be tightened out at all.

**Tighten these out — they are driver commits, not phase work:**

| commit | what |
|---|---|
| `dcdcff6ce` | a `LOOP.md` rule |
| `fe12005b1` | the § *Owed* repair |
| `f730eef0f` | the guard repair ("silence is not proof of completion") |

**These are NOT driver commits — they are phase work and belong *in* the delta:**

| commit | what |
|---|---|
| `91ac79006` | the notes-shard mint |
| `fe2e0bdb9` | `VA-1` off-jail run, owed items 158–161 |
| `34c6181eb` | the row 7 payload ruling |

`f730eef0f` sits *after* `PHASE-09` `T9`'s commits while `PHASE-10` is also open,
so **both phases' ranges can claim it**. Tighten it out of whichever one sweeps
it up, and then check the other.

**`1d33865ed` is interior and probably cannot be tightened out at all.** It
landed *between* `PHASE-09` `T9`'s commits, while the worker was live — the slice
owner committed the off-jail audit (`offjail-audit.md`, `offjail-handback.md`)
and swept up this file with it, both being untracked at the time. A phase's delta
is one contiguous range, so an interior commit is unreachable by any
`--start`/`--end`. `PHASE-06` has the precedent. Declare it rather than fight it:
the audit files are slice evidence and arguably *are* phase work; this file is
orchestrator infrastructure and is not. Say so at reconciliation instead of
leaving the auditor to infer it.

The general rule this re-teaches: **hold your own commits while a worker is
live** — and that applies to the human as much as the orchestrator, so a
handback is the moment to say "safe to commit now".

`record-delta` only if the sweep caught commits genuinely not the phase's —
dropping real phase work is what the CLI refused at `PHASE-07`, correctly.
`verify-vt` cannot attribute before the flip and the boundary (`notes.md`
item 57).
