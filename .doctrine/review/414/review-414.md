# Review RV-414 — reconciliation of SL-273

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Conformance audit of SL-273 (library ownership and `lib:` citations), run in
the adopted capsule worktree `.worktrees/SL-273-c32` on branch
`capsule/SL-273/c32` (HEAD `0ec9904a9`). Surface reviewed: that branch's tree;
the slice's commits are `d804787d7..HEAD`, sweep base `cff015419`.

Lines of attack:

1. **Invariants I1–I6** (design §7.1) hold on the landed tree: both shipped-roots
   tests run by default and pass; `essentials.md` ≤ 88 lines; one constant for
   the marker, the exemption set and the test roots.
2. **RV-408 control obligations** — F-1 (per-occurrence sweep verifier),
   F-2..F-5 (seeded-walk, scanner-suffix, disk-manifest, degraded-read
   controls) became plan criteria; verify each against its test or notes
   evidence, then close the deferred findings on RV-408.
3. **VT/VA evidence** — `slice verify-vt`, the VA outputs in `notes.md`.
4. **Path conformance** — `slice conformance` undeclared / undelivered cells.
5. **Design §6.4 audit re-pass** — a fresh inventory of the landed tree
   (`inventory-audit.toml`) compared with `inventory.toml`.
6. **Recorded deviations** — PHASE-04 ran as Claude capsule-workers, not the
   planned DeepSeek confined dispatch (user decision 2026-09-28).
7. `doctrine check gate` green.
