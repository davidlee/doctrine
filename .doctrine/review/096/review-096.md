# Review RV-096 — reconciliation of SL-117

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-117 (claude-force-subprocess-dispatch) against
design.md. The slice is one phase, three files changed, no dependencies.

### Lines of attack

1. **DispatchConfig field** — is the bool present, serde-default false,
   correctly parsed, with exhaustive tests?
2. **dtoml roundtrip** — does the combined-keys test survive the full
   `DoctrineToml` deserialize path?
3. **Skill prose** — does step 3 include config path, absent-file default,
   `.claude/` detection, pi fallback, and env-marker inference?
4. **Gate** — clippy zero warnings, no regressions in existing tests.

### Evidence

- Commit: d4760450
- `just check`: 1892 passed, 2 pre-existing failures (catalog::scan, SL-103)
- 13/13 dispatch_config + dtoml tests pass

## Synthesis

SL-117 is a clean, minimal implementation — one bool field, four unit tests, one
dtoml roundtrip test, and a ~10-line skill prose update. Every design invariant
holds:

- `claude_force_subprocess_dispatch: bool` with `#[serde(default)]` → default `false`
  via both serde and Rust `Default` derive.
- Four dispatch_config tests cover true, false, absent-key, and combined-keys parse.
- One dtoml test proves the full `DoctrineToml` round-trip with both dispatch keys.
- Step 3 routing prose includes: `doctrine.toml` path, absent-file default `false`,
  `.claude/` presence detection, `pi` fallback, and env-marker inference.
- Description line mentions the config override for discoverability.
- Clippy zero warnings. No regressions (2 pre-existing `catalog::scan` failures are
  SL-103's, unrelated).

No design-governance findings. No code divergences. The slice is audit-clean.

### Standing risks

- **Prose-only enforcement.** The config key has no binary consumer — correctness
  rests on orchestrator LLMs faithfully reading skill prose. Consistent with
  `preferred-subprocess-harness` posture.
- **`preferred-subprocess-harness` unwired (IMP-101).** The dispatch-subprocess
  skill still has two hardcoded spawn templates. SL-117's routing prose names `pi`
  as a concrete fallback, so the system degrades gracefully.

## Reconciliation Brief

No spec or governance changes needed. The implementation matches design.md
exactly — no per-slice edits, no REV write surface changes.

### Per-slice (direct edit)

None.

### Governance/spec (REV)

None.

## Reconciliation Outcome

No findings were raised — the implementation conforms exactly to design.md.
No per-slice edits, no REV authoring, no withdrawals or tolerations needed.
Reconcile pass complete — handoff to /close.
