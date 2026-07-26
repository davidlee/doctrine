# REV REV-034 — Verify attests against a claim-relevant clean tree, not a globally clean one

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SPEC-007 asserts that `verify` attests **against a clean working tree, refusing a
dirty one**. That contract has been partly false since `--allow-dirty` shipped
(2026-06-18/21), and SL-230 makes it fully false: the gate becomes *claim-relevant*
cleanliness rather than global cleanliness.

The substantive change is not a loosening. SL-230 refuses on *more* of what
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
inventory omitted REQ-147). This revision is the discharge of both. `SL-230 needs
REV-034`.

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

## Application

Applied at SL-230 close, after the implementation lands — the spec and the code
must agree at the same moment, not before. Verified by RV-307's closure criteria
(§ 9 of `design.md`).
