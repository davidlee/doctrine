# Review RV-046 — plan of SL-079

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This Inquisition holds the plan of SL-079 ("Finish the CLI colour story") to the
locked SL-079 design, the prior RV-045 verdict, the storage rule, and the
project's pure/impure split. The accused artifact is the plan aspect:
`.doctrine/slice/079/plan.toml` and `.doctrine/slice/079/plan.md`.

The plan claims three phases: PHASE-01 foundation plumbing (IMP-038 + IMP-040),
PHASE-02 priority table colour (IMP-039a), and PHASE-03 status-line colour
(IMP-039b). The tribunal presses these lines of interrogation:

1. Does every phase objective and EN/EX/VT criterion follow the locked design and
   RV-045 corrections, or has any rejected design path crept back in?
2. Are the phase boundaries executable without hidden work, especially the
   `--color` shell plumbing and list/priority/status call-site injection?
3. Do the plan criteria name the right surfaces: survey/next for priority
   tables, and adr/policy/standard/knowledge/revision for status lines?
4. Does the plan accidentally require blockers/explain to behave like priority
   tables after the design excluded them from `render_columns`?
5. Does the plan account for stale slice-scope text that still names
   install/reconcile/corpus, or does it leave a future executor with conflicting
   marching orders?
6. Is verification strong enough to prove byte preservation for `color:false`,
   ANSI emission for `color:true`, `--color` precedence, and the `NO_COLOR` path?
7. Are criteria ids and phase ids stable and doctrine-shaped for later
   `/phase-plan` and `/execute` use?
8. Does any criterion demand impossible or misleading commands, flags, or
   terminal observations?

The guilty shall not hide behind pretty tables; each criterion must either serve
the design or be put to the torch.

## Synthesis — Inquisitorial Verdict

The plan of SL-079 is not damned root and branch, but it is tainted enough that
execution must not proceed until penance is applied. Seven charges were proved
under interrogation. None require expanding the slice; all demand correction of
the plan, design, or stale scope before PHASE-01 is handed to `/phase-plan`.

The design spine remains broadly sound: PHASE-01 before colour surfaces,
PHASE-02 for survey/next, PHASE-03 for the five lifecycle status lines. Yet the
plan lets false criteria and unstable verification through the gate, and so the
Inquisition brands it unclean.

Ordered penance:

1. **F-1, fix-now:** Correct PHASE-03 status criteria. ADR `proposed` is plain,
   not yellow. `accepted` is green. If VT-4 says every mapped token, include the
   full `status_hue` mapped set or narrow the wording.

2. **F-2, design-wrong:** Reconcile the `--color` parser claim. Either add
   `ignore_case = true` and test uppercase/lowercase values, or admit that clap
   accepts the lowercase `auto|always|never` tokens and remove the
   case-insensitive promise from design and plan.

3. **F-3, design-wrong:** Make `resolve_color` verification executable. Either
   add an injectable pure helper for the full mode/env/tty matrix, or narrow
   VT-2 to direct `Never`/`Always` checks plus `Auto` delegating to
   `stdout_color_enabled`, with existing `color_enabled` tests covering
   `NO_COLOR` and tty.

4. **F-4, fix-now:** Name the PHASE-03 injection seam. `main.rs` should resolve
   `tty::resolve_color(cli.color)` at the status command arms and pass a
   `color: bool` into `adr`, `policy`, `standard`, `knowledge`, and `revision`
   `run_status`, unless a different seam is explicitly designed.

5. **F-5, fix-now:** Replace live mutable VA commands with fixture-safe
   verification. Use the actual CLI shapes from `--help`, bind expected ids and
   starting states to the fixture, and never ask an executor to mutate the live
   corpus as a vague acceptance step.

6. **F-6, design-wrong:** Reconcile stale scope text. The scope still names
   install/reconcile/corpus while the locked design and plan name
   standard/knowledge/revision. Amend the scope, or gate PHASE-01 on that
   reconciliation.

7. **F-7, fix-now:** Add PHASE-01 integration evidence for the global colour
   override on an existing `CommonListArgs` surface: `--color=always` emits ANSI
   when piped, `--color=never` suppresses ANSI, and default `auto` keeps the
   existing piped golden plain.

Standing risk: the prior design review `RV-045` corrected the main design
direction, but this plan copied two design-level inaccuracies into executable
criteria. Before implementation, revise the affected artifacts together; do not
let the plan become a second scripture against the design.

Judgement: HERESY CONFIRMED IN THE PLAN CRITERIA. The plan is repairable, but
until corrected it should not be executed. Let false yellow `proposed`, phantom
case-insensitive parsing, and unsafe live-corpus status commands be nailed to
the church door as warnings to the next penitent executor.

> **HERESIS URITOR; DOCTRINA MANET**
