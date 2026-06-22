# IMP-010 Design — write_class_tests refactor + reseat atomicity

Two independent remediations from the SL-032 review, consolidated into one
cleanup pass. No new features, no behavioural changes.

---

## §1 F-3 — write_class_tests: argv-driven assertions

**Problem.** The `write_class_tests` module (`src/main.rs:249`, ~200 lines)
hand-builds every `Command` variant with all fields, so any field addition to
any command breaks the table for reasons unrelated to classification. It also
bypasses clap entirely — it cannot catch a CLI-wiring regression the way the
e2e `WRITE_VERBS` tests do.

**Approach.** Replace struct-literal construction with `Cli::try_parse_from`
at the call site, extracting `Command` from the parsed CLI:

```rust
fn cls(args: &[&str]) -> Option<&'static str> {
    use crate::commands::cli::Cli;
    match write_class(&Cli::try_parse_from(args).unwrap().command) {
        WriteClass::Read => None,
        WriteClass::Write(v) | WriteClass::Orchestrator(v) | WriteClass::Hookmint(v) => Some(v),
        WriteClass::MarkerClear => None,
    }
}
```

Each test becomes a compact `assert_eq!(cls(&["doctrine", "install"]), Some("install"))` —
no field noise, and a wiring regression (e.g. removing `install` from `Command`)
would fail at parse time.

**Coverage.** The compiler's exhaustiveness match on `write_class` (no wildcard
arm) already proves every `Command` variant is handled. The unit tests pin only
what cannot be proven by types: the Read/Write split direction and the verb
labels. The e2e `WRITE_VERBS` table in `tests/e2e_worker_guard.rs` already
proves the full guard fires end-to-end — the unit- and integration-level tables
are intentionally redundant at different seams, which is fine. No coverage
reduction.

**Out of scope.** Moving the tests to `src/commands/guard.rs` (where
`write_class` now lives) is a nice-to-have but not required — SL-115 left them
in `main.rs` and moving them adds noise. Leave them unless the compiler forces
it (e.g. `CommonListArgs` becoming private).

---

## §2 F-4 — reseat atomicity

**Problem.** `run_reseat`'s post-guard fs operations are a multi-step sequence
(rename dir → rename internal files → rewrite toml id → swap aliases) with no
commit point. A mid-sequence failure leaves a half-reseated entity *at the
canonical id* that `validate` will flag — and guard-1 then refuses a retry. Note
the current first op (`fs::rename(src_dir, dst_dir)`) is *already* atomic
(same-mount dir rename); the non-atomicity is purely the steps that follow it.

**Approach (decided).** Stage in a sibling temp dir, transform there (invisible),
then one atomic rename is the commit point:

```
1. copy_dir_all(src_dir → tmp)        # tmp = tree_root/.NNN.tmp, same mount, invisible
2. rename internal files in tmp (stem-NNN → stem-MMM)
3. rewrite toml id in tmp             # plain write — tmp invisible, no write_atomic needed
4. rename(tmp → dst_dir)              # SINGLE atomic commit
5. swap aliases (rm old + ln new)
6. rm -rf src_dir
```

`src_dir` stays intact and seated through step 4. The entity does not appear at
the canonical id until the atomic rename. Crash at any point:

| Crash after | State | Recovery |
|---|---|---|
| Step 1–3 | only orphan `.NNN.tmp`; `src_dir` intact & seated; nothing at `dst_dir` | `validate` clean — retry just works; guard cleans/ignores the orphan |
| Step 4 | fully functional entity at `dst_dir`; `src_dir` still present | clean — steps 5/6 idempotent |
| Step 5 (partial) | aliases may be missing, entity content fine | `validate` flags alias, rerun fixes |
| Step 6 | done | — |

No detect-and-resume probe, no "is dst_dir complete?" logic — the atomic commit
makes a partial build invisible, so the only mid-build residue is an ignorable
`.NNN.tmp`. Guard the staging dir: refuse (or clean) if `.NNN.tmp` already
exists, the same no-clobber posture as guard-1.

**Layering.** Add a `copy_dir_all(src, dst)` to `fsutil.rs` (leaf tier) — a
simple recursive `read_dir` + `create_dir_all` + `fs::copy`, no new
dependencies. `run_reseat` (command tier) calls it into `tmp`, transforms `tmp`
in place with `std::fs::rename` + `toml_edit` (already imported), then commits
with a single `std::fs::rename(tmp, dst_dir)`.

**Boundary conditions.**
- Entity directory is small (2–15 files, <50KB typical) — copy is cheap.
- `tmp` and `dst_dir` must be on the same mount for the commit rename to be
  atomic — both are siblings under `.doctrine/<kind>/`, so guaranteed.
- alias symlinks live in `tree_root`, not inside `src_dir`, so
  `copy_dir_all(src_dir)` never copies them.
- The non-zero-on-success dangler exit is unchanged (accepted SL-032 R-3
  contract: forces human to act before committing).
- Symlinks in entity dirs: none exist today; `fs::copy` follows symlinks and
  copies the target, which is fine for a repair verb.

---

## §3 Acceptance checks

- [ ] `write_class_tests` uses `Cli::try_parse_from` — no struct-literal `Command` construction
- [ ] All existing `write_class_tests` pass with argv-driven inputs
- [ ] `copy_dir_all` in `fsutil.rs` handles a typical entity dir (toml+md+subdirs)
- [ ] `run_reseat` produces the same result (dir renamed, internal files renamed, toml id rewritten, aliases swapped) via stage-tmp → atomic-rename commit
- [ ] orphan `.NNN.tmp` present does not block a clean retry; `src` stays seated and `validate`-clean
- [ ] Existing reseat tests pass unchanged
- [ ] `just gate` passes (layering, clippy, tests)
