# Notes SL-260: Design-review finding routing convention and trial

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-20 · plan authored, slice ready · fa227ce48

### Produced
- plan authored — `plan.toml` (3 phases) + `plan.md`, phase sheets materialised,
  slice `proposed → ready` (commits 279175ebd, 0c34a6ee6, fa227ce48)
- standing substrate from the design stage: `design.md` locked at run revision
  79; `RV-371` concluded and done; `DEC-263`–`DEC-273`, `DEC-275`, `DEC-276`,
  `CON-006`
- minted: `ISS-474` — the `slice plan` scaffold and the plan skill ship a
  multi-line, and therefore invalid, TOML example of the `VT` mandate;
  `ISS-475` — no lifecycle stage settles a slice's own `DEC` records
- two friction observations recorded this stage and committed with this sweep
- no `src/` change; the tripwire's commit-scoped check and its positive control
  both run clean. `doctrine check gate` not run this stage — no code modified

### Learned
- `mem.fact.design-run.fragment-renders-via-resume`
- `mem.pattern.plan.path-tripwire-commit-scoped-not-range-diff`

### Open
- `CON-006` — the residue register. Binds criterion authoring at every stage: no
  criterion may assume a check it records as absent.
- design §6's `Q1`/`Q2`/`Q4`/`Q5` remain unminted. `Q2` and `Q4` are now
  **scheduled**: `PHASE-03` `EX-6` mints them onto the trial chore once that
  chore exists. `Q1` and `Q5` stay unminted by design.
- `PHASE-01` `EN-3` — the rendered-turn check needs some slice's design run
  sitting at stage `reviewing` (`SL-253` at plan time). None available →
  `/consult`, never a source-file read.
- the thirteen `DEC` records this slice minted still read `proposed`; discharged
  for this slice at `PHASE-03` `EX-7`, generalised as `ISS-475`
- `ISS-320` — re-adopting an edited `design.md` needs a section map nothing
  emits; relevant to anyone editing the locked design
- `ISS-472`, `ISS-473` — filed out of the design stage; neither blocks

---

## Design surface triage — exploring stage, 2026-09-19

Bulk evidence lives in `.doctrine/slice/260/research/research.md` (runtime tier,
gitignored; baseline current). This section is the triage over it, plus what the
memory sweep added. Pointers, not copies.

### Constraining governance

Binding: `DEC-103` (two-surface placement is *required*, corollary 2; the
honour-system rules must be labelled unenforced-by-construction, residue rule) ·
`DEC-138` (disposition binds the responder's turn; its consequences make stating
the two-gate asymmetry an obligation on this design) · `ADR-007` D-C5/D-C9b ·
`DEC-126`+`DEC-125` (the design gate attests, it does not check) · `ADR-005`
(the two targets sit in different knowledge tiers) · `DEC-101` (step ids are
API — the authority behind the `reviewing.toml` non-goal) · `POL-001` ·
`SPEC-031` (criterion identity; silent on promotion) · `ADR-017`/SL-060.

Checked, not applicable, each with a reason: `STD-001`, `STD-002`, `STD-003`,
`POL-002`, `ADR-003`, `ADR-013`. `ADR-013` cannot fire at all — `revision change
add` takes a live entity FK and an embedded asset has no entity id.

`RFC-026` is **provenance, not authority** (`ADR-014`: an RFC asserts no canon).
The design *adopts* `P10`; it is not bound by it.

Revision candidates: none, and none possible.

### Shaping decisions already taken in scope

Five research deltas were scope-level and were folded into `slice-260.md` before
this run opened (`4b100ff6f`): lead §1 on `DEC-103`; name the binding gate as the
slice close, not the design lock; carry `P10`'s settle-first test and its
accumulation rule; label the unenforced-by-construction items; and resolve §6's
script hedge to **no tooling at all**, with the tripwire that a `src/` change
newly binds three standards.

### Open questions carried into design

`OQ-1`..`OQ-5` in `research.md`, plus the two the memory sweep raised:

- `OQ-1` the raiser-side ruling redefines what `verify` means. Name it as a
  convention-level redefinition or the two readings collide inside the trial's
  own counting.
- `OQ-2` the finding→criterion promotion leg has no governance owner. Convention
  text, knowledge record, or agent practice?
- `OQ-3` enforcement is the counting pass and only the counting pass. Accept and
  say so.
