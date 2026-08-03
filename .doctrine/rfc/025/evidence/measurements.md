# RFC-025 · the measurement table

Produced by **SL-241 PHASE-06** (EX-4), 2026-08-03. Companion to `README.md` and
`matrix.md`; **read `README.md`'s limits first** — they bound everything here.

`probe-specs.md` § Measurements defines a **before/after** count "on the same
real phase". The rig measures capsule runs. **The incumbent column has no source
unless one is named**, so every row below names *both* sources or records itself
as after-side-only with its reason (F-10). A row that quoted the design's own
parenthetical would be an invented incumbent wearing a citation; rows 2 and 5
were counted against the code, and doing so **corrected the design's list** — see
row 2's note.

Rows countable by static inspection of the two models are the honest ones. The
runtime rows either name an incumbent source or are recorded after-side-only.

## Citation legend

| mark | meaning |
|---|---|
| **static** | counted by inspection of source / spec, with the location cited |
| **measured** | taken trusted-side by the rig, from a banked TSV |
| **capsule-reported** | the source is INSIDE the trust boundary — see row 8 and F-P06-5 |
| **not measured** | no source exists in scope; recorded, never estimated |

---

## 1 · trust-bearing lifecycle states

| before (incumbent) | after (capsule) |
|---|---|
| **7** — static | **4** — static |

**Before.** `funnel-machine.md` § Transitions: `none` → `spawned` →
`worker-committed` → `imported` → `verified` → `concluded` → `reaped`
(`src/funnel_machine.rs:46-51`, plus the `None` origin). Each gates verb legality
(C7, REQ-384–387), so each is trust-bearing rather than merely informational.

**After.** The four stages of design § 5.1: harvest, conform, verify, advance.

---

## 2 · mutable refs written per accepted phase

| before (incumbent) | after (capsule) |
|---|---|
| **3 attributable per phase, + a share of 3–4 slice-level refs** — static | **1** — measured, and *asserted* |

**Before.** Enumerated from **REQ-311** (SPEC-022 FR-001, the ref taxonomy and
its two mutability classes), then checked against the code. Attributable to one
accepted phase:

1. `dispatch/<N>` — the coordination SSoT and "the funnel's sole write target";
   advanced under CAS once per funnel beat (`src/dispatch.rs:4357`).
2. **the worker's fork branch** — worker forks ride the `dispatch/<name>` prefix
   too, so the prefix does not distinguish them (`mem_019f191431d2`).
3. `phase/<N>-NN` — exactly one per phase with a non-empty code range, cut from
   `boundaries.toml` (`src/dispatch.rs:4310`). Immutable *class*, but written.

Amortised over the slice's *k* phases rather than attributable to one:
`review/<N>` (`src/dispatch.rs:4256`), `candidate/<N>/<label>`, trunk, and
optionally `edge`.

> **The design's parenthetical was wrong in two ways, and this row corrects it.**
> It reads *"coord branch, projected refs, candidate branch, `candidates.toml`,
> trunk"*. `candidates.toml` is a **file, not a ref** — it cannot appear in a
> count of refs. And the list conflates **per-phase** writes with **per-slice**
> ones, which is the difference between 3 and 7 depending on how *k* falls. This
> is exactly why the sheet required the row be counted rather than quoted.

**After. One** — and it is not merely counted, it is **asserted every run**:
`assert_outcome` checks that exactly one canonical ref changed (I1, EX-10), and
that leg passed in both P-C1a and P-C1b. The after-side of this row is therefore
*measured*, not static.

---

## 3 · security-significant hooks (target 0)

| before (incumbent) | after (capsule) |
|---|---|
| **2 shipped assets** — static | **0** — measured |

**Before.** Static grep of the shipped hook set: `install/git-hooks/pre-commit`
(the coord safe-commit / funnel-reversion backstop, installed by
`dispatch.rs:1831 install_coord_hook`, ISS-234) and
`plugins/doctrine/hooks/hooks.json` (the SubagentStart/Stop matcher set, census
B5).

**After.** `control/audit-nohooks.sh`, and **not as a bare grep**: rows B1–B6
each carry a token leg *and* a witness leg (the subject is unrepresentable in the
model). A grep returning nothing proves only that the grep ran
(`mem_019fa18161f4`).

---

## 4 · role-detection rules (target 0)

| before (incumbent) | after (capsule) |
|---|---|
| **4** — static | **0** — measured |

**Before.** Census rows **B1–B4**: the on-disk worker marker; the
`DOCTRINE_WORKER` env leg plus `worker_mode`; the fail-closed
marker-absent-in-a-linked-worktree rule (REQ-192); and marker recovery /
`describe_mode`.

**After.** `audit-nohooks`'s B1/B2/B4 legs. Authority is conferred by the
sandbox, so there is no marker to stamp, clear, or recover.

---

## 5 · git operations between doorbell and accepted-ref advance

