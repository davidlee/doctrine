# deepseek-v4-pro is a capable reviewer, not just an implementer

User-stated (2026-07-27, SL-231): deepseek is "generally pretty decent at
review. Not opus decent, but decent enough to find good dents in Opus' code."

So the pi/deepseek arm is a legitimate REVIEW resource, not only an
implementation worker. Run it as a review pass over Opus-authored or
Opus-orchestrated code; expect real findings, not rubber-stamping.

## Do not over-generalise one bad pass

A single SL-231 PHASE-01 review turn went poorly — one confidently-wrong
finding (it ran `cargo test` inside the jail where `DOCTRINE_WORKER=1` is set,
so ADR goldens skipped, and it called the worker a liar) and one silent skip of
the check flagged as most important. That was recorded in a handover, and a
later session used it to justify skipping review passes entirely on SL-231
PHASE-02 and PHASE-03. That inference was wrong: it generalised one bad run
into a capability judgement, and it silently substituted orchestrator judgement
for a user-decided arm plan.

Treat a bad review turn as a bad turn. Adjudicate its findings against the
code — which is required anyway — and keep using the reviewer.

## Observed failure mode, which review directly targets

On SL-231 PHASE-03 the same model wrote three gate-invisible defects
(a Latin-1 `char::from(u8)` escaper corrupting all non-ASCII, a hand-rolled
`filter_and_order` duplicating an existing service capability, and a newline
row-injection vector in table rendering) and self-reported success each time.
Given a precise diagnosis it then fixed all three correctly, first try.

Its weak point is SELF-review, not capability. That is exactly the gap a
separate review turn closes — a fresh turn reviewing another turn's output is
not the same as asking an agent to check its own work.

Mechanics for a read-only review turn: `PI_REUSE_FORK=1` to attach to the
existing fork plus `PI_TOOLS=read,bash,grep,find,ls` to withhold edit/write.
See [[mem.pattern.dispatch.pi-rpc-worker-protocol]].


## Confirmed in use — RV-317 (SL-231 PHASE-01..03 code review, 2026-07-27)

Three read-only deepseek passes (`PI_REUSE_FORK=1`,
`PI_TOOLS=read,bash,grep,find,ls`, `PI_THINKING=high`), run lens-diverse rather
than phase-shaped. The user's correction holds: the passes produced **real
findings that survived independent adjudication**, including a blocker — a
terminal escape / row-injection vector re-opened through unescaped envelope
metadata, with accurate line numbers, correct threat model, and correctly
proposed severity. Also genuine: discarded read diagnostics, a publication temp
leak, and a whole-suite VT assertion audit that caught three tests asserting a
tautology, the opposite of their name, or a classification instead of the
diagnostic they were named for.

### Two calibration failures worth planning around

1. **Line numbers can be systematically offset.** Pass A's citations were ~410
   lines off throughout — it numbered against a `git diff` hunk rather than the
   file. Substance was mostly right; every location needed re-deriving. Two
   findings even cited *overlapping* ranges in one file, which is the cheap tell.
   Prompt for `grep -n` output or function names alongside line numbers, and
   never raise a finding on a reviewer's line number without re-reading it.
2. **A "CLEAN" section is not evidence of absence.** Pass B declared "no
   production-code panics on hostile input" clean while a reachable panic sat one
   module away in a file it had open. The single worst defect of the review
   (`shard_dir` byte-slicing a UTF-8 uid) came from orchestrator probing —
   literally running the binary with a non-ASCII argument — not from any pass.

### Practical shape

Reviewer passes and orchestrator probing are **complementary, not redundant**.
The passes read broadly and cheaply; empirical probing of the built binary finds
what reading misses. Budget for both. Lens-diverse passes (layering / hostile
input / test-input coverage) beat phase-shaped ones on a multi-phase delta —
cross-phase contract drift is invisible to a per-phase pass by construction.

See also [[mem.pattern.review.sweep-defect-class-not-instance]].
