# IMP-458: Qualified foreign-corpus citation form

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

Doctrine now cites records held in **another project's doctrine corpus** —
oubliette (`~/dev/oubliette`), the capsule host, which runs its own doctrine
install with its own id namespace. Both corpora mint `SL-`, `ADR-`, `REQ-`,
`RSK-` ids from 001 upward, so **the spaces have started colliding in practice**,
not merely in principle.

The live instance, caught by hand during `RFC-025`'s 2026-09-18 update pass:
the RFC needed to point at oubliette's byte/disk ingestion-bound risk, whose id
over there is `RSK-006`. Doctrine's own `RSK-006` is an unrelated coverage-scan
perf risk from June. A bare citation resolves cleanly to the wrong record, and
nothing anywhere says otherwise.

Both projects' docs already warn about the separate id spaces in the abstract —
oubliette's `status.md` tombstone and its `docs/contract-doctrine.md` both state
that an `SL-`/`ADR-`/`REQ-` id in one repo's `.doctrine/` means nothing in the
other. Neither says what to *write* instead, so the warning has no form to
discharge into.

## The convention (in use now, unratified)

    <project>.<ENTITY-ID>        e.g. oubliette.RSK-006, oubliette.ADR-001

Lowercase project token, a literal `.`, then the canonical prefixed id
unchanged. Chosen on three grounds: it is regex-amenable, it reads as
qualification rather than as a new id kind, and it leaves the durable id
verbatim on the right of the dot so existing habits and greps still find it.

Applied in `RFC-025` § *The spike became a repo* as of 2026-09-18. **In use, not
codified** — this item is the codification.

## Why tooling cannot ignore it

The prose-citation scanner (`prose_cite_findings`, `src/doctor_checks.rs:249`,
run by `doctrine doctor` / `check`) matches
`[A-Z]{2,}-[0-9]+(-[A-Za-z0-9]+)*`. Two consequences, and the second is the one
that makes this more than cosmetic:

1. **The qualifier does not save you.** The regex matches the `RSK-006`
   *substring* inside `oubliette.RSK-006`, so once a qualified citation appears
   outside a code span the scanner resolves it locally — against the wrong
   corpus — exactly as a bare one would. The scanner has to learn the prefix and
   skip (or, better, resolve against a declared foreign corpus) or the
   convention makes the false resolution *more* likely by making foreign
   citations more common.
2. **Today the whole class is invisible.** The scanner skips inline code spans,
   and authored prose backticks entity ids by house style — so neither the bare
   nor the qualified foreign citation produces a finding at all. The `RSK-006`
   collision passed `doctrine check quick` at exit 0. A silent skip over a
   real class is `STD-003` territory.

## What codifying it means

- **Agent guidance.** `install/routing-process.md` § *Reference forms* (inlined
  into the boot snapshot, so it reaches every agent) and `install/glossary.md`
  § reference forms, which `install/using-doctrine.md:195` already points at.
  Both currently define the *unqualified* form as the whole story.
- **Scanner.** Teach `prose_cite_findings` the qualified form: recognise
  `<project>.` and stop resolving the right-hand side locally. Whether it then
  resolves against a declared foreign root or simply abstains is the design
  question — abstaining is cheap and honest; resolving needs a registry of
  foreign corpora and their paths, which nothing has asked for yet.
- **The code-span blind spot** is arguably separable and arguably the bigger
  finding. Raise it separately if this item is scoped to the convention alone.

## Open questions

- **`OQ-1` — is the project token free text or declared?** Free text is zero
  machinery and drifts (`oubliette.` vs `Oubliette.` vs `oub.`); a declared set
  needs somewhere to declare it. A `[foreign]` table in project config is the
  obvious home if one is wanted.
- **`OQ-2` — does this want a `STD`?** It is a naming rule that binds authored
  prose across projects, which is `STD-002`'s shape (*Naming conventions — short
  entity titles, ids not slugs*). Extending `STD-002` may beat minting a new one.
- **`OQ-3` — does the reverse direction bind?** Oubliette cites doctrine's ids
  constantly and holds no record text; under this convention its citations of
  doctrine would become `doctrine.REQ-454`. That is a change to the *other*
  project's authored prose and its owner's call, and the convention is worth
  nothing if it only runs one way.
