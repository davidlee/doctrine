# Review RV-369 — reconciliation of SL-245

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed.** Not a dispatch candidate branch. SL-245 was implemented in
an **oubliette capsule** (ADR-020) and handed back on `capsule/SL-245/c11`, a
strict fast-forward of `edge` (`b3030d71e..7ff010a1d`, plus the audit's own
fix-now commit). The audit ran in the linked worktree `.worktrees/SL-245-c11`;
the ledger is driven from the parent tree, because `doctrine review`'s
baton verbs refuse a fork (IMP-024).

**Why this audit was not a formality.** The capsule could run every automated
leg and none of the human ones: PHASE-05's `VH-1..VH-4` are the acceptance test
the design nominated (`DEC-257`), and the design explicitly rested on them —
*"whether the picture is right is a human judgement, and that judgement is the
acceptance test."* The slice therefore arrived gate-green with its acceptance
criteria entirely unrun. That gap is the primary line of attack.

### Lines of attack

1. **The gap between green and working.** Every VT is pure or a refusal path;
   `-X`'s success path had never executed anywhere. Probe specifically for
   defects that live in what the *impure shell feeds* the pure decisions, which
   is by construction the blind spot of a suite built from injected values.
2. **POL-002 facet (3)** — a feature-scoped host capability must be opt-in and
   must fail naming what was missing and what would satisfy it. Every refusal
   message is user-facing text and must name the flag and its remedy.
3. **The refusal paths must leave stdout untouched.** `DEC-254`: no escape byte
   may reach a non-terminal, and no half-written escape may survive a failure.
4. **Design/plan/scope coherence.** The locked design is deliberately ahead of
   the slice (the 2026-09-15 scope cut — `IMP-451`, `IMP-452`, `CHR-072`). Where
   the code departs from `design.md`, the departure must be the *declared* one,
   not a new one.
5. **`DEC-256`'s placement rule is provisional** and was to be decided by VH-1
   and VH-2, including on HiDPI.
6. **ADR-001 layering** — three new leaves (`graphviz`, `kitty`,
   `terminal_image`), and `kitty → tty` is a leaf→leaf edge that must stay
   `WindowGeometry`-only.
7. **STD-001** — the kitty protocol is nothing but meaningful literals; the same
   bytes are written by the encoder and read by the classifier, so a drift
   between them would silently misclassify every terminal.

### Invariants held

- `build_graph_output` is untouched and every path without `-X` is
  byte-identical (PHASE-05 `EX-4`).
- No `unsafe`, no `poll`/`select`, no `O_NONBLOCK` in the tty query path
  (PHASE-03 `VA-1`, `DEC-259`).
- Raw mode is restored on every return path, and a failed restore outranks
  whatever the exchange produced.
- `"dot"` is single-sourced; the only surviving literal in `map_server` is the
  `/health` wire key (PHASE-02 `VA-1`, `DEC-143`).

### Evidence run

| leg | result |
|---|---|
| `doctrine check gate` | green (clippy zero warnings, all suites) |
| `doctrine slice verify-vt 245` | 17/17 PASS across PHASE-02/03/05 |
| `doctrine slice conformance 245` | 13 conformant · 0 undelivered · 1 undeclared |
| `VA-1` ×3 (PHASE-02/03/05) | pass |
| `VH-1..VH-4` | run by the user in ghostty during this audit — see findings |

## Synthesis

**SL-245 was gate-green, 17/17 on its automated verification, conformant against
its own design-target selectors — and its central feature did not work at all.**
That is the finding this audit exists to have made, and everything else here is
downstream of understanding why it was possible.

### What happened

`-X` is guarded by five checks before it draws. The second of them asked "is
stdout the terminal I am running in?" and answered it by comparing the device id
of stdout against the device id of an fd opened from `/dev/tty`. That comparison
is not merely wrong sometimes; it is wrong always. `/dev/tty` is a devnode in its
own right and the fd inherits its identity `(5,0)`, while stdout on a pty carries
`(136,N)`. `Endpoint::Same` was unreachable. Every terminal on earth was refused
with a message explaining that it was the wrong terminal.

The second blocker sat behind the first, and could only be found once the first
was fixed: the whole-corpus graph rasterises to a 61 MiB PNG that ghostty
declines, `q=2` suppresses the rejection, and the user is left looking at the 237
newlines the encoder appends. `DEC-256` bounds the placement in *cells*, and a
199 × 237 cell rectangle is entirely unremarkable — the cell bound was mistaken
for a resource bound, and nothing anywhere measured bytes.

### Why the tests could not see either

Not because the suite was thin. It is a good suite: the placement table is
reproduced row by row, the reply classifier is tested against a frame split at
every byte boundary, `bracket` is tested on all five restore paths. The shape is
the problem, and it is a shape worth naming because it generalises.

Every pure test **injects** the values the pure rule compares. Every e2e test runs
with stdout on a **pipe**, so it stops at a refusal. Between them, the suite
verified the rule exhaustively and never once exercised the *premise* — whether
what the impure shell measures is the same kind of thing the pure rule compares.
`-X`'s success path had never executed anywhere when the slice was handed back as
complete.

