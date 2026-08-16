# REQ-478: Report every recorded mutation on the change log

## Statement

<!-- The sister TOML's `description` field is the primary, normative statement.
     Prose here may elaborate, expand upon, or disambiguate it — never
     duplicate it. -->

Two clauses in the statement do separate work, and separating them is the
point.

**Completeness** is the positive obligation: the change log is a projection of
what the run did, so a mutation it does not report is a hole in the read model,
not a stylistic omission. This is stated over the *emittable* roster rather than
over the whole event vocabulary, because `DEC-239` splits the two — the readable
half exists to keep historical snapshots parsing, and holding it to an emission
obligation would force a live path for vocabulary that is deliberately dead.

**The bound** is the negative one: the row says *this was recorded*, and nothing
more. It is not an acknowledgement that every key in the submission was
understood. Unknown-key absorption is `ISS-333` / `ISS-346` / `ISS-327` /
`ISS-328`, gated on `QUE-219`, and a reader who takes a change row as coverage
of the payload has been misled by the very surface added to stop them being
misled. The clause is in the requirement rather than left to review because that
misreading is the failure mode most likely to be introduced by someone
strengthening the requirement in good faith.

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->

`ISS-355`: a successful `agent_declaration` apply exits 0 and prints only
`revision N stage <stage>`. The output is byte-identical to the output of an
apply whose payload was silently discarded, so the one observable that could
separate *landed* from *dropped* is absent exactly where it is needed. The
guardrails tell agents not to read raw runtime state, which leaves confirmation
with no sanctioned route at all.

The generalisation `DEC-238` reached is the durable part: a derived row can only
report changes in the *key* of the set it differences. The change log differences
material events; a mutation that records itself outside that key is invisible by
construction rather than by oversight. `ISS-367` is the second instance, which is
what moved this from a bug to a requirement.

**Why this sits in `SPEC-029` and not in `STD-003`.** `STD-003` (*no silent
skip*) is the near-miss, and its second prohibition — *no empty success* —
describes the defect almost word for word. It does not reach it: the standard's
statement scopes to "a read of authored corpus data [that] fails or degrades",
and this is emission on a *successful write* path. That boundary is deliberate
and stated twice — "tolerate-and-disclose is a rule for readers", and "a degraded
read on a **write** path refuses". The phrasing overlap is a coincidence of
wording, not of subject: `STD-003` governs a reader that could not see, this
governs a writer that did not say.

Widening `STD-003` was considered and rejected. It is a `required`
cross-cutting standard, so broadening its scope clause would retroactively bind
every write path in the repository to a disclosure obligation nobody has
audited — a large unbudgeted commitment bought to settle one container's defect.
Minting the statement here instead keeps the blast radius honest and puts it
where the next person editing `ChangeEvent` will actually meet it: spec coverage
checks it, rather than review having to remember it.

If the class later proves genuinely cross-cutting — other derived-row surfaces,
the review ledger, dispatch receipts — the right move is a new standard with this
requirement as its first adoption, not a retrofit of `STD-003`. Two instances
inside one container is not yet evidence for a cross-cutting rule; `STD-003`
itself was extracted only after a slice had lived with real ones.
