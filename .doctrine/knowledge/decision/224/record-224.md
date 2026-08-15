# DEC-224: Contract ships as verb, help pointer, and library doc

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The candidate set

`DEC-219` fixed what a row carries and `DEC-221` fixed what keeps it honest. What
remained was how many surfaces render it. The scope named two candidates; inquiry
found four.

1. **A fetch verb** — `doctrine design contract [--format json|prompt]`.
2. **An enumerating `design apply --help`**.
3. **A generated reference doc published into the library**, on `artifact.rs`'s
   pattern.
4. **Injection into `resume`'s existing contract channel** — `contract_section`
   (`commands/design.rs:2343`) already renders gate-condition contract prose for
   the current stage edge unless the caller declares it holds it, via
   `ResumeArgs::known_contracts`.

The fourth was not in the scope or the research. It surfaced from reading the CLI
and matters because the project instruction is to ride existing seams rather than
parallel them, which gives it a prior claim that had to be examined rather than
assumed away.

## A retraction: `X3` does not survive

The research artefact claimed the published doc is *the only form reaching an
installed client project with no source*. That is wrong, and the error was
material enough to have decided this question by itself.

A client project installs the **binary**; what it lacks is `src/`. So a fetch verb
reaches it perfectly well. And under `ADR-019` reference docs are **published, not
projected** — there is no copy on disk in a client project either, they are read
with `doctrine library show`. Both forms are binary-mediated, and neither has a
reach advantage over the other.

Stripped of the reach claim, the doc's case is narrower and honest: it appears in
the library index an agent browses, so it can be found by someone who does not
know the verb exists. That is a discoverability argument, and it is the one the
user accepted it on.

## Why the verb is primary

`design show --format json` is the standing precedent for a machine-readable
projection of run state, so a contract verb is not a new idea in this command
surface. Classification is safe by construction: `guard.rs:430` sorts design
verbs in an exhaustive per-variant match — `Start`/`Apply`/`Materialise` are
`Write(<label>)`, `Show`/`Resume` are `Read` — so a new variant that fails to
declare itself is a compile error there *and* in `dispatch` (`design.rs:240`),
not something a reviewer has to catch. One caveat to carry into the design: the
`Write` labels are plain `&'static str` typed only in that arm, with no test
pinning a label to its verb, so the new verb's label is unguarded prose even
though its classification is not.

The verb alone discharges the scope's first objective and the cost RFC-026 `E8.7`
measured. Everything else here is additive.

## `--help` reduced to a pointer

An enumerating help output was rejected on two grounds. Mechanically, clap derive
wants literals, so a generated table means builder-side injection — real work for
a rendering the verb already serves better. Editorially, it bloats the output a
caller scans for flags with content that belongs behind a subcommand.

A one-line pointer costs a line, sits where a caller already looks, and answers
half of the discoverability question `DEC-225` finishes.

## One generator, three consumers

`STD-001` forbids three renderings that restate each other, so all three render
from one generator over the const contract table (`DEC-123`). This is what makes
the doc's second surface safe rather than a second thing to keep honest.

The layering constraint is tighter than first triaged, and it shapes where that
generator sits: `ADR-001` classifies `design_run` as **leaf, out-degree 0, std +
serde derive only** (`layering.toml:31`), not engine tier. So the shared source is
a *pure renderer producing strings*, and everything that embeds, publishes, or
attaches to the CLI sits above it in command tier. The three consumers cannot
share a generator that reaches `crate::install` the way `contract_section` does.

## The fourth candidate

Deferred to `inq-7` rather than rejected here, on the grounds that it is a
delivery mechanism rather than a rendering. `DEC-225` takes it up and declines it
for now, with the escalation path recorded.
