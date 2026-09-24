# Review RV-380 — code-review of SL-263

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Closure-grade adversarial **code review** of the SL-263 implementation delta only
(`PHASE-01`..`PHASE-04`); `PHASE-05` (governance leg) is out of scope. Subject is
the slice (`SL-263`); the code is the evidence. Reviewed per recorded boundary:
P1 `3dd5e8de4..c7efe9f37`, P2 `14d7b3ebb..553f6dec8`, P3 `6d3811eee..c1583705b`,
P4 `b18beeda1..0a2756c7c`.

**Foreign span.** P1's range carries an SL-261 commit (`58a4c57f3 design adopt
--diff`) touching `src/commands/design.rs`, `src/design_run/snapshot.rs`,
`tests/e2e_design_state.rs`, `Cargo.toml`/`Cargo.lock`. Not reviewed — recorded
here so the delta is not read as SL-263's.

**Lens.** Design §5.1–§5.10 (esp. §5.6 handler-level codex fields, §5.7 the six
pi contract bullets, §5.9 the two-form Claude upgrade); plan `EX`/`VT`/`VA` per
phase; `STD-001` (named constants), `STD-003` (a skipped leg is said), `ADR-001`
(layering), `POL-002` (no host coupling in the neutral core).

**Lines of attack** (confirm or kill against the code, not against assertion):

1. `entry_is_canonical`'s new `None ⇒ key-absent` rule — correct invariant, or
   over-heal that drops a key an operator added by hand?
2. Claude two-form ownership vs the codex predicate — disjoint for every
   rendering (deleted / portable / spaced-program)? Can a re-install duplicate?
3. The pi adapter's `[...event.content]` / `ctx.sessionManager` throws — does
   that break the fail-open contract?
4. `templates/surface.ts` hardcoding pi tool names + neutral classes that also
   exist as Rust consts — STD-001 breach or unavoidable cross-language dup, and
   what detects drift?
5. The generic `PiExtension` / `install_extension` / `write_ext_outcome`
   refactor — behaviour genuinely preserved? `#[cfg(test)]` wrappers dead?
6. The behavioural VTs use a stub child — is the codec↔adapter contract covered,
   or is VT-2 theatre a field-name typo would pass?
7. `write_ext_outcome` allocation, and stale docs / under-specified assertions
   in the codex registry loop.
8. Test depth — VTs asserting implementation (source substrings) where behaviour
   was intended.

### Outcomes of the candidates

- **#1 CONFIRMED** → raised (F-1). Reproduced live.
- **#2 KILLED.** `is_doctrine_command` suffix-strip + `is_doctrine_program`: for
  any program half the two Claude args (`…--input claude`, bare `…memory
  surface`) and the codex arg (`…--input codex`) cannot both match; `plan_hook`
  drops all owned and re-inserts exactly `matchers.len()`, so duplication is
  structurally impossible.
- **#3 KILLED.** pi's `ExtensionRunner.emitToolResult` wraps every handler in
  `try { await handler(...) } catch (err) { emitError(...) }` (verified in the
  installed `@earendil-works/pi-coding-agent` dist), so a post-await throw is
  absorbed and the tool result is left intact — the fail-open outcome holds. The
  EPIPE path (a process-level `'error'` event, not a rejection) is separately
  handled by the no-op stdin listener.
- **#4 PARTLY KILLED.** STD-001's declared scope is `src/` (and tests), so the
  `templates/*.ts` class tokens are not a STD-001 breach; but the tokens are
  duplicated cross-language with **no drift detector** — folded into F-2.
- **#5 KILLED.** The generic core is faithful (report strings, dry-run,
  foreign-skip, parent-dir creation all preserved); the `#[cfg(test)]` wrappers
  are referenced by their per-extension test suites, so not dead.
- **#6 CONFIRMED** → raised (F-2): no test round-trips the TS envelope through
  the real `--input neutral` decoder; the stub ignores its argv.
- **#7 KILLED for staleness** (docs and the two-element slice assertion are
  up to date); a small in-Rust STD-001 duplicate surfaced instead → raised (F-3).
- **#8 CONFIRMED in part** → folded into F-2 (the child's argument vector is
  verified only by a `content.contains("--input")` source-substring check).
- **Extra**: `discover_surface_anchor` returns an unlabelled `(PathBuf,
  PathBuf)` while a `SurfaceAnchor` type exists → raised (F-4, nit).

## Synthesis

**Overall: revision-required** — the architecture and the design implementation
are sound, but two majors introduced by this slice must be reconciled before
close. Neither is a blocker (the ledger is done, nothing gates the RV); both are
small and well-scoped.

