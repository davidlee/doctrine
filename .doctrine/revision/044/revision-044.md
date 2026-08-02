# REV REV-044 — Materialise carries a same-file CAS guarantee

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

RV-324 F-2 (blocker) established that `materialise` can erase a post-check human
edit to `design.md` and then certify its own replacement. SPEC-029's watermark
responsibility is one of the two authored surfaces that made that lawful, and it
is the surface a Revision can reach: DEC-092 is a knowledge record and therefore
not a `revises` target (it is handled by a superseding decision record instead).

### What is wrong with the authored text

The responsibility says the watermark is "re-checked immediately before every
**snapshot** write". Two of the three re-check sites match that wording
(`design.rs:988`, `:1223` — verbs whose write is a runtime-snapshot write). The
third does not: `materialise` re-checks (`design.rs:1382`), then writes the
**authored document** (`crate::entity::write_body(.., BodyMode::Replace)`), and
only then writes the snapshot. So the authored write sits *inside* the window the
responsibility describes as ending at the re-check, and the spec's own sentence
does not govern it.

The consequence is worse than a wider window. `materialise` then re-baselines the
watermark from the body it just rendered (`design.rs:1406`), so the next entry
check compares Doctrine's own output against Doctrine's own fingerprint, finds
alignment, and reports nothing. The clause "a divergence abandons the write
without advancing the run" is unreachable on this path, because by the time the
next verb could detect the divergence the watermark has already absorbed it.

### Why the stronger guarantee is the right correction, not a wider caveat

DEC-092's rationale derived the weaker, delayed-detection guarantee from an
explicit premise: `src/review.rs::with_turn` "holds a writer lock and hashes the
very file it atomically replaces; a design run hashes `design.md` while writing a
runtime snapshot, a checkpoint journal, and possibly an authored record."

That premise is true for every run-advancing verb **except** `materialise`, which
hashes and replaces the same file. So the asymmetry that justified the weaker
wording does not hold for the one verb that destroys authored bytes, and the
stronger mechanism is not hypothetical — `with_turn_hooked` (`src/review.rs:2056`)
already ships it in this codebase: `LockGuard` acquisition, an entry
compare-and-swap, and a `mid_turn` seam its own comment names "the pre-write CAS
test seam". `fsutil::write_atomic` performs no compare-and-swap of its own
(`src/fsutil.rs:52-75`), and `entity::write_body` skips only the byte-identical
case, so nothing below the seam supplies the guarantee either.

Tiering the responsibility by *what is being written* — rather than weakening it
uniformly or claiming a guarantee the snapshot path cannot honour — keeps both
halves honest and matches the mechanism actually available at each site.

### Before / after

**Before** (`spec-029.toml`, `responsibilities[2]`):

> Own the authored-design watermark — the fingerprint of the design document the
> run last agreed with — checked at entry by run-advancing verbs and re-checked
> immediately before every snapshot write, where a divergence abandons the write
> without advancing the run while journalled effects remain and stay recoverable.

**After:**

> Own the authored-design watermark — the fingerprint of the design document the
> run last agreed with — checked at entry by run-advancing verbs and re-checked
> immediately before every write it guards, where a divergence abandons the write
> without advancing the run while journalled effects remain and stay recoverable.
> The strength of that guard is tiered by what is being written: a
> runtime-snapshot write carries delayed detection, because the file hashed is not
> the file replaced; the authored-document write `materialise` performs carries a
> same-file writer lock and compare-and-swap, because there the bytes fingerprinted
> and the bytes replaced are the same file, and a re-baseline that certified an
> overwritten hand-edit would make the divergence permanently undetectable.

### Scope

One responsibility sentence on SPEC-029. No REQ member changes — SPEC-029's
eleven requirements (REQ-428…REQ-438) are `pending` skeletons with no prose, so
the responsibility list is the governing text. No ADR is touched: ADR-001's
layering and the pure/shell split are unaffected, and the fix rides an existing
`src/review.rs` mechanism rather than introducing a tier.

The code change this authorises is carried by SL-233 PHASE-15 alongside the rest
of RV-324's remediation, gated on this revision being approved.
