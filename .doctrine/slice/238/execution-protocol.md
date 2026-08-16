# SL-238 execution protocol — the `DEC-242` seats, as practised

**Authored, committed, binding on PHASE-02…PHASE-08.** Read before `/phase-plan`.

`DEC-242` decides *that* implementation runs through three seats and *why*. It
does not say how to operate them — it names the machinery gap instead
(*"separate red and green agents are new machinery … and need a subagent
definition"*, and `IMP-434` is still open). PHASE-01 ran the seats without that
machinery and this file is the result: what worked, what it cost, and what the
next agent should do differently.

Nothing here supersedes `DEC-242`, `design.md` or `plan.toml`. Where this file
and the design disagree, the design wins and the disagreement is a finding.

---

## 1. The mechanism that worked

**One resumable subagent for the whole phase, not one per task.**

- Spawn a `general-purpose` subagent once, at the start of the phase, with the
  full blind brief (§2). The built-in `Explore` and `Plan` agents are one-shot
  and return no agent id — they cannot be resumed, so they are unusable here.
- Drive it with one `SendMessage` per task, addressed to the agent id from the
  spawn result. A completed subagent auto-resumes with its whole transcript; no
  re-orientation, no re-reading of the design.
- `SendMessage` needs no environment variable and no agent-teams enablement.
  Only structured team-protocol messages do (`docs/claude/subagents.md:958`).

PHASE-01 cost: **one spawn, four messages, ~215k subagent tokens**, across T1,
T2, T4 and T3. Per-task spawning would have paid the design-reading cost four
times over and lost the accumulated test idiom between tasks.

The seat stays warm across a phase and dies with the session. That is the real
constraint on phase shape: **finish a phase in one sitting, or expect to pay the
spawn again.**

### The blindness is a property of what you send it, not of a sandbox

There is no enforcement. The seat is blind because its brief forbids four paths
and because the orchestrator never pastes prototype content into a message.
Both halves are yours to keep.

Order is what makes the second half honest: the blind seat writes a task's red
tests **before** the code author writes green, so it never has cause to look at
an implementation. If a red test later needs repair, relay the *failure
symptom*, never the code.

---

## 2. The blind brief — reusable, four prohibitions

Every spawn and every resume carries these. They are the whole arrangement.

Do not read:

1. anything under `.worktrees/` — in particular `.worktrees/proto-SL-238-types`;
2. any git object on branch `proto/SL-238-types`;
3. `.doctrine/slice/238/handover.md` — it describes the prototype;
4. `.doctrine/state/slice/238/phases/phase-NN.md` — where the phase sheet is
   written by a planner who *has* read the prototype. Under `R1`'s default the
   planner is blind and the sheet IS readable, except the oracle-derived section
   its `## Findings` names — say which when handing it over.

Everything else in the repo is open. Point 4 is the one most easily forgotten and
the one that leaks most quietly.

Also give it, every time:

- **its sources of truth** — `design.md` (§7 names the assertions, §3 specifies
  behaviour, §8 lists files) and `plan.toml`'s `VT-` rows, which are the spec;
- **the grep hazard** (§5). It bites test authors hardest, because a fixture
  built on a mangled identifier fails for a reason that looks like a design
  disagreement;
- **`cargo test --bin doctrine <filter>`** — this project's focused run, never
  `--lib`;
- **the standing instruction**: if the design does not state an output string, a
  type shape or a behaviour, *report it, do not infer it*. That is `DEC-242`'s
  intended signal, and it is where all the value came from.

---

## 3. What the arrangement actually caught

Four rounds, three defects that had survived a locked design run, a two-round
external adversarial review (`RV-358`), and a type prototype:

| | what | severity |
|---|---|---|
| **F-3** | §3 rule 2 / §7 promise `VT-5` catches an *omitted* derived-status kind. An equality pin catches addition only. | harmless — the strict read fails loudly anyway. Prose fix at reconcile. |
| **VT-2 contradiction** | §7's fixture (*"no toml at all"*) is unreachable given §3's own arm table. | **real.** Built the way §7 implied, a corrupt `RV` returns `Ok(Unavailable)` — an `STD-003` violation in the slice whose purpose is removing a surface that stated unchecked claims. |
| **`VT-3`'s hole** | `VT-3` fixtures only the common arm, leaving the two arms where laundering was actually *possible* unconstrained. | the seat wrote the missing test unprompted. |

The pattern is worth naming, because it predicts where the next one will be:
**all three sit at the seam between a design's prose rule and the criterion meant
to enforce it.** Prose review reads both and sees agreement; only someone
obliged to *write the assertion* discovers the criterion cannot carry the claim.

**PHASE-02 found a fourth at the same seam, approached from the other side.** §6
and §7 both state that `--prune` has no test coverage; five SL-105-era goldens
exist (`notes.md ### Open`). They pin the *decision* — which edges survive — and
never the rendered reason, so the design's substance held and its claim did not.
The generalisation is cheap and worth keeping: **before pinning a before-state,
grep for the pin.** A design's account of what is *untested* ages exactly as
badly as its account of what is true, and it is not a claim a prose reviewer
thinks to check.

Two further returns that are not defects:

- The seat drew boundaries around its own tests rather than letting them look
  stronger than they are — `VT-6` cannot distinguish delegation from a duplicated
  inline arm; `VT-7` pins non-independence, not parse *count*. Both fall to
  `VA-1`'s grep. Keeping those honest matters here specifically, because `VA-1`
  is grep-shaped and grep in this jail lies (§5).
- Given a choice, it quantified over a kind *set* rather than naming kinds — so a
  kind added later is already covered. Ask for that when the criterion allows it.

---

## 4. Refinements for PHASE-02 onward

**R1 — plan the phase blind, THEN read the prototype against the finished plan.
This is the default from PHASE-02 onward.** The planner is what got contaminated
in PHASE-01; `DEC-242` permits that but does not require it. A planner who never
opens the fork can author its own red suite, and the phase runs **one-seat**.

The trade looked real when this was written — as planner-with-the-fork, PHASE-01
caught the prototype's stale `engine (17)` header and its total absence of lint
attributes *before* they became implementation bugs. **Ordering dissolves the
trade.** Plan blind, commit the plan, and only then read the fork as an oracle
against it. The plan cannot be contaminated by something read after it was
written, and the prototype's defects still surface — they just land as
carried-forward findings instead of silently shaping the plan.

PHASE-02 ran it. The blind sheet was written and committed, then the fork was
read: **it poked no hole in the plan.** It corroborated the ground truth (its
diff removes exactly the code the phase characterises), changed one *annotation*
— the backlog copy's canonical echo is preserved across the collapse, not
superseded, so labelling it superseded would have been the precise confusion
`EX-4` exists to prevent — and yielded two prototype defects logged forward to
PHASE-07 (a `Terminal` branch minting reasons for states `authored_class` may
make unreachable; an `eprintln!` against `print_stderr = "deny"`). See
`phase-02.md` `## Findings` F-3.

Two caveats on the default:

- The oracle pass is thinnest on a **characterisation** phase, where the ground
  truth is today's tree and fully readable, and richest where the prototype
  lands real consumers. Run it either way; it is cheap.
- The oracle's returns then live in the sheet, so **the sheet stops being
  prototype-clean.** Say so when handing it to an implementer: which section is
  oracle-derived, and that the fork itself stays closed.

PHASE-03 remains the freest case — §5 is greenfield, there is no prototype code
for it at all, so the oracle pass has nothing to return and the blindness costs
literally nothing.

**PHASE-05 ran it a third time, and three runs are enough to say what the oracle
is FOR.** It poked no hole in the plan again. What it returned divides cleanly,
and the division is the useful part:

- **Corroboration of decisions the sheet already made** — worth having, and it
  arrives free. `D-1`'s argument ceiling was real; `D-2`'s Table-arm placement
  was the better of two working options.
- **A shape worth borrowing.** The fork sited §4's *"a resolvable backlog target
  renders bare"* rule in a four-line `ref_annotation` wrapper over
  `render_ref_state` — a rule the landed renderer did not carry and the sheet had
  not homed. Shapes transfer even when the code around them does not.
- **Defects to avoid copying — and these are SYSTEMATIC.** The fork's
  `probe_item_refs` omits the memoisation §4 mandates; so does its
  `probe_boundary` (`O-3`, PHASE-04). It threaded an 8th parameter past a denied
  `too_many_arguments` with no suppression at all, which means **it was never
  linted**. A prototype that does not lint and does not memoise is not a
  correctness reference.

**So: read the fork for shapes and for corroboration, never for correctness.** Its
omissions repeat across surfaces, which is exactly what makes a
prototype-contaminated planner dangerous and `R1`'s ordering worth its cost — the
plan is written before the fork can suggest that skipping the memoisation is
normal. The corollary for PHASE-06/07, which read this same fork again: expect
the same two omission classes, and expect them not to be flagged by anything the
fork itself runs.

**R2 — hand the seat a task, not a criterion.** Messages that named the criterion
*and why it carries weight* produced better assertions than messages that quoted
`plan.toml`. The `VT-4` brief said *"`Terminal` is the one class the footer
suppresses — this is the path by which an open prerequisite vanishes"*, and got
back an assertion quantified over all 24 kinds plus a positive control. Quoting
the row alone would have got one `assert_ne!`.

**R3 — correct the seat's model when it drifts, immediately.** After T2 it
concluded "every arm performs the same lenient title read". Half right, and it
would have written `VT-7` expecting `Ok` where the answer is `Err`. One sentence
in the next brief fixed it. Read the seat's closing notes for model errors, not
just its findings.

**R4 — batch a task's red suite with the green that consumes it when staging
would otherwise need a throwaway lint attribute.** T2 and T4 landed as one commit
because `read`/`Authored` had no production consumer until the overlay; splitting
them would have added and removed a `dead_code` attribute inside one phase. Take
both red suites first, then one green. This is also the commit boundary the
design implies — §3 treats the reader and its overlay as one collapse.

**R5 — when a criterion is wrong, amend it in `plan.toml` at the moment you find
it.** Deferring costs more than the edit: anyone reading the plan meanwhile sees
a criterion the code deliberately contradicts, and the audit has to reconcile a
`VT` that never passed as written. Criteria **ids are immutable** — replace the
text in place, keep the id, put the reasoning inline (see PHASE-01 `VT-2`). The
`design.md` half cannot be edited — the run is locked at rev 63 — so it goes to
`notes.md` `### Open` as a reconcile action. Both halves, every time.

**R6 — put findings in `notes.md`, not the phase sheet.** The sheet is
gitignored and `rm -rf`-able. PHASE-01's two reconcile actions live in
`notes.md ### Open`; the sheet holds only the pointer.

---

## 5. Environment hazards that bit, or nearly did

**Confirm a quoted identifier or line number with `Read`; suspect your own
transcription before the tool**
(`mem.pattern.verification.suspect-transcription-before-tool`).

**Corrected 2026-08-17 — this entry previously said the opposite, and the
correction is the point.** It read *"proxied grep rewrites identifiers and line
numbers, silently and non-deterministically"*, citing an `rtk` output filter as
the mechanism and `fn status_and_title_for` rendering as `fn status_and_n` as the
evidence. **`rtk` was real but had been removed months before those sightings** —
not on `PATH` in the jail, no hook rewriting commands to it. Re-running the
recorded case, `grep -n` / `rg -n` / `awk NR` agree with each other and with
`Read`. The sightings were most likely agent misreads, and a documented mechanism
sitting in context supplied a diagnosis nobody had traced. `grep` is
deterministic; the agent quoting it is not.

The practice below is **unchanged** — it was always cheap and it never depended
on the mechanism. Only the reason for it changed.

Consequences to work around, not just know:

- Never quote an identifier, signature or line number from grep *out of working
  memory*. Locate with grep; confirm the quote with `Read` before it enters a
  finding, a criterion, or a memory.
- `VA-1` is a grep-shaped criterion. Run it **with a positive control** — a
  search that must return hits — so an empty result is a demonstrated absence
  rather than a broken search. This rule stands on its own footing
  (`mem.pattern.harness.grep-negative-needs-positive-control`) and is untouched
  by the correction above.
- Every `file:line` in this slice's PHASE-01 records was reconfirmed that way, as
  was every one in PHASE-03's sheet.
- **Do not name a mechanism in a durable record without evidence it was present.**
  "I may have misread" is a legitimate and usually correct entry.

**Shell working directory persists between tool calls.** A `cd` into the
prototype worktree silently redirected three later "production" greps in
PHASE-01. Use absolute paths, or `git -C`.

**Two lint asymmetries, both measured, neither predictable:**

- an `enum` deriving `PartialEq`/`Eq` is *live* even with no constructor — no
  `dead_code` attribute needed;
- a plain `fn` with no production caller needs
  `#[cfg_attr(not(test), expect(dead_code, reason = "…"))]`, and it self-clears.

Add the item bare, compile, let rustc name the set. Under `warnings = "deny"`
that costs one cycle and is never wrong. Three prior slices (SL-244, SL-248,
SL-249) recorded predictions here that were wrong.

**The layering gate is blind to a module with no edges.** A new root module
declared in `main.rs` with no `use` lines passes the entire suite. It reports
`Unclassified` only once edges land. Do not read "gate green" as "classified".

---

## 6. The loop, concretely

Default, one-seat (`R1`):

```
/phase-plan PHASE-NN            # BLIND — design + plan + today's tree only
  amend a wrong criterion in place (R5); findings → notes.md (R6)
  commit the plan
oracle pass                     # NOW read the fork, against the finished plan
  → returns append to the sheet's ## Findings, flagged as oracle-derived
doctrine slice phase 238 PHASE-NN --status in_progress
  hand the phase to ONE implementer — task-shaped, not criterion-shaped (R2)
                              →  red first, green, refactor
                              →  cargo fmt; just gate
                              →  path-limited commit (git commit <paths> -F -)
doctrine slice phase 238 PHASE-NN --status completed --note "…"
/harvest                        # notes.md ## Harvest, then this file if the protocol moved
```

Fallback, two-seat — when the planner has already read the fork, or the phase is
large enough that a separate red author earns its spawn:

```
  spawn blind seat once (§2)  →  per task: SendMessage red brief
                              →  verify red is for the right reason
                              →  orchestrator writes green
```

`just check` for the inner loop, `just gate` before every commit,
`doctrine check gate` at close. **Path-limit the commit itself**, not just the
`add` — agents share one index.

Phase status is runtime state and the flip is not optional: PHASE-01 was executed
before being flipped to `in_progress`, and both transitions had to be recorded
retrospectively. Flip first.

**The cost of not flipping first, discovered at PHASE-02.** `slice verify-vt`'s
gate 4 attributes a mandated file through the slice's **source-delta registry**,
which the `in_progress`→`completed` pair populates (`code_start` at the first
flip, `code_end` at the second). A retrospective flip therefore records an empty
or truncated range, and every one of that phase's `VT` rows reads
`≈ UNATTRIBUTABLE` **forever** — not a `Fail`, so it never halts anything, it
just silently withholds the evidence an audit is going to want. PHASE-01's eight
rows sat that way until repaired with

```
doctrine slice record-delta 238 PHASE-01 --start 8e0c4fbdd^ --end 13808f354
```

after which all eight read `PASS`. Two lessons: flip first, and **re-run
`verify-vt` after the `completed` flip** — mid-phase `UNATTRIBUTABLE` is gate 4
working as designed (`mem_019f89125fb275a2895bf58b5e29ed95`), so it is only
after the flip that the verdict means anything.

**The other end of the same boundary, discovered at PHASE-03.** Flipping first is
necessary but not sufficient: the span runs from the flip to the `completed`
flip, so **anything committed in between is attributed to the phase**, including
work that has nothing to do with it. PHASE-03's span opened with a memory-corpus
correction that happened to land after the flip. The `completed` transition
*warns* and names the commits; it does not refuse. Read that warning and tighten:

```
doctrine slice record-delta 238 PHASE-NN --start <first own commit>^ --end <own code tip>
```

Cheapest habit: land unrelated `.doctrine/` work **before** the `in_progress`
flip, not after.

**And a caveat on reading the result.** `verify-vt` is per-slice in its
attribution, so once a phase touches a file that *later* phases name as their
`test_file`, those later rows stop reading `UNATTRIBUTABLE` and start reading
`PASS` wherever their keywords happen to occur anywhere in that file. PHASE-03
landed `src/backlog.rs` and did exactly this to four later rows. Trust a `PASS`
only for a phase that is `completed`; `ISS-441` carries the defect.
