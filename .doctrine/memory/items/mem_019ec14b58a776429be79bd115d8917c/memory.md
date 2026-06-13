# Relate entities via doctrine link, not hand-authored relation rows

**When** you want to connect two entities (a slice to the spec it serves, a slice
to the ADR that governs it, a backlog item to its slice, one governance entity to
a peer), express it as a relation — but do NOT hand-write a `[[relation]]` row or a
typed key into the `.toml`. Hand-authored rows drift malformed (SL-058 was the
clean-up); the writer validates `(source, label, target)` legality you would
otherwise get wrong silently.

**How** — `doctrine link <source-id> <label> <target-id>` writes the outbound
`[[relation]]` row; `doctrine unlink` removes it. Storage is outbound-only and
reciprocity is derived (ADR-004), so you link from the source side only; `inspect`
/ `show` render both directions.

**Vocabulary** — the legal `(source, label) → target` table is
`RELATION_RULES` @ `src/relation.rs` (ADR-010). It is the single source of truth;
do not memorise or transcribe it — read it (or let `link` reject an illegal pair).
Each rule also carries a `LinkPolicy` that decides whether `link` is the right
tool at all: most relate-intent axes are `link`-writable, but the spec relational
spine (`descends_from`/`parent`/`members`/…) and kin stay typed keys written by
their own flows — NOT by generic `link`. Read the rule's policy; don't assume.

**Trap — verify against a FRESH dev build.** `link`/`unlink`/`inspect` are recent;
the on-PATH binary and `./target/debug/doctrine` are routinely STALE and lack them,
and in the jail `cargo build` writes to `~/.cargo/doctrine-target-jail/debug`
([[mem.pattern.build.jail-target-redirect]]). A "half-wired / missing verb" symptom
is almost always a stale installed binary, not a real gap
([[mem.pattern.relation.authored-rows-tooling-half-wired]]) — re-resolve the dev
binary via `cargo metadata … target_directory` before concluding anything.
