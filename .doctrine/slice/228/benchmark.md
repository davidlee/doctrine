# SL-228 PHASE-07 — OQ-5 memory-blind benchmark

Terminal-acceptance evidence for the SL-228 cluster. Records the protocol, the
harness, what each scenario measured, and what it cost. Read the *Stated limits*
section before quoting any number from here.

## What OQ-5 asks

> A fresh orchestrator with **zero dispatch memories** completes a standard run
> plus the top-5 quirk scenarios **by following verb output alone**, measured
> against the Cluster-1-cleaned baseline (SL-224/225).

D4 is the claim under test: `dispatch next` covers the full ladder, and drift is
caught by verb gates "whose refusals name the fixing verb".

## Harness

**Subject** — a headless `claude -p` orchestrator (`claude 2.1.220`, model
`claude-opus-5`) spawned by `bench-spawn.sh`, structured on `pi-spawn-confined.sh`.
Not a subagent: `dispatch_import` / `dispatch_conclude_phase` / `dispatch_reap`
are MCP-only, and the MCP server binds **one project root at startup**
(`src/mcp_server/mod.rs:38`), so a subagent sharing this session's server could
only ever address the real repo. A subprocess gets its own server bound to the
benchmark clone (`--mcp-config` + `--strict-mcp-config`).

**Substrate** — a clone of this repo branched from `dispatch/228` (the funnel
machinery is not on trunk), carrying fixture slice **SL-230**: two phases over
`bench/*.md`, plan + materialised sheets, so the funnel record is **populated**
by the subject's own spawn rather than exercising the degenerate row-absent
fall-through (the PHASE-06 lesson).

