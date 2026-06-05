# Notes SL-011: Cache-friendly session boot context

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Mechanism (what shipped)

`src/boot.rs` — the cache-friendly governance snapshot. A pure assembler
(`boot_sequence` + `render_boot`) projects stable governance into the agent's
cached session-start prefix; a thin impure shell (`produce`, `write_if_changed`,
`run`, `wire`/`run_install`) gathers section bodies, resolves `current_exe()`,
and writes only on content change (the content-diff cache key). Verbs:

- `doctrine boot` — regenerate `.doctrine/state/boot.md` (the hook target).
- `doctrine boot install` — wire the `@`-import + per-harness SessionStart hook.
- `doctrine boot --check` — DISK sentry: stale (≠ recompute) / unpopulated.

Harness seam is `enum Harness {Claude, Codex}` + `match` (NOT trait/Box<dyn> —
D8/Charge IV; mirrors `skills.rs`). Identity-unification with `skills::Agent` is
deferred debt until SL-012 frees `skills.rs` (the §3 concurrency gate).

## Durable findings (harvested from phase sheets 04 + 05)

- **`--check` is a DISK sentry, never a session sentry** (codex F2 / design §5.4).
  It sees the *file* fresh while the *current inlined prefix* stays stale until
  `/clear`/restart. CLI wording is disk-scoped on purpose; in-session freshness is
  `/route`'s ≤2-session lag warning + the `/canon` freshen-now ritual (regenerate
  THEN `/clear`), never an `--check` claim.
- **`boot_check` is deterministic — no clock, no in-content timestamp.** A
  generation timestamp baked into the snapshot would bust the cache every session
  and defeat §1, so freshness is reported out-of-band only. One render path:
  `build_sections` is shared by `regenerate` and `boot_check` (no second fork).
- **Clippy gate footgun.** `just check` runs plain `cargo clippy` (bins/lib only).
  `cargo clippy --all-targets` lights up ~800 `unwrap_used`/`expect_used` denials
  in TEST code (the workspace restriction lints apply to test targets too). Mirror
  the gate, not `--all-targets`. Captured in AGENTS.md conventions.
- **`--check` flag can't `conflicts_with` the `install` subcommand** — clap
  subcommands aren't a conflictable arg group. Dropped the attribute; dispatch
  precedence (`Some(Install)` wins, else `None if check`) makes `--check` a no-op
  when `install` is present (documented in the arg help).
- **Ownership match is dep-free `rsplit`, not shell-split** (PHASE-04, A4). Hook
  command is always `<program> boot` — split on the *last* whitespace keeps a
  spaced program path whole, zero dep. FOOTGUN: a second/spaced hook arg breaks
  it → switch to a real shell-word split (shell-words crate). Guarded by a NOTE on
  `is_doctrine_boot_command` + the spaced-path test.
- **JSON merge does not enable serde_json `preserve_order`** — the Value
  round-trip preserves every foreign key/hook (tested) but re-sorts object keys
  alphabetically. Acceptable: `settings.local.json` is gitignored/regenerable.
- **Symlink dedup writes through to the real inode.** `resolve_target` =
  `canonicalize`-or-original; the `CLAUDE.md → AGENTS.md` case writes the resolved
  `AGENTS.md` (one inode, one write) and the symlink survives — never `rename`d
  over (which would split the inode).
- **Integration tests exercise `wire`/`regenerate`/`boot_check`, not the
  `current_exe()` shells.** Under `cargo test`, `current_exe()` is
  `doctrine-<hash>`, rejected by the ownership match — so inject a fake exec and
  test the inner fn, never through `run_install`/`run`.

## Closure gate (POST-BUILD, USER-RUN)

The build self-installs and dogfoods clean (`boot install` wired this repo's
`@`-import + hook; `boot` regenerates; `boot --check` reports in-sync, only the
genuinely-empty `Accepted ADRs` unpopulated). Two confirmations remain that only
a live harness can give — recorded here as the closure verification:

1. **The live ordeal.** A real Claude session must witness (a) the
   `startup|clear` SessionStart matcher firing the hook, and (b) the ≤2-session
   in-session lag (an edit to governance becomes visible only after the
   freshen-now ritual or ≤2 session-starts). The matcher token is asserted by the
   research doc and `clear`-fires-SessionStart is already witnessed; the OR-token
   string itself wants live confirmation.
2. **The live codex run.** Confirm codex inlines the `AGENTS.md` `@`-import into
   its system prompt (no SessionStart equivalent → unbounded staleness if it does
   not) and honours the routing.

**Codex cut-from-v1 fallback:** if the import does not inline for codex, or the
unbounded codex staleness is unacceptable, cut codex from v1 — keep `import_targets`
Claude-only and leave the `Harness::Codex` arm as the staged seam for a later
slice. Claude (with its hook) is the supported v1 harness regardless.
