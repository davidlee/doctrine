# IMP-367: Remove or reach the design-run read surfaces with no reader

`src/design_run/attestation.rs` carried a **module-wide** `#![expect(dead_code)]`
from PHASE-02, on the honest grounds that nothing reached the module yet. PHASE-12
made most of it live. PHASE-10 narrowed the gate to per-item, and narrowing it
surfaced what the blanket had been covering: **seven accessors that no caller and
no test reads at all**, in either the binary or the e2e suites.

- `Attestation::reviewer`
- `AcceptanceAttestation::{authority, basis, turn, digest}`
- `IntegratedReview::id`
- `LockAcceptance::attestation`
- `IntentState::as_str`

Each is a plausible read surface for its type, and none has a consumer. Two of them
are more than idle: `AcceptanceAttestation::basis` is the *auditable* half of
DEC-088 — a lock rests on an agent's attributed claim, and the basis is what makes
the claim auditable — yet nothing reads it back out, so the audit trail is written
and never surfaced. `IntentState::as_str` documents itself as the STD-001 single
source for the token, but the serde rename is what actually puts the token on the
wire, so it is a single source for nothing.

**The disposition is per item, not a sweep.** Either give it a reader (a projection
row, a `show` line, a recovery report) or delete it. What must not happen is a
third phase adding a per-item `expect` beside these and moving on — that is how a
narrowed gate silently becomes a blanket again
(`mem.pattern.lint.dead-code-blanket-masks-siblings`).

PHASE-10 deliberately did **not** delete them: removal inside another phase's work,
with no test to notice the loss, is a change whose blast radius nobody sized. The
expects it added name this item.

Originates from SL-233 PHASE-10 (sheet finding F-4).
