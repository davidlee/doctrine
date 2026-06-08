# Audit SL-027 — DRY backlog test-fixture TOML builders

Mode: **conformance** (post-implementation, single phase PHASE-01).
Reconciled against `design.md` (D1, §3–§5), `plan.toml` (EX/VT/VA), and the
behaviour-preservation gate (CLAUDE.md). Implementation commit: `c3fb9dc`.

## Evidence

- `just check` green end-to-end: fmt + `cargo clippy` zero warnings + test +
  build (full run 2026-06-09).
- Backlog suite: 40 `#[test]` in `src/backlog.rs` `mod tests`, all green.
- Refactor diff (`git show c3fb9dc -- src/backlog.rs`): single file, +151/−67,
  every hunk ≥ line 1410 (test module starts :994) → **zero production change**.
- `git show c3fb9dc | grep assert` → empty → **zero assertion edits** (VT-1).

## Findings

| # | Expected (cite) | Observed | Disposition |
|---|---|---|---|
| F1 | EX-1: one core builder owns the literal — `render_fixture_toml` + `write_fixture` + `toml_list`; head/dates/path/quoting once each | All three present (`:1441`/`:1451`/`:1484`); head literal + `2026-06-08` dates + `backlog-{name}.toml` path + list quoting each appear once | **aligned** |
| F2 | EX-2: `write_item`/`write_assessed_risk`/`write_related` survive as thin wrappers, ~30 call sites unchanged | Wrappers at `:1499`/`:2005`/`:2031` delegate to `write_fixture`; signatures match design §3.3; diff carries no call-site edits beyond F3 | **aligned** |
| F3 | EX-3: `:1813` (`backlog_show_json_is_faithful_item_state`) becomes one `write_fixture` call; manual `create_dir_all`+`fs::write` removed; assertions untouched | Folded to a single `write_fixture(Fixture{..})` (`:1882`); manual dir/write gone; the `read_item`+`show_json` asserts intact | **aligned** |
| F4 | EX-4: deliberately-explicit parser/error fixtures left intact | The three in-memory/error-path literals untouched — now at `:1157` (parser round-trip), `:1190` (unknown-enum validate), `:2159` (edit-malformed) | **aligned** |
| F5 | VT-1: backlog suite green with zero assertion edits | 40 green; diff has no `assert` lines | **aligned** |
| F6 | VT-2: `just check` passes | Green | **aligned** |
| F7 | VA-1: `grep -c 'created = "2026-06-08"'` == 4 | The plan's grep recipe matches **0** — Rust source escapes the quotes; the correct pattern `created = \"2026-06-08\"` returns **4** (unified builder `:1455` + the three explicit survivors `:1157`/`:1190`/`:2159`). Verification *intent* (4 copies → 1 builder + 3 explicit) is met. | **aligned** — VA-1 substance holds. The grep recipe in `plan.toml` is a doc nit (unescaped quotes); corrected at `/close` so the recipe reproduces. |

## Byte-equivalence (design §4)

Confirmed by the gate, not re-derived: the 40 downstream assertions are
unchanged and green, so the unified builder emits byte-identical TOML for every
shape (head-only, +facet+rels, fully-assessed risk). Any transcription drift in
the literal (residual risk R-residual) would have reddened the suite — it did not.

## Reconciliation

No code finding requires a fix. Design and plan tell a coherent story with the
code. One authored-doc nit (F7) — the VA-1 grep pattern needs escaped quotes —
to correct at close. ISS-001 transitions to its resolving state at `/close`.

Risks R1/R2 retired by D1 (design §7); no durable gotcha worth promoting to
memory beyond what `mem.pattern.lint.string-build-no-push-format` already covers
(honoured here despite `cfg(test)` being unlinted).

**Audit-ready for `/close`.**