| before (incumbent) | after (capsule) |
|---|---|
| **4 in the equivalent window, one of which materialises a tree** — static | **2 touching the canonical, one of which moves a ref** — static + measured |

**After.** In the whole window, exactly two git operations touch the **canonical**
repository:

1. `git fetch --no-tags` of the quarantine ref — **objects only, no ref created**
   (`control/pipeline.sh:627`).
2. `git update-ref <ref> <new> <old>` — the three-argument form **is** the
   compare-and-swap; one ref move (`control/pipeline.sh:636`).

Everything else in the window runs against non-canonical clones (the quarantine,
the verify capsule). Cross-checked by P-C1a and P-C1b's `assert_outcome`.

> **A third fetch into canonical exists in this file and is deliberately not
> counted.** `control/pipeline.sh:391` is the *deleted second hop* — what
> `probe-specs.md` originally described and this design removes, because it puts
> hostile objects in the canonical database **before** the refusal meant to stop
> them. It is wired behind `RIG_DEFECT_CANONICAL_HOP`, off by default, and exists
> **only** so the object-count clause can be shown going RED against a real wrong
> admission rather than a hand-waved one (F-3, VA-3). Naming it here rather than
> letting the grep-count speak: an unmentioned conditional in the window is
> indistinguishable from an undercount.

**Before.** The equivalent window is worker-commit → coord-ref advance. On the
**subprocess arm** the orchestrator imports the working-tree diff
(`src/worktree/import.rs:352-403`): `rev-parse` (read), `diff` ×2 (read), then
**`git apply --index`** — which mutates the coordination tree's *index and
working tree*. The conclude beat then commits the journal onto `dispatch/<N>`
under CAS.

> **The load-bearing difference is the kind, not the count.** The incumbent
> materialises a tree and stages into a shared index — the hazard class that
> produced ISS-234 and the pathless-commit guard. The capsule model never
> materialises a tree at all: it moves objects and one ref. A row reporting "4 vs
> 2" and stopping there would understate this by describing the wrong axis.
>
> **Arm caveat.** The count above is the subprocess arm. The claude arm's worker
> self-commits through the gated `worker_commit` tool, so its import leg differs.
> Both arms converge on the same coord-ref advance.

---

## 6 · ~~git ops between worker-done and candidate-create~~ — RETIRED

There is no `candidate create` on the after side (D8), so the metric has no
endpoint. Re-endpointed by row 5. Retained as a numbered row so the table's
row-count matches the design's and no reader concludes one was dropped.

---

## 7 · wall-clock and disk per accepted phase

| before (incumbent) | after (capsule) |
|---|---|
| **not measured** | **measured** — absolutes, never deltas (VA-2) |

**Before — not measured, and the reason is scope, not oversight.** No
instrumented incumbent run is in scope for this spike. Nothing here is a
before/after *comparison*; the after column stands alone.

**After.** From P-C1a's banked TSV (`~/capsules/probes/c1a/results.tsv`), the
light fixture, five runs. Warm steady state:

| step | range |
|---|---|
| setup | 0.022 – 0.033 s |
| clone / provision | 0.047 – 0.052 s |
| build (`tsc`) | 0.319 – 0.408 s |
| test | 0.243 – 0.264 s |
| phase (stub worker) | 0.046 – 0.051 s |
| harvest → advance | 0.345 – 0.365 s |
| peak disk, worker capsule | 704 512 – 716 800 B |
| peak disk, verify capsule | 663 552 – 667 648 B |

**The first run is the outlier and it is kept, not smoothed:** clone 7.736 s and
harvest 7.691 s against a cold page cache. A table reporting only the warm
numbers would misrepresent a first-capsule-of-the-day.

With a **real agent** (P-C1b, scored run): phase 51.130 s, harvest 0.357 s, peak
disk 757 760 B worker / 667 648 B verify. The phase step is the agent; every
other step is within P-C1a's range, which is the point — the model's overhead
does not move when the worker becomes non-deterministic.

---

## 8 · tokens per accepted phase

| before (incumbent) | after (capsule) |
|---|---|
| **not measured** | **capsule-reported**, n = 1 |

**Before — not measured.** As row 7.

**After.** P-C1b, the scored run of 2026-08-03 (`rig=1a1383b0`), **n = 1**:

| field | value |
|---|---|
| `input_tokens` | 15 |
| `output_tokens` | 3 415 |
| `cache_creation_input_tokens` | 15 079 |
| `cache_read_input_tokens` | 195 816 |
| `total_cost_usd` | 0.334952 |

**Three caveats travel with these numbers and none is optional.**

