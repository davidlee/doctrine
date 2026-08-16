# Prototype probe — SL-256

## Why you are here

A design document is prose. Prose can describe a type model that cannot exist:
a function pointer whose signature cannot hold the function it must point at, an
extracted helper that has lost the data its own error message interpolates, a
roster split that under-covers. Models re-reading that prose do not reliably
find those, because finding them is a compiler's job, not a reader's. Several
rounds of adversarial review have already run over this design and it survived
them.

So: you are the compiler's proxy. You will implement the design's core types
disposably and let `cargo check` adjudicate. The code you write is an
instrument. **The findings are the deliverable.** Nothing you write will be
merged; the worktree you work in is thrown away when you are done.

## Where you are

You are alone in a git worktree forked from `edge`, inside a sandbox. Everything
outside the worktree is mounted read-only at the OS level, and the worktree's
own `.git` is read-only too — so **you cannot commit, and you should not try.**
Hand back an uncommitted working tree; someone else collects it. If a write
fails with a permission error, that is the sandbox, not a bug: don't fight it,
work somewhere else.

You will find an `AGENTS.md` in your worktree describing a governance process —
routing, slices, phase plans, conventional commits, lint gates. **Ignore it. It
does not apply to this run.** This prompt is your whole mandate.

The design is at `.doctrine/slice/256/design.md`, current as of your HEAD, so
read it from disk and trust it. Scope context is
`.doctrine/slice/256/slice-256.md`. The code surface is `src/design_run/`; the
design's **Code impact** section names every file it expects to touch. Do not
edit `design.md` — it is a managed artefact and your findings reach it through
someone else's hands.

## What the design claims

Read it yourself; this is only so you know where the weight sits. Three
load-bearing claims:

1. **A new `ChangeEvent::ActRecorded`** with exactly one payload term, so that
   every successful act-recording writes an observable row.
2. **`ChangeEvent::ALL` splits into `READABLE` (23) and `EMITTABLE` (22)** — what
   a persisted snapshot may *contain* versus what this binary may newly *write* —
   plus a `#[serde(try_from = "String", into = "String")]` pair replacing the
   current `rename_all` / `alias` / `rename` attributes.
3. **One `admit_and_record` seam** returning a mandatory `Pending`, replacing
   `admit_against`, with `record_declaration` and `record_act` re-signatured
   above it — `record_act` returning `Vec<Pending>` whose order is contractual.

## In scope

- The types, their signatures, and the interfaces between them.
- Enough call-site wiring to make `cargo check --bin doctrine` tell you the
  truth about whether the model holds together.
- Running the binary, or a throwaway `#[test]`, if that is the cheapest way to
  see whether the behaviour actually falls out right.

## Out of scope — actively do not spend tokens here

- **Connective tissue.** You are not building the production feature. If a call
  site is mechanical churn that teaches you nothing, stub it, `todo!()` it, or
  leave it broken and say so.
- **The existing test suite.** It will stop compiling once you change these
  signatures. That is expected and fine. Do not repair it.
- **Lint, clippy, formatting, doc comments, `just gate`.** All of it. Ignore.
- **Completeness.** A prototype that proves three claims and abandons a fourth
  is worth more than one that half-proves all four.

## What to care about, maximally

The point is to validate and improve the design, so give your full attention to:

- **Modelling.** Is this the right shape? Does the sum type carve the space at
  its joints? Does the roster split earn itself?
- **Interfaces.** Can each declared signature actually hold what it must? Does
  anything crossing a boundary arrive without data it needs?
- **Behaviour.** Does the described behaviour fall out of the model, or does it
  need propping up?
- **Direction.** Does this leave the codebase somewhere better?

## You may diverge from the design — and should

If you find a defect, or a cleaner approach, **take it and work it through.**
Do not implement the design faithfully at the cost of implementing it wrongly.
The value of this work is proving out and improving the design, not reproducing
it. A finding you reached by trying the design's way, hitting a wall, and
building the better thing is the most valuable output you can produce.

## What counts as a finding

A finding is something **the design document would have to change** to be right.

- ✅ A declared signature cannot hold the function it must point at.
- ✅ An extracted helper cannot render its own error message.
- ✅ A claimed ordering, coverage, or invariant does not hold.
- ✅ A cleaner model the design should adopt instead.
- ❌ Missing `#[derive(Debug, Clone, PartialEq)]`. True, but a design document
  declares no attributes — that belongs to whoever implements it. Same for
  imports, visibility keywords, and formatting. If you hit one, put it in a
  short "for the implementor" list at the end, not among the findings.

For each finding state: **what the design says** (with `design.md` line
numbers), **what the source says** (with `file.rs:line`), **why both cannot be
true**, and **what you would change.** Mark each as either *verified against
source* or *reasoned but unverified* — that distinction is load-bearing for
whoever reads this, and an honest "unverified" costs you nothing.

## How you know you succeeded

Not by finishing. You have succeeded if, at hand-off:

1. `cargo check --bin doctrine` either passes, or every remaining error is one
   you deliberately chose to leave and can name.
2. `PROTO-FINDINGS.md` exists at the worktree root and stands on its own — a
   reader who never sees your code can act on every finding in it.
3. You have said plainly which parts of the design you exercised and which you
   never reached, so nobody mistakes your silence for a clean bill of health.

Zero findings is a legitimate result and a useful one. Do not manufacture them.

## Budget

You get one long turn and it is not unlimited. **Create `PROTO-FINDINGS.md`
before you write any code, and append to it the moment you hit something** — a
finding recorded when you find it survives; one saved for a tidy write-up at the
end does not. When you sense you are near the end, stop implementing and spend
what is left making the report good.

Work in whatever order you like. Start with whichever claim you think is most
likely to be wrong.
