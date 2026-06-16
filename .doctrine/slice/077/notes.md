# Notes SL-077: Render requirement prose in spec show

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## 2026-06-16 — Inquisition (RV-042)

Design reviewed and purified. Six findings corrected in design.md:

- **F-1 (BLOCKER):** Struck `build_registry` from D1 table — it keeps its inline
  parse for `is_second_parent` classification.
- **F-2 (MAJOR):** Added `description`/prose reconciliation to D4 — both render,
  neither deprecated; description is the structural summary, prose is the full body.
- **F-3 (MAJOR):** Specified comment-detection contract in D3 — single-line HTML
  only, per-line evaluation, non-HTML syntax treated as content.
- **F-4 (MAJOR):** Defined dangling-member prose column (`—`) in D6.
- **F-5 (MAJOR):** Changed `load_body` to degrade-and-continue (`Option<String>`,
  not `Result`).
- **F-6 (MINOR):** Added aspirational caveat to D4 demo example.

One nit (F-7, path helper) followed-up, not blocking.

Design is now ready for plan. Key watchpoints for implementation:
- `prune_empty_headings` must use exact comment-detection contract from D3
- `load_body` returns `Option<String>`, never errors
- `build_registry` is NOT converted to `read_spec` — preserve the inline
  `is_second_parent` match
- Existing test suites must stay green (behaviour-preservation gate)
