# No fixture supersede edges on live ADRs

`doctrine supersede NEW OLD` is not a demo op. It flips `OLD.status →
superseded` on a real governance entity. Never run it to manufacture an example
`supersedes` row.

**What happened.** SL-155 G5b noted the corpus had no `[[relation]] label =
"supersedes"` row and ran `doctrine supersede ADR-012 ADR-004` to make one — an
arbitrary, unrelated pair (ADR-012 = dispatch topology; ADR-004 = outbound-only
relations). Collateral: ADR-004, the live relation principle cited ~40× across
the active corpus (SPEC-017, SPEC-018, ADR-010 §5), got stamped `superseded`.
Months later RV-236's inquisition read that false status and raised F-1 against
SPEC-003 for "citing dead authority" — a misfire whose true cause was the fake
edge, not the citation. Reverted under REV-020 (ADR-004 → `accepted`, edge
removed).

**Rule.** A `supersedes` edge must encode a real supersession or not exist. The
governance corpus is not a CLI/storage test fixture — exercise verbs against
throwaway entities or unit tests, never live canon.

**Inquisition corollary.** A `superseded_by` flag is not proof of dead
authority. Before flagging a citation, check the *superseder's* topic and whether
the rest of the active corpus still cites the target as live. See
[[mem.pattern.review.superseded-by-is-adr004-carveout]].
