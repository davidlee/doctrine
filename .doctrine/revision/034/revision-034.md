# REV REV-034 — SPEC-007 tells the truth about the memory verification surface after SL-232

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Scope — one revision, one turnover

**This revision widened.** It was opened against `verify`'s clean-tree contract
alone, for SL-230. Two things changed it: **DEC-027** split the gate out into
**SL-232** (the `needs` edge moved with it — SL-230's is now empty), and **RV-314
F-3** established that SL-232's objectives 4 and 7 falsify two further SPEC-007
requirements that no revision covered.

Its subject is therefore no longer one contract but one **turnover**: *SPEC-007
agrees with the memory verification surface at the moment SL-232's code lands.*
Four rows, one `apply`.

The alternative — a second revision for the two new requirements — was rejected on
**atomicity**. All four touched sites go false at the same instant, when SL-232
lands. ADR-013 makes `revision apply` the "forcing-function" tying recorded
approval to the truth-write, so splitting the rows across two revisions guarantees
two applies for one landing and opens a window in which SPEC-007 asserts a mix of
retired and current contracts — the queried-surface trap of RV-307 F-39, which is
the very failure REQ-147's row exists to close. ADR-013's model also favours
accumulation directly: a Revision is "born as content-light pending intent" and
"accumulates staged deltas as it is worked", giving dependents "a crisp single
anchor". Two anchors for one slice's governance dependency is the shape it was
designed to avoid.

Recorded as **DEC-076**; discharges **RV-314 F-3**.

**Re-drafted 2026-10-07 under DEC-371** (`RFC-035`). The four rows are unchanged;
their *after* text is rewritten. The parked SL-232 design measured
*claim-relevant cleanliness* through git's view of the tree; DEC-371 replaces that
with a claim-shaped stamp — the claim set's bytes, mode and set compared directly
against a commit — drops the rest-of-tree anchor, and deletes `--allow-dirty`.

*Note: the slug still reads `verify-attests-against-a-claim-relevant-clean-tree…`.
The id is identity and the slug is never authoritative (STD-002), so it is left
rather than renamed.*

## Rationale

SPEC-007 asserts that `verify` attests **against a clean working tree, refusing a
dirty one**. That contract has been partly false since `--allow-dirty` shipped
(2026-06-18/21) — today 62 of 110 attested memories carry a `checkout_state_id`
in `verified_sha`, not a commit — and SL-232 makes it fully false.

The substantive change is not a loosening. SL-232 refuses on *more* of what
matters and *less* of what does not:

- dirt the memory does not claim no longer blocks — it was never evidence about
  the claim;
- the **claim set** must now match the stamped commit, which today it need not.
  A memory can currently be stamped `verified_sha = HEAD` while its body is
  untracked and HEAD demonstrably does not contain it (RV-307 F-1, proven
  empirically).

The claim set is the memory's declared scope (`scope.paths` plus the files its
`scope.globs` expand to) plus its own `memory.md` — exactly the path set the
staleness seam later counts commits on (REQ-146), so the stamp never claims more
than verify measured. `memory.toml` is excluded: verify writes the stamp into it.

So the honest contract is: *verify stamps commit X only when the claim set's
bytes, mode and membership match X, refusing otherwise; `verified_sha` is always
a commit.*

Raised as RV-307 F-4 (ADR-013 dependency uninstantiated) and F-5 (the amendment
inventory omitted REQ-147). This revision is the discharge of both — and of
**RV-314 F-3**. `SL-232 needs REV-034` (moved from SL-230 by DEC-027).

## Change rows

### REQ-147 — `modify` (primary)

The requirement's **title is itself the old contract, verbatim**, so a spec-only
edit would leave an active member of SPEC-007 asserting the opposite of the
implementation.

> **before** (`requirement-147.toml:6`)
>
> Attest a memory with `verify` by stamping its verification axis against a clean
> working tree, refusing a dirty tree so no false attestation is recorded

> **after**
>
> Attest a memory with `verify` by stamping its verification axis with a commit
> whose blobs match the memory's claim set — its declared scope paths and globs
> plus its own body — in bytes, mode and membership, refusing when they diverge so
> no false attestation is recorded