**Blindness** — removed from the benchmark branch: `.doctrine/slice/228/**`
(design, plan, notes, handover — the answer key), `.doctrine/dispatch/**` (a
worked example of the exact ladder), `.doctrine/rfc/011/**` (the quirk
analyses), and the entire 726-item local memory corpus. Dispatch prose cut from
`governance.md` and `AGENTS.md`. Regenerated boot snapshot reads
`Memory: not yet populated`. Confinement additionally masks `/workspace` (the
real repo was otherwise readable — the answer key on the desk),
`~/.claude/projects` (the orchestrator's own transcripts), and re-binds the
funnel-capable binary over the jail's stale PATH `doctrine` 0.29, which lacks
`dispatch next` entirely.

Contamination is defined as **harness-injected** knowledge. The subject reading
a shipped skill or `dispatch-mechanics.md` is *data*, not contamination — it is
OQ-6's question in the first place.

**Instrument** — `--output-format stream-json`. The terminal `result` event
carries `total_cost_usd`, per-model token counts, `num_turns`,
`permission_denials` and `terminal_reason`: both halves of EX-2 from one
artefact, with no self-reporting by the subject.

## VA-1 — protocol conformance

| Check | Result |
|---|---|
| Zero dispatch memories in the subject's context | **Confirmed** — local corpus removed; boot Memory index empty |
| Zero memory retrieval during the run | **Confirmed** — 0 `memory_*` tool calls and 0 `doctrine memory` CLI invocations across all rounds |
| Scenarios from the RFC-011 prioritisation | **Partially** — see *Scenario set was re-derived* |
| Zero harness rescue | **Confirmed** — the subject asked the harness nothing; no round ended on a question |

## Scenario set was re-derived

The nominal top-5 were verified against their own backlog ids rather than taken
from `case-notes-analysis.md`, which predates its own remediation:

| # | Nominal quirk | Backlog | Determination |
|---|---|---|---|
| 1 | prepare-review phase-status split-brain | IMP-272 | **FIXED** — excluded |
| 2 | object-db import leaves coord stale | *no item* | **LIVE** — kept (S1) |
| 3 | worker_commit false-red (stale PATH) | ISS-218 | **FIXED** (SL-225 DEC-003) — excluded |
| 4 | selector under-declaration | IMP-256 | **FIXED at plan time** — kept in narrowed runtime form (S2) |
| 5 | split-lineage close friction | IMP-127/236 fixed; IMP-201/174 open | **OUT OF SPAN** — beyond `next`'s terminal beat |

Four of five are substantially remediated. Running the named set would have
measured other slices' fixes. Replacements were drawn from the funnel's own
claim surface.

## Results

| id | Scenario | Outcome |
|---|---|---|
| **S0** | Standard run | **PASS** — both phases spawn→worker_commit→import→verify(pass)→conclude→reap, plus `sync --prepare-review` with two phase cuts. No questions asked. |
| **S1** | Object-db staleness (live quirk #2) | **PASS, quirk confirmed live** — the subject hit `record-boundary` leaving the coord index stale and diagnosed it unaided. |
| **S2** | Undeclared-scope refusal | **DID NOT FIRE** — pre-empted. Both S0 and S4a independently compared `plan.toml` EX-2 against the selector set and declared `bench/index.md` *before* importing. Competence, not tooling: `slice selector doctor` was run and returned **"selectors healthy — no findings"**. |
| **S3** | Verify red → triage | **NOT EXERCISED** — pre-empted; the subject read `.bench/suite.sh` before writing and satisfied the strict form the phase spec under-specified. A design that a competent operator can pre-read is not an injection. Re-arm as a mid-run change if wanted. |
| **S4** | Crash mid-funnel → heal-forward | **PASS** — subject killed at `worker-committed` (fork present, import not run). A fresh session with no notes reconstructed position and drove through to completion. `dispatch next` gave the literal next command; 2 calls were lost to argument shape (positional `SL-230` rejected; `--slice SL-230` accepted). |
| **S5** | Fork advanced past its imported tip | **GUARD PASS / REMEDY FAIL** — see below. |

### S5 in full — the run's most consequential result

The harness advanced `dispatch/sl230-p01` by one commit after import
(`802f4b62` → `045e754c`) while the row's `import.fork_tip` stayed `802f4b62`.

1. `dispatch_reap` returned **`Refused{not-landed}`**. D9's three-check
   conjunction held and the branch survived — RV-308 F-1 works.
2. The refusal's `detail`, verbatim: *"if this fork is funnel-managed, reap it
   with `dispatch_reap` … otherwise `--force` to reap knowingly, or
   `--superseded-head <SHA>` to assert it is spent-and-abandoned."* The caller
   **was** `dispatch_reap`. The only non-circular advice left is the override pair.
3. The subject followed it: `worktree gc --fork … --superseded-head 045e754c`.
   The fork was deleted; `dispatch_reap` then advanced the row on the
   fork-absent path (I4) and reported `Reaped`.

**The guard fired and its own remedy text walked the operator around it**, via
the CLI escape NEW-OQ-C deferred. The commit beyond the imported tip was
destroyed — the exact outcome F-1 exists to prevent. Filed **ISS-250**.

This also discharges NEW-OQ-C's own stated trigger — "revisit … sooner if an
operator is observed reaching for CLI gc on a funnel-managed fork". Observed on
the first blind run, and not by accident: the funnel's refusal sent them there.

## Findings

| id | Finding | Status |
|---|---|---|
| **ISS-249** | Coord pre-commit hook refuses **every** commit when no chained hook exists (`set -eu` + failing `resolve` assignment aborts before the `[ -x $next ]` guard). Verified by reproduction; this repo satisfies the trigger. Survived because SL-228's own coord worktree predates its hook install — the slice never ran its own hook. | Filed |
| **ISS-250** | `dispatch_reap`'s not-landed refusal prescribes `dispatch_reap`; routes the operator to the CLI gc escape. | Filed |
| **ISS-251** | `slice selector doctor` false green: reports healthy when a file named in a phase **objective / exit criterion** has no design-target selector. IMP-256's check covers VT `test_file` paths only. Confirmed twice independently. | Filed |
| — | NEW-OQ-C's revisit trigger observed (above). | Feeds reconcile |

## Measurement (EX-2)

Baseline is the **case-notes ledger**, not a controlled A/B — no numeric
SL-224/225 figure exists anywhere (searched RFC-011, SL-224/225 notes/design,
the SL-228 scope). "Cluster-1-cleaned baseline" names a state of the world.

| Round | Turns | Terminal | Output | Cache w/r | Cost |
|---|---|---|---|---|---|
| probe ×2 | 10 | completed | — | — | $0.07 |
| S0 (standard) | 121 | max_turns | 56,449 | 149,544 / 13.0M | $9.41 |
| S4a (to crash point) | ~89 msgs | killed by harness | (undercounted) | 116,911 / 7.9M | ~$4.71 est |
| S4b (blind resume) | 90 | **completed** | 30,993 | 85,657 / 4.8M | $4.04 |
| **Total** | | | **~90k** | **~352k / 25.7M** | **~$18.23** |

**Completion:** a memory-blind orchestrator drove a populated funnel end to end,
twice, including a full recovery from a dead context — with zero harness
rescue and zero memory access.

**Against the ledger baseline:** the frictions this run actually paid for were
(a) ~15 turns on two harness artifacts of my own making, (b) 2 calls on the
`--slice` argument shape, (c) the coord-staleness quirk, diagnosed unaided.
None of the four remediated top-5 quirks recurred. The two genuine defects found
were *new* — and one of them (ISS-250) is a consequence of PHASE-08's own fix.

## Stated limits

1. **n=1.** No statistical claim. One subject, one model, one fixture.
2. **The fixture is a 2-phase toy** over markdown documents, with cheap
   configured cadences. Scenarios carry the difficulty, not the fixture.
3. **The measured arm was the subprocess (pi) arm, not the claude arm.** The
   clone lacked untracked `.claude/`, which is what the dispatch skill routes on,
   so the subject correctly took the subprocess path. `dispatch arm-spawn` + the
   Agent-tool spawn were **not exercised**. Harness defect, recorded not hidden.
4. **Two harness artifacts cost the subject ~15 turns**: the missing `.claude/`
   above, and gitignored `web/map/dist/` being absent from a clone, which made
   the baked `prove` fallback red — and `worktree import --from-worktree` runs
   `prove` as a reject-and-halt gate, so an unrouted `prove` halts every
   subprocess-arm import.
5. **S3 not exercised; S2 did not fire.** Both were pre-emptable by an operator
   who reads before writing. Recorded rather than re-run, per the budget stop.
6. **S0's per-rung funnel rows were lost** to a substrate restore before the
   end-state archiver existed; S0's ladder is attested by the captured
   `dispatch status` and PHASE-02's record. Later rounds are archived under
   `end-states/`.
7. **Baseline is qualitative** (limit 6 of the protocol, not a defect of it) —
   see EX-2 above.

## Verdict against the OQ-5 bar

The verb surface carried a memory-blind orchestrator through a full funnel and
through a crash, unaided. On the evidence, `next` earns D4's "forcing function"
claim for *positional* guidance. The claim that **refusals name the fixing verb**
does not survive intact: S5's refusal named the verb that produced it, and the
operator's resulting action destroyed work the guard had just protected. That is
the finding this benchmark exists to produce, and it is actionable (ISS-250).

**VH-1 — accepted by the User, 2026-07-27.** Accepted on the record as written,
stated limits included. Limit 3 (the claude arm was never on trial) is carried
forward rather than discharged: it is not re-run here, and it keeps the 15
claude-arm memories held in `oq6-retirement.md` Tier B2 held. `oq6-retirement.md`
remains a draft list — acceptance of this benchmark is not an instruction to
retire anything.
