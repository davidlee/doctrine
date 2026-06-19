---
seq: 0026
scope: codebase
target: discovered — backlog staleness detection (complements 0025, feeds 0011)
confidence: med
reversible: yes (read-only analysis; nothing authored)
---
## What
The done-but-open class has **at least two instances**, and doctrine has **no
detector** for it:
- **IMP-081** — `--trust`/`--severity` flags shipped (`memory record --help`), item
  still `open` (confirmed, proposal 0025).
- **IMP-103** — "clarify `--trunk` dry-run semantics": `dispatch sync --help` now
  documents `--trunk` ("Absent ⇒ trunk is left untouched") and `--integrate` stages,
  which reads as the clarification asked for. Item still `open` (med confidence —
  no body to confirm full intent).

Finding these required me to *guess* a capability and check `--help`. There is no
mechanizable signal that an open backlog item is actually finished, so done-but-open
items accrete silently and pollute `survey`/`next` (the actionability graph the
product sells). 0025 proposes the *manual* sweep; this proposes the *detector* that
makes the sweep automatic — and it is the natural new check for the `doctrine doctor`
health gate (proposal 0011).

Two tractable signals (no NL understanding required):
1. **Linkage signal** — an open backlog item whose linked `slices:` are *all* in a
   terminal state (done/closed) is very likely resolved. This is a pure graph query
   (item → slice → status) over data doctrine already holds. (Caveat: I could not
   confirm instances this session — `slice list` surfaced no done-state slices in
   the snapshot, so the signal needs the done-slice corpus to be non-empty to fire;
   worth checking against the real status set.)
2. **Capability signal** — weaker/heuristic: items whose title names a flag/verb
   (`--trust`, `sync --integrate`) that now exists. Not reliably mechanizable; better
   left to the manual sweep (0025).

## Options
1. **Add a linkage-based staleness check** (signal 1) to `backlog validate` /
   `doctrine doctor`: list open items whose linked slices are all terminal, as
   advisory "candidate-resolved." Tradeoff: pure graph query over existing edges, no
   NL; advisory-only (never auto-closes); the highest-precision mechanizable signal.
2. **Manual periodic audit only** (0025's approach, no detector). Tradeoff: zero
   build; relies on someone remembering to sweep; misses items between sweeps.
3. **Leave as-is.** Tradeoff: done-but-open accretes; `next`/`survey` advertise
   finished work (IMP-081 proves it already happens).

## Recommendation
Option 1, scoped as an **advisory** check folded into the `doctrine doctor` gate
(0011): an open item whose every linked slice is terminal is flagged "candidate
resolved — verify and close," never auto-transitioned (the human disposes, per the
fence's own logic). Rationale: it reuses the relation graph (item→slice edges +
status) doctrine already computes — same reuse posture as the other proposals — and
turns 0025's manual sweep into a standing signal. Keep it advisory: a terminal slice
doesn't *prove* the item is done (an item may outlive its first slice), so it
proposes, never disposes — exactly the right default.

Decisions deferred to YOU:
- (a) **build the linkage detector, or rely on manual sweeps** (0025)?
- (b) where it lives — `backlog validate`, `doctrine doctor` (0011), or `status`?
- (c) confirm signal-1 fires: are there open items linked to terminal slices in the
  real status set? (I couldn't confirm from the snapshot — `slice list` showed no
  done-state rows; verify the done-slice corpus exists before building.)

## Next doctrine move
```
# confirm the signal would fire (read-only):
doctrine slice list                       # which states exist? any terminal?
# (then for each open backlog item, check if its linked slices are all terminal)

# capture the detector (NOT executed — fence forbids backlog transition):
doctrine backlog new improvement "Backlog staleness detector: advisory flag for \
  open items whose linked slices are all terminal (done/closed) — graph query over \
  item->slice edges; fold into doctrine doctor (proposal 0011); automates the \
  0025 done-but-open sweep. Advisory only, never auto-close" --tag area:backlog \
  --tag area:cli
```
(Verbs described, NOT executed.)

## Illustration (optional)
None — a graph query over existing item→slice edges + status; design question is
*where it lives* and *advisory-only*, which a speculative diff would prejudge.