- `OQ-4` "first phase" is asserted, not governed. Needs a fallback for a `probe`
  that wants a worktree branch.
- `OQ-5` the probe-evidence home rests on `.gitignore`, not on any authority.
  Pick a name; do not cite an authority that does not exist.
- `OQ-6` **(new, from `mem_019f97fcab2e77a28902371f80743605`)** — no verb
  transitions a finding out of `verified` (`src/review.rs:703-728`). So `P10`'s
  accumulation rule can reopen the *design decision* but never a verified
  routed finding's *disposition*; the remedy there is a prose amendment on the
  `RV` `.md`. Which does the convention mean?
- `OQ-7` **(new)** — does the convention name a default route for an
  unclassifiable finding, or require an explicit `review`? (Scope's own `OQ-1`.)

### Risks

- `R1`/`R2`/`R3` as scoped: read-and-not-followed is a real result; three
  ledgers is thin and carries a combined intervention; the routes may need the
  owner to interpret (a `P10` would-kill).
- `R4` **(new, from `mem_019fbc7514097c42aa042ab7bb2206c6`)** — the probe
  adversary clause goes into `--response`, the one field where a double-quoted
  shell argument silently eats every backtick span. `dispose` succeeds, the
  ledger is turn-based, there is no amend verb, so the damage is permanent until
  the raiser contests. The counting pass cannot tell an eaten clause from an
  absent one, which corrupts exactly the `P10` *Open* item §4 answers.
- `R5` **(from `mem_01a0b45d375873e3ae691d75bea8760d`)** — a repair inherits the
  finding's scope and leaves the twin arm; observed four times in one review
  (`RV-370`/`SL-246`). A `demonstrate` route can discharge one arm and read as
  complete, so routing does not by itself close this sibling failure mode.

### Assumptions

- `A1` design-run findings already land on the `RV` ledger, so *"on the existing
  ledger"* is true today and `IMP-392` is not a prerequisite.
- `A2` `--disposition` accepts free text and nothing validates it.
- `A3` **(new)** the shipped convention reaches agents only through a rebuild +
  `doctrine install`; cargo does register `install/` as a build dependency
  (`mem_019e98a783ea7471ac4bfcefdc04ae5e`, re-probed), so the rebuild is not
  manual — but a stale embed is silent, so closure verifies through the render.

## Further review passes — what one would probe, and why we are not running one

Written 2026-09-20, after `RV-371` concluded (`done`, 10 findings, 1 withdrawn,
9 verified) and after the post-pass critical re-read that repaired its residue.
Recorded for the `review.passes` runbook step.

**The decision.** No further design-review pass. The user's call, taken with the
evidence below in hand: the design is text-only, its remaining uncertainty is
about how the convention behaves in use rather than about whether it is
internally sound, and behaviour-in-use is exactly what the trial exists to
measure. Another pass buys re-argument where the next three ledgers buy
observation.

**What a further pass would probe, if one were run.** These are live and
unexamined by any adversarial reader; they are the honest cost of stopping here.

1. **The material `RV-371` never saw.** The design grew from 804 to about 1130
   lines under the pass. §5.1's precedence procedure, §5.2's four inserts as they
   now stand, and §9.4's collection procedure were all authored *as repairs* and
   have been read by the responder and by one critical re-read — never by an
   adversarial reader. `review_pass` says `STALE` for exactly this reason, and
   that status is accurate, not a bookkeeping artefact.
2. **Whether the precedence is total in practice, not just on paper.** Steps 2
   and 3 are now total over the five by construction. What is untested is whether
   step 1 — *route on the claim whose failure would make the rest moot* —
   actually discriminates on real findings, or whether responders reach straight
   for step 3 and let the fixed order do the work. That is a trial observation,
   not a review one.
3. **The three inserts as shipped text.** §5.2 carries proposed text. Nobody has
   attacked it as an agent would *receive* it — with only the fragment in hand and
   no design.md. The one defect of that shape found so far (the undefined term
   *instrument route*) came from a re-read, not from the pass.
4. **§9.4's joins against a real ledger.** The procedure is specified but has
   only been run for its route-extractor column. Whether the manual joins are
   executable at acceptable cost is unknown until the first eligible slice.

**Why the pass is nonetheless not the right instrument.** `RV-371`'s own
synthesis is the argument. Five defects survived the adversarial pass and were
found afterwards by a fresh critical re-read; `R5` fired three times *inside the
repair round* and `R4` twice. The lesson is not that another adversarial reader
would find the next five — it is that a single pass does not converge, and that
the marginal find came from re-reading under different conditions rather than
from an additional reader. Stacking a second pass on a design whose remaining
risk is behavioural would spend the reader on the wrong question.

**What carries the residual instead.** The trial, which is what this slice
exists to make possible; `CON-006`, which enumerates every unenforced clause with
its code site; and `R1`/`R3`/`R6`, which name at ledger grain what to watch for
while the convention operates. If the trial observes routed obligations dropped
between plan and execute, `Q4`'s code-backing is the named next move and the
`src/` tripwire is what it costs.

## PHASE-01 delivery evidence (2026-09-23)

Recorded outcome for `PHASE-01` `VA-1` (phase-sheet `D2`: the render subject is
another slice's runtime run, so it will not outlive the phase).

- Commit `7c329fde0`; transcription byte-identical to design §5.2's two fences
  (`HEAD~` minus the stand-alone rule, plus both fences, `diff` empty).
- After `cargo build`, `./target/debug/doctrine design resume 253` (`SL-253` at
  stage `reviewing`) renders `## Routing a severe finding (provisional — RFC-026
  P10 trial)` at output line 127, the carve-out at 122, the route table
  (`probe` row, 143) and the `CON-006` pointer (200). `SL-253`'s state TOMLs
  md5-identical before and after.
- `VT-2` tripwire: `git log --oneline 423c181d1..HEAD --grep='SL-260' -- src/`
  empty; without the pathspec, 17 commits. `doctrine check commit` exit 0.

## PHASE-02 evidence and deviations (2026-09-23)

- Pointers `4822916bf` (verbatim from design §5.2); fix `768dd1bde`.
- `VA-1`: axis reads back from `library show reference/review-ledger.md`
  (:167); pointer from installed `.claude/skills/plan/SKILL.md` (:43). Installed
  with `install -a claude -s plan -y` — the unscoped agent set runs `npx` legs
  that can rewrite tracked `skills-lock.json`.
- `VA-2` sweep (positive control: fragment line 3): every shipped class has one
  owner, `reviewing.md`; zero leakage into the pointers — **after** `DEC-277`.
- **Deviation, `DEC-277` (user: "A").** The §5.2 drafts dropped two required
  clauses: `DEC-264`'s post-`verified` residue sentence (0 owners) and the plan
  pointer's `CON-006` citation (`EX-5`). Both shipped in `768dd1bde`.
  **`/reconcile` owes:** amend design §5.2's fragment and pointer drafts to
  match the shipped text (direct edit; the run stays locked).
- **Gate slip.** `4822916bf` was committed ungated and left `edge` red: SL-244's
  `design_prompts_have_no_consumer_outside_the_design_run`
  (`tests/e2e_claude_install.rs`) allowlists every file naming `design-prompts`,
  and the §4 axis names its owner by that address. User chose to allowlist it as
  a pointer (the `name@digest` allowlist untouched). This is a `tests/` change,
  not `src/`; the tripwire still holds (`VT-3`: empty, 20-commit control).
  The PHASE-01 sheet's `A2` ("no test pins the fragment") grepped for the path,
  not for the store name, and missed it.

## Interaction with ISS-476 (2026-09-23)

ISS-476 (`f0d7804ae`) edited two of this slice's surfaces ahead of execution —
`reviewing.md` (pass-ledger paragraph, lock example) and `review-ledger.md` §2 —
and touched `src/`. No overlap with the §4 route axis or the routing rule.

- **Base.** Design §9.2 row 7 (`git diff --stat <base>..HEAD -- src/` empty)
  false-reds on a base older than `f0d7804ae`. Fork from `edge` at or after it.
- **Line citations drifted** (IMP-467, then ISS-476): the stand-alone rule cited
  as `reviewing.md:63-64` (plan `EX-3`, design §5.2) is now `:98-99`; the
  `/consult` guardrail cited as `review-ledger.md:164-166` (scope §2) is now
  `:172-173`. Locate by the quoted text, not the numbers.
- **Trial effect.** The envelope now names the run's pass RV, which should stop
  split passes (an empty run RV beside the real one, SL-256's RV-359/RV-360)
  from muddying the trial's per-slice ledger count.
