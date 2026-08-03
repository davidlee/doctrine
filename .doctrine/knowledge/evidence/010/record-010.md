# EVD-010: Bundle adds a trusted-side file-ingestion boundary that fetch does not carry

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Datum

H13 is the row that exists on one mechanism only. Under **M-B (bundle)** the
parent must read a file the capsule wrote, so it needs its own refusal legs —
four of them, each **observed `pass` at `model-level` on both fixtures**:

| leg | token |
|---|---|
| the bundle is missing | `harvest/bundle-absent` |
| the bundle does not verify | `harvest/bundle-invalid` |
| the bundle names an unsafe path | `harvest/bundle-unsafe-path` |
| the bundle exceeds the resource bound | `harvest/resource-cap` |

Under **M-A (fetch)** the whole row is `n/a`, and the `n/a` is **structural, not
unwritten** (R-C's rule): M-A reads no trusted-side artifact at all, so there is
nothing to refuse. That is the asymmetry — measured now, rather than argued.

## Why this is linked `disputes`

QUE-200 records candidate 2's advantage as: *"A bundle is a single flat file —
pure data, no config, no hooks — so the control-plane boundary is `git bundle
verify` + fsck'd fetch from the file. Cleanest trust story."*

The probe shows the surface is **moved, not removed**. The flat file is still
attacker-controlled input, parsed on the trusted side, and it needs a
four-legged boundary of its own that M-A does not need at all. "No config, no
hooks" is true; "cleanest trust story" does not follow from it, because the
comparison M-A wins is *fewer trusted-side reads of capsule-authored bytes*.

The `disputes` edge is against **that stated advantage of candidate 2**, not
against the question. Read it as counter-evidence to a claim QUE-200's own body
makes, which is what the label is for.

## What it does not say

This is a cost, not a defect. All four legs hold. Nothing here says M-B is
unsafe — it says M-B buys its "pure data" property by taking on a boundary M-A
does not have, and that the two arms should be compared on total trusted-side
surface rather than on the config-and-hooks axis alone (where they are equal —
EVD-006).

## Related

- [[safe-capsule-ingestion-mechanism]] — QUE-200, the question this informs.
- EVD-006 — the config/hooks axis, where the two mechanisms are equal.
- EVD-008 — the forensic axis, where they are also equal.
- SL-241 PHASE-05 T4d; `~/capsules/probes/c3/results.tsv` row H13.
- SL-241 `notes.md` § Harvest → Open — where this input was first banked.
