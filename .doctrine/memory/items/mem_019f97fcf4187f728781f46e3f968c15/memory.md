# dispatch sync --integrate projects .doctrine/** too — coord-authored corrections DO reach trunk

`--integrate` is **not** code-only. It carries the candidate's `.doctrine/**`
alongside `src/`/`tests/`, fast-forward-only under CAS, fail-closed on any
authored path the slice did not author (escape hatch: `--allow-corpus-clobber
<path>`, repeatable, global across both the `--trunk` and `--edge` legs).

The consequence people get wrong in both directions:

- **Corrections authored on `review/<N>` / `dispatch/<N>` DO propagate to trunk.**
  Selector-registry fixes and `plan.toml` `test_file` corrections made by the
  coordinator during dispatch land at integrate. Redoing them on `edge` during
  reconcile is redundant work.
- **Reconcile edits made on `edge` do NOT reach trunk via integrate.** Integrate
  projects the *candidate's* corpus, so trunk receives the pre-reconcile
  `design.md` / `slice-NNN.toml`. Edge leads, main follows — the two diverge and
  must be reconciled edge↔main *after* integrate, not by a plain
  `git fetch . edge:main` (integrate has by then advanced main, so that FF is
  refused).

**How to apply.** Before authoring reconcile edits, diff the candidate's corpus
against the primary tree (`git diff <candidate-ref> refs/heads/edge --
.doctrine/slice/<N>/`) to see what integration will already carry. Note also that
`slice conformance` run on the primary tree reads a **stale** registry while the
coord's corrected one still sits on `review/<N>`.

Observed at SL-227: the RV-302 Reconciliation Outcome recorded a "code class only,
so corrections do not propagate" premise and redid the selector fixes on edge; the
post-integrate diff showed `review/227`'s corrections had in fact reached main, and
the only genuine remaining delta was `status` plus one stale `src/main.rs` selector.

Related: [[mem.pattern.dispatch.close-lands-via-candidate-integrate-trunk]],
[[mem.pattern.dispatch.integrate-needs-close-target-first]].
