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

## Synthesis

SL-273 delivers what its design locked. The `lib:` form has one constant
(`LIB_PREFIX`), a pure scanner/resolver in `src/lib_citation.rs`, and two
callers: build-repo tests over the shipped roots and a doctor leg over client
`.doctrine/**`. `library show` accepts the prefix verbatim. `routing-process.md`
became `essentials.md` through REV-069, `boot-footer.md` is retired, and the
sweep converted 120 inventory rows with every change checked per occurrence.

Evidence on the landed tree:
- `doctrine check gate` green (9165 passed, 0 failed). Both invariant tests
  run by default and pass: `every_shipped_lib_citation_resolves` (I1) and
  `no_bare_library_mention_in_shipped_text` (I2).
- `verify-sweep.nu --through U4`: 0 violations; the only extra edit is the
  plan-authorised `#[ignore]` removal (I5).
- RV-408's five control findings (F-1 blocker, F-2..F-5) verified against
  their tests and the recorded control runs; RV-408 concluded.
- `slice verify-vt`: all pass except PHASE-01 VT-2, superseded by design (F-2).

Fixed in the audit: `essentials.md` had drifted to 89 lines through PHASE-04's
longer `lib:` citations; re-broken to 88 with no word change (F-1).

Standing risks and accepted tradeoffs:
- **I4 has no test.** The line budget is agent-verified only, which is how
  PHASE-04 broke it unnoticed. Cheap to pin if it matters.
- **Restate depth.** The fresh re-pass found 58 restate leads the first
  inventory did not hold (F-4). Accepted here as a bar question for IMP-505;
  until then skills still carry summary paragraphs beside `lib:` citations,
  and three of them already contradict their owners (F-5, ISS-505).
- **Process deviation.** PHASE-03/04 and the re-pass ran on Claude workers,
  not the planned DeepSeek pi workers (no API keys outside the jail; user
  decisions 2026-09-28 and 2026-09-29). Independence came from fresh
  contexts and the per-occurrence verifier rather than a different model.
- **Conformance tooling** attributes a path to the broadest matching
  selector and reports the exact selectors undelivered (observation
  recorded); read that cell by hand until fixed.

## Reconciliation Brief

### Per-slice (direct edit)
- F-3 — selector registry: `doctrine slice selector rm SL-273 src/lib.rs`, then
  `doctrine slice selector add SL-273 src/main.rs` (design-target; the file
  that declares `mod lib_citation`). Mirror in design.md §8.2's code-impact row
  "`src/lib.rs` or `src/main.rs`" → `src/main.rs`.

### Governance/spec (REV)
- None. REV-069 already carried the ADR-005 / ADR-024 / SPEC-011 changes and is
  applied.

## Reconciliation Outcome

### Direct edits applied
- Selector registry (`slice-273.toml`): removed `src/lib.rs`; added
  `src/main.rs` (design-target) — the file that declares `mod lib_citation`
  (finding F-3).
- design.md §8.2 code-impact row: "`src/lib.rs` or `src/main.rs`" →
  "`src/main.rs`" (finding F-3).

`slice conformance SL-273` no longer lists `src/lib.rs`. Remaining
undelivered cells: the eight exact `src/*` selectors shadowed by `src/**`
(conformance-tool behaviour, per F-3 disposition), and
`.doctrine/slice/273/inventory-audit.toml`, which is committed at 94b434384
but still reported undelivered — noted for /close, not re-audited here.

### REVs completed
- None needed. REV-069 (ADR-005 / ADR-024 / SPEC-011) was applied in PHASE-02
  and is `done`.

### Withdrawn / tolerated / follow-up
- F-1: fixed during audit.
- F-2: tolerated — PHASE-01 VT-2 checked the `#[ignore]` marker PHASE-04 was
  planned to remove.
- F-4: tolerated — restate-half inventory handed to IMP-505.
- F-5: follow-up — pre-existing skill/owner contradictions handed to ISS-505.

Reconcile pass complete — handoff to /close.
