# RFC-025 · capsule spike — evidence

Produced by **SL-241**, 2026-08-01 … 2026-08-03. The bulk is PHASE-05's C3
ingestion probe, whose verdict this file opens with; the cost, confinement,
selector and rig-selftest tables from the other phases were archived alongside
it at audit — same reason, same treatment (RV-343 F-11, and the disposal sweep
that followed it). This directory is the durable record; everything else the
spike produced lives in disposable scratch or runtime state and will not
survive close.

RFC-025's next-design-question 1 asks what the minimal safe parent-side mechanism
is for ingesting a phase result from a potentially hostile capsule git
repository. It is tracked as **QUE-200**, and the probe exists to answer it with
measurements rather than argument. **QUE-201** (where a client's
interpretation-surface declaration lives) gained one evidence input from the same
rig.

## The result

**The mechanism axis barely matters, and the one measured asymmetry runs against
the bundle.**

Two mechanisms were rigged side by side — **M-A**, fetch into quarantine
(RFC-025's leading hypothesis), and **M-B**, bundle. Sixteen hazard rows ran
against both, on two fixtures each. On every axis expected to separate them they
are equal:

| axis | result | record |
|---|---|---|
| hostile `.git/config` + hooks; oversized objects | identical — 8 cells, all pass | EVD-006 |
| where the trust-bearing lines are | all trusted-side, in the mechanism-*independent* downstream pipeline | EVD-007 |
| forensic retention of worker history | both preserve the worker's commit identity | EVD-008 |
| wall-clock cost | equal (heavy cell: M-A 2.50s, M-B 2.55s) | EVD-009 |

The single measured asymmetry is **H13**: M-B must read a file the capsule wrote,
so it needs four trusted-side refusal legs (`bundle-absent`, `bundle-invalid`,
`bundle-unsafe-path`, `resource-cap`). On M-A the row is **structurally** `n/a` —
no trusted-side artifact is read, so there is nothing to refuse. That is
counter-evidence to the bundle's stated advantage in QUE-200 ("cleanest trust
story"): the surface is **moved, not removed**. Recorded as **EVD-010**, the only
`disputes` edge in the set.

Two further costs sit on the M-B side, neither a security failure:

- `harvest/fsck-failed` is emitted at **two sites** with git's stderr discarded
  at both, so one token stands for "the ingested objects are bad"
  (security-relevant) and "this quarantine's derived cache is stale"
  (operational) — F-P05-28, EVD-007;
- M-B's larger transfer trips git's own auto-maintenance inside the quarantine,
  which produced a spurious refusal M-A did not — F-P05-28, EVD-009. Bounded at
  N=1 and fixture-caused; **not** generalised to "bundles are fragile".

## What this evidence does NOT establish

Read these before citing anything above.

1. **QUE-200's `upload-pack` surface is SAMPLED, not cleared.** *(Restated in
   PHASE-06 — F-P06-11. This limit previously said the vector "was never
   exercised", which the rig's own plant disproves.)* `upload-pack` **did** run
   in the capsule repo's context on every M-A cell — a plain-path `git fetch`
   spawns `git-upload-pack '<path>'` (verified in-jail, git 2.54.0) — and H6
   plants `uploadpack.packObjectsHook` there deliberately, observing git's
   protected-config defence hold rather than trusting the documentation for it.
   What is **not** established:
   - QUE-200's claim is a **universal** — *"git's protected-config rules cover
     everything a hostile repo config can do to upload-pack"*. Two keys do not
     discharge it, and the result is bound to git 2.54.0.
   - **One of the two keys was not defended at all.** `core.fsmonitor` *is*
     honoured from repo-level config; it stayed silent because nothing in the
     M-A harvest path refreshes an index (`git status` fires it, `rev-parse`
     does not). That safety is a property of which commands
     `harvest-fetch.sh` runs — `rev-parse` at `:57`/`:66` and the `fetch` at
     `:73`, three touches of the capsule's clone — not a property of git.
   - The dissolution's reasoning, *"config and hooks are repo-local, never
     objects"*, covers what **travels** and is silent on the trusted side
     **going to** the hostile config. M-B runs git in the capsule repo **zero**
     times; M-A runs it three.

   **"Fetch is proven safe against hostile config" is still not what this matrix
   says** — the reason is that the surface was sampled, not that it was never
   entered.
2. **Candidate 3 (tree materialization) was never rigged**, on QUE-200's own
   recommendation. There is no measurement for or against it here beyond the
   structural argument that it forfeits forensic history.
3. **`n/a` is not "not attempted".** Every `n/a` in the table names a
   *structural* absence — H13/M-A has no artifact to refuse; H12/light has no
   `.envrc` or `flake.nix` to plant. A cell that was merely hard was never
   recorded as `n/a` (R-C).
4. **`counts-toward-nothing` legs count toward nothing.** The conflict sub-probe
   (H10/H16's candidate-layer legs) is explicitly excluded from the model-level
   claim. See `guards.md`.
5. **Altitude caps the claim.** `model-level` means the row held on both
   fixtures. `unproven-beyond-rust` (H12) means one fixture had nothing to plant,
   so portability is not established. The vocabulary is design § 5.4 / A-3.
6. **One row's altitude is earned by two different boundaries and the table
   cannot say so** — F-P05-36. The column is not as flat as it reads.
7. **Four `fail` rows in `results-c3.tsv` are four successes.** The conflict
   sub-probe recorded its falsification round in-band; those rows are stamped
   `MUTATED=m32…m35` in the preceding `p-c3:` preamble, and a mutant that reds is
   the mutant working. Anything counting outcomes must respect that stamp. See
   `guards.md`.

## What is here

| path | tier | what |
|---|---|---|
| `README.md` | committed | this file — the verdict and its limits |
| `matrix.md` | committed | the sixteen hazard rows, per-mechanism |
| `guards.md` | committed | the five guard probes and the conflict sub-probe |
| `measurements.md` | committed | the § 9 measurement table, nine rows, each naming both sources or saying which is absent — **required reading for the go/no-go**, whose opening sends you to its closing section |
| `results-c3.tsv` | committed | **the generated measurement table** — the scored matrix, verbatim |
| `results-guards.tsv` | committed | the guard probes' scored table, verbatim |
| `results-c2.tsv` | committed | **P-C2's confinement rows, verbatim** — five appended runs; the last (`p-c2: 2026-08-03T04:36:53Z`) is the scored one, after the D-P06-5 profile change. Archived at audit (RV-343 F-11) from `$SPIKE_CAPSULE_ROOT/probes/c2/results.tsv`, which is outside the repository and disposable |
| `results-smoke.tsv` | committed | the A2 legs — network, credential, and F-P06-6's capability leg. Same provenance and same reason as `results-c2.tsv` |
| `results-c1a.tsv` | committed | **P-C1a's cost rows, verbatim** — the stub-worker timings `measurements.md` cites. Five appended runs; the last (`p-c1a: 2026-08-03T04:17:39Z`) is the scored one |
| `results-c1b.tsv` | committed | **P-C1b's cost rows, verbatim** — the *real-agent* run. Two appended runs; the last (`p-c1b: 2026-08-03T04:38:58Z`, rig `1a1383b0`) is the scored one. See the reading note below — the two blocks differ in a way that matters |
| `results-selftest.tsv` | committed | **the rig's own falsification record** — 51 appended runs, 513 scored legs, none failing. This is what makes "the falsifiers actually falsify" measured rather than asserted; `results-c3.tsv` cites it |
| `results-r2.tsv` | committed | the R2 selector/conformance probe — seven edges, one table, no preamble. Carries the R2b decisive row (a *declared* `.doctrine/` touch is conformant to `--strict`) |
| `exec-proof-smoke` | committed | six bytes — the kernel string the capsule wrote from inside. `results-smoke.tsv` records that the capability leg passed; **this file is the artefact that was the pass** |
| `run-c1b/` | committed | the one real-agent run's forensic tree — `contract`, `stages` (the four-stage ledger), `pinned-oid`, `worker-stdout`, `interpretation-surface.txt`. See below |
| `phase-sheets/` | committed | **archived runtime phase sheets** — see below |
| `drivers/` | committed | the falsification and diagnostic drivers, re-runnable |
| `.doctrine/state/rfc-025/raw/` | runtime, gitignored | raw run logs — the exhibit, not the evidence (design § 5.3 as amended) |

`results-c3.tsv` is the authority for the ingestion half; `results-c2.tsv` is
the authority for the confinement half; `results-c1a.tsv` and `results-c1b.tsv`
are the authority for the cost rows in `measurements.md`. The summaries here
cite them; where they disagree, the TSV wins.

**The `results-*.tsv` are append-only.** Every one except `results-r2.tsv` (a
single table, no preamble) accumulates run blocks, so **a bare grep returns rows
from superseded profiles**. Only the block under the *last* preamble is scored.
Superseded blocks are kept, not pruned — a correction is only legible against
what it replaced.

- **`results-c2.tsv`** — five `p-c2:` runs. The earlier `api-cred` rows are the
  ones F-P06-8 found asserting on a proxy; they no longer stand.
- **`results-c1a.tsv`** — five `p-c1a:` runs, the first three against an older
  base (`169e7ac82c22`) and two of those against a dirty rig.
- **`results-c1b.tsv`** — two `p-c1b:` runs, and **the difference between them is
  the point**. The superseded first (`03:24:26Z`, phase 142.235s) ends
  `agent-committed=no tree-dirty=yes`; the scored second (`04:38:58Z`, phase
  51.130s) ends `agent-committed=yes tree-dirty=no`. That flip is the claude
  arm's self-commit ritual arriving. **Cite 51.130s, not 142.235s** — and note
  the `n = 1` caveat stamped on every token row: these support "a phase reaches
  green at roughly this cost", never a comparison, and the token counts are
  capsule-*reported* from inside the trust boundary (F-P06-5).
- **`results-selftest.tsv`** — 51 `selftest:` runs, all green. **`fail` appears
  75 times and never as an outcome**: 45 are the refusal token `suite-failed`,
  29 the word `failure` in prose. Field-exact, it is 513 `pass` and zero `fail`.
  Same trap as limit 7 above — count on the column, never on a substring.

**Reading `run-c1b/`:** the forensic tree of the scored real-agent run.
`stages` is the four-stage ledger (`harvest`/`conform`/`verify`/`advance`, all
`pass`, no token) — RFC-025's central pipeline structure observed rather than
argued. `contract` names the base, accepted ref and verify command;
`pinned-oid` is what the capsule declared. `worker-stdout` (64K) is the agent's
own transcript, and it is **inside** the trust boundary: read it as testimony,
not as measurement. The run's git clones (`canonical/`, `quarantine/`,
`capsule/`, `verify-capsule/`) were **not** archived — they are 2.8M of
rebuildable object stores and nothing cites them.

## What was deliberately NOT archived

So that a later reader does not go looking for it, or assume it was lost:

- **The fixtures (177M, `$SPIKE_CAPSULE_ROOT/fixtures/`).** Reconstitutable, not
  lost: the builders `scripts/spike-capsule/control/fixture-{heavy,light}.sh`
  are tracked, and heavy's pin `4fc9d32f0` is recorded here (phase-06.md, and
  six `results-c3.tsv` rows). Recipe plus pin rebuilds them. A-P06-3's "do not
  rebuild fixtures" was a *within-spike* freeze so measurements stayed
  comparable — it never claimed they were irreplaceable.
