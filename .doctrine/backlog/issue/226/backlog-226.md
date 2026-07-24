# ISS-226: verify-vt UNATTRIBUTABLE says keyword present when absent

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed

`doctrine slice verify-vt SL-214` run pre-implementation (plan authored, no
work landed) reports for every mandated file that exists but is untouched:

```
≈ UNATTRIBUTABLE VT-2 — keyword present but `plugins/doctrine/skills/design/SKILL.md` not modified by this slice
```

But the keyword is verifiably absent:

```
$ grep -F "/knowledge" plugins/doctrine/skills/design/SKILL.md; echo $?
1
```

Same for every UNATTRIBUTABLE row that run (PHASE-01 VT-2..4, PHASE-02
VT-1..2; keywords `/knowledge`, `unsettled`).

## Expected

The message should not assert keyword presence it hasn't established (or the
keyword check should run and be reported truthfully). Pre-work, an honest
status is something like "file not modified by this slice — mandate pending",
distinct from "keyword present". As written, an agent must re-grep every
mandated file to disprove the tool before trusting its own keyword floors.

## Guess at mechanism

UNATTRIBUTABLE looks like a short-circuit on the modified-by-slice check with
a canned message that hard-codes "keyword present" regardless of whether the
substring scan ran.

Surfaced while planning SL-214 (RFC-011 case-noted).

## Additional datum — universal UNATTRIBUTABLE on a dispatch-concluded slice (SL-227 audit, RV-302 F-4)

A distinct trigger of the same UNATTRIBUTABLE path, worth recording because it is
*total*, not per-file: at SL-227's conclude/audit, `doctrine slice verify-vt 227`
returned **UNATTRIBUTABLE for EVERY VT across all three phases** — exit 0 (PASS,
non-halting), no Fail/WAIVED/UNCHECKABLE.

- **Not** missing/failing tests: the mandated tests demonstrably exist and pass
  (independent `cargo test --bin doctrine` on `review/227` = 3812 pass; e.g. 80+
  keyword hits in `src/commands/library.rs`; the crux gate in `src/install.rs`).
- **Mechanism (narrower than the title's "keyword present" message):** for a
  slice driven to conclusion via `/dispatch`, the source-delta **base** is
  mis-computed — an 8-commit refresh-base moved the fork-point, so the
  modified-by-this-slice set resolves to empty and every keyword hit is orphaned
  as "keyword present but <file> not modified by this slice". The message is the
  ISS-226 symptom; the base miscomputation for a dispatch-concluded slice is the
  upstream cause of the *universal* verdict.
- **Impact on the conclude gate:** UNATTRIBUTABLE is treated as a gate-fidelity
  artifact (exit 0, non-halting) rather than a delta defect, so it does not block
  audit/close — but it makes verify-vt uninformative for every dispatch-concluded
  slice, forcing a manual re-grep + independent test run to establish the floors.
- See also IMP-228 (`imp-228-vtgate-unattributable` worktree) if a fix is scoped
  there. Surfaced by SL-227 audit RV-302; `.doctrine/review/302/`.
