# Review RV-062 — reconciliation of SL-089

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation reconciliation audit of SL-089 (Map Explorer — backlog
filtering, actionability graph, and prioritization views). All five phases
complete; code lives on `dispatch/089` at a64313e.

**Lines of attack:**

1. **Backend conformance** — Do the ActionabilityView types, survey_for_map
extraction, survey_view_for_map, and DataStores wiring match design.md D1–D12?
Does the frontend remain render-only (no eligibility/blocking/rank logic)?

2. **Frontend integration** — Does the view toggle work? Are kind checkboxes
individual? Does the kind filter contract hold (entity list only, never the
actionability graph)? Are D3 vendor files correct (version, checksum, license)?

3. **Dark theme + styles** — Do CSS variables resolve in both light and dark
modes? Are priority colours legible?

4. **Gate hygiene** — cargo clippy zero warnings, eslint zero warnings, all
phase exit criteria met, existing tests unchanged.

**Evidence sources:** `dispatch/089` branch, `just check` gate results,
code inspection of all touched files.

## Synthesis

SL-089 implementation is clean and conformant. All five phases delivered to spec;
the two findings raised were surface-level nits — a design-prose-to-implementation
mismatch (F-1, aligned) and a redundant DOM element (F-2, fix-now, resolved).

**What went well:**
- Backend types and surface functions match design.md D1–D12 exactly:
  `ActionabilityView` carries `policy_version: "priority.v2"`, nodes with
  server-computed rank, and `needs`/`after` edges.
- `survey_for_map` extraction preserved zero behavioural divergence from CLI
  `survey()` — confirmed by VT-7 byte-for-byte test.
- `DataStores` under single `RwLock` eliminates the torn-read window (D9).
- All existing route handlers ported cleanly to `state.stores` — zero test
  regressions.
- Frontend is genuinely render-only: `priority.js` only calls `d3-dag` layout +
  SVG DOM construction; no eligibility, blocking, or rank computation in JS.
- Kind filter contract holds: checkboxes filter entity list only; actionability
  graph and relationship table are unfiltered.
- CSS variables for all 8 priority-colour properties present; dark-theme
  compatible (semantic status colours, same in both themes).

**Standing risks:**
- The `d3-dag` UMD bundle must extend the `d3` global — if the upstream package
  changes its global injection pattern, the graph will silently blank. Mitigation:
  vendored file with checksum; explicit `<script>` load order.
- One pre-existing test failure (`e2e_memory_sync::sync_produces_all_shipped_dirs`)
  is unrelated to SL-089 and reproducible on `main`.

**Tradeoffs accepted:**
- View toggle not persisted (D7) — acceptable for MVP; follow-up can add
  `localStorage`.
- `--accent-color` CSS variable was never defined; implementation used `--kind-SL`
  (consistent with `.depth-btn.active`). No behavioural impact.

## Reconciliation Brief

### Per-slice (direct edit)

None. All findings are terminal and resolved within the audit.

### Governance/spec (REV)

None. No governance or spec changes surfaced.

## Reconciliation Outcome

All findings were terminal at audit close-out:
- **F-1** (aligned): design.md CSS snippet referenced non-existent `--accent-color`;
  implementation correctly used existing `--kind-SL`. No change needed.
- **F-2** (fix-now, resolved): duplicate "Edge types" label removed from priority
  legend at 90f3f59 on dispatch/089.

No per-slice direct edits or REVs required. Reconciliation pass complete —
handoff to /close.
