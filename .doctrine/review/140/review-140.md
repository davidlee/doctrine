# Review RV-140 — design of SL-141

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This Inquisition probes the SL-141 design document and scope for doctrinal purity
under ADR-001 (layering), the lexical contract (SL-017), the catalog scan contract
(SL-071), and the entity engine path invariants.

**Lines of attack:**

1. **ADR-001 layering.** Does search.rs cross a tier boundary upward? Are its
dependencies catalog (engine) + lexical (leaf) only?

2. **Lexical contract.** Does entity_lex_doc produce a valid LexDoc matching the
retrieve::lex_doc shape? Is BM25 corpus construction correct (full corpus, not
query-hits only)? Is tokenize the one lexer?

3. **Catalog scan change.** Does ScanMode gate body reading correctly? Do all
existing call sites pass include_bodies: false (zero behaviour change)?

4. **Kind selector.** Is the resolution pipeline unambiguous (expand aliases →
validate)? Are error cases specified (unknown prefix, empty set)?

5. **CLI design.** Are --kind / --with / --no interactions defined for all
combinations? Is --short / --context conflict enforced? Is zero-score suppression
specified?

6. **Output & snippet.** Is the snippet extraction algorithm specified precisely
(cased text, span identification)? Are edge cases covered (empty body, no match)?

7. **Verification coverage.** Do the listed tests cover the critical paths? Are
missing test cases acknowledged?

8. **Risks.** Are the three risks adequately mitigated, or does the design
hand-wave past real threats (template noise, memory pressure, snippet quality)?

## Synthesis

### Verdict: CONDITIONAL PASS WITH PENANCE

The SL-141 design survives its inquisition — but not unscathed. Six wounds
were opened; six were cauterised. The design is doctrinally sound at the
architectural level: the search module sits correctly in the command tier,
dependencies flow downward or within-tier, the lexical contract is honoured,
ScanMode gates I/O faithfully, and the BM25 corpus construction mirrors the
proven `retrieve` pattern. No upward edges. No cycles introduced that cannot
be accounted for. The ".md body → LexDoc → BM25" pipeline is the right reuse
of existing seams.

Yet the design was found wanting in precision where precision is non-negotiable.

### Confessed Sins and Ordered Penance

1. **F-1 (MAJOR): The snippet span-reconstruction algorithm was a hand-wave.**
   "Map the matched token back to its span in the original cased body text" is
   not an algorithm — it is a prayer. The tokenizer is destructive (lowercase,
   split on non-alnum). Reconstructing byte offsets from its output requires a
   parallel span-tracking pass. **Penance:** Amend the design with a concrete
   two-pass algorithm: tokenize for matching, then re-scan for span offsets.
   Test: exact match → correct span; multi-match → first; no match → fallback.

2. **F-5 (MAJOR): The command-tier tangle ratchet was ignored.**
   ADR-001 freezes the command-tier cyclic-edge count at 120. `search.rs` adds
   three new command→command edges (`search → catalog::scan`, `search →
   integrity`, `cli → search`). The design breezily asserts "just gate green"
   without acknowledging the ratchet. **Penance:** Add a Tangle Impact section
   to the design. Show the 3-edge delta. Either reduce command→command edges
   elsewhere beforehand, or document the baseline increase and secure it in
   the implementation PR with a `[tangle_baseline]` bump.

3. **F-2 (MINOR): Dependency tier labels were wrong.** `catalog::scan` is
   command, not engine. `integrity` is command, not engine. `listing` is leaf,
   not engine. The "No upward edges" claim holds — this is a documentation
   sin, not a structural one. **Penance:** Correct the Architecture section.

4. **F-3 (MINOR): The `spec` alias collided with the SPEC prefix.** A user
   writing `--kind spec` meant tech specs; the resolver expanded it to
   [prd, spec]. Ambiguity is a UX sin. **Penance:** Rename the alias to
   `specs` (plural). One token. Zero ambiguity.

5. **F-4 (MINOR): Page-boundary edge cases were left to the void.**
   No tests for `--page 0`, `--page` beyond results, or `--page` without
   `--limit`. **Penance:** Specify the default limit; add edge tests.

6. **F-6 (MINOR): The `body` serde skip was unpinned.** The design promises
   body exclusion from catalog JSON but guards it with no test. A serde
   attribute drift could silently include large body text. **Penance:** Add
   a serialization-exclusion test.

### Standing Risks

- **RSK-003 (template noise):** Still standing, only noted as follow-up.
  The first release WILL pollute BM25 IDF with boilerplate tokens from
  every entity's template skeleton. "Scope" and "objectives" will match
  every entity. Tolerated for v1 but the inquisition notes it.

- **Memory pressure (RSK-002):** Tolerated. The corpus IS small. But the
  `body: Option<String>` field on `ScannedEntity` adds ~25 bytes struct
  overhead even when `include_bodies: false`. Structural taint, not
  allocation taint. Revisit at 10x scale.

### Tolerated Taint

- The `--rfc` kind is in the default set alongside ADRs and specs. RFC
  bodies carry discussion prose — potentially noisy, potentially useful.
  The design makes the call; the inquisition does not second-guess it.

- No status filtering on results. Status is display-only. Follow-up
  territory. Tolerated.

- The snippet quality gap (naive token-window, no sentence boundaries) is
  acknowledged. Tolerated for v1.

### On What Survived

The design's architecture is honest. The pure/imperative split is clean:
`KindSelector::resolve`, `entity_lex_doc`, `snippet` are pure; `build_corpus`
and `run` are the thin impure shell. The BM25 fit-over-full-corpus pattern
is correct — matching `retrieve`'s proven design. The `ScanMode` gate is
the right seam. The CLI is clap-idomatic with proper conflict enforcement.

The penitent must complete their penance before the PR lands. The
inquisition grants conditional passage.

**HERESIS URITOR; DOCTRINA MANET**
