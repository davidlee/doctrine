# Host-source gates must match raw, not strip language syntax (POL-002)

A platform gate that inspects **host-project source** to decide Pass/Fail must
NOT bake in language syntax — comment markers (`//` Rust, `#` Python, `--` SQL),
string-literal delimiters, block-comment forms. Doing so load-bears the gate's
*correctness* on the host project's language, which **POL-002 facet-1 forbids**
("conventions may inform a default; they must never carry correctness").
doctrine governs arbitrary repos.

## The trap (SL-170 PHASE-03, codex F-3 walked back)

The S3 VT gate (`src/vtgate.rs`) originally comment-stripped + string-stripped
`test_file` before matching `keywords`, to defeat "a keyword hidden in a comment
/ dead string." Two faults:

1. **POL-002 heresy.** The stripper knew Rust/C comment + string syntax. Wrong
   for `.py`/`.rb`/`.go`/`.sql`. A gate's verdict cannot depend on host language.
2. **Zero-false-fail breach.** Real e2e tests reference CLI tokens as STRING
   literals (`cmd.arg("check")`) and output assertions as `stdout.contains("…")`.
   String-stripping nuked exactly those → FALSE-FAILED already-completed work
   (PHASE-02's own VT-7/9). The dominant e2e keyword case IS the string literal.

## The fix — raw substring + author-owned regex escalation

- Match `keywords` as plain **raw substring** over the unmodified file. No
  stripping. Language-agnostic, POL-002-clean.
- Offer an optional `patterns` (line-anchored **regex**) escalation for an author
  who wants a code shape — the regex is the author's, not a baked-in lexer, so it
  stays host-independent.
- **Threat-model honesty:** raw substring is the proportionate floor for the
  *omission* threat (omitted work ⇒ keyword absent entirely ⇒ Fail). Comment /
  dead-string *bait* is an ADVERSARIAL move — out of scope (if the worker is
  adversarial the dispatch trust model fails upstream, ADR-012). Don't defend a
  threat your design declares out of scope at the cost of POL-002 + false-fails.
- **Accepted weakness (document it):** a keyword present only in a comment now
  satisfies. Tolerable — still catches genuine omission; bait is out of scope.

## How to apply

Designing any gate/check that greps host source: ask POL-002's question — "does
this depend on a convention a different host would not share?" Comment/string
syntax is such a convention. Match raw; if you need shape, take a regex the
author supplies. Surfaced by DOGFOODING the gate against its own slice plan
(design §9) — the gate judged its own `plan.toml` and exposed the F-3 fault
before close. See POL-002 and
[[mem.pattern.design.product-not-compromised-by-project-local-ops]].
