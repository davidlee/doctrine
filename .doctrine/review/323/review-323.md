# Review RV-323 — design of SL-241

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject.** `.doctrine/slice/241/design.md` at `3966256a` — the capsule spike
rig. Aspect under trial: **design intent**. Posture: inquisitor. This is the
first *external* hostile pass; the internal pass (§ 10.1, eight findings) is
integrated and is not relitigated.

**Scope fences honoured** (operator rulings, `handover.md` + design § 1.1):

- **CON-005.** The claim tried here is the narrow one — *capsules bound what a
  worker process can do, not what it can say, nor make its outputs
  trustworthy.* No charge below requires watertightness against a fully
  compromised in-capsule agent. Lying test suites, prompt injection into the
  control-plane agent, orchestrator authority (RT-8), and
  malicious-but-passing code are out of scope with RFC-022/023 governing.
- **POL-002 § Scope.** The rig is client-local by construction; `cargo`, `nix`,
  `just`, the edge/main split are all permitted. Only the *evidence* carries an
  altitude obligation. "The rig is doctrine-specific" is not a charge.
- **RT-1 / RT-2 / RT-3 verdicts are settled.** DEC-104's narrowing of RT-2 is
  fair game; the underlying verdicts are not.
- **RFC-023** will revise plan gates; current machinery adopted as-is.

**Lines of interrogation.**

1. **Matrix sufficiency (the central claim).** § 5.1 asserts fourteen of the
   sixteen P-C3 rows run on the four-stage pipeline and only H10/H16 need the
   candidate layer. Walked row by row against each row's *expected kill*
   boundary in `probe-specs.md` — not against the design's summary of it.
2. **Deleted machinery still load-bearing.** Every row whose expected kill
   names the journal, the candidate layer, or the artifact channel that this
   model removes, and which the design did not re-derive.
3. **Reuse vs re-derivation (DQ-1's spirit, and the standing no-parallel-
   implementation rule).** Where the design hand-rolls in shell what the Rust
   belt already implements — and whether the hand-rolled leg carries the
   belt's explicit hardening.
4. **"One mutation" as a testable invariant (CON-004 / I1).** Whether the
   trusted side truly takes one write, or whether harvest writes too.
5. **Evidence integrity.** Whether the altitude column (D6), the portability
   control (D5/DEC-101), ASM-007's falsification, and the measurement table
   can be filled honestly by the rig as specified — or whether any of them can
   be satisfied vacuously.
6. **Trust placement of the interpretation-surface declaration (DEC-099).**
   Which side authors it, and which side it is *read from*.
7. **Internal coherence.** The artifact against itself.

**Method note.** Following § 10.1's own lesson — the first draft twice wrote
cheques the codebase had not cashed — every claim about existing machinery was
checked against `src/`, not against help text. Confirmed sound:
`slice conformance --against A..B --strict` exists with those exact flags and
its range fold is belt-hardened (`src/slice.rs:2890-2906`). Confirmed
*unsound*: the submodule rejection the matrix leans on is index-scoped
(`src/git.rs:2432`), unreachable from an object-only pipeline.

## Synthesis

### Judgement

**The design is not heresy. Its architecture is sound and its reasoning is, in
places, exemplary** — D2's observation that the simplification *dissolved* F3
rather than working around it is the mark of a direction that is right, and D8's
refusal to mint a synthetic `Verified` row is the correct answer to a genuine
temptation. The four-stage decomposition, the outcome-conditional restatement of
CON-004 (I1), the I6 guard asserted first, and the positive controls on both
audits are all the work of an author arguing in good faith against themselves.

The taint is of one kind, and it recurs. **Where this design deletes machinery,
it has not everywhere followed the deletion through to the artifacts that
depended on it.** The candidate layer, the journal, and the coordination-artifact
channel are all correctly removed — and the sixteen-row matrix, the measurement
table, and the assumption ledger still speak the vocabulary of the model that
carried them. F-1, F-2, F-9 and F-10 are four faces of that single sin. It is a
sin of *completeness*, not of judgement, and the penance is proportionate: derive
what was previously inherited.

The second thread is thinner but sharper. **Three claims are load-bearing and
undecided at the exact point where they must be mechanised** — where harvest
writes (F-3), where the runner comes from (F-11), where the declaration is read
from (F-5). Each is one sentence away from being true. Each, left as it stands,
is one provisioning reflex away from being false: a `cp` instead of a read-only
bind quietly undoes RT-1, the programme's only blocker. Let the record show these
were not errors of reasoning but silences, and that silences in a trust boundary
are where the rot enters.

