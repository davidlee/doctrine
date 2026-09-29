# HYP-002: Raw byte+mode+set comparison against HEAD replaces the three legs for the claim question

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The proposition

For `verify`'s **claim question** — *is the claim's evidence committed?* —
comparing raw worktree bytes, **plus the mode bit and an explicitly enumerated
set**, against the HEAD blob is sufficient; and it **subsumes** the three-leg
git-semantic dirt observation together with everything the design layers on it to
make that observation faithful: `NORMATIVE_FLAGS`, `DEC-089`'s attribute-source
partition, `DEC-090`'s `unmeasurable` refusals, and `CON-002`'s git-2.40 floor.

The reasoning is polarity, not preference. The three legs answer *"is git
satisfied?"*; `I6` claims *"the attested commit contains these bytes."* Every
mechanism in the design exists because those two questions can be separated —
content conversion on one axis, freshness suppression on the other — and each
separation needs its own neutralisation or refusal. Comparing the bytes directly
does not have the gap to close.

## What would refute it

Sufficient-and-subsuming predicts:

- **P1** every `RV-314` F-19/21/22/24/33/37/38/42 fixture flips from
  *false-clean* to *detected* under bytes+mode+set. **Tested: held** (`EVD-030`).
- **P2** the surviving failures are exactly the three structural residuals the
  probe measured — the set (deletion, staged add), symlinks (`hash-object`
  follows), and the mode bit — and **no** hazard in the conversion or freshness
  families survives. **Partly tested:** the residuals are measured; the claim of
  their exhaustiveness is an argument from the mechanism's shape, not a census.
- **P3** the **anchor** question and `--allow-dirty` become *separable* from claim
  measurement, so the apparatus serving *them* can be deleted rather than
  amended. **Untested** — this is the design decision the RFC frames, not a
  measurement.

Falsified by any conversion or freshness route byte comparison reads clean on, or
by a claim surface whose correct set or symlink equality cannot be recovered from
one `ls-files(--cached, --others)` + `ls-tree(HEAD)` enumeration.

## What it does not claim

It does **not** claim the anchor question should go — only that it is a separate
question byte comparison does not answer. It does not claim `verified_sha`'s
meaning is settled. Both are `RFC-035`'s subject. It also does not claim the
three-leg `capture()` is worthless elsewhere: `capture()` has three callers
(`record`, `verify`, `retrieve::freeze`) and only the claim leg is in scope.
