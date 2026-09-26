# ISS-490: Search default kind set omits POL, STD and REQ — revisit the exclusions

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Two coverage holes in the discovery corpus. The first is live today; the second
is scheduled but bounded by a decision that should be re-examined before the
work lands.

## 1. `doctrine search` indexes 17 of 24 entity kinds by default

`SEARCH_DEFAULT` (`src/kinds/mod.rs`) is `SL, ADR, PRD, SPEC, RFC, ISS, IMP,
CHR, RSK, IDE, ASM, DEC, QUE, CON, EVD, HYP, CPT`. Seven kinds are outside it:
`POL`, `STD`, `REQ`, `RV`, `REC`, `CM`, `REV`.

This is **deliberate, with recorded rationale** — not an oversight. `SL-141`
(which built the verb) states it:

> Explicit exclusions from default: `pol`, `std`, `rv`, `rec`, `req`, `rev`,
> `cm`. Rationale: REQ bodies are thin; RV/REC are process byproducts; POL/STD
> are rare; REV is change-axis metadata; CM is graph not prose.

Three of those rationales hold up. **Two have decayed:**

- **"POL/STD are rare"** — rarity is a poor proxy for salience. Policies and
  standards are the *governing rules*, and they are exactly what an agent asks
  after with "what is the right way here?". It is a small, load-bearing,
  high-priority class, not a rare one. `SL-141`'s own design provides a
  `governance` group alias (`adr, pol, std`) — the exception case was known at
  the time.
- **"REQ bodies are thin"** — plausible, and possibly still true of body *length*.
  But a requirement is the normative content of a spec; omitting it means a
  spec-aware search silently under-reports.

**Concrete cost.** `doctrine search "single search entry point"` returns
`PRD-017` and `SPEC-026` and quietly omits `REQ-364` — the requirement that *is*
that behaviour. Nothing in the output says the search was partial.

**The affordances do not close it either.** The groups taught as the widening
path are asymmetric: `specs` expands to `prd, spec` — *not* `req` — so "widen
with `-k specs`" still misses requirements, and the only routes to `POL`/`STD`
are `-k governance`, `-k all`, or `--with pol,std`. An agent has to know the
kind taxonomy *before* it can search for the thing it does not yet know the kind
of. That inverts the purpose of a discovery verb.

**What guidance can and cannot fix.** `IMP-488` documented the scope and the
widening flags across the shipped surfaces (`reference/using-doctrine.md` is the
canonical home). That makes the omission *discoverable* but does not remove it:
a documented default is still a default an agent will use bare.

## 2. The library's bytes are out of the index by design — reconsider?

`SPEC-026` `D10` scopes the library search provider to **published metadata
only** — title/summary, kind, address, licence — and keeps asset content out:

> Content indexing is a deferred option, not a C1 obligation. There is no secrecy
> boundary at stake (every published asset is open-source licensed under the
> declared set); the library corpus is simply the manifest, so unpublished bytes
> are out of search by construction, not by a guard.

So even after federation (`PRD-017` `REQ-364/365/366`; `SPEC-026` `REQ-377/378`)
lands, the reference corpus — `glossary.md`, `using-doctrine.md`,
`dispatch-mechanics.md`, the authority model — remains **unsearchable prose**.
That is the corpus an agent most needs by *concept* rather than by address
("what does `VT` mean?", "who may I take direction from?"). Today it is
reachable only by already knowing its logical address.

Worth saying plainly: `D10` is principled. It is a structural claim (the library
corpus *is* the manifest), and it is a deliberate scope bound rather than an
omission — hence a question, not a defect. But the rationale is about what the
corpus *is*, not about whether its prose should be findable, and those are
separable. `IMP-154` already seeds the general widening (non-entity docs + a
path column), so the mechanism this would ride is being built regardless.

## The question

Should either exclusion be revisited, or reversed?

1. **Default kind set** — widen to every prose-bearing kind? Keep the default but
   make the omission *loud* (report the kinds not searched, so a partial result
   announces itself rather than needing prior knowledge)? Add a `--all-kinds`
   affordance and make `specs` include `req`? Or leave as is, on the argument
   that `IMP-488`'s guidance is sufficient?
2. **`D10`** — index library asset content, or keep metadata-only? If kept,
   should the published docs carry a stated reason a reader can see, so "not
   found" is not read as "not present"?

## Related

- `SL-141` — built `doctrine search`; owns the default-set design and rationale.
- `RV-142` — the SL-141 audit.
- `SL-159` — added `EVD`/`HYP`; `SEARCH_DEFAULT` was maintained then, so new
  kinds *are* swept into the default — which sharpens the question of why
  `POL`/`STD` never were.
- `IMP-154` — `search --all`: non-entity docs + path column; the widening seam.
- `IDE-041` — whether the library lands before or with federated search. The
  `D10` question belongs *inside* that planning, not after it.
- `PRD-017` (`REQ-364`–`REQ-366`) and `SPEC-026` (`REQ-377`–`REQ-378`, `D10`) —
  governing the federation this scope decision sits within.
- `IMP-488` — the guidance-side fix; this item is the scope-side question it
  deliberately did not open.
