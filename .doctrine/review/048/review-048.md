# Review RV-048 — reconciliation of SL-069

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Audit of SL-069's shipped memory corpus against its design.md, with two
specific lenses:

1. **Heresy detection** — project-local conventions (build tools, repo-specific
   paths, doctrine-internal jargon) that leaked into the universal orientation
   corpus.
2. **Gap analysis** — capability areas that remain uncovered after the 13 new
   memories.

**Invariants held:** every shipped memory carries ADR-002 signature
(`repo=""`, `anchor_kind="none"`); new memories use `signpost` or `concept`
type per design D7; cross-references form a connected graph; CLI verb map and
file map are current; boot snapshot renders signpost-only per design D2; boot
--check warns on empty governance per design D5.

**Evidence:** read all 27 shipped `.md` bodies, checked all 27 `.toml` headers
for ADR-002 conformance, verified boot snapshot behaviour (signpost filter +
governance warning), confirmed CLI verb list and file map freshness.

## Synthesis

### Conformance summary

All 13 new memories carry correct ADR-002 signatures (`repo=""`, `anchor_kind="none"`),
correct types (2 concept, 11 signpost per D7), and appropriate cross-references
forming a connected graph. The 14 refreshed existing memories are substantively
current — CLI verb map covers all post-SL-018 verbs, file map includes all new
directories, lifecycle-start includes the reconcile phase, skill-map paths are
corrected from `plugins/` to `.doctrine/`.

Boot snapshot changes work correctly: signpost-only Memory section filter (D2),
governance empty-section nudge comment (D5), and `boot --check` governance
warning. All verified by existing tests + new VT tests.

### Findings

**F-1 (major): Project-local build tool leakage.** `mem.pattern.doctrine.conventions`
and `mem.pattern.doctrine.tdd-loop` contain `just check`, `cargo test`, and
`cargo clippy` references — build-repo-specific tool choices. Non-Rust client
projects get misleading tooling advice. The conventions memory points at the
project's own `CLAUDE.md` for the authoritative rules, partially mitigating, but
the shipped text bakes in assumptions. These two memories should be rewritten to
describe process conventions in tool-agnostic terms (e.g. "lint clean" not
"zero clippy warnings", "tests pass" not "cargo test clean").

**F-2 (minor): Knowledge records gap.** `doctrine knowledge` exists as a CLI verb
and `.doctrine/knowledge/nnn/` as a directory, but no signpost or concept memory
introduces the epistemic record kinds (assumption/decision/question/constraint) or
when to use them vs ADR vs memory. This was Tier 3 gap #13 in research.md —
explicitly excluded from the 13 new memories. Remains a real discoverability gap.

**F-3 (minor): Review (RV kind) signpost deferred — dependency resolved.** SL-068
is now done, so the deferral condition has resolved. The audit signpost mentions
the RV kind, but a dedicated signpost covering review ledger mechanics (baton,
raise/dispose/verify/contest/withdraw) is still absent.

### Standing risks

- **Self-updating shipped memories (OQ-1)** remains unsolved. The corpus will
drift again across future slices. Design records this as an open question with
candidate mechanisms; no decision was made in this slice.
- **Review (RV kind) signpost** is a follow-up item — capture as backlog.

### Tradeoffs accepted

- Knowledge records gap (F-2) was a conscious Tier 3 deferral — acceptable for
  the slice's scope.
- Build tool leakage (F-1) exists because the conventions were refreshed
  mechanically rather than rewritten for audience. The `CLAUDE.md` pointer
  mitigates but doesn't eliminate the issue.
