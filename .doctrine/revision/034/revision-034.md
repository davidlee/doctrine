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

*Note: the slug still reads `verify-attests-against-a-claim-relevant-clean-tree…`.
The id is identity and the slug is never authoritative (STD-002), so it is left
rather than renamed.*

## Rationale

SPEC-007 asserts that `verify` attests **against a clean working tree, refusing a
dirty one**. That contract has been partly false since `--allow-dirty` shipped
(2026-06-18/21), and SL-232 makes it fully false: the gate becomes *claim-relevant*
cleanliness rather than global cleanliness.

The substantive change is not a loosening. SL-232 refuses on *more* of what
matters and *less* of what does not:

- dirt anywhere in doctrine's own authored corpus that the memory does not claim
  against no longer blocks — it was never evidence about the claim;
- the memory's **own item directory** and its **declared scopes** must now be
  committed, which today they need not be. A memory can currently be stamped
  `verified_sha = HEAD` while its body is untracked and HEAD demonstrably does not
  contain it (RV-307 F-1, proven empirically).

So the honest contract is: *verify attests against a tree that is clean in the
claim's own evidence surface, refusing when that surface is dirty.*

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
> Attest a memory with `verify` by stamping its verification axis against a tree
> whose claim-relevant surface — the memory's own item directory and its declared
> scopes — is committed, refusing when that surface is dirty so no false
> attestation is recorded

REQ-147's `.md` body is currently an unfilled template (statement and rationale
both empty). The revision fills both, since the title alone has been carrying the
whole contract.

### SPEC-007 — `modify`

Two sites, both stating the retired contract:

> **before** (`spec-007.toml:22`, capability line)
>
> Carry the `verify` attestation verb (stamp the verification axis against a clean
> working tree, refusing a dirty one) …

> **after**
>
> Carry the `verify` attestation verb (stamp the verification axis against a tree
> whose claim-relevant surface is committed, refusing when it is dirty) …

> **before** (`spec-007.md:132-133`, § `verify` and the global/derived orientation class)
>
> `verify` attests a memory against the current working tree, stamping the
> verification axis; it refuses a dirty tree so no false attestation is recorded.

> **after**
>
> `verify` attests a memory against the commit its claim-relevant surface resolves
> to — the memory's own item directory plus its declared scopes — stamping the
> verification axis; it refuses when that surface is dirty, so no false attestation
> is recorded. Dirt outside the claim's evidence surface is not evidence about the
> claim and does not block. `--allow-dirty` remains the explicit escape hatch,
> stamping `checkout_state_id` instead of a commit.

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
§ Scope for why they are not split. Verified by SL-232's closure criteria
(§ 9 of `design.md`), which require REV-034 applied so SPEC-007, REQ-146, REQ-147,
REQ-155 and the implementation agree.
