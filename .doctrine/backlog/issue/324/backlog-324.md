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

**Sequencing: none.** This one stands alone — the study names it the smallest
next step in either brief and recommends it as a backlog item rather than a
slice. It does not wait on CHR-058; the SPEC-002 amendment gives it a
conformance home afterwards.

Evidence: `.doctrine/rfc/027/proof-binding-study/conclusion.md` § R2; RFC-027
Stage 4.
