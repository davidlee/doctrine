# Script-format change must co-update the pi-spawn parity scraper (pi_spawn_core_tokens)

`src/worktree/jail.rs::tests::pi_spawn_core_tokens()` proves flag parity between
the Rust `bwrap_core_argv()` builder and `scripts/pi-spawn-confined.sh` by
**scraping the script's text** — dropping comment lines, splicing `\`-continuations,
tokenizing on whitespace, then taking the tokens between the `bwrap` token and a
**boundary token**. It is therefore coupled to the script's exact surface form.

## What happened (SL-185 PHASE-03, 2026-07-02)

The original scraper took tokens `bwrap … until "pi"` because the script had a
single inline `timeout bwrap … pi …` invocation. P03 hoisted the flags into
`PREFIX=( bwrap … )` and drove them through a single
`timeout "${PREFIX[@]}" pi …` exec site — so `pi` no longer trails the flags; the
scraper slurped the array-close `)` plus the intervening `timeout "$BACKSTOP"
"${PREFIX[@]}"` tokens and the parity assertion failed. Fix: move the boundary
token from `"pi"` to `")"` (the `PREFIX=( … )` array close). Flags are byte-identical;
only the scraper's terminator changed.

## How to apply

- Any edit to the `bwrap …`/`PREFIX=( … )` region of `scripts/pi-spawn-confined.sh`
  (or the (B) follow-up that swaps the Linux inline array for a `jail-prefix --out`
  reader) **must** re-check `pi_spawn_core_tokens()`'s boundary assumptions and run
  `cargo test --bin doctrine bwrap_core_argv_matches_pi_spawn_core_flags`.
- A text-scraping parity test is brittle by construction. If it breaks a third
  time, consider replacing the scrape with a generated/emitted artifact (have the
  script `source` a doctrine-emitted flag list, or assert against `jail-prefix`'s
  own `--out`) so parity is structural, not lexical.

**Relates to RFC-005** (dispatch funnel integrity — hazard survey): the Linux
subprocess arm's confinement parity is a funnel-correctness invariant (the shell
wrap must equal the audited `bwrap_argv`); a scraper that silently mis-parses a
reformatted script could hide a real divergence. Boundary-token fragility is the
weak point.

Surfaced via [[mem.pattern.dispatch.worker-prompt-run-full-suite]] (the worker
didn't run this `--bin` test, so shipped it red).
