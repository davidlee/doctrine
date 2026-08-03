# DEC-120: Condition satisfaction is derived, attested, or claimed

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The defect this replaces

`Condition::is_derived()` is a boolean answer to a three-valued question, and the
proof is in the vocabulary it partitions. `user-acceptance-attested` is *derived*;
`user-accepts-sufficiency` is *claimed*. Nothing about the two acts differs — in
both a human says yes and an agent reports it. What differs is only that
`reviewing` built `ReviewStanding` to hold the answer and `exploring` did not.

So the boolean does not name a property of the condition. It names whether the
engine happens to check it yet — which is why applying `DEC-102`'s seal criterion
condition by condition reproduces `is_derived()` exactly, and explains nothing
(SL-244 research, cross-thread finding 1).

## The three kinds

**Derived** — the engine computes the answer from framework-owned run state. No
one may claim it; a payload alleging one is already refused
(`Refusal::DerivedConditionClaimed`). Liveness is not a concept: the answer is
recomputed, so it cannot go stale. `required-sections-exist`,
`materialisation-current`.

**Attested** — a human act the engine records and binds but must never compute.
The engine's job is custody, not inference: it holds what was attested, binds it
to the content it was given about, and reports when that binding breaks. Liveness
is the whole mechanism. Both acceptance conditions.

**Claimed** — an agent asserts a fact the engine neither computes nor binds to a
human. This is the defect class, not a tier: `DerivedDesignFacts::satisfies` is
existential over evidence rows, so a claim is satisfied by *someone having said
so* about *some* subject whose bytes have not moved. `DEC-066`'s fingerprint
binding makes such a claim **expire**; it never makes one **true** (`ISS-285`).

The decision is that Claimed has no legitimate surviving members in this
vocabulary. Every condition resolves to Derived or Attested, or it retires. Where
a member cannot, that is a finding about the member, not a licence for the tier.

> **Sharpened 2026-08-03** — *Attested names the provenance of an input, not a
> second way of deciding.*
>
> The three kinds above read as three evaluation mechanisms. They are not. **Every
> condition is derived**; what varies is whether the state it derives from can be
> authored by the engine or only by a human act.
>
> - *Derived over run state* — every input is engine-authored.
>   `materialisation-current`, `required-sections-exist`.
> - *Derived over an attested artefact* — some input can only be put there by a
>   human, and the engine derives over it once it is there.
> - *Claimed* — there is no artefact to derive over, only an existential scan of
>   evidence rows. Still the defect class, and now visibly so: it is the case
>   where nothing was recorded to derive *from*.
>
> **The incumbent already does this.** `ReviewStanding::acceptance_current` is
> documented as *"a user acceptance covers current content"* (`gate.rs:248-249`)
> and `sections_attested` as *"every section carries an attestation bound to its
> current content"* (`:242-243`). Both are `is_derived() == true`. The reviewing
> edge has been deriving over attested artefacts since SL-233 — the pattern is
> built, used, and unnamed, which is why the boolean looked like it partitioned
> mechanism when it was partitioning coverage.
>
> **What this buys.** `satisfied()`'s branch dissolves: there is no
> `if is_derived() { standing } else { facts }` fork to preserve, because
> derivation is uniform and the kind describes where the inputs come from.
>
> **The general form, which is why this generalises past the gate.** A step that
> looks too subjective to derive is usually one whose *artefact* was never
> recorded. Record the artefact as structured state and the condition over it
> becomes derivable without making the judgement mechanical — the human still
> judges, the engine still derives, and the state is then available to renderers
> and to downstream consumers that have not been imagined yet. Subjectivity is
> not the obstacle; the missing artefact is.

## Why this is not merely documentation

The three kinds differ in engine behaviour, which is the test the alternative
failed:

| | who authors the input | storage | liveness | payload claiming it |
|---|---|---|---|---|
| Derived | engine | none — recomputed | n/a | refused |
| Attested | human | bound artefact | binding breaks | refused; the *artefact* is submitted instead |
| Claimed | agent | evidence row | fingerprint expiry only | admitted, unbound |

Had the rows been identical, prose over the existing boolean would have been the
cheaper truth. Read with the 2026-08-03 sharpening: the "who answers" question is
uniform — the engine derives in every row — and this table is the input-provenance
axis it derives *over*.

## The attestation primitive already exists

This does not invent machinery. `DEC-088` established content-bound user
attestation for checkpoints: `AcceptanceDeclaration` carries a `basis` and an
optional turn, and Doctrine derives and binds the digest over the payload, the
disposition and the run revision, so an acceptance cannot be transplanted onto
different content. Authority is deliberately **not** a wire field — *"this
declaration is the user's, and offering an `authority` key would let a payload
claim one."*

The Attested kind generalises that primitive from checkpoint dispositions to gate
conditions. What SL-244 owes is the generalisation and the subject rule, not a new
attestation mechanism.

## Scope

This record decides the taxonomy for **design-run gate conditions**. It does not
decide how attestation is obtained, how strong the binding must be, or whether an
agent-only trust chain can substitute for one — `RFC-022` holds that question and
is deliberately untouched here. The kinds are useful to that discussion precisely
because they separate it: Attested is where a human's authority enters, Claimed is
what `RFC-022` asks whether anything can be built out of, and Derived needs
neither.

Related: `DEC-088` (the attestation primitive), `DEC-101` (open→closed narrowing
is a type error — the kinds are a closed dimension of a closed vocabulary, and
nothing is narrowed into it), `DEC-102` (seal vs craft — which this supersedes as
the partition's explanation, not as an asset policy), `DEC-066`/`DEC-067`
(liveness and cumulative revalidation), `ISS-285`, `IMP-361`, `RFC-022`.
