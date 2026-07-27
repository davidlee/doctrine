# IMP-324: No durable sink for design-round probe evidence

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`/design` frequently produces **executable evidence** — scratch-repo probes that
establish or refute a claim the design load-bears on. There is nowhere authored
for it to live.

- `.doctrine/slice/*/research/` is **gitignored** (`.gitignore:48`), so it is the
  disposable tier. It is also scoped to the `/research` pre-design round by
  convention (`raw/<thread>.md`), not to design-round probing.
- The established alternative is to embed measured results in `design.md` as
  tables annotated *"verified, git 2.54.0"*. That preserves the **result** and
  discards the **method**, so the next reviewer re-derives the setup rather than
  re-running it.

## Why it matters here specifically

RV-307 ran eight rounds and 39 findings against one design, and its named dominant
cost driver was claims reasoned about rather than measured — *"a tool property is
a claim needing a falsifier, not a premise"*, plus F-17/F-23's rule that the
falsifier must be **registered before** the probe. A process that generates
falsifiable probes and then throws away the probe pays that cost again at every
review round and every re-read.

Concretely: SL-232's design round reproduced RV-307 F-37's three routes, ran a
shape matrix over ~25 entry shapes, tested a replacement rule against five
registered falsifiers (two of which **failed**, which is the load-bearing part),
and censused the live corpus. All of it is re-runnable in seconds — and none of it
had a home.

## Interim action taken

SL-232 placed its probes at **`.doctrine/slice/232/probes/`** with a README
carrying the falsifiers, the git version, the corpus HEAD, and the three results
that came out *against* the hypothesis. That directory is authored-tier by
accident of not matching the ignore glob, not by convention.

## What to decide

1. Is design-round probe evidence **authored** (committed, diffable, reviewed) or
   **disposable**? The RV-307 experience argues authored: it is the substrate a
   reviewer needs to attack a measured claim, and an unreproducible measurement
   is an assertion.
2. If authored, where — `slice/NNN/probes/` (SL-232's interim choice), or a
   subdirectory of `research/` carved out of the ignore rule?
3. Should `/design` **prompt** for it, the way `/research` prompts for
   `research.md`? A convention nobody is asked about is a convention nobody
   follows.
4. Should the probe README's shape be templated — falsifiers registered up front,
   tool version, corpus HEAD stamped (RV-313 F-1's denominator-drift lesson), and
   an explicit section for results that refuted the hypothesis?

## Provenance

Surfaced during SL-232's design round while persisting the F-37 reproduction; the
first attempt to commit the probes was refused by `.gitignore:48`. Recorded in
RFC-011's case notes as a token-efficiency observation before being promoted here.
