# A shipped delivery copy cannot cite its governance owner

When a rule is *owned* by repo-local governance and *delivered* through a shipped
asset (a published reference doc), the two ends cannot be linked the obvious way:

- the governance record (an ADR) **may** cite the published address — a published
  `reference/<name>.md` is a client-resolvable form, and the record is repo-local
  in any case;
- the shipped doc **cannot** cite the ADR. A repo-private entity id in shipped
  text does not dangle in a client repo — the client has minted its own record at
  the same number, so the citation silently resolves to *their* unrelated record.

So the linkage is one-way by construction: governance → published address. The
shipped copy must describe its owner in prose ("durable governance, not itself
shipped") without naming it, and the "single owner" claim is carried by content
identity, not by a hyperlink.

**The rule's own delivery copy is the highest-risk site for violating it.** Hit
live in SL-267 PHASE-01: the first draft of `install/shipped-corpus-authoring.md`
cited `ADR-024` twice — in its header comment and in a "where this is recorded"
section. The phase's own grep-with-positive-control check caught it. Expect this
shape: a doc whose subject is citation discipline wants to cite its own authority.

The grounding rule itself is owned by ADR-024 (shipped-corpus grounding: any
address a client can resolve, never a repo-private id or path). This memory
records only the delivery-copy linkage consequence, not the rule.