### Synopsis

SL-263 ports `memory surface` to codex and pi by keeping every harness-shaped
thing in a codec and letting the core see only a `(Surface, ScopeProbe)`. The
implementation honours that: `SurfaceRequest`, `SurfaceAnchor`, the three
`*_request` codecs and `paths_from_patch` are pure and live in `src/memory.rs`;
the pipeline is composed once; `ScopeProbe`'s path arm widened to a set with no
query/ranking change; no new module (ADR-001 layering intact); harness names are
confined to the codecs and the boot registry. `STD-001` is largely honoured, and
where it isn't the fix is a one-liner (F-3).

The work earns the ledger's praise in the places that are hardest to get right.
The codex handler fields are golden-pinned as a whole-file literal, so the exact
nesting (inside `hooks: […]`, beside `command`) is asserted — the failure mode
§5.6 warned about (a key on the matcher group, invisible to a presence check)
is genuinely foreclosed. The Claude two-form ownership is disjoint by
construction and cannot duplicate on re-install. The generic `PiExtension`
refactor preserved every observable behaviour (report strings, dry-run,
foreign-skip, parent-dir creation). The pi adapter's fail-open outcome holds: I
verified against the installed `@earendil-works/pi-coding-agent` that
`ExtensionRunner.emitToolResult` wraps each handler in `try/catch`, so a post-await
throw is absorbed and the tool result is left intact — the concern that motivated
candidate #3 is killed by the host, not by the handler's own guards. The EPIPE
hazard (a process-level `'error'` event, which the host does *not* catch) is
correctly handled by the no-op stdin listener. The PHASE-01 fixtures are genuine
captures, and the unobtainable `apply_patch`-via-shell case is recorded as an
absence with evidence rather than fabricated.

**F-1** is the one functional regression: `entry_is_canonical`'s
`None ⇒ key-absent` rule makes every doctrine-owned hook claim ownership of
`additionalContextLimit` and `timeout` even when its spec sets neither, so
`plan_hook`'s drop-and-reinsert silently deletes an operator's hand-set `timeout`
(a documented Claude Code per-hook field) on every install. Reproduced live.
The fix is to compare only fields the spec sets, which preserves the intended
codex healing.

**F-2** is the verification gap the design itself tried to close: PHASE-04 VT-2
was written to round-trip the generated extension through the real
`memory surface --input neutral` decoder, but was implemented against a shell
stub that ignores its argv, leaving the only cross-language seam (the neutral
envelope's key/class vocabulary) with no end-to-end detector. The child's arg
vector is asserted only by a `content.contains("--input")` source grep — the very
check §9 forbade here.

**Standing risks, accepted rather than raised.** A multi-path patch competes for
one cap (design R-risk, bounded and noted); pi's nudge is post-execution and
codex's is retrospective (R-3); the subagent-parity delta stands on both ports
(R-6); `SURFACE_CONTEXT_LIMIT_CODEX`'s unit is unverified (headroom holds under
either reading); and the codex handler being baked on an absolute exec path
re-arms `/hooks` trust on every upgrade (R-7).

**Range hygiene.** P1's stored span (`3dd5e8de4..c7efe9f37`) includes an SL-261
commit (`58a4c57f3`) touching `design.rs`/`snapshot.rs`/`e2e_design_state.rs`/
`Cargo.*`. That is foreign to this review and was excluded; noting it so no future
reader mistakes it for SL-263's delta.

**Killed candidates** (recorded so the reasoning is not re-derived): the codex vs
Claude ownership predicates are provably disjoint for every rendering including
the `(deleted)`, portable and spaced-program halves; the `#[cfg(test)]` `plan_*`
wrappers are referenced by their test suites, not dead; no stale doc or
under-specified assertion survives in the codex registry loop or `RefreshReport`;
and the whole-file codex golden is intentionally byte-exact, not brittle.

### Action items

1. **F-1 (major)** — `None` must mean "do not compare"; add a regression test
   seeding a handler field on a spec that sets none. Must land before close.
2. **F-2 (major)** — add a `tests/` round trip using
   `env!("CARGO_BIN_EXE_doctrine")`; re-run PHASE-04 `VT-2`. Must land before close.
3. **F-3, F-4 (nit)** — one shared `BIN_PATH_MARKER`; return a `SurfaceAnchor`
   from `discover_surface_anchor`. Same region, cheap.

### Haiku

Harness names stay out —  
but a hand-set timeout  
still leaves without a fight.