**(a) `capsule-reported` — the one measurement with no trusted-side source**
(F-P06-5). The harness that counts tokens is `claude`, and `claude` runs inside
the capsule, which is precisely what EX-3 requires. `probe-c1a.sh:82-85` states
the rule this breaks in the rig's own words. Two mitigations, both taken, neither
a fix: the JSON is **streamed over the worker's stdout** so it never becomes a
file the capsule could revise, and it is **recorded, never asserted** — no stage,
token, outcome or altitude reads it, so I5 holds. A capsule that wanted to lie
about its cost could. That is the honest column header.

**(b) No single headline figure is published** (D-P06-3, F-P06-2). The four
figures differ by ~4 orders of magnitude and a system-prompt cache floor rides
every headless run regardless of the phase. A bare "tokens" number would be
ambiguous by orders of magnitude. **The phase-attributable subset is
`output_tokens` plus `input_tokens`**; the cache figures are dominated by that
floor.

**(c) n = 1, one scored attempt** (D-P06-2). It can support *"a phase reaches
green in a capsule at roughly this cost"*. It **cannot** support a comparison.

### The prior attempt — disclosed, not discarded (D-P06-2, D-P06-4)

An earlier run reached the model and is therefore an *attempt*, whatever it
returned. It is recorded here rather than dropped:

| field | prior attempt | scored run |
|---|---|---|
| `input_tokens` | 24 | 15 |
| `output_tokens` | 8 131 | 3 415 |
| `cache_creation_input_tokens` | 25 872 | 15 079 |
| `cache_read_input_tokens` | 388 449 | 195 816 |
| `total_cost_usd` | 0.6571435 | 0.334952 |
| phase wall-clock | 142.235 s | 51.130 s |
| ritual | `agent-committed=no tree-dirty=yes` | `agent-committed=yes tree-dirty=no` |

**The prior attempt's numbers describe a degraded agent, not a phase** (F-P06-7).
The sandbox profile ro-bound the agent home, so the harness could not create its
per-session working directory and **every shell call failed**. The agent wrote
correct code by reading the tests and reasoning through them by hand, and
committed nothing. Raw evidence preserved at
`~/capsules/probes/c1b/attempt-1/`.

**One result worth carrying out of this table: a degraded agent is not a cheap
one.** The attempt that executed nothing cost **2.4× the output tokens and 2.0×
the cache reads** of the run that did the work. That points against the intuition
that a blocked agent gives up early, and it is a cost of the failure mode, not of
the model.

---

## 9 · distinct failure states requiring operator action (target 0)

| before (incumbent) | after (capsule) |
|---|---|
| **qualitative** — the affordance census | **0 of 18 enumerated** — static + observed |

**Before — qualitative by design, not by omission.** The affordance census (§ 2)
and the SPEC-021 D7 catalogue (census § E, 8 rows) are before/after *context*.
This row deliberately replaced an earlier "recovery affordances reachable
(target 0)", which was an assertion wearing a number's clothes — one cannot count
the affordances of a model that does not exist yet.

**After.** The closed refusal-token vocabulary (`control/pipeline.sh:92-95`) —
**18 (stage, token) pairs across four stages**, 17 distinct token strings
(`resource-cap` is legal in two stages):

| stage | tokens | n |
|---|---|---|
| harvest | `fsck-failed` `oid-mismatch` `resource-cap` `bundle-invalid` `bundle-absent` `bundle-unsafe-path` | 6 |
| conform | `ancestry-not-descendant` `ancestry-merge-commit` `undeclared-path` `forbidden-path` `gitlink` `gitmodules` | 6 |
| verify | `suite-failed` `verify-timeout` `sandbox-failed` `resource-cap` | 4 |
| advance | `stale-base` `cas-lost` | 2 |

**None requires operator action.** Each terminates the run with the accepted ref
unmoved and the capsule discarded; there is no partial state to repair and no
prompt to answer. Observed across every P-C1a, P-C1b and P-C3 run: each
terminated on a stage verdict, none paused for an operator.

**The honest limit, stated rather than papered over:** the set is closed but
**not fully exercised**. `cas-lost` is legal and owned by no matrix row —
producing it means racing the accepted ref between stage 4's precondition read
and its CAS, which probes the rig's own scheduling rather than a hostile capsule.
*Reachable-but-unexercised* is a weaker and more accurate claim than "the rig
cannot produce it"; an unexercised path stated as impossible is how a gap stops
being looked at.

---

## What this table does not establish

Beyond `README.md`'s seven limits, which apply unchanged:

- **No row here is a before/after comparison of the same real phase**, which is
  what `probe-specs.md` § Measurements literally asks for. Rows 7 and 8 have no
  incumbent source at all; rows 1–5 compare *models by static inspection*, not
  runs. The table is honest about which is which, and that is the most it claims.
- **Row 8 is n = 1 and capsule-reported.** Both caveats are structural, not
  provisional — the second cannot be fixed without dissolving EX-3.
- **Row 5's incumbent count is the subprocess arm.** The claude arm differs.
- The rig ran **in-jail on Linux/bwrap** throughout. Nothing here is portable to
  macOS without re-measurement.
