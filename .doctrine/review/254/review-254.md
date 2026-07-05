# Review RV-254 — design of SL-205

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Target.** `design.md` of SL-205 — ambient memory surfacing via a Claude
`PreToolUse` hook. Design aspect only (scope + plan are not on trial here).

**The design in one breath.** A new `doctrine memory surface` subcommand
(`src/memory.rs`) reads a `PreToolUse` envelope on stdin, discriminates a *path*
surface (`Read|Edit|Write` → `path_scope`/`glob`) from a *command* surface
(`Bash` → `command`, gated `severity ≥ high`), calls a new thin `retrieve_rows`
helper (`src/retrieve.rs`) that composes the existing `load_query`/`query`/
`check_retrievable` path, and emits `additionalContext` — advisory only, `exit 0`
always, main-thread only (`agent_id.is_none()`). Wired via `hooks.json` glue;
classified `Read` in `guard.rs`. Precedent: `doctrine worktree pretooluse`.

**Doctrine it is held to.** POL-002 (no load-bearing on host convention);
ADR-011 (harness-agnostic altitude — Claude-specific behaviour as a per-harness
adapter, neutral core untouched); ADR-001 (leaf ← engine ← command, no cycles);
STD-001 (no magic strings); the fail-open hook invariant
(`mem.fact.claude.pretooluse-hook-fail-open`); the behaviour-preservation gate
(existing memory/retrieve suites green unchanged); DRY / no-parallel-implementation.

**Lines of interrogation.**

1. **POL-002 escape.** Does *any* residue of host-command knowledge survive in
   the neutral core? The severity gate claims to rest only on doctrine-owned
   metadata — prove it launders nothing (e.g. does the command surface parse or
   branch on the command string beyond passing it to `retrieve --command`?).
2. **ADR-011 altitude honesty.** Is the neutral/adapter seam truly clean, or does
   the "thin" `retrieve_rows` smuggle Claude-envelope concerns into `retrieve.rs`?
   Is `retrieve_rows` genuinely a composition of the existing seam, or a parallel
   query implementation wearing a composition mask (DRY / ADR-001)?
3. **Fail-open completeness.** Enumerate every failure path (unparseable stdin,
   absent root, retrieve `Err`, seen-set/log IO error, `session_id` absent). Does
   *every* one fold to emit-nothing/`exit 0`? Any path that could `exit 2`, panic,
   or block a tool call is a mortal heresy (advisory INV-1/INV-2).
4. **Main-thread gate correctness.** `agent_id.is_none()` as the main-thread
   discriminator — is that the true contract, or can a main-thread call ever carry
   an `agent_id` (or a subagent lack one), leaking surfacing into confined workers?
5. **Holdback composition.** Does the severity gate compose *correctly* on top of
   the in-core trust holdback, or can the design surface a held-back memory?
6. **STD-001 / magic strings.** `SEV_FLOOR = "high"` is a string fed to
   `severity_rank` — is the scale single-sourced, or does a second severity scale
   still bleed (the prototype's dead `"major"`)? Any other unnamed constant?
7. **Seen-set / session semantics.** Per-`session_id` file: does a missing/empty/
   reused `session_id` degrade safely? Unbounded `.doctrine/state/` litter — is the
   deferral honest or a swept-under-rug correctness gap?
8. **Verification adequacy.** Do VT-1..11 + VA-1 actually pin the invariants, or
   are there untested seams (the impure shell's IO ordering, the two-spawn
   interaction with the jail on a shared matcher, the guard.rs classification)?
9. **Design-target completeness.** Are the four declared targets (`memory.rs`,
   `retrieve.rs`, `guard.rs`, `hooks.json`) the true touch-set, or does shipping
   demand an undeclared edit (embed roots, manifest, gitignore negation for the
   new state files, `mod`/`use` wiring)?

The internal pass already confessed F1–F6 (see design §10). The external
inquisitor must presume those fixes themselves harbour fresh heresy, and hunt the
seams the author was too close to see.

## Synthesis

**Judgement.** The design of SL-205 is fundamentally sound and doctrinally
aligned — but it was arraigned on three counts and confessed to all three under
cross-examination. No mortal heresy: no POL-002 escape (the severity gate launders
no host-command knowledge — the command string is passed verbatim to `retrieve
--command`, never parsed), no ADR-011 altitude breach (the neutral/adapter seam
holds — `retrieve_rows` composes `load_query`/`query`/`check_retrievable`, no
parallel query), no holdback bypass (the in-core trust holdback runs before the
severity gate), no magic-scale bleed (`severity_rank` is single-sourced). The
reviewer confirmed each innocent plea it tested.

The guilt was concentrated in one underexposed seam: **the emit / IO-failure
contract**. The design proclaimed `exit 0` always (INV-2) yet leaned on a
precedent (`pretooluse.rs:433`) that `?`-propagates its stdout write; it declared
`session_id` optional yet specified only the happy path; and it appended the
seen-set before the emit, so a failed delivery could poison future dedup.

**Penance (all fix-now, all executed on the design artifact):**

1. **F-1** — INV-2 now forbids `?`-ing the emit write; the emit swallows a stdout
   `Err` to emit-nothing/`Ok`. Verified by VT-12 (failing writer).
2. **F-2** — §5.3 defines absent/empty `session_id`: dedup disabled for that fire
   (no IO, no synthetic key, no panic; still surfaces). Verified by VT-13.
3. **F-3** — INV-6 (dedup integrity) + §5.4 reorder: seen-set/log append only
   after a successful non-empty emit. Verified by VT-12/VT-14.

**Standing risks (tolerated, with rationale).** `.doctrine/state/` litter from
per-session seen files — tiny, gitignored, `rm -rf`-able; GC deferred to a named
follow-up (R4), not a correctness gap. The two-spawn interaction on the shared
`Bash`/`Edit|Write` matcher (jail no-op + surface) — cheap, R1-measured; the
design is honest about it.

**Harvest.** Nothing promoted now — the three findings are design-local and fully
reconciled into `design.md`. The emit-error-swallow contract may generalise to the
pi/codex port follow-ups; harvest it as a memory at implementation *if* it proves
cross-harness, not before.

> **HERESIS URITOR; DOCTRINA MANET**