The design saw the gap and reasoned past it: *"A pty harness is deliberately not
built: the decisions it would exercise are pure and already tested."* Read again
after the fact, that sentence is the defect's charter. The decisions were pure and
tested; the feed was neither. The lesson is not "build pty harnesses" — it is that
**a pure core plus injected inputs leaves exactly one thing uncovered, the
adapter, and that is where a real-environment test earns its keep**. One `script`
spawn was enough.

### What the capsule could and could not do

The capsule executed three phases to a green gate, with per-phase notes,
a self-raised issue (`ISS-456`) where it hit something outside its file set, and
no drift from the design that was not the declared scope cut. That is good work.
It could not run `VH-1..VH-4`, and `DEC-257` had already named those as **the
acceptance test** — *"whether the picture is right is a human judgement, and that
judgement is the acceptance test."*

So the slice arrived with its acceptance criteria unrun and every proxy for them
passing. Both blockers surfaced within minutes of a human typing the command. The
uncomfortable reading is that the capsule's green gate carried almost no
information about whether the feature worked, and nothing in the process said so
— `verify-vt` reports `VT` rows and is silent about `VH`, and the phase was
marked complete with four `VH` criteria outstanding. The handover was explicit
about this ("Do not mark PHASE-05 complete without them") and it happened anyway.

### Standing risks

- **`q=2` keeps a class of terminal-side failure unreportable** (`F-7`,
  tolerated). `F-6`'s fix removes the case that actually bit, by refusing
  oversized images rather than sending them. A rejection for any *other* reason
  still lands as a silent no-op.
- **`MAX_IMAGE_BYTES = 8 MiB` is calibrated, not derived.** It is anchored to
  measurements on this corpus and to no published ghostty limit. Being refused is
  cheap and the message names the fix, so a miscalibration is recoverable.
- **The 36 s wait before a refusal on a large graph** is untouched. `IMP-452`
  deferred the CLI deadline deliberately; `-X` is interactive and Ctrl-C is the
  timeout. Now that the outcome can be a refusal rather than an image, this is
  more irritating than the deferral assumed, and `IMP-452` should be read with
  that in mind.
- **macOS remains unverified** (`CHR-072`). `F-1`'s fix removes the *compile*
  barrier `ISS-456` named — there is no `dev_t` left to widen — but it does not
  verify that `VMIN`/`VTIME` timed reads behave on macOS `/dev/tty`, which is
  what `CHR-072` is actually for. Do not let `ISS-456`'s closure read as
  clearance for the platform.
- **`DEC-256`'s placement rule is judged only on this hardware.** VH-1 passed;
  the HiDPI question sec-9 raised was not separately exercised.

### Tradeoffs consciously accepted

- `q=2` stays, with both halves of the tradeoff now written down rather than
  only its benefit.
- The size bound refuses rather than scales. `dot -Gsize` was measured (834 ×
  1987 px, 1.8 MiB, still 25 s) and rejected: it converts a blank screen into an
  illegible grey one, which is not an improvement worth 25 s.
- The bound is on PNG bytes, not on DOT bytes. A pre-spawn bound would refuse in
  2 s instead of 36, but DOT size is a proxy that could refuse a graph that
  renders perfectly well. Accuracy beat latency.

## Reconciliation Brief

Nine findings, all terminal. Four were repaired under audit (`F-1`, `F-5`, `F-6`,
`F-8` — committed as `3a38acc50`); one is dissolved by another's fix (`F-4`); one
is a recorded tradeoff (`F-7`); one is noise correctly read (`F-9`). What remains
below is prose and backlog that must catch up with what shipped.

### Per-slice (direct edit)

- **`design.md` sec-2 + `DEC-259`** (`F-1`): the endpoint decision is specified
  over `st_rdev` device ids. Replace with POSIX session ids via `tcgetsid`, and
  state WHY device ids cannot serve — an fd opened from `/dev/tty` fstats as the
  `/dev/tty` devnode, never as the pts it redirects to, so the equality is
  unsatisfiable. This is the load-bearing correction; leaving it would re-teach
  the defect to the next reader of the design.
- **`design.md` sec-8** (`F-2`): delete the claim *"A pty harness is deliberately
  not built: the decisions it would exercise are pure and already tested."*
  Replace with the rule the slice learned — a pure decision plus injected inputs
  leaves the ADAPTER untested, and that seam needs one real-environment test.
  Add the `script`-based pty row to the VT table.
- **`design.md` sec-3** (`F-3`, `F-6`): the message table has twelve rows and the
  code ships fourteen messages. Add rows for `MSG_TERMINAL_INSPECT` (the terminal
  could not be inspected at all) and `MSG_IMAGE_TOO_LARGE` (the image budget).
- **`design.md` sec-4 + sec-9** (`F-7`): sec-4 states `q=2`'s benefit and not its
  cost. Record the tradeoff both ways — a suppressed failure reply means a
  terminal-side rejection is undetectable — and add the residual exposure to
  sec-9's residuals.
