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

That matters for the specific worry QUE-200 records against candidate 1: *"git's
protected-config rules cover everything a hostile repo config can do to
upload-pack"* is a trust-bearing claim to test, not assume. This row does **not**
discharge that claim — it shows the hazard class does not reach the parent under
either mechanism, which is a different and weaker statement. `upload-pack`
running in the capsule's context remains untested here.

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
