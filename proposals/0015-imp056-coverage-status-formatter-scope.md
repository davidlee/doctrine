---
seq: 0015
scope: backlog
target: IMP-056 (Coverage CLI status rendering)
confidence: high
reversible: yes (proposal only; no code/backlog transition — fence holds)
---
## What
IMP-056 ("Coverage CLI status rendering: stable kebab-case formatter, not Debug")
is correct and the defect is real, but it reads as a one-line change and **isn't** —
it touches a tested, possibly-persisted register. Grounding:

`CoverageStatus` (`src/requirement.rs:144`: `Planned/InProgress/Verified/Failed/
Blocked`) is rendered to a user/log token via **Debug** at two sites, not one:
- `coverage_verify::status_label` (`src/coverage_verify.rs:258-261`):
  `format!("{status:?}")` — the CLI verify-report token (`old→new`).
- `coverage_store::withdrawal_line` (`src/coverage_store.rs:178-183`):
  `"withdrew …/{} [{status:?}]"` — and `status_label`'s own doc says it exists "matching
  `coverage_store::withdrawal_line`'s `[Failed]` register," so the two are
  *intentionally coupled* to the same spelling.

So a "stable kebab-case formatter" must be a **single shared formatter** both sites
route through (a `Display` impl or `fn token()` on `CoverageStatus`) — exactly the
DRY shape, and consistent with IMP-056's intent. But:
- `withdrawal_line` has a **golden test** asserting the PascalCase Debug spelling:
  `assert_eq!(line, "withdrew SL-057/REQ-256/SL-057/VT [Failed]")`
  (`src/coverage_store.rs:800`). Kebab (`failed`) breaks it.
- `withdrawal_line` is a **register line** ("withdrew …") — if it is persisted to a
  store/log (its name and `[Failed]` "register" framing suggest so), changing the
  token is a **format migration**, not just display polish.

That is the real, hidden decision in IMP-056: is the kebab token **CLI-report-only**
(change `status_label`, leave the withdrawal register as-is — but then the two
deliberately-coupled spellings *diverge*), or **register-wide** (change both, update
the golden test, and accept a withdrawal-line format change)? The item as written
doesn't say, and the two sub-options have different blast radii.

## Options
1. **Shared formatter, register-wide.** Add `Display`/`token()` on `CoverageStatus`
   (kebab), route BOTH `status_label` and `withdrawal_line` through it, update the
   `withdrawal_line` golden test. Tradeoff: DRY + consistent + matches IMP-056's
   "stable formatter" intent; but changes the persisted/loggable withdrawal format —
   needs a check that nothing parses the old `[Failed]` spelling back.
2. **CLI-report-only.** Kebab-format `status_label` alone; leave `withdrawal_line`
   on Debug. Tradeoff: smallest, zero register-format risk; but breaks the
   deliberate coupling (the doc comment) — report says `failed`, withdrawal log says
   `Failed`, a new inconsistency.
3. **Shared formatter that *equals* the current Debug spelling** (PascalCase via an
   explicit map, not `{:?}`). Tradeoff: removes the Debug-as-display fragility (the
   real risk: a variant rename silently changes output) without changing any visible
   token or test. Doesn't deliver "kebab," but delivers "stable" — which is the
   actual correctness goal IMP-056's title leads with.

## Recommendation
Option 1 (shared kebab formatter, register-wide) **if** the withdrawal register is
not parsed back anywhere — verify that first. The single shared formatter is the
DRY-correct shape and honours the deliberate `status_label`↔`withdrawal_line`
coupling. If the register *is* consumed/parsed, fall back to Option 3 (stable
explicit formatter at the current spelling) which kills the Debug-as-display
fragility — the genuine risk — without a format migration, and leave the kebab
cosmetics for a deliberate later change. Either way, the fix is "one shared
formatter," never two divergent ones (Option 2).

Decisions deferred to YOU:
- (a) **is the `withdrawal_line` register persisted/parsed**, or display-only? (sets
  Option 1 vs 3 — the load-bearing question).
- (b) **kebab vs stable-PascalCase** — is the goal cosmetic (kebab) or robustness
  (stop using Debug as a display contract)? The title says both; they're separable.
- (c) confirm the shared formatter lives on `CoverageStatus` in `requirement.rs`
  (the type's home), not in either consumer.

## Next doctrine move
```
# confirm the register's consumers before choosing scope (read-only):
grep -rn 'withdrew \|withdrawal_line\|\[Failed\]\|\[Verified\]' src/   # is it parsed back?
doctrine backlog show IMP-056

# the fix is code — route it (NOT executed; small, but still code):
/route      # → small slice or boot.md-Governance "small backlog item" quick-design.
```
(Verbs described, NOT executed.)

## Illustration (optional) — ILLUSTRATIVE, not applied
Hand-authored (no worker), the shared-formatter shape (Option 1):
```rust
// src/requirement.rs — one stable token, both consumers route through it.
impl std::fmt::Display for CoverageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Planned => "planned",
            Self::InProgress => "in-progress",
            Self::Verified => "verified",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
        })
    }
}
```
```diff
- // coverage_verify.rs
- fn status_label(status: CoverageStatus) -> String { format!("{status:?}") }
+ fn status_label(status: CoverageStatus) -> String { status.to_string() }
- // coverage_store.rs::withdrawal_line
-        "withdrew {}/{}/{}/{} [{status:?}]",
+        "withdrew {}/{}/{}/{} [{status}]",
  // and update the golden test: [Failed] → [failed]
```
