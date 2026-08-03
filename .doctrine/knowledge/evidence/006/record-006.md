# EVD-006: Hostile git config, hooks and disk-cap hazards behave identically on fetch and bundle

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Datum

The two hazards RFC-025 named as *ingestion-boundary* risks — a hostile
`.git/config` plus hooks (H6) and an oversized object / pathological tree (H7) —
score identically on both mechanisms. Eight scored cells, all `pass` at
`model-level`, from the SL-241 C3 probe matrix:

| row | M-A (fetch) light / heavy | M-B (bundle) light / heavy | boundary |
|---|---|---|---|
| H6 | pass / pass | pass / pass | **dissolution** |
| H7 | pass / pass | pass / pass | `harvest/resource-cap` |

**H6 dissolves on both, and the reason is structural, not defensive.** The
capsule sets `core.hooksPath` and writes eight executable hooks in its own
clone; the plant is live, not vacuous — a reference-transaction hook was
*observed firing inside the capsule*. It never crosses the boundary because
config and hooks are **repo-local, never objects**, and both mechanisms
transfer objects. Neither `git fetch` nor `git bundle` carries them.

## What this row does and does not say about QUE-200's `upload-pack` worry

**Corrected in SL-241 PHASE-06 (F-P06-11).** This section previously read
*"`upload-pack` running in the capsule's context remains untested here."* That
was wrong, and the row's own plant is what disproves it.

QUE-200 records a trust-bearing claim against candidate 1: *"git's
protected-config rules cover everything a hostile repo config can do to
upload-pack"* — to test, not assume. **The row tests it, deliberately.** Beyond
`core.hooksPath`, `H6_mutate` plants two config-borne execution triggers that are
not hooks (`instantiations.sh:640–641`):

| key | honoured from repo-level config? | why it did not fire trusted-side |
|---|---|---|
| `uploadpack.packObjectsHook` | **no** — protected config only | **git defended it.** `upload-pack` runs this instead of `pack-objects` when serving a fetch, i.e. as a child of the trusted side's own `git fetch`. The defence is *observed*, not taken from the documentation |
| `core.fsmonitor` | **yes** | **nothing triggered it.** Firing needs an index refresh; `rev-parse` does not perform one, `git status` does |

`upload-pack` therefore ran in the capsule repo's context on **every M-A cell** —
verified in-jail rather than inferred, `GIT_TRACE=1` on a plain-path fetch
spawning `git-upload-pack '<path>'` (git 2.54.0).

**What is untested is narrower and sharper.** One key was defended and one was
merely not reached, so the row samples the surface without clearing it:

- The **universal** QUE-200 states is not discharged by two keys, one of which
  turns out not to be protected at all. It is bound to git 2.54.0 besides.
- `core.fsmonitor`'s silence is a property of **which commands the M-A harvester
  runs**, not of git. `harvest-fetch.sh` touches the capsule's clone exactly
  three times — `rev-parse` (`:57`, `:66`) and the `fetch` (`:73`). A `status`,
  `diff`, `add` or `checkout` added there would turn this same passing plant into
  trusted-side arbitrary execution. **This row is the regression detector for
  that** — `c3_assert_never_ran_in` reads the execution log — but it detects it
  only when re-run.

**And the dissolution's stated reasoning is sound but incomplete.** *"Config and
hooks are repo-local, never objects"* covers what **travels**; it is silent on the
trusted side **going to** the hostile config. On M-A the parent runs git three
times with the capsule's `.git/config` on its effective cascade. On M-B it runs
git in the capsule repo **zero** times — `harvest-bundle.sh` reads a flat file.
That asymmetry is M-A's, and it is the mirror of EVD-010's.

**"Fetch is proven safe against hostile config" is still not what this matrix
says** — but the reason is that the surface was *sampled*, not that it was
*never entered*.

**H7 refuses at `harvest/resource-cap` on both**, same stage, same token.

## Two caveats that change what this can be cited for

1. **The disk cap is enforced *in-capsule*, not at ingestion** (F-P05-37).
   `sandbox.sh` applies `ulimit -f` inside the namespace and `du -s` at the end
   of the capsule run — before control returns to the harness. Every other row
   plants trusted-side between `pipeline_capsule` and `pipeline_run`; that seam
   is *upstream of nothing* for H7, so H7 needed genuine capsule-time hostility.
   The mechanism-neutrality H7 shows is therefore about the harvest stage's
   *reading* of an in-capsule enforcement outcome — not about what the mechanism
   itself transfers.

2. **The observable is the refused write's signature, not its product**
   (F-P05-38). `ulimit -f` refuses the write *at* the bound, so an oversized
   file never materialises: the capsule's attempted file is present and
   **zero-length**, beside a 201-deep tree, with the capsule's cumulative
   footprint *under* the cap (1.5 MB light / ~201 MB heavy against a 512 MB
   bound) — which is what says the per-file leg fired and `du` had nothing to
   say. A first draft of this assert counted files at or over the cap and could
   only ever have red, on every host, forever.

## What it bears on

Neither hazard discriminates M-A from M-B. Whichever mechanism QUE-200 settles
on, these two are handled the same way, so they carry no weight in the choice.

## Related

- [[safe-capsule-ingestion-mechanism]] — QUE-200, the question this informs.
- SL-241 PHASE-05 T4c; `~/capsules/probes/c3/results.tsv` rows H6, H7.
- Findings F-P05-37 (in-capsule enforcement point), F-P05-38 (a clause that
  cannot fail is not a control).
