# glossary

The kinds below group into a small set of durable entities + typed facets — see
[entity-model](entity-model.md) for the architecture.

| kind                         | abbr     | folder |
|------------------------------|----------|:------:|
| **specs**                    | --       |        |
| product requirements doc     | PRD-001  |   y    |
| technical specification      | SPEC-001 |   y    |
| revision                     | REV-001  |   y    |
| requirement                  | REQ-001  |   y    |
| requirement label (membership) | FR-001 / NF-001 |  |
| **slices**                   | --       |        |
| slice                        | SL-001   |   y    |
| tech design                  | DES-001  |        |
| design review                | RVW-001  |        |
| implementation plan          | PLN-001  |        |
| phase sheet                  | PHASE-01 | phases |
| audit                        | AUD-001  |   y    |
| **governance**               | --       |        |
| policy                       | POL-123  |        |
| standard                     | STD-123  |        |
| architecture decision record | ADR-001  |        |
| **backlog**                  | --       |        |
| issue                        | ISS-001  |   y    |
| improvement                  | IMP-001  |   y    |
| chore                        | CHR-001  |   y    |
| risk                         | RSK-001  |   y    |
| idea                         | IDE-001  |   y    |

## reference forms

How to cite things in prose, commits, and comments. The id is identity; the slug
is never authoritative.

**Entity ids — prefixed, 3-digit zero-padded** (the `abbr` column above):
`SL-020`, `ADR-004`, `PRD-009`, `REQ-059`, `RSK-004`, `ASM-001`. Cite the *durable*
id, never a mobile membership label (`FR-`/`NF-` move per spec — use the `REQ-NNN`
they label).

**Document-local enumerations — bare** (prefix + integer, no zero-pad, no dash):
they are scratch refs within one document, not entity ids.

| ref | meaning | authored in |
|---|---|---|
| `OQ-1` | open question | spec / slice / design |
| `D1`   | decision | design §7 |
| `R1`   | review finding | design §10 |
| `Q1`   | design question | design / slice |
| `C1`   | charge | inquisition |

**Phase ids — `PHASE-01`** (2-digit, immutable; edits append, never renumber). The
sheet *file* is `phase-01.md` (lowercase).

**Criteria ids** (authored in `plan.toml`, immutable; bare, no pad):

| ref | meaning |
|---|---|
| `EN-1` | entry criterion |
| `EX-1` | exit criterion |
| `VT-1` | verification by **test** (automated) |
| `VA-1` | verification by **agent** check |
| `VH-1` | verification by **human** acceptance |

`VT`/`VA`/`VH` are the three verification modes — pick by *who/what* confirms the
criterion. Non-retroactive: existing `VT-` criteria stay valid as "by test."

## folder conventions

 - inside slice folder:
   - ./notes.md
   - ./handover.md
   - ./phases/phase-*.md
 - inside slice, spec or backlog folder:
   - ./research/*
   - ./context/*
