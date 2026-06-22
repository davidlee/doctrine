# Review RV-141 — design of SL-141

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Round 2 of the SL-141 design Inquisition arraigns the amended design after
RV-140's six verified penances. The first tribunal forced precision on snippet
spans, tier labels, alias ambiguity, pagination edges, tangle impact, and body
serde exclusion. This tribunal asks whether the penance itself bred fresh sin.

**Lines of attack:**

1. **Penance reflection.** Are all RV-140 findings truly incorporated, or merely
   confessed in words?
2. **Tokenizer doctrine.** Does the snippet design reuse `lexical::tokenize` as
   the single lexical authority, or does it spawn a parallel lexer under a monk's
   hood?
3. **Pagination authority.** Does the CLI surface cite and follow the right
   memory/search precedent for default limits, `--limit 0`, `--page`, and
   truncation behaviour?
4. **Catalog ingestion.** Are missing, unreadable, and malformed `.md` body files
   specified clearly enough for `scan_entities` to preserve existing consumers?
5. **Kind and output UX.** Are aliases, status exposure, table/JSON flags, and
   help tests complete enough to prevent a first-release UX auto-da-fé?
6. **ADR-001 ratchet.** Does the command-tier tangle analysis remain explicit and
   implementable under the 120-edge ceiling?

**Held doctrine:** ADR-001 layering, the `lexical` leaf contract, catalog scan
behaviour preservation, existing `listing`/memory pagination conventions, and the
curated RV-141 `domain_map.toml` invariants.

## Synthesis

### Verdict: HERESY REMAINS; THE PENANCE BRED NEW OFFSPRING

RV-140's six wounds were largely cauterised. The amended SL-141 design now
names the tangle ratchet, corrects the tier labels, uses the plural `specs`
alias, adds pagination boundary tests, sketches the snippet span pass, and pins
body serde exclusion. The old devils were not ignored.

But the new words harbor four fresh taints. Two are major: the snippet design
creates a parallel tokenizer by reciting `lexical::tokenize`'s split rule in
prose, and the pagination section misquotes memory-find/retrieve precedent by
turning a retrieve maximum into a search default while blessing `--limit 0` as
unlimited despite memory-find rejecting it. These are not aesthetic blemishes;
they are coupling and CLI-contract sins fit for the wheel.

### Ordered Penance

1. **F-1 (major): unify tokenizer machinery.** Amend the design so snippet spans
   come from shared lexical code (`tokenize_with_spans` or a shared iterator),
   not a duplicate char-scan algorithm. Add equivalence and Unicode-boundary
   tests.
2. **F-2 (major): settle pagination truth.** Pick memory-find, memory-retrieve,
   or explicit search-specific constants. Remove the false citation to
   `RETRIEVE_LIMIT_MAX` as a default and align the `--limit 0` tests with the
   chosen rule.
3. **F-3 (minor): specify body-read errors.** Missing files are not the whole
   tribunal. Choose and test the policy for unreadable or invalid UTF-8 `.md`
   bodies.
4. **F-4 (nit): pin CLI help and `--json` precedence.** Add help/parse coverage
   and state whether `--json --format table` follows the existing `--json wins`
   convention.

### Standing Risks

The template-noise risk remains tolerated but real. Status filtering is still a
follow-up, so first-use searches for `scope`, `context`, and similar liturgical
boilerplate will summon the unquiet dead from every template. This is allowed for
v1 only because the design names it; the Inquisition does not bless it.

RV-141 remains active and awaits the responder. Let no one mark the design clean
until the charges have been answered and the corrected text dragged into the sun.

**HERESIS URITOR; DOCTRINA MANET**