REQ-147's `.md` body is currently an unfilled template (statement and rationale
both empty). The revision fills both, since the title alone has been carrying the
whole contract.

### SPEC-007 — `modify`

Two sites, both stating the retired contract:

> **before** (`spec-007.toml:23`, capability line)
>
> Carry the `verify` attestation verb (stamp the verification axis against a clean
> working tree, refusing a dirty one) …

> **after**
>
> Carry the `verify` attestation verb (stamp the verification axis with a commit
> whose blobs match the memory's claim set, refusing when they diverge) …

> **before** (`spec-007.md:178-179`, § `verify` and the global/derived orientation class)
>
> `verify` attests a memory against the current working tree, stamping the
> verification axis; it refuses a dirty tree so no false attestation is recorded.

> **after**
>
> `verify` attests a memory by comparing its **claim set** — the declared scope
> (`scope.paths` plus the files `scope.globs` expand to) and its own `memory.md` —
> against a commit: bytes, mode bit and membership. It stamps that commit's sha as
> `verified_sha` only when all three match, and refuses otherwise, so no false
> attestation is recorded. `verified_sha` is always a commit. The claim set is the
> same path set the staleness seam counts commits on, so the stamp never claims
> more than was measured. Dirt outside the claim set is not evidence about the
> claim and does not block; there is no escape hatch.

### REQ-146 — `modify`

Added by RV-314 F-3. SL-232 objective 4 limb (a) widens the historical staleness
seam from `scope.paths` to `paths ∪ globs` in **both** consumers
(`memory validate` Check 2 and `retrieve::git_facts`, which gate on the same
predicate — invariant I11). The requirement's title names the narrow seam, so it
becomes false the moment that widening lands. Measured: **13 of 43**
scoped-and-attested memories are glob-only and are therefore ranked on a 30-day
calendar instead of by commits touching their evidence (QUE-175, answered `yes` on
measurement).

Both tiers carry "scoped paths"; the title is the queried surface.

Under DEC-371 this widening is also what the claim set must match: the seam and
the claim cover one path set, or a glob-scoped memory's staleness would count from
a commit whose glob-matched bytes verify never compared.

> **before** (`requirement-146.toml:6`)
>
> Compute git-anchored staleness in four explicit modes — scoped+attested by
> commits touching **scoped paths** since verified_sha, scoped-unattested and
> unscoped by days-since-reviewed, and the global/derived class evergreen and
> decay-exempt

> **after**
>
> Compute git-anchored staleness in four explicit modes — scoped+attested by
> commits touching **its declared scope entries (paths and globs alike)** since
> verified_sha, scoped-unattested and unscoped by days-since-reviewed, and the
> global/derived class evergreen and decay-exempt

### REQ-155 — `modify`

Added by RV-314 F-3. REV-041 (approved, done) established that the five-state
resolution is the **render contract**, binding `find`/`retrieve`, and that the
prohibition on silent over-trust is **surface-independent** — a surface emitting
findings "discharges this by emitting a finding, not by falling silent."

SL-232 objective 7 makes `memory validate` exactly such a surface: it absorbs
ISS-257 and gives the two silently-swallowed `None` cases an explicit
undeterminable finding. But REQ-155's title offers only the five *render* states,
so a conformant findings surface has no vocabulary in the requirement it is
conformant to. The title must admit the second discharge mode REV-041 already
recognised in the body.

> **before** (`requirement-155.toml:6`)
>
> Resolve every undecidable git-reachability case to an explicit
> fresh/stale/unknown/unanchored/reference state, never a silent hide or silent
> over-trust

> **after**
>
> Resolve every undecidable git-reachability case explicitly — a rendering surface
> to one of the fresh/stale/unknown/unanchored/reference states, a findings surface
> to an emitted finding — never a silent hide or silent over-trust

## Application

Applied at **SL-232** close, after the implementation lands — the spec and the code
must agree at the same moment, not before. All four rows apply together; see
§ Scope for why they are not split. Verified by SL-232's closure criteria, which
must require REV-034 applied so SPEC-007, REQ-146, REQ-147, REQ-155 and the
implementation agree (the redesign under DEC-371 re-authors those criteria).