The third is the one I press hardest, for it is the one the spike exists to
prevent. **Three separate mechanisms can be satisfied vacuously and would be
scored as evidence** — a light-fixture cell with nothing planted counts as
`model-level` (F-7); an exhaustiveness assumption is discharged by a rig that can
only confirm (F-8); two rows measuring the incumbent count toward capsule-model
coverage (F-9). This design's own § 9 already knows the principle — *"a negative
grep without a positive control proves only that grep ran"* — and applies it to
the two audits while leaving the matrix, where the portability claim is actually
made, unguarded. **A spike that launders assertion into evidence is worse than no
spike, for it retires the doubt that would have saved us.** Burn the vacuous cell;
salt the ground where it stood.

Four charges are **blockers**. None impugns the model. All four would, unaddressed,
produce a results table that says something the runs did not establish.

### Sentencing — the ordered penance

Before any rig code is written:

1. **F-1** — re-derive the expected-kill column for all sixteen rows against the
   four-stage model. Rows dissolved by construction are recorded as such, with
   reasons. This is evidence, not bookkeeping.
2. **F-3** — rule on the harvest destination. Quarantine is a disposable
   *repository*; canonical takes exactly one write, at stage 4. Then I1 is true
   and `assert_outcome` can be written.
3. **F-11** and **F-5** — two sentences, stated as invariants: runner scripts
   enter the sandbox as read-only binds from outside the writable root; the
   interpretation-surface declaration is read from the contracted base `B`,
   never from the harvested result `S`.
4. **F-2** — add the mode-aware leg (`ls-tree -r`, refuse `160000`, refuse
   `.gitmodules`). Object-only, no worktree, gives H8 a boundary.
5. **F-4** — the forbidden-path leg takes `-c core.quotePath=false`,
   `--no-renames`, `-z`. Cite `src/mcp_server/dispatch.rs:487` as the reference
   form, as § 5.2 already cites the conformance verb.
6. **F-12**, **F-13** — settle the stage count and the class cardinality. Ten
   minutes, and the results vocabulary stops being ambiguous.

Before any row's result is scored:

7. **F-7** — per-instantiation positive controls, or an explicit `n/a` excluded
   from the altitude computation.
8. **F-6** — M-B's bundle file gets its RT-4 hygiene paragraph; H13 re-homes onto
   it; the trusted-side surface enters QUE-200's verdict inputs.
9. **F-8** — the independent enumeration of the TypeScript trigger surface, or
   § 5.5 and ASM-007 both stop naming the fixture as the falsification vehicle.

Before closure:

10. **F-9** — mint the question owning the capsule model's conflict/staleness
    admission path; carve H10/H16 out of the coverage count in § 9 Closure.
11. **F-10** — name both columns' sources for every measurement row; retire the
    `candidate-create` metric; mark tokens `n=1`.

### Verification

Each penance carries its own proof, per the design's own standard:

- F-2, F-4 — hostile rows with positive controls: a gitlink, a non-ASCII
  `.doctrine/` path, a rename out of `.doctrine/`. Each must be *observed*
  refusing. A guard never seen to fire is not known to work (§ 9, I6).
- F-3 — the happy-path self-test asserts exactly one canonical ref changed; a
  refused row asserts the canonical object count is unchanged.
- F-7 — every mutate step verifies its own payload landed before the pipeline
  runs. `n/a` is a legal, recorded outcome; a silent pass is not.
- F-11 — a scripted probe overwrites the in-capsule `verify.sh` and rings; the
  verdict must be unaffected.
- F-5 — a capsule rewrites its own declaration; trusted-side behaviour must be
  byte-identical to the run that did not.

### Standing risks, consciously named

- **CON-005 holds and is not re-tried here.** Every charge above concerns what a
  worker *process* can do. Not one requires the in-capsule agent to be honest.
- **F-9's residual outlives this slice.** Even fully remediated, the spike will
  not establish how the capsule model admits the second of two results from one
  base. That is the largest known gap in the programme and it must leave this
  slice as a *record*, not as a sentence inside a rejected alternative.
- **ASM-007 may survive unfalsified even after F-8's penance.** An
  independently-derived enumeration over two ecosystems is modest evidence, not
  proof. It should be recorded as strengthened, never as discharged.
- **R2 stands where the design left it.** `slice conformance --strict` is
  confirmed to exist and to be belt-hardened; whether its `--strict` semantics
  agree with the belt's at every edge remains for the rig skeleton to probe. The
  design was right to keep this open and right not to guess.

*The findings await disposition by the accused. Thirteen charges stand upon the
record; four bar the gate.*

> **HERESIS URITOR; DOCTRINA MANET**