- **`design.md` sec-4 (Placement) + `DEC-256`** (`F-6`): the placement rule bounds
  columns and explicitly does not bound height, and bounds nothing in bytes. Add
  the image budget as part of the rule, and record that a CELL bound is not a
  RESOURCE bound — the case that convicted it placed into an unremarkable
  199 × 237 rectangle.
- **`design.md` sec-9** (`F-6`): the "Very large graphs" risk names
  `RENDER_TIMEOUT` as its mitigation, and the scope cut removed `RENDER_TIMEOUT`
  (`IMP-452`). Re-state the risk against what actually mitigates it now (the
  byte budget), and record that VH step 2 observed it and rejected the
  "acceptable for an explicit opt-in" reading.
- **`design.md` sec-5** (`F-8`): `RasterOutcome::Png` is described as carrying
  bytes only; it now carries `dot`'s stderr alongside them.
- **`slice-245.md` § Scope cut** — no change needed; the declared departures
  (`IMP-451`, `IMP-452`, `CHR-072`) all held. Recorded so reconcile does not
  re-derive it.

### Governance/spec (REV)

- **`SPEC-027` resp. 5 + `REQ-396`** — the Revision the design already queued
  (`DEC-258`), amending the `run_graph` acceptance criterion to describe the
  render branch. Unchanged by this audit; still owed.

### Backlog

- **Close `ISS-456`** (`F-4`), citing `RV-369` `F-1` as what resolved it. The
  `dev_t` widening problem has no subject any more. Do NOT read this as clearing
  macOS — `CHR-072` is still open and is the real platform question.
- **New issue** (`F-5`, leg 2): `tests/spawn_cwd_convention.rs::bin_refs` cannot
  see a `doctrine_bin` reference inside a macro invocation, because `syn::visit`
  does not descend into macro token streams. Incumbent, repo-wide, and its
  anti-vacuity test (`scan_separates_a_spawn_from_a_mention`) does not cover the
  macro case. Wants a fix plus that missing row.
- **Annotate `IMP-452`** (`F-6` synthesis): the deferred CLI deadline was
  weighed when the slow path ended in an image. It can now end in a refusal
  after ~36 s, which changes the calculus.

### Off-surface — named so reconcile does not attempt it

- **`PHASE-03` `EX-2`** pins `endpoint(stdout_is_tty, stdout_device, tty_device)`
  "over `st_rdev` values", which `F-1`'s fix makes false as written. Plan
  criteria are immutable-append and are NOT a reconcile direct-edit surface, so
  this is recorded here as a design/plan divergence rather than a brief item.
  The behaviour `EX-2` was reaching for — a failed `/dev/tty` open maps to
  `NotControlling` — is preserved exactly.
- **`PHASE-02` `EX-1`** names `RasterOutcome { Png, … }` without pinning `Png`'s
  payload, so `F-8`'s struct variant remains conformant. No action.

### VH re-run after the fixes — 2026-09-17

Prose amendment, not a reopened disposition (findings are terminal once
verified). `F-1`'s and `F-6`'s dispositions were written before the acceptance
test could be re-run; this records the result.

All four `VH` criteria now pass on the fixed binary, in ghostty:

- **VH-1** — the focused graph renders at native size, prompt directly below,
  no stray reply text. Re-confirmed after `F-6`'s size stop was added (143 KB is
  far under the 8 MiB budget, so the stop correctly does not fire).
- **VH-2** — the whole corpus is refused, with both new behaviours visible:

      warning: dot: graph is too large for cairo-renderer bitmaps. Scaling by 0.668933 to fit
      Error: --render needs a smaller graph: 'dot' produced a 62 MiB image and the
      terminal is sent at most 8 MiB; narrow it with a focus id or --depth, or
      drop -X to emit DOT

  `F-8`'s disclosure and `F-6`'s refusal, doing exactly what they were added
  for. `DEC-256`'s placement rule stands for the case it governs (VH-1); the
  case it did not govern is now a refusal rather than a blank screen.
- **VH-3** — the pipe is refused with the not-a-terminal message.
- **VH-4** — tmux is refused with the *unsupported* message, i.e. from the
  support probe rather than the topology stop. Worth stating plainly: before
  `F-1`'s fix this step would have "passed" for entirely the wrong reason, since
  every terminal was refused on topology. VH-4 was only a real test once F-1 was
  fixed.

One cosmetic defect surfaced by the re-run and fixed in the same pass:
`MSG_DOT_NOTES` read `warning: 'dot' reported: dot: graph is too large …`.
graphviz already prefixes its stderr with `dot: `, so the constant is now
`warning: {stderr}` and lets the tool identify itself. The measured figures in
the code comments are aligned to what the refusal prints (62 MiB, rounding up),
so a future reader comparing the two does not stumble.

**Acceptance is therefore complete.** `DEC-257` nominated `VH-1..VH-4` as the
acceptance test; all four have now been run by the user against the landed
behaviour, and the slice is clear to proceed to `/reconcile` on that axis.