- **The raw run logs (`.doctrine/state/rfc-025/raw/`, ~60 files).** Design § 5.3
  as amended rules them the exhibit, not the evidence. The ruling was checked
  before they were let go rather than taken on faith: **no committed artefact
  cites any raw log by name** (zero hits for the path pattern across
  `.doctrine/`), so nothing here dangles without them.
- **`probes/c1b/attempt-1/results.tsv`.** Verified a strict subset — byte-equal
  to the first eighteen lines of `results-c1b.tsv`.

## Citations — how to resolve them

The summaries are **citational**: they state the results and point at the
reasoning rather than restating it. Three id families appear.

- **`F-P05-nn` / `D-P05-nn`** — findings and decisions from PHASE-05's runtime
  sheet. That sheet is disposable and is discarded at slice close, so it is
  **archived here**: `phase-sheets/phase-05.md`. Findings are newest-first under
  `## Findings`; decisions under `## Decisions`. Earlier phases' `F-P0n-nn` /
  `D-P0n-nn` resolve in the sibling sheets.
- **`EVD-006` … `EVD-011`** — the six evidence records carrying the verdict
  inputs, linked to QUE-200 / QUE-201. Read with
  `doctrine knowledge show EVD-006`.
- **`I1`–`I5`, `F-2`…`F-14`, `A-1`–`A-3`, `R5`, `S1`–`S4`, `§ 5.n`** — design
  invariants, falsifiers, assumptions, rules and sections. They resolve in
  `.doctrine/slice/241/design.md`.

The slice's durable harvest — decisions, findings and evidence lifted out of the
sheets — is `.doctrine/slice/241/notes.md`. Prefer it for anything you expect to
outlive RFC-025.

### On the archived phase sheets

They are **frozen exhibits, not live tracking.** They carry task checkboxes,
in-flight uncertainty, hypotheses later falsified, and at least one finding whose
first reading was wrong and is corrected further down the same entry. That is
deliberate — the reasoning behind an expensively-obtained finding is worth more
than a tidied summary of it, and F-P05-39 is the standing lesson about what it
costs when the working artefacts are not kept.

Nothing in `phase-sheets/` is authoritative for slice state. `doctrine slice show
241` and `notes.md` own that.
