# Review RV-402 — code-review of SL-271

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Subject: the SL-271 implementation delta, 12 commits on
`slice/SL-271-codex-mcp-registration-during-install` in
`.worktrees/SL-271` (unlanded at review time) — `src/boot.rs` +915/-47 (the
bulk), `src/install.rs` +36, `tests/e2e_codex_install.rs` +320, README prose.
Read as evidence in that worktree; the ledger itself is driven from the primary
tree, so `review prime`'s path-set (230 paths) hashed the pre-change `src/boot.rs`
— the staleness signal is meaningless for this pass, by construction, not a
refusal.

Locus: `doctrine show SL-271`, design rev 44 sec-5.2/5.3/5.4 (`doctrine design
resume 271`), `plan.toml` PHASE-01..03 EX/VT rows, `notes.md` (four RV-399
passes, 44 findings), `RV-399` (design review, done). Prior review depth is high,
so this pass attacked the *implementation* of the accepted decisions rather than
the decisions themselves.

Lines of attack, in the order they paid:

1. **Does the probe answer the question the notice asks it?** PHASE-03 turns on a
   live `codex features list` bound to `cwd = install root`. Probe the trust
   gate: does the project's `.codex/config.toml` layer even load before the
   operator has trusted the project — the step the notice prints immediately
   after? Run codex for real (2x2 over trust), do not reason about it.
2. **Ownership predicate.** `classify_codex_entry` (`boot.rs:2290`) against design
   sec-5.2's formula: index guards, `keys_ok`, byte-exact `current` vs normalised
   `owned`, `env_vars` absent-or-exact. Look for a shape that reads as ours when
   it is not, or as foreign when it is ours (thrash).
3. **Edit preservation.** Does the narrow-path write keep comments, siblings,
   `[features]`, and an inline parent's spelling? Does a second run byte-stable
   no-op? Does a non-UTF-8 read stay untouched?
4. **Report seam.** One wording for two arms and two causes, `dry_run` verbs,
   the trust-caveat ordering against the activation notice, and whether the
   Wired/Refreshed distinction survives for Codex.
5. **Payload contract.** `mcp_invocation()` derivation, and whether
   `RefreshOutcome`'s doc comment (EX-7's required refresh) now describes both
   arms or only Claude's.
6. **Behaviour preservation.** The untouched `plan_mcp_*` suite as the proof;
   confirm no assertion was edited, and that the Claude `.mcp.json` payload and
   its printed form are unchanged.
7. **Test confidence.** Which of these claims is a test *incapable* of failing —
   the probe input above all (the fakes inject the answer, so no test can
   observe what real codex would have said).

Executed: `cargo test --bin doctrine codex` (22 pass), `cargo test --test
e2e_codex_install` (12 pass), `cargo test --test architecture_layering` (25
pass), plus live `boot install --agent codex` runs (dry, real, idempotent,
non-UTF-8, foreign) in scratch roots and a codex 0.155.1 trust experiment.

## Synthesis

**Overall: acceptable** (revision-required on one point of *design*, not of code).

**Synopsis.** SL-271 adds a second MCP registration arm to `doctrine boot install`:
a `toml_edit` narrow-path write of `[mcp_servers.doctrine]` into a project
`.codex/config.toml`, a strict ownership predicate that only ever heals doctrine's
own emitted shapes, a per-harness report seam, and a probe that folds the codex
hooks-feature state into the activation notice. The 12-commit delta is high
quality work: the shared `mcp_action`/`McpEntryClass` table genuinely merges the
two arms' judgement rather than duplicating it, the ownership formula is
implemented with its index guards in the right order (`args.len() == 2` before any
index; `as_table_like`, so an inline parent classifies on content, not spelling),
the write is edit-preserving and byte-stable on a second run, and the Claude
refactor's behaviour preservation is proved by the untouched `plan_mcp_*` suite
plus a payload now derived from `desired_mcp_entry` instead of re-spelled. I
re-ran the evidence rather than trusting the hand-back: the focused unit suite (22
pass), the new e2e crate (12 pass), the layering gate (25 pass), and live installs
in dry, real, idempotent, non-UTF-8 and foreign states.

It stands at one blocker. F-1 is not a coding defect: `codex features list`
reflects the *user* codex layer, because codex applies a project's
`.codex/config.toml` only once that project is trusted — and install runs before
trust, which the notice's own step 2 prescribes. So the probe can suppress step 1
("ensure `[features] hooks = true` in `.codex/config.toml`") for a project that
disables hooks, leaving the operator with silent hooks — the exact failure the
disclosure exists to prevent. The criterion (PHASE-03/EX-4) is met as written, so
the reconcile belongs to DEC-329 / design sec-5.2, whose claim "the probe answers
for the project being installed" the evidence falsifies. Standing risks are the
two nits (F-4's lost wire/refresh distinction for Codex, F-5's undeclared probe
dependency), both cheap and coupled to that same decision, and F-2/F-3, which are
small and in the hot tree.

Consciously accepted tradeoffs, none of which I raised: the portable `sh`-wrapped
entry (a POSIX shell dependency in exchange for no abspath, declared in README);
strictness that heals doctrine's output and nothing else, including a user's own
`env_vars` (a deliberate non-clobber, and the source of the `OwnedStale`
refresh); a `Malformed` file producing the same one-line fallback as a foreign
entry (one wording, two causes, by decision); and `install_mcp`'s pre-existing
blanket `.ok()` left alone as out-of-scope (`ISS-495`). Credit where it is due:
the ownership predicate and the inline-parent handling are the parts of this
delta a reviewer most expects to find broken, and they are not.

**Haiku** — *a shell wraps the bin; / the probe asks codex, not the file / the project has not yet trusted.*
