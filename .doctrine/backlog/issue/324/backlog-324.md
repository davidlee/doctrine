# ISS-324: coverage literal-command entries bypass the matcher requirement

`coverage::valid` (`src/coverage.rs`) requires a non-empty matcher **unless a
literal command is set**. A literal command that exits 0 having selected zero
tests therefore lands `Verified`, flagged only as `[exit-code-only]` in the
report for a human to notice. That is a live false-green path.

Removing the exemption makes the matcher unconditional across every mode: the
author must state what success looks like in the runner's output, and the
existing `MatcherRequired` refusal does the work. No schema change, no new
outcome variant, no language knowledge in core.

**Why it matters beyond the hole itself.** RFC-027's `H13` (Doctrine addresses
proof subjects without understanding source languages) survives *only* if the
matcher is unconditional. The alternative way to tell "ran zero tests" from
"ran and passed" is parsing runner output — which `src/regression.rs` INV-5
does, and which is cargo-bound, i.e. exactly what `P8` forbids in core. A
mandatory matcher pushes that knowledge out to project-authored config, where
`REQ-257` already says it belongs.

**Scope.** Bounded by a corpus query — how many existing coverage entries rely
on the exemption. That is a count, not a design question.

## Corpus query — measured 2026-08-08

**Zero entries rely on the exemption.** The removal is a clean break: no
migration, no grandfathering, no authored-corpus churn.

Across all 16 `.doctrine/slice/*/coverage.toml` stores (90 entries): 85 are
`VA`/`VH` attestations with no `check` at all, and **5** are `VT`-check
recipes — all five in `.doctrine/slice/228/coverage.toml`
(`REQ-384`, `REQ-385`, `REQ-386`, `REQ-388`, `REQ-389`), all literal-command,
and **all five already carry a matcher**. Four use the pattern
`[1-9][0-9]* passed; 0 failed` — which is precisely the zero-tests-selected
guard the exemption's absence would otherwise leave open, authored by hand.
Not one alias-based or project-default-base check exists in the corpus.
Positive control: a raw `grep` over the same stores returns exactly the same
five `command` keys and five `pattern` keys.

**Reachability.** Not merely a schema quirk — `doctrine coverage record
--command …` with no `--matcher-pattern` is a live authoring path, and
`coverage_store.rs:128` runs `coverage::valid` before any write, so removing
the exemption closes the path at the write seam.

### A quieter sibling hole, found by the same query

`valid` treats a matcher as empty when it is absent **or** its pattern is `""`
(`src/coverage.rs:541`–`549`), and the exemption lets both through alongside a
literal command. But `coverage_verify.rs:184` computes the report flag as
`command.is_some() && matcher.is_none()` — pattern-only. So an entry with a
literal command and `matcher = { pattern = "" }`:

- passes validation,
- always matches (`evaluate_matcher("", …) == Some(true)`, pinned at
  `coverage.rs:1254`), so it lands `Verified` unconditionally,
- and is **not** flagged `[exit-code-only]` in the report.

That is strictly worse than the path this issue names, because the human tell
that justifies the exemption is absent. No corpus entry exhibits it. Making the
matcher unconditional closes both, since `matcher_empty` already covers the
empty pattern — the fix is deleting the `&& check.command.is_none()` guard, not
adding a case.

### Blast radius

- `src/coverage.rs` — the guard at 547; test
  `valid_accepts_empty_matcher_with_literal_command` (1304) inverts to a
  rejection test.
- `src/coverage_verify.rs` — the whole `exit_code_only` apparatus (field 60,
  `exit_code_only_count` 92, derivation 184, `[exit-code-only]` flag 273, the
  audit line 299, test 997) becomes unreachable. Decide deliberately whether it
  is deleted or kept as a defence-in-depth reporting leg for pre-existing
  entries — with zero such entries, deletion is the honest option.
- `src/coverage_store.rs:710` — a test comment states the alias case; extend to
  the command case.

**Sequencing: none.** This one stands alone — the study names it the smallest
next step in either brief and recommends it as a backlog item rather than a
slice. It does not wait on CHR-058; the SPEC-002 amendment gives it a
conformance home afterwards.

Evidence: `.doctrine/rfc/027/proof-binding-study/conclusion.md` § R2; RFC-027
Stage 4.
