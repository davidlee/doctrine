# Sketch: the stable-section-ID marker grammar

PHASE-06 design gate (EN-2). Authored artefact under `.doctrine/slice/233/`,
durable and diffable — not runtime scratch. It answers the seven questions
`plan.toml` PHASE-06 EN-2 enumerates, under the constraint EX-10 places on (a)
and (d), and it is the surface a dedicated RV ledger is raised against.

design.md §5.3 rule 2 makes re-adoption the sole lawful crossing back from
authored bytes into a managed run, and §5.5 makes it turn on "the complete
stable-marker map must validate". This sketch is the grammar those markers are
written in.

**A marker writer and reader already exist at head**, landed by an earlier
phase: `MARKER_OPEN` / `MARKER_CLOSE`, `authored_sections`, and
`render_document` in `src/commands/design.rs`. So this gate is not specifying
a greenfield format — it is settling the contract an incumbent already writes
into authored documents, and naming what about that incumbent is unsafe. The
delimiters below are the incumbent's, unchanged. See § *The incumbent*.

> **Revision 5** (2026-07-29), after RV-323's fifth round. Rev 4 changed the
> *method* for checking claims about the incumbent — execute, don't read. Rev 5
> extends that to claims about the **proposed** rules, which is where round 5
> found it still missing, and then turns the same instrument on the threat model
> the rules were defending against.
>
> **The extraction rule was wrong twice, and a generated differential found both
> where a hand-written table of heading forms had not.** Round 5 contested one
> case (`## ###` derives `###`, but CommonMark reads that heading as empty).
> Building the oracle instead of adding the row found a second, larger one — a
> cascade, because the strip ran once where the formatter strips repeatedly —
> and then found that the fix that looked obviously right for the first family
> closed **nothing**. That is the third enumeration failure in this one rule,
> after rev 3's table twice. See § *The oracle this rule owes its shape to*.
>
> **And the property those rules were defending is withdrawn.** Chasing
> formatter stability is unbounded: closing sequences, cascades, whitespace
> runs, then inline emphasis, and a per-character probe shows no charset can
> close it because the instability is a *token-pair* property. A title derived
> from source bytes cannot be stable under a tool that rewrites source bytes
> while preserving meaning. The closing-sequence rule is **kept on CommonMark's
> authority instead** — those bytes are heading framing, not content, formatter
> or no formatter — and the whitespace collapse is **not adopted**, because its
> only justification was the withdrawn one.
>
> **Then the threat model itself was measured, and it is much smaller than rev 4
> assumed.** Markers survive a realistic formatted document byte-identically;
> prettier cannot parse TOML **at all**, so the entire authored entity tier was
> never exposed; and prettier is **not installed in this repository** — no
> binary, no config, no ignore file, not a dependency of the one `package.json`.
> Three grammar rules were derived from an untested hypothesis about a *client*
> project. §(f) now scopes the posture instead of defending it.
>
> **One correction is disclosed rather than quietly amended**: rev 4's claim that
> prettier is idempotent on its own output is **false** (2,560 of 39,019 bodies
> move on a second pass; convergence takes four). F-1 was *verified* partly on
> that claim. Its substance survives — the non-idempotence is confined to
> degenerate hash-only headings — but the reviewer, not the responder, should
> decide whether the verification stands. See § *Row 10's resolution*.
>
> **Revision 4** (2026-07-29), after RV-323's third round. **This revision
> changes method, not wording.** Rev 3's falsifier — *a third partial function
> shipped as total means every remaining answer gets an explicit totality
> check* — fired: F-3's second contest found the heading table neither
> exhaustive nor disjoint. F-1's second contest found the framing rule stated
> against a simplification of `render_document` rather than against what it
> does. F-6 found `VA-7` answering a proof obligation with a grep, which is the
> instrument this document's own §(d) says does not prove a property.
>
> So rev 4 does three things rev 1–3 did not. **Every incumbent row is now an
> executed check** run through the real binary against a throwaway project, and
> the exact commands and outputs are in § *The incumbent*; the two rows that
> cannot be executed are labelled as readings. **Every answer carries an
> explicit totality argument** — the input domain enumerated, the partition
> shown to cover it disjointly. **The `VA` criteria become behavioural**: `VA-5`
> and `VA-7` were both greps and are both replaced, because F-6's argument
> against `VA-7` applies verbatim to `VA-5` and answering only the one that was
> found would be the same error in a smaller place.
>
> The method change paid immediately. Executing the table found **two further
> defects neither review nor three revisions of reading had reached** — rows 13
> and 14 — and falsified this document's account of the framing in a way the
> contest had only half-identified. Details in § *What executing the table
> found*.
>
> **Revision 3** (2026-07-29), after RV-323's verify round. F-2, F-4 and F-5
> verified; **F-1 and F-3 contested, both rightly.** F-1's contest found rev 2's
> *repair* silently altering declared prose — and rev 2's justification for it
> simply wrong, having conflated trimming with framing. A fixed one-newline
> separator that parse removes exactly once is invertible on every body, so
> nothing is normalised and the oracle strengthens to byte equality. F-3's
> contest found "the body's first ATX heading" shipped as a total function when
> it is partial: zero and several headings were undefined. Both are the
> now-familiar shape — the defect living in the previous round's fix. No new
> findings, so rev 2's falsifier did not fire. See § *Revision history*.
>
> **Revision 2** (2026-07-29), after RV-323's first round. Five findings, all
> verified, none contested. Three of them — F-1, F-3, F-5 — falsified claims
> rev 1 *asserted about code it had read rather than run*, which is the
> projection sketch's own defect class arriving in a new document. F-3 is the
> worst and its consequence is larger than the finding stated: re-adoption
> updates a section's fingerprint and never its body, so a `materialise` after
> a hand edit silently reverts the edit and re-baselines the watermark over it.
> The nine-defect incumbent table is now twelve. F-4 withdrew a maximality
> claim the sketch had no derivation for. See § *Revision history*.

**Evidence, not recall.** Every claim in (f) about formatter survival was
measured against `prettier@3.9.6` in this jail during authoring, at three print
widths and against indented, trailing-whitespace, fenced, blockquoted and
list-nested lookalikes. One measurement changed the grammar — see (c)'s
right-trim rule. Where a claim is unmeasured it is labelled as such.

**Rev 4 extends that standard to the whole document.** Every claim about
*incumbent behaviour* is now an executed check too, run through
`./target/debug/doctrine` against a throwaway project outside the repo, one
slice and one design run per scenario. § *The incumbent* carries the result of
each. The two claims that are not behavioural — a call site, a type — are
labelled **read**, not **run**, and are the only readings left in the document.

## Why this gate exists here

R1 — *protocol ceremony exceeds its value* — was decided in the projection
sketch. This gate decides something narrower and harder to walk back: **the
authored file's on-disk contract with its own history.**

Markers are the only durable link between a section's runtime identity and its
prose. Once a real `design.md` has been materialised, every marker in it is a
compatibility surface: changing the grammar later means either re-materialising
every managed document or carrying a second parser forever. The projection
sketch's bounds can be retuned against fixture evidence; this grammar cannot.

That asymmetry is the argument for spending a design gate here, and it is also
the reason the sketch prefers *narrow and boring* at every fork.

## The governing claim

**Materialise-then-parse is the identity on the section map, and every
departure from it is a named refusal rather than a guess.**

Three properties carry it, and they are separable — each can fail while the
other two hold:

1. **Recognition is exact.** A marker is recognised by whole-line, whole-token
   match at column 0. No prefix matching anywhere, so two ids sharing 31 bytes
   are as distinct as two ids sharing none.
2. **Escaping is invertible.** Any body text that could be read as a marker is
   neutralised on write and restored on read, by a transformation that is a
   bijection on the set of marker-shaped lines.
3. **Divergence is refused, never repaired.** Seven named refusals partition
   every way an authored document can fail to describe the run it came from.

Property 3 is what makes 1 and 2 worth having. A parser that recovers *some*
map from a mangled document is worse than one that refuses, because the
recovered map re-baselines the watermark and the damage becomes the new truth.

## (a) The exact syntax, byte for byte

A marker is one line, at column 0, and nothing else on that line:

```text
<!-- doctrine:section sec-1 -->
```

Term by term, with byte counts, because the total is load-bearing in (f):

| term | bytes |
|---|---:|
| `<!--` | 4 |
| one space | 1 |
| `doctrine:section` | 16 |
| one space | 1 |
| the section id | 5 … 32 |
| one space | 1 |
| `-->` | 3 |
| **whole marker line** | **31 … 58** |

The delimiters are `MARKER_OPEN` = `"<!-- doctrine:section "` and
`MARKER_CLOSE` = `" -->"` — the constants already at
`src/commands/design.rs:66,68`, single-sourced per STD-001. This sketch adopts
them unchanged: they are already written into any document the incumbent has
materialised, and re-spelling them would be a migration bought for nothing.

Exactly one space at each of the three separator positions. Not "one or more":
a normalising formatter that collapsed runs of spaces would otherwise be able
to turn a two-space lookalike into a marker, which is the same promotion hazard
(c)'s right-trim rule exists to close. One space is also what materialise
emits, so the strict form is the only form Doctrine ever writes.

**This tightens the incumbent, which is deliberate.** `authored_sections`
currently applies `.trim()` to whatever sits between the delimiters, so
`<!-- doctrine:section␠␠sec-1␠-->` is accepted as `sec-1` today. Two spellings
of one marker is exactly what "recognition is exact" denies, and with (a)'s
charset excluding whitespace from ids the trim protects nothing. A document
carrying a slack spelling after this change gets `MissingMarker` for the
section it meant to name — a refusal, not a misparse.

### The section-id grammar, and its maximum encoded length

```text
section-id  ::=  "sec-" body
body        ::=  ( ALPHA | DIGIT | "_" | "-" )+
```

subject to the whole id being at most `DESIGN_ID_BYTES` = 32 bytes. So `body`
is 1 … 28 bytes and **the grammar's maximum encoded length is 32 bytes**,
which satisfies EX-10(a) at equality.

Three **constraints** are genuinely derived from what the marker needs — they
say what an id may not contain, and they bound the choice without determining
it. The marker is tokenised by splitting a line on single spaces and comparing
the third token whole, so an id may not contain:

- **whitespace**, or the token split is ambiguous;
- **`>` or `-`-runs forming `-->`**, or the id could terminate its own comment
  early — excluded by the charset admitting `-` only as an ordinary body byte
  with no adjacency rule needed, since `-->` requires `>`, which is not in the
  charset;
- **non-ASCII**, so a byte count equals a column count and the column-0 rule is
  checkable without a Unicode segmentation pass.

**Totality of the id rule, which rev 3 did not state.** The rule is a
recogniser, so its domain is every `&str` a caller can supply and totality means
every input reaches an outcome. It does: `DesignId::parse` tests length, then
prefix, then non-empty remainder, then — new here — that every remaining byte is
in the charset. Each test is decidable on any input, and the arms are ordered,
so exactly one of `IdTooLong`, `MalformedId` or acceptance results. Whitespace,
`>`, `\n`, `\r` and every non-ASCII byte all land in `MalformedId` by the
charset arm; there is no input that falls through. Rows 8 and 14 are what
falling through looks like today.

**The charset is a choice, not a derivation, and rev 1 dressed it as one.**
RV-323 F-4 is right: `.`, `:`, `/`, `+`, `=`, `@`, `[` and `]` all satisfy the
three constraints above, so `[A-Za-z0-9_-]` is emphatically *not* the maximal
satisfying subset. The maximality claim is **withdrawn** rather than repaired,
because the provenance rule's whole point is that a decision wearing a
derivation's clothes will later be used to settle questions it has no standing
to settle.

What actually justifies `[A-Za-z0-9_-]`, stated as the choice it is:

- it is the identifier charset the corpus already uses for entity ids and
  slugs, so it needs no new convention and reads as an id to a human;
- the three constraints establish a *safety floor*, not an optimum, and above
  that floor the cheap direction is conservative: widening admission later is a
  free change, narrowing it after ids exist in committed authored documents is
  a migration.

So the grammar takes a **conventional safe set**, not the widest one. If a
future need argues for `.` or `:`, the constraints already say adding them is
safe, and this paragraph is the record that nothing was proved against them.

**The bound is inherited, not invented.** 32 is `DESIGN_ID_BYTES`, whose own
provenance is already proved rather than asserted — `src/design_run/gate.rs`
carries `const _: () = assert!(widest_condition(&Condition::ALL) <=
DESIGN_ID_BYTES)`, and the projection sketch derives the 32-byte id term in the
264-byte worst-case change row. This sketch adds no new number. Per the
provenance rule, a marker-specific length constant would be exactly the kind of
underivable bound that rule deletes.

**Admission is wider than minting, deliberately.** `apply`'s `declare` contract
already takes a caller-supplied `subject` — `{"declare":[{"subject":"sec-1",
"body":"…"}]}` — so section ids are named by the run's driver, not claimed from
a counter. Landed fixtures name `sec-1`, `sec-9`, `sec-01`, `sec-late`,
`sec-moved`, `sec-stable`; every one is legal under this grammar and no landed
test changes. That is a fact about this grammar worth stating plainly, because
a digits-only alternative was considered and rejected precisely for breaking
them, and a grammar whose first act is to invalidate the interface it sits
under is the wrong grammar.

### Enforcement — at admission, by refusal

The rule lives inside `DesignId::parse` in `src/design_run/ids.rs`, which is
already **the one validating constructor** with a private field and no raw
route in. Every construction site inherits the charset rule the same way it
already inherits the length rule, so enforcement is universal by construction
rather than by each path remembering (EX-10(b)).

**Two refusals, and rev 1 named the wrong one** (RV-323 F-5). Precisely, as the
code has it at `ids.rs:75-92`:

- **over `DESIGN_ID_BYTES`** → `Refusal::IdTooLong { raw, limit }`, returned by
  the length guard *before* the prefix is examined;
- **unknown prefix, or an empty body after it** → `Refusal::MalformedId { raw }`
  — and this is the arm the new charset rule joins, since a bad byte is a
  malformed id rather than an over-long one.

Both are **a refusal, never a trim**, riding the reasoning already recorded in
that module: a truncated identity is a *wrong* identity, and two distinct
subjects rendering identically is the failure the layer rule exists to prevent.
EX-10(b) is satisfied by the existing length guard; the charset rule extends the
same enforcement to the same constructor, so no new enforcement mechanism is
introduced.

**The charset rule applies to all four id kinds, not only sections**, and the
cost is stated rather than hidden. Only section ids appear in markers, so only
sections *need* it. It is applied uniformly because (i) every kind's id is
rendered into a change row whose encoding is equally corrupted by whitespace or
a control byte, and (ii) two id rules kept in agreement is one more than the
model needs — the projection sketch lost a review round to exactly that shape
when its distance metric and rank table disagreed. The cost is that admission
narrows for three kinds this gate does not constrain. Widening later is cheap;
narrowing later is not. Landed `inq-root`, `att-1`, `cp-017` are all legal.

### A worked example

Run state: two sections, `sec-1` (title *Design Problem*) and `sec-11` (title
*Current State*), in that document order. `materialise` emits exactly:

```text
<!-- doctrine:section sec-1 -->
## 1. Design Problem

The managed run has no way to write prose back to the authored tier.

<!-- doctrine:section sec-11 -->
## 2. Current State

`slice design` scaffolds a template and refuses to clobber it.
```

Parsing that document recovers `{sec-1, sec-11}` with the two bodies verbatim
and the document order `[sec-1, sec-11]` — note the order is the *marker
sequence*, not the id sort, under which `sec-11` precedes `sec-2`. See
§ *A consequence this gate creates*.

### What the marker does not carry

The payload is the id and nothing else. Not the title, not the fingerprint, not
the order, not a schema version.

Each omission is a decision:

- **No title** — see (b); a title in the marker is the retitle hazard.
- **No fingerprint** — a fingerprint in the authored file would be a second
  copy of a runtime fact in the tier that cannot be trusted to hold it, and
  re-adoption already declares the whole-document fingerprint explicitly
  (§5.5). Two fingerprints that can disagree is worse than one.
- **No order** — order is positional and therefore cannot desynchronise from
  itself.
- **No version** — the grammar is the compatibility surface; a version field
  would invite a second parser, which is the outcome this sketch is trying to
  avoid. If the grammar must ever change, the marker's fixed `doctrine:section`
  token is itself the discriminator, and the migration is a re-materialisation.

## (b) What happens to a marker's payload when a section title changes

**Nothing.** The marker carries the id alone, and the id is invariant under a
retitle.

The mechanism, stated in the order the bytes move:

1. The title is *authored prose inside the section's region* — the `## 1.
   Design Problem` heading is body content, not marker content.
2. So a retitle changes the region's bytes, which changes the section's
   fingerprint.
3. A moved fingerprint invalidates evidence bound to the old one, under
   DEC-066: attestations and gate clearances that named the old fingerprint are
   no longer live. That is correct — a section whose title changed has changed,
   and prior review attested to prose that no longer exists.
4. The section's *identity* does not move, so its inquiry links, its review
   history and its position survive the retitle.

**The load-bearing negative: a section id is never derived from its title.** A
slug-derived id (`sec-design-problem`) is the obvious readable alternative and
it is rejected here, because under it a retitle is an *identity* change wearing
the costume of a content change. Doctrine would see one section vanish and
another appear, silently re-keying every attestation, link and finding — the
one failure mode in this whole grammar that produces a plausible-looking wrong
answer rather than a refusal. The charset in (a) permits a slug; the *contract*
forbids deriving one, and the round-trip fixture must include a retitle that
holds the id fixed while the fingerprint moves.

### Where rev 1 was wrong, and what it exposed

Rev 1 wrote that `Section.title` "is a projection convenience, extracted from
the region's first ATX heading", and that "at materialise the snapshot is
authoritative; at re-adopt the document is". RV-323 F-3 checked this against
the code and **no part of that path exists**:

- `title` and `body` are *independent wire fields* (`submission.rs:124-132`),
  both caller-supplied;
- `render_document` materialises `body` and **never emits `title`**
  (`design.rs:1148-1157`), so a section declared with title *Visible* and body
  *plain body* produces a document with no heading at all;
- `authored_sections` extracts id and body only (`design.rs:249-268`) — nothing
  reads a heading back.

So rev 1 asserted a mechanism instead of checking for one. That is the
projection sketch's F-1 defect class, in a new document and in the *answer to a
gate question* rather than in a bound.

**The repair: the heading is body, and `title` is derived.** One source, not
two kept in agreement:

- a section's body **begins with its own heading** — the driver declares
  `body` whose first non-blank line is `## 1. Design Problem`, exactly as
  `render_document` already assumes by emitting body alone;
- `Section.title` is **derived** from that heading, never independently
  supplied. The `title` wire field is removed, on the same reasoning EX-12
  removes the annotation spelling: two ways to say one thing, where one of them
  is unvalidated, is the defect — and a derived title cannot desynchronise from
  the prose because there is nothing to disagree with.

### The derivation, stated totally

Rev 2 wrote "the body's first ATX heading"; rev 3 tabulated four cases. RV-323
contested the table twice, the second time for **being neither exhaustive nor
disjoint**: an empty body was caught by two rows, and setext headings (`Title`
over `===`), an ATX heading inside a fenced code block, and a bare `#hashtag`
were classified by no row at all. That contest is correct, and what it exposes
is that a table of *cases someone thought of* is not a partition. So the
derivation is restated as a decision procedure over the whole input domain,
with the domain named first and the arms proved to exhaust it.

**The domain** is every value the `body` field can hold: an arbitrary `String`,
an arbitrary sequence of Unicode scalar values. Not "a reasonable Markdown
body" — the caller supplies it and nothing has filtered it.

**The recogniser, spelled out, because "an ATX heading" is what rev 3 left to
the reader.** A line *L* is an **ATX heading line** iff, reading *L* left to
right:

1. zero to three U+0020 SPACE characters, then
2. a run of one to six `#`, then
3. **either** the line ends there **or** the next character is a space or a tab.

Nothing else is an ATX heading line. This is a decidable predicate on every
possible line, which is what lets the arms below be exhaustive rather than
enumerated.

**The procedure**, evaluated in order. Let *f* be the first line of the body
that is not blank, where *blank* means "contains no character other than space
and tab".

| # | guard | outcome |
|---:|---|---|
| 1 | no such *f* exists | refused — `SectionBodyEmpty { id }` |
| 2 | *f* is not an ATX heading line | refused — `SectionBodyHeadingMissing { id }` |
| 3 | *f*'s extracted text is empty | refused — `SectionTitleEmpty { id }` |
| 4 | otherwise | `title` = *f*'s extracted text; every other line, heading or not, is ordinary body content |

**Why this is total and disjoint, rather than asserted to be.** Each guard is
the negation of its predecessors conjoined with one decidable test, so at most
one arm applies (disjoint); the fourth is the unguarded remainder, so at least
one applies (total). Arm 1 decides whether a non-blank line exists; arm 2
applies a predicate defined on every line; arm 3 tests a string for emptiness.
There is no input for which the procedure fails to terminate with exactly one
outcome. The argument is short precisely because the domain was named first —
rev 3's table had no domain, only rows.

**Extraction, byte by byte.** From an ATX heading line: drop the leading spaces
and the `#` run, and call what remains the **content region** — by the
recogniser it is empty or begins with a space or a tab. Then, **on the content
region, with its leading whitespace still present**: drop trailing whitespace;
then, while what remains ends in a run of `#` that is either preceded by
whitespace or is the whole of the region, drop that run and any whitespace
before it. Finally trim leading and trailing whitespace. The remainder is the
title.

**Two things about that order are load-bearing, and rev 4 got both wrong.**

*The leading whitespace must survive until after the closing-sequence test.*
Rev 4 dropped "the whitespace run that follows" the opening `#`s **first**, so
on `## ###` the region became `###`, the trailing run was preceded by nothing,
the closing-sequence step could not fire, and the title derived as `###`.
CommonMark reads `## ###` as a heading with **empty** content, because the
closing sequence is preceded by the delimiter space. This is not a formatter
question — the rule was simply wrong against the standard it names.

*The strip must run to exhaustion, not once.* One pass leaves a **cascade**:
`## # # #` derives `# #`, and a document that gets formatted derives `#`, then
nothing. Stripping while the guard holds lands on the same fixed point from any
starting spelling.

Both defects were found by a **generated differential** rather than by reading
— see *The oracle this rule owes its shape to*, below. Neither was visible in
rev 4's hand-written table of heading forms, and rev 4's table was itself the
repair for two earlier hand-written tables. That is the third enumeration
failure in this one rule.

**Why the closing sequence is dropped at all — and what that is *not*.** Rev 4
justified this step by formatter behaviour: `prettier@3.9.6` rewrites
`## Title ##` to `## Title`, so an extraction that kept the `#`s would derive
`Title ##` before a format run and `Title` after one. **That justification is
withdrawn as the reason, and the rule is kept on a better one.** CommonMark
defines a trailing `#` run preceded by whitespace as the heading's *optional
closing sequence* — framing, not content. Extraction drops it because those
bytes are **not part of the heading's text**, which is true whether or not any
formatter exists. The formatter merely made the error visible.

The distinction matters because the formatter justification does not survive
scrutiny, and §(f) now says so.

**The three cases the contest named, now classified — and one of them cannot
occur at all.**

- **Setext** (`Title` then `===`) — *f* is `Title`, not an ATX heading line, so
  arm 2 refuses. This is a **decision**, not an oversight: setext needs
  two-line lookahead, and `Title` over `---` is ambiguous with a thematic break
  and with a table delimiter row. Requiring ATX keeps the predicate single-line
  and decidable. The cost is a refusal for an author who writes setext, and the
  fix is one line.
- **A bare `#hashtag`** — not an ATX heading line, because rule 3 requires a
  space, a tab, or end-of-line after the `#` run. Arm 2 refuses. Classified by
  **measurement** rather than by recalling a spec: prettier strips the closing
  sequence from `## Title ##` (the positive control — the probe fires on a real
  heading) but leaves `#hashtag ##` untouched, so prettier does not parse
  `#hashtag` as a heading. The same probe classifies `####### Title` (seven
  `#`) and a four-space-indented `## Title` as non-headings; both fall to arm 2.
- **An ATX heading inside a fenced code block** — **this case cannot arise**,
  and the argument is a proof rather than a claim. For such a heading to be
  *f*, every line before it must be blank, by the definition of *f*. A blank
  line does not open a fenced code block. So no fence is open at *f*. If a body
  *does* begin with a fence, the fence opener is itself the first non-blank
  line and arm 2 refuses it. The same argument disposes of indented code blocks
  and HTML blocks: arm 2 asks only whether *f* matches a single-line shape, so
  no block context can reach it.

**Requiring the heading rather than defaulting to an empty title** is what makes
the derivation total, and it costs nothing a design document would want: a
section that does not begin with its own heading produces a document whose
structure is not readable, which is the outcome materialise exists to prevent.
Deeper headings staying as content is what lets a section carry subsections
without fragmenting into more sections than the run declared.

**And the procedure runs at adoption, not only at declare.** If it ran only at
declare, a hand-edited document could carry a section whose body fails arm 2,
and the run would hold a section whose title is underivable — the partiality
reintroduced through the other door. Running it at both admissions is what makes
the derivation total *for the run* rather than for one entry path.

Note the rule is *first non-blank line*, not *first heading anywhere*. "First
heading anywhere" would let prose precede the title and still derive one, so a
body whose heading sat in its third paragraph would silently take that as its
title — a partial function wearing a total one's clothes for the third time.

Because the heading is inside the body, it is inside the fingerprint, so a
retitle *does* move the fingerprint and DEC-066 *does* invalidate the
attestations bound to it. Under rev 1's account — with `title` a separate field
never materialised and never fingerprinted — a retitle would have moved nothing
at all, and prior review would have survived a change to the section's most
visible claim. That is the failure this repair actually prevents.

### The oracle this rule owes its shape to

Rev 4 answered F-6 with a **table of heading forms** — ATX one through six,
seven hashes, `#hashtag`, tab-after-hashes, closing sequence, indents, setext,
fence-first, empties, bare `##`, two headings. RV-323 round 5 contested it for
omitting one row, and the omission was real. But adding the row would have been
the wrong repair, for the reason this document has now hit three times: a table
of cases someone thought of is not a partition, and it is not an oracle either.

So the instrument is a **generated differential**, and it is what found every
defect above. The property:

> for every body *B*, `derive(B) == derive(format(B))`

`derive` is the procedure above; `format` is a Markdown formatter. The corpus
is a **product** — indents × hash runs × delimiters × contents × trailers, plus
non-heading first lines for arms 1 and 2 — not a list. 39,019 bodies, run
against `prettier@3.9.6` in-process.

What it measured, each rule differing from the one above it by a single change:

| rule | divergences | family it closed |
|---|---:|---|
| rev 4, as written | 4,680 | — |
| + leading whitespace survives the test | 4,224 | `## ###` at the head |
| + refuse a bare-`#`-run title | 4,224 | **nothing — the guard was wrong** |
| + strip to exhaustion | 1,664 | the whole hash cascade |
| + collapse internal whitespace | 832 | `# a  b` → `# a b` |

Three results from that run matter more than the counts.

**The contested case was the least of it.** `## ###` is degenerate. `# a  b` is
a double-space typo, and the row-3 guard — the fix that looked obviously right
when the family was still "bare hash runs" — closed **nothing**, which reading
would not have revealed at all.

**The last family cannot be normalised away.** What survives at 832 is a
formatter rewriting *inline markup*: `# *em*` → `# _em_`. Probing every
printable ASCII character plus unicode in four positions each found **zero**
characters unstable in isolation: the instability needs a matched *pair*, so it
is a token property, not a character property, and the obvious cheap defence —
an admissible-title charset, the shape (a) uses for ids — is **measurably
unavailable** here.

**Chasing it is unbounded**, and that is the finding. Closing sequences,
cascades, whitespace runs, emphasis — the families fall one at a time and the
next one is always a formatter behaviour nobody enumerated. A title derived
from a heading's *source bytes* can never be stable under a tool that rewrites
source bytes while preserving meaning. Only a title derived from **rendered
inline content** could be, and that is a Markdown inline parser — a dependency
this CLI does not have and PHASE-06 was not scoped for.

**So the stability claim is withdrawn rather than repaired**, exactly as (a)'s
maximality claim was under F-4, and for the same reason: a property this
document cannot derive must not be asserted. What remains is a rule justified
by CommonMark, and a scope stated in §(f).

**The whitespace collapse is therefore NOT adopted.** It closes a real family
and it was tempting — one line, 832 fewer divergences. But its only
justification was one formatter's behaviour, and with the stability claim
withdrawn that justification is gone. Keeping a rule whose derivation has been
retracted, because it is already written and looks useful, is precisely the
habit F-4 named. A double space in a heading stays in the derived title.

### The larger defect F-3's evidence exposed

Following F-3's citation of `adopt_authored` "leaving both title and body
unchanged" reaches something worse than a title problem, and it is now this
sketch's most serious finding about the incumbent.

**Re-adoption does not adopt the prose.** `run.rs:281-295` updates a section's
`fingerprint` and nothing else; `body` is assigned in exactly one place in that
file (`run.rs:497`, the declare path). It is not an oversight that can be
patched locally either — `DerivedInput.authored_sections` is a
`BTreeMap<DesignId, Fingerprint>` (`run.rs:64`), *digests only*, so the pure
layer is not given the authored text and **cannot** adopt it.

The consequence is silent data loss, and it is the worst outcome this grammar
can produce:

1. a user hand-edits `design.md`; `adopt_authored` validates and records the
   new fingerprint, so the run reports itself current;
2. the snapshot still holds the *pre-edit* body;
3. the next `materialise` renders that stale body over the user's edit and
   re-baselines the watermark to the reverted bytes.

Nothing refuses, nothing warns, and the watermark now certifies the reversion.
This is precisely the failure the governing claim names — a map recovered from
a document, then made the new truth — reached through the adoption path rather
than the parse path.

**And the landed test over this exact path is the archetype of a test that
passes against the defect.** `adopt_authored_crosses_divergence_and_rebaselines_alone`
(`tests/e2e_design_state.rs:281`) declares a section body `"first draft"`,
hand-writes `"hand written prose"` over the document, adopts it — and then
asserts exactly two things: that the watermark re-baselined, and that the
section's fingerprint moved. **It never asserts the stored body.** Both of its
assertions hold *while the prose is silently reverted*, so the one test
covering this path cannot fail on it.

That is worth more than the code reading, because it makes VA-6's requirement
empirical rather than predicted: a fingerprint assertion demonstrably passes
against this defect, in this repository, today. RV-323's verify round
independently confirmed the defect **by execution**.

**Rev 4 ran the whole loop end to end, and the reversion is worse to watch than
to read about.** Three sections were declared, materialised, hand-edited
(`middle` → `HAND WRITTEN PROSE`), and re-adopted with the hand-edited
fingerprints. Every step reported success — the adoption emitted
`section_fingerprint_changed sec-2 old=f8461cf0859b new=5bf3d6453af6`, which is
the run agreeing that it had taken the edit on board. The next `materialise`
then reported `materialised … at revision 6`, and the hand-written prose
occurred **zero** times in the resulting document while the stale `middle`
occurred once. No refusal, no warning, no non-zero exit anywhere in the
sequence. The user's only signal that their prose is gone is reading the file
again.

**PHASE-06 must widen `DerivedInput.authored_sections` to carry the authored
body alongside its digest, and `adopt_authored` must store it.** Carried as
EX-13 (see § *A consequence this gate creates*), because a criterion that does
not exist obliges no phase to build the thing this sketch depends on.

## (c) Escaping — what a body may contain, and how it is neutralised

A section body may contain anything, including a line that is byte-identical to
a marker. This document is itself an instance: the worked example in (a) is
prose containing marker lines, and design.md will document the grammar the same
way. An escaping answer that only works for bodies nobody would write is not an
answer.

### The shape test

A line is **marker-shaped** iff, after right-trimming whitespace and removing
up to three leading spaces, it matches:

```text
"<!-- doctrine:" ":"* "section " <section-id> " -->"
```

with `<section-id>` legal under (a)'s grammar. Let *k* ≥ 1 be the number of
colons.

Two details in that definition are the whole answer, and both were found by
measurement rather than reasoning:

- **Right-trimming is required, not defensive.** Measured: prettier strips
  trailing whitespace. So a body line `<!-- doctrine:section sec-6 -->` followed
  by three spaces — not a marker when written, because recognition demands the
  line end at `-->` — is *normalised into a marker* by a formatter run. Without
  the right-trim in the shape test that line escapes escaping, and formatting
  promotes body text into a section boundary. This is the sharpest edge in the
  grammar and nothing about the syntax suggests it.
- **The three-space left-trim is defensive, and labelled so.** Measured:
  prettier preserves 1-, 2- and 3-space indentation on a top-level HTML comment,
  so it cannot promote an indented lookalike today. Three spaces is CommonMark's
  limit before a line becomes an indented code block, so it is the widest
  promotion a conforming formatter could perform. The rule costs nothing and
  removes the dependency on one formatter's current behaviour.

**Totality of the escaping rule.** The domain is every line of every body. The
shape test is a decidable predicate on a line — right-trim, remove up to three
leading spaces, match a fixed regular shape whose id term is the total
recogniser above — so every line is either marker-shaped or not, with no third
outcome. On the shaped set, write is *k ↦ k+1* on {*k* ≥ 1} and read is
*k ↦ k−1* on {*k* ≥ 2}; on the unshaped set both are the identity. Two total
functions on a two-part partition of the domain, so the composition is total.
There is no line for which the transformation is undefined, which is the
property the round-trip oracle would otherwise be silently quantifying over a
subset of.

**The id grammar is part of the shape test in both directions.** A line like
`<!-- doctrine:section not a valid id -->` is not marker-shaped, so it is not
escaped on write and not recognised on read — consistent, and the round trip
holds. If the two directions disagreed about what counts as shaped, escaping
would stop being invertible; that symmetry is a test obligation, not a comment.

### The transformation

- **On materialise (write):** every marker-shaped body line with *k* colons is
  emitted with *k+1*.
- **On re-adopt (read):** a marker-shaped line at column 0 with *k = 1* is a
  **marker**. Any marker-shaped line with *k ≥ 2* is **body**, restored to
  *k−1*.

Write maps *k ↦ k+1* on {*k* ≥ 1}; read maps *k ↦ k−1* on {*k* ≥ 2} and
consumes *k* = 1. The two are mutually inverse on marker-shaped lines and
identity on everything else, so the composition is the identity on all bodies.
This is delimiter doubling, chosen because it is the one escaping scheme with
no reserved sequence a body can be unable to express: any lookalike at any
nesting depth is one increment away from safe, and the scheme has no escape
character that itself needs escaping.

Worked, from this very document:

| written by a user in a body | stored on disk after materialise | read back |
|---|---|---|
| `<!-- doctrine:section sec-1 -->` | `<!-- doctrine::section sec-1 -->` | unchanged |
| `<!-- doctrine::section sec-1 -->` | `<!-- doctrine:::section sec-1 -->` | unchanged |
| `<!-- doctrine:section sec-1 -->␠␠␠` | `<!-- doctrine::section sec-1 -->␠␠␠` | unchanged |

**Fenced code needs no special case, and that is the point.** A marker-shaped
line inside a fence is escaped like any other body line, because escaping is
lexical and does not consult Markdown block structure. The alternative — a
parser that skips fenced regions — requires the parser to track fence state
across a document a human may have left with an unbalanced fence, and an
unbalanced fence would then silently swallow every subsequent marker. Lexical
beats structural here precisely because the input is untrusted.

A human who *hand-types* an unescaped marker-shaped line into a fence gets a
refusal — `UnknownMarker` or `DuplicateMarker` by (e)'s table. That is the
correct outcome and it is stated rather than smoothed: Doctrine cannot
distinguish an intended section boundary from an illustrative one, so it
refuses instead of guessing, and the fix is one colon.

## (d) Collision behaviour

### Exact duplicates

Two markers bearing the same id in one document is `DuplicateMarker { id }` —
refused. There is no last-wins, no merge, no positional disambiguation. The
document asserts two regions are the same section and they are not, and
choosing either one silently discards the other's prose.

### Distinct ids sharing a long common prefix — EX-10(c)

**They cannot collide, and the reason is structural rather than probabilistic.**

Recognition compares the *whole* id token for byte equality. Nothing in the
read path uses `starts_with`, a prefix index, or an abbreviation:

- ids are bounded at admission and **never truncated**, so the renderer never
  produces a shortened form two ids could share (the projection sketch's
  round-3 finding, already settled);
- the marker carries no fingerprint, so there is no digest abbreviation here
  analogous to `ENVELOPE_FINGERPRINT_SHORT_BYTES`;
- section lookup is `SectionGroup::find`, which is `==` on `DesignId`.

This is not a hypothetical case in this grammar, which is why it earns a
fixture rather than a paragraph. Under (a), `sec-1` and `sec-11` are both legal
and one is a proper prefix of the other — the ordinary case, not the
adversarial one. The adversarial case is two 32-byte ids differing only in
their final byte; both were carried through the (f) measurement and recovered
whole and distinct.

Three obligations follow, and they are the testable content of this answer:

1. `sec-1` and `sec-11` present in one document resolve to their own regions,
   in a fixture that would pass under prefix matching only if the shorter id
   were also declared — so the test is arranged with `sec-11` declared *first*
   in document order, where a prefix matcher misassigns.
2. Two ids differing only in byte 32 resolve distinctly.
3. A grep-level obligation: no `starts_with` against a section id in the parse
   path. Stated as an obligation and not as a mechanism — per the projection
   sketch's own conclusion that detecting one spelling is not proving a
   property, this one is carried by review, and the fixtures above are what
   actually fail if it is violated.

**Totality of the collision answer.** The domain is every pair of ids the parse
path can produce. Resolution is `==` on `DesignId`, which is byte equality on a
`String` — a total function on every pair, with exactly two outcomes and no
input for which it is undefined. That is why this answer needs no partition
table: the operation is already total, and the only way to make it partial would
be to introduce normalisation, which the next paragraph declines to do.

### Ids that differ only by case, or by leading zero

`sec-1` and `sec-01` are **distinct ids**, as are `sec-a` and `sec-A`. The
grammar does no normalisation, and comparison is byte equality.

This is a deliberate refusal to be clever, and it has a real cost: a user who
writes `sec-01` where the run holds `sec-1` gets `UnknownMarker` rather than a
match. That is the better failure — case-folding or zero-stripping would make
two ids the run considers distinct collide in the document, which is the
collision this section exists to prevent, reintroduced by a convenience.

## (e) What a user editing or deleting a marker produces

Every departure of the document from the run is one of **seven** refusals. The
set is a **partition** — total (no document state falls through) and disjoint
(evaluation order makes at most one fire). Both properties are needed: a
non-total set means some mangled document is silently adopted, and an
overlapping set means the refusal a user sees depends on evaluation order
nobody wrote down.

Rev 3 listed five and rev 3's own § *For the reviewer* said nothing proved
totality. Rev 4 proves it below, and proving it added two: the framing evidence
(§ *Row 10's resolution*) produced a document state the five could not classify,
and a fourteenth incumbent defect produced another.

Checks run in this fixed order:

| # | refusal | fires when | the edit that produces it |
|---:|---|---|---|
| 1 | `CarriageReturnInDocument` | the document contains any `\r` byte | saving with CRLF line endings |
| 2 | `MarkerFreeAddition` | any non-blank bytes precede the first marker | typing a preamble, or breaking the first marker's syntax |
| 3 | `DuplicateMarker { id }` | one id marked twice | copy-pasting a section |
| 4 | `UnknownMarker { id }` | a marker names an id the run does not hold | editing an id, or inventing a section in prose |
| 5 | `MissingMarker { id }` | a run section has no marker in the document | deleting a marker line but keeping its prose |
| 6 | `StructuralDeletion { id }` | a marker's region is empty | deleting a section's prose but keeping its marker |
| 7 | `UnterminatedDocument` | the final region is non-empty and does not end in a newline | stripping the file's trailing newline |

### Why the set is total — the argument rev 3 owed and did not give

Totality is not a property of the *list*; it is a property of the **region
decomposition** the list is evaluated over. So the decomposition comes first.

A document is a byte string. Its **marker lines** are the lines matching (a)'s
syntax at column 0. They cut the document into a **head** (everything before
the first marker line, possibly empty) and one **region** per marker: the bytes
from just after that marker line's terminating newline up to the first byte of
the next marker line, or to the end of the document for the last.

That decomposition is defined for *every* byte string, including one with no
marker lines at all (head = the whole document, zero regions). Every byte of
the document lands in exactly one part. So classifying the head and each region
totally is sufficient, and the head is easy — non-blank bytes present, or not.

For a region *R*, exactly one of three things is true, and the third is where
rev 3's rule was undefined:

| *R* | classification |
|---|---|
| ends with `"\n"` | body = *R* minus **exactly one** trailing `"\n"` |
| is empty | `StructuralDeletion { id }` — the marker is present, its region is not |
| non-empty and does not end with `"\n"` | `UnterminatedDocument` |

**And the third case can only occur at the end of the document** — which is
what makes the rule well-defined rather than merely enumerated. A marker line
begins at column 0, so either it starts at offset 0 or the byte immediately
before it is a newline. That byte is the last byte of the preceding region.
Hence every region except the last either is empty or ends in a newline, and
the third row is reachable only for the final region. That is a proof, and it
is the thing F-1's second contest was asking for when it said the repair was
"undefined at EOF".

With the decomposition total, the seven checks are a partition over it: #1 is a
whole-document byte test, #2 classifies the head, #3–#5 compare the region ids
against the run's section ids (a set comparison, total by construction), and
#6–#7 are the two non-body rows of the table above. Disjointness is the fixed
evaluation order, as before.

Reading the table as user acts, which is how the anti-theatre criterion VA-2
will read it:

- **Deleting a marker, keeping the prose** (#5). The orphaned prose merges into
  the preceding region, so the preceding section's fingerprint also moves — but
  that is not what is reported, because the *cause* is the absent marker.
  Refusing on the cause rather than the symptom is what makes the message
  actionable.
- **Editing a marker's id** (#4). The old id goes missing and the new one is
  unknown, so #4 and #5 both hold. The order resolves it to `UnknownMarker`,
  which is the more informative of the two: it names the token actually on the
  line in front of the user.
- **Breaking a marker's syntax** — a lost space, a mangled `-->` — demotes the
  line to body text. If it was the first marker, #2 fires; otherwise the named
  section goes missing and #5 fires.
- **Deleting a whole section, marker and prose together** (#5). Not a separate
  class: v1 has lifecycle transitions rather than deletion (DEC-063), so
  removing a section in prose is not the deletion interface and the refusal
  says so.
- **Deleting a section's prose but leaving its marker** (#6). Distinguished
  from #5 because the user's act and the correct advice differ: they emptied a
  section rather than removed it, and adopting an empty body would let prose
  deletion masquerade as an edit.
- **Saving the file with CRLF line endings** (#1). This one is new in rev 4 and
  it is a refusal rather than a tolerance on purpose. The incumbent silently
  drops every `\r` (row 14, executed), so a CRLF save today is adopted as if it
  were LF and the next materialise rewrites the user's line endings without
  saying so — the silent-alteration class this whole document exists to close.
  The alternative to refusing is to preserve `\r` as an ordinary body byte,
  which means the writer must detect and reproduce a per-document line-ending
  convention. Doctrine's authored documents are LF; a document that is not gets
  a named refusal naming the fix, which is one `dos2unix`. Narrow and boring, as
  at every other fork here.
- **Stripping the file's trailing newline** (#7). An editor configured not to
  end files with a newline produces a final region that cannot be framed. This
  is a refusal rather than a tolerance for the same reason: accepting it would
  mean parse removes "at most one" newline, and the next materialise would add
  one back — a byte the user did not type, appearing in their next diff, from a
  rule that repaired rather than refused.

**No refusal changes runtime state.** Per §5.3 rule 2 and EX-4, an invalid
adoption moves neither clearance nor the watermark. The document stays diverged
and ordinary mutation keeps entry-refusing until the user repairs it or adopts
a valid candidate — the failure is sticky by design, because a refusal that let
the run proceed would leave the snapshot describing prose that no longer
exists.

**Whitespace-only text before the first marker is not an addition.** A leading
blank line is what a formatter produces, and refusing it would make the
document unformattable. Non-blank bytes are the trigger.

## (f) Survival of formatter reflow — measured

Measured against `prettier@3.9.6`, the formatter named in EN-2, in this jail.
Fixtures carried markers at minimum and maximum id length, prefix-sharing ids,
escaped lookalikes at one/two/three colons, an inline mid-sentence lookalike,
indented lookalikes at 1–3 spaces, a trailing-whitespace lookalike, a
marker-shaped line inside a fenced code block, one inside a blockquote, one
inside a list item, consecutive markers with an empty region between them, and
a marker as the final line.

**Result: every marker line survived byte-identically**, at `--print-width`
80, 40 and 20, in both `--prose-wrap preserve` (default) and
`--prose-wrap always`. Both 32-byte prefix-sharing ids were recovered whole and
distinct.

The mechanism is that a block-level HTML comment is an **HTML block** in
CommonMark, which prettier passes through verbatim. That is why print width is
irrelevant: the 58-byte worst-case marker survived a 20-column width, where any
prose-wrapping rule would have broken it. The measurement is what licenses (a)'s
maximum-length answer to ignore column budgets entirely.

### What does *not* survive, stated plainly

The claim is about the **marker map**, not the bytes. Formatting changed the
surrounding document in three measured ways:

- a **blank line inserted** between a marker and an immediately following
  heading;
- **table cells padded** to column width;
- **trailing whitespace stripped**, and prose rewrapped under
  `--prose-wrap always`.

So a formatter run is a **foreign edit**. Section fingerprints move, ordinary
mutation entry-refuses per §5.3 rule 1, and re-adoption is the crossing back —
exactly as for a hand edit. This is the honest reading of "survives reflow": the
document remains *parseable into the same section map*, so re-adoption
succeeds and no prose is lost. It does not mean evidence survives, and it
should not: prose that was reformatted was changed, and DEC-066 invalidating
attestations bound to the old bytes is the correct outcome, not a limitation.

Two consequences the parser must therefore honour, both directly from the
measurements:

1. **Marker recognition may not depend on adjacency.** Arbitrary blank lines
   may appear between a marker and its content. A parser keyed on "marker then
   heading on the next line" breaks on the first formatter run.
2. **The right-trim in (c)'s shape test is mandatory**, per the trailing-space
   promotion measured above.

### What the formatter can actually reach — measured at rev 5

Rev 4 designed against formatter behaviour in three places without ever asking
what the exposure was. Rev 5 asked. Three measurements, all against
`prettier@3.9.6`:

- **Markers survive a realistic document byte-identically.** A materialised
  document carrying three markers, a table, a list, a long line, a
  closing-sequence heading, a double-spaced heading and inline emphasis was
  formatted: all three marker lines came through unchanged. **The structure the
  whole grammar rests on is not at risk.** What changed was prose spelling —
  blank line after each marker, table padding, `## Last ##` → `## Last`,
  `## Detail  with  spaces` → `## Detail with spaces`, `*emphasis*` →
  `_emphasis_`.
- **Prettier cannot touch TOML at all.** `getSupportInfo()` lists no TOML
  language and a `.toml` path infers a **null parser**; formatting one requires
  the third-party `prettier-plugin-toml`. So the **entire authored entity tier
  — every `.toml` under `.doctrine/` — is out of reach of stock prettier.** The
  exposure was only ever Markdown: `design.md`, `notes.md`, the prose tier.
- **Prettier is not installed in this repository.** Not on `PATH`, no
  `.prettierrc`, no `prettier.config.*`, no `.prettierignore` anywhere in the
  tree, and the single `package.json` (`web/map`) does not depend on it — its
  toolchain is eslint, tsc, vite and vitest. The rev 4 measurements were taken
  against a copy fetched via `npx --yes` **for the measurement itself**.

**That last point reframes §(b)'s whole chase.** The formatter hazard is not a
property of this repository; it is a hypothesis about a *client* project that
installs Doctrine and happens to run a Markdown formatter over `.doctrine/**`.
That hypothesis was never stated as one, never tested, and three grammar rules
were derived from it. It is real but narrow, and it does not warrant a
Markdown inline parser in a CLI that has none.

**So the posture is scoped rather than defended.** Doctrine states that
`.doctrine/**` is **not formatter-safe Markdown** and should be excluded from
project formatting — the one-line `.prettierignore` entry being the idiomatic
mechanism in a client repo. Two caveats belong with that:

- **Doctrine should not write that line itself.** `doctrine install` seeding a
  `.prettierignore` into a client repo is the platform reaching into a
  host-project convention, which is what POL-002 forbids. It is documentation
  or an opt-in, never an installer side effect. **A narrower variant that may
  clear POL-002 is captured as IMP-355**: write a `.prettierignore` into a
  *kind folder* when the first record lands there. A file Doctrine writes
  inside its own tree is Doctrine describing what it owns, not a statement
  about the host repo — which is why the trigger is first-write rather than
  install time. Deliberately sized low, and whether even that crosses the
  policy is left open there rather than settled here.
- **The guidance is advice, not an invariant.** A user can hand-format, use
  another formatter, or have format-on-save configured. What makes that
  survivable is the first measurement above: markers survive, so a formatted
  document still parses into the same section map, adoption still crosses back,
  and the damage is bounded to prose spelling and a moved fingerprint — which
  §(f) already calls the correct outcome for a foreign edit.

### The limits of this evidence

- **Only prettier.** EN-2 requires at least one and prettier is the one
  available in this jail; `mdformat`, `dprint`, `markdownlint --fix` and
  `pandoc` are absent and **unmeasured**. Editor format-on-save for Markdown
  usually *is* prettier, which is why it remains the right single instrument —
  but "measured against one formatter" is now the stated scope of the claim,
  not a limitation footnote under a universal one.
  Prettier is also the strongest available test for the wrapping hazard because
  it is the one that rewraps prose at all. An untested formatter that rewrote
  HTML blocks would defeat the grammar, and no grammar expressible in Markdown
  survives that, so the exposure is named rather than mitigated.
- **Fixtures for the marker claims; a generated corpus for the title claims.**
  §(f)'s marker measurements used purpose-built fixtures, not a materialised
  real `design.md` — which does not exist until PHASE-06 builds it. §(b)'s
  title measurements do not have that limit: 39,019 bodies from a product, not
  a list. The round-trip property test EX-9 requires is what closes the
  remaining gap, by generating the inputs rather than enumerating them.
- **Prettier's own config is not pinned.** A repo-level `.prettierrc` could set
  options not measured here. The three measured axes (print width, prose wrap)
  are the ones that could plausibly interact with a single long line.

## (g) Visibility in rendered Markdown

**A reader of rendered Markdown sees nothing.** An HTML comment is not rendered
as content by CommonMark, GitHub, or any renderer in ordinary use. The blank
line a formatter inserts after a marker is likewise not visible — Markdown
collapses it.

**A reader of the source sees the marker**, and that is where the honest cost
sits, because design.md is read as source at least as often as it is rendered:
in the editor, in `git diff`, in review. Three costs, and what each is worth:

- **A line of machine noise above every heading.** Mitigated by the marker
  being short (31–58 bytes), fixed in form, and unmistakably not prose.
- **Diff noise.** Mitigated by the payload being id-only per (b): markers do not
  churn on retitle, reorder or edit. A marker line appears in a diff only when a
  section is created — which is exactly when a reader wants to see it.
- **An invitation to edit.** The marker looks editable and is not, and (e) is
  the entire answer to what happens when someone tries. The refusal messages
  should name the marker as Doctrine-owned rather than merely reporting a parse
  failure.

The rejected alternative is worth recording: a heading attribute
(`## Title {#sec-1}`) is *visible* in renderers that do not support the syntax —
CommonMark has no attribute syntax, so GitHub renders the braces as literal
text in the heading. That fails "unobtrusive" in the one place it matters most,
the rendered document, and it also couples the marker to the heading line,
which (b) needs to keep separate.

## The incumbent, and what this grammar changes about it

The seven answers above are not a proposal in the abstract. `render_document`
already writes markers and `authored_sections` already reads them, so each
answer either ratifies incumbent behaviour or names a defect. Distinguishing
the two is the most useful thing this sketch can hand PHASE-06, because it is
the difference between work and re-work.

**Ratified, unchanged:** the delimiters (a); whole-token id comparison, so the
`sec-1` / `sec-11` prefix case is already safe (d); marker-to-next-marker
region boundaries; recognition at column 0, which `strip_prefix` already
enforces.

**Defects this grammar names.** Each is a current, reachable behaviour, and
each becomes a refusal or a rule above.

**Every row is now RUN, not read.** Rev 1–3 shipped this table as twelve
readings of code, and rev 3's § *For the reviewer* said so and predicted a
thirteenth. Rev 4 executed each row against the real binary — `doctrine slice
new`, `design start`, `design apply` to declare, `design materialise`, then a
hand-written document and a `design apply --input` carrying `adopt_authored`,
one throwaway slice per scenario. **Adoption is the readout**: it accepts iff
the caller's section digests match what Doctrine parsed out of the document, so
"adoption succeeded with digest *X*" *is* the statement "Doctrine read body
*X*". No re-implementation of the parser is involved anywhere in the evidence.

| # | incumbent behaviour | where | evidence | this grammar |
|---:|---|---|---|---|
| 1 | no escaping whatsoever — `render_document` concatenates `section.body` raw, so a body containing a marker line splits the section on the next read | `design.rs:1146` | **RUN.** A body carrying `<!-- doctrine:section sec-9 -->` materialised that line raw; the document then reads as two sections and adoption of the declared body refuses `1 mismatched` | (c) |
| 2 | trailing whitespace defeats `strip_suffix`, so a formatter that strips it **promotes body text into a marker** | `design.rs:255` | **RUN, both polarities.** A body line `…sec-9 -->` plus three spaces: adoption of the whole body **succeeds** (not a marker). Strip trailing whitespace from every line, as prettier does, and the same adoption **refuses** — the line was promoted and split the section | (c) right-trim |
| 3 | `.trim()` on the id admits non-canonical spellings of one marker | `design.rs:260` | **RUN.** `<!-- doctrine:section␠␠␠sec-1␠␠␠-->` adopts as `sec-1` | (a) |
| 4 | bytes before the first marker are **silently discarded** — the loop ignores lines while `current` is `None` | `design.rs:252` | **RUN.** A hand-typed `PREAMBLE THE USER TYPED` above the first marker: adoption **succeeds**, the preamble is in no section, and the next materialise deletes it | (e) #2 |
| 5 | duplicate ids silently last-wins, because `authored_section_digests` collects a `Vec` into a `BTreeMap` | `design.rs:272` | **RUN.** Two `sec-1` markers over bodies `## FIRST` and `## SECOND`: adoption succeeds against `## SECOND`'s digest, and the emitted row is `section_fingerprint_changed sec-1` | (e) #3 |
| 6 | an unparseable marker id is **silently dropped** by `.ok()`, so a mangled document yields a plausible partial map | `design.rs:276` | **RUN.** `<!-- doctrine:section not-a-sec-id -->` over `## Ghost`: adoption succeeds, the ghost region is dropped | (e) #4 |
| 7 | sections render in **id order**, because `SectionGroup` is id-sorted and `render_document` iterates it directly | `design.rs:1148` | **RUN.** Declared `sec-2` then `sec-11`; the materialised document emits `sec-11` first | § *A consequence* |
| 8 | no charset rule on ids, so a section id may contain a **newline** and produce a marker that cannot be read back | `ids.rs:75` | **RUN, and worse than "cannot be read back".** `sec-a\nb` is accepted at declare, materialises as `<!-- doctrine:section sec-a` / `b -->`, and adoption then refuses. Materialise cannot produce a readable marker for that id, so **the run is permanently unadoptable** — wedged, not merely lossy | (a) |
| 9 | `materialise` calls `fsutil::write_atomic` directly rather than `entity::write_body` | `design.rs:1122` | **READ** — a call site, not a behaviour. One of the two readings left in this document | EX-1, already stated |
| 10 | **terminal body whitespace is lost**: `render_document` emits `section.body` raw, but `authored_sections` `trim_end`s each region (`:258`, `:266`) | `design.rs:258` | **RUN.** Of three declared sections, the two whose bodies ended in whitespace failed to round-trip and the one that did not succeeded — declared `e1fd2bd8ec7c`, read back `59d4082467a6` | (c), below |
| 11 | `title` is a wire field that is **never materialised**, so it can drift from the prose forever with nothing to detect it | `design.rs:1148` | **RUN, with a positive control.** Titles `Alpha`/`Beta`/`Gamma` occur **0** times in the materialised document; the bodies occur 3 times and `Alpha` is present in the snapshot. So it is stored, and it has no path to the authored tier | (b) |
| 12 | **re-adoption never adopts the prose** — only the fingerprint is updated, and `DerivedInput` carries digests without bodies, so a later `materialise` reverts the user's edit and re-baselines over it | `run.rs:281`, `:64` | **RUN end to end.** Hand edit → adopt (reports the fingerprint moved) → materialise (reports success) → the hand-written prose occurs **0** times and the stale body is back. Every step exited 0 | (b), EX-13 |
| **13** | **the adoption completeness check never consults the document's marker set.** `missing` is `held − declared` and `unknown` is `declared − held` (`run.rs:263-264`) — the run's sections against the *caller's assertion*. The document enters only through `mismatched`'s digest lookup. So a marker the document carries and the run does not is invisible, and there is no `UnknownMarker` refusal at head at all | `run.rs:263` | **RUN.** A legal, unheld `sec-42` marker over `## Invented` added by hand: adoption **succeeds**. Row 6 is the same hole reached with an *unparseable* id; this is the parseable one | (e) #4, EX-5 |
| **14** | **carriage returns are silently dropped.** `authored_sections` iterates `str::lines()`, which strips a trailing `\r`, and rejoins with `\n` | `design.rs:251` | **RUN.** A CRLF-saved document whose region on disk is `## One\r\n\r\nbody line\r\n` adopts against the digest of the **LF** form. So a CRLF save is adopted as LF and the next materialise rewrites the user's line endings | (e) #1 |

**Row 8, corrected** (RV-323 F-2). Rev 1 claimed a space or `>` in an id
produces a marker that cannot be read back. That is **false at head**, and the
counterexample is decisive: `authored_sections` extracts the whole substring
between the delimiters and `.trim()`s it, so `sec-a b` and `sec-a>` are emitted
and read back unchanged — the incumbent does not tokenise on spaces at all.
What genuinely cannot be read back is a **newline** (or CR) in an id, which
`DesignId::parse` admits today and which splits the marker across two lines.

The distinction matters and rev 1 collapsed it: excluding a newline repairs an
incumbent **defect**, while excluding a space or `>` is a **forward-looking
contract change** required by (a)'s single-space tokenisation, not a bug fix.
Row 8 now names only the defect; the space and `>` exclusions are justified in
(a) as part of the new grammar and nowhere claimed as incumbent breakage.

### What executing the table found

Running the rows rather than re-reading them was rev 4's whole method change,
and it is worth being precise about what that bought, because "we tested it"
is the kind of claim this document has been wrong about before.

**It found two defects three revisions of reading had not.** Row 13 — the
completeness check comparing the caller's map against the run's sections and
never against the document's markers — is visible in four lines of `run.rs` and
three revisions read past it, because reading `missing`/`unknown`/`mismatched`
tells you the *intent* and only running it tells you the *extension*. Row 14
was invisible to reading twice over: `str::lines()` is idiomatic and its
`\r`-stripping is a documented convenience nobody re-reads.

**It sharpened row 8 from "lossy" to "wedging".** Reading said a newline in an
id produces a marker that cannot be read back. Running showed the run reaches a
state where *no* document can be adopted, because materialise itself cannot
emit a readable marker for that id. Same fact, different severity.

**It changed nothing about (f).** The one section that was already measured
produced no corrections in three review rounds and none here. That asymmetry is
now the third piece of evidence for the same conclusion, and it is why the rev-4
falsifier below is about method rather than about care.

**And it falsified this document's own framing account** — the subject of the
next section, which is the thing F-1 has now contested twice.

### Row 10's resolution — the framing, measured rather than described

Rev 2 answered this at **admission**: normalise terminal whitespace at declare
time. RV-323's first verify round contested that and was right — the repair
**silently alters prose the caller declared**, a smaller version of the defect
class this sketch exists to close.

Rev 3 replaced it with a fixed one-newline separator that parse removes exactly
once. RV-323's third round contested *that*, on two grounds: the rule is
**undefined at EOF**, and it was stated against a simplification of
`render_document` rather than against what `render_document` does. **Both are
correct, and the second is worse than the contest claimed.** Here is the
incumbent's actual output, `od -c` over a three-section document, which no
revision of this sketch had run:

```text
…  s   e   c   -   1       -   -   >  \n
#   #       A  \n  \n   t   e   x   t          \n  \n  \n  \n
…  s   e   c   -   2       -   -   >  \n
#   #       B  \n  \n   m   i   d   d   l   e  \n  \n
…  s   e   c   -   3       -   -   >  \n
#   #       C  \n  \n   l   a   s   t          \n
```

The declared bodies ended in `\n\n`, nothing, and two spaces respectively. So
`render_document` (`design.rs:1146-1157`) builds each block as marker + `"\n"` +
body + `"\n"` and then `blocks.join("\n")`, which means **an interior block
carries two framing newlines — its own and the join's — and the last block
carries one.** Rev 3's "exactly one `\n` as the block separator" describes
neither. A parse rule that removed exactly one would leave a spurious blank line
on every interior body and be correct only on the last.

**The repair is to make the framing uniform, which is also what makes it
total.** The affix must not depend on position, because a position-dependent
affix is what put the EOF case beyond the rule:

- **Emit:** for each section in `seq` order, the marker line, `\n`, the escaped
  body **verbatim**, then `\n`. Blocks are **concatenated with no separator** —
  `blocks.join("\n")` is deleted, and it is the join, not the trim, that was
  carrying the position dependence.
- **Parse:** the region is the bytes from just after a marker line's newline to
  the first byte of the next marker line, or to the end of the document.
  **Exactly one** trailing newline is removed. The two cases where a region has
  none are classified by (e)'s decomposition — empty region is
  `StructuralDeletion`, non-empty-without-newline is `UnterminatedDocument` and
  is reachable only at EOF, which § *(e)* proves.

Remove-exactly-one inverts append-exactly-one, and now it does so at every
position including the last. Verified over the adversarial set — `""`, `"x"`,
`"x  "`, `"x\n"`, `"x\n\n"`, `"\n\nx"`, `"  "`, `"\n"`, `"a\nb  \n\n\n"` — all
round-trip byte-exactly under the stated rule.

**The cost, and then the measurement that reverses it.** Dropping the join
removes the blank line between a body and the next marker, so the emitted
document is tighter than the incumbent's. The obvious objection is that
`prettier` will therefore reformat every document Doctrine writes. Measured, at
`prettier@3.9.6`:

1. prettier's normal form inserts a blank line **after** each marker and does
   **not** insert one before the next marker;
2. prettier is idempotent on its own output — **CORRECTED AT REV 5, SEE BELOW**;
3. and — the part worth the measurement — **`materialise(adopt(prettier(D)))`
   is byte-identical to `prettier(D)`.**

So a formatted document is a foreign edit exactly once. Adopt it and
re-materialise, and the document is already at prettier's fixed point; every
subsequent format run is a no-op and every subsequent materialise reproduces
the same bytes. The tighter emitted form is a one-cycle transient, not a
permanent fight with the formatter.

> **Rev 5 correction — claim 2 is false as stated, and this is disclosed rather
> than quietly amended because RV-323 F-1 was VERIFIED partly on it.** The
> differential run in §(b) formatted 39,019 generated bodies and then formatted
> the output again: **2,560 moved on the second pass**, and convergence took
> four (2,560 → 1,084 → 320 → 0). Prettier is *not* idempotent on its own
> output in general.
>
> **What survives.** The non-idempotence is confined to headings whose content
> is `#` runs and spaces — `## # #` → `## #` → `##`, one group eaten per pass.
> No ordinary document in the corpus moved twice, and claims 1 and 3 stand as
> measured. So F-1's substance — one foreign-edit cycle, then a fixed point —
> holds for every document a user would write, and the framing repair it
> licenses is unaffected.
>
> **What does not.** "Idempotent" was a universal, and rev 4 asserted it from a
> single-pass measurement. The honest form is: *prettier reaches a fixed point
> in at most four passes on the measured corpus, and in one pass on every
> non-degenerate document in it.* This is the same defect class the document
> keeps finding in itself — a property checked on the cases at hand and stated
> as though checked on the domain. The reviewer should decide whether F-1's
> verification stands on the corrected claim; the responder's position is that
> it does, and that saying so is not the responder's call to make alone.

**The test oracle**, which rev 2 owed and rev 3 under-specified: a property test
over generated bodies — terminal spaces, terminal blank lines, marker
lookalikes, adversarial whitespace — asserting `parse(materialise(S)) == S`
**byte for byte**, and asserting it over **section counts one, two and three**,
because the defect rev 3 shipped was invisible at *n* = 1 and lived entirely in
the interior/last distinction.

**What this does not claim.** A formatter still strips trailing whitespace, so
bytes do not survive the *first* external `prettier` run — see (f). That is a
foreign edit, correctly detected by fingerprint divergence and crossed by
re-adoption. The round-trip property is about materialise-then-parse with no
external editor in between, and there it is now exact.

Defects 4, 5, 6 and 13 share one shape and it is the shape the governing claim
names: **the incumbent recovers a map from a document it should have refused.**
Each silently produces a *plausible* section map from a document that does not
describe the run, and that map then re-baselines the watermark, making the loss
the new truth. They are the reason (e)'s refusals are a partition rather than a
list — four separate silent-recovery paths is what a non-total refusal set looks
like in practice.

Defects 2 and 14 are the ones no amount of reading would have found: one took
measuring a formatter, one took running the parser. Defect 9 is already
PHASE-06's under EX-1 and is listed only so the count is honest.

## A consequence this gate creates

Naming a required change discharges nothing unless it is applied — the
projection sketch lost a verify round to exactly that, and the standard it set
is that a criterion which does not exist obliges no phase to build the thing
the sketch depends on.

**The gap, and it is live rather than hypothetical.** `SectionGroup` orders
sections by id, and its doc comment says so: *"ordered by id so serialisation is
deterministic"*. `render_document` iterates that group directly
(`design.rs:1148`), so **the incumbent already materialises in id order**. Id
order is not document order: under (a)'s grammar `sec-11` sorts before `sec-2`,
so the document a run emits is in an order no author chose, and the order is not
stable under adding a tenth section — declaring `sec-10` silently moves existing
prose. The envelope's section rows inherit the same defect;
`render/envelope.rs:767` sorts "outstanding review first, then section order",
where *section order* currently resolves to id order.

**The resolution**, which PHASE-06 must build rather than assume:

- `Section` gains a `seq`, claimed from the run's existing `next_seq` counter —
  the same monotonic, pure, snapshot-derived counter inquiry nodes already use.
  No new machinery, and it inherits `claim_seq`'s existing properties.
- Materialise emits sections in `seq` order. `SectionGroup`'s id ordering stays
  exactly as it is: it is a *serialisation* determinism rule and this is a
  *document* order, and conflating them is what produced the gap.
- **Re-adopt takes document order as authoritative** and renumbers `seq` to the
  marker sequence. Hand-reordering sections is therefore a supported edit rather
  than a sixth refusal — it loses no information and refusing it would make the
  authored tier less editable than a plain file, which is the failure R1 names.

**Applied, not merely named:** PHASE-06 gains an appended `EX-11` carrying this,
and `VT-3` naming its tests.

### EX-13 and EX-14 — what rev 2 added and rev 4 adds

RV-323's round 1 produced three obligations, appended as one criterion because
they are one defect seen from three sides: **the authored document and the
snapshot can disagree with nothing to detect it.**

- **Bodies round-trip byte-exactly**, by a **uniform framing affix** — every
  block, including the last, is marker + `\n` + body + `\n`, concatenated with
  no separator, and parse removes exactly one trailing newline (row 10). Rev 4
  replaces rev 3's one-newline *separator*, which described neither the
  incumbent nor a total rule, and which the third round rightly contested.
- **`title` is derived** by the total procedure in § *The derivation, stated
  totally*, and the wire field is removed, so the heading is fingerprinted and a
  retitle invalidates the evidence bound to it (row 11, answer (b)).
- **Re-adoption adopts the prose**: `DerivedInput.authored_sections` carries the
  authored body alongside its digest, and `adopt_authored` stores it — closing
  the silent-reversion path in row 12.

**EX-14 is rev 4's, and it exists because F-6 was right about the instrument.**
Removing a wire field does not stop an old payload carrying it: `Declaration`
derives `Deserialize` without `#[serde(deny_unknown_fields)]`
(`submission.rs:106`), so an unknown key is accepted and ignored. **Run, with a
positive control**: a declare payload carrying `totally_bogus_key` was accepted
and the section created; a payload with a type error on a real field was
refused, so the deserializer does reject what it is asked to. Therefore the
field removals in EX-12 and EX-13(b) are unfalsifiable without a
`deny_unknown_fields` on `Declaration`, and that is the criterion. It is scoped
to `Declaration` deliberately: `ApplyRequest` carries `#[serde(flatten)]` on its
envelope (`submission.rs:467`), which is incompatible with the attribute, so
PHASE-06 must not try to put it there. The attribute is an established
convention in this codebase — `observation/wire.rs`, `observation/request.rs`,
`publication.rs`, `worktree/jail.rs` all carry it — so this rides a seam rather
than introducing a mechanism.

Criteria are immutable, so `EX-11` … `EX-14` are appended and nothing is
renumbered.

## For the reviewer

Where I am least confident, in descending order:

**Rev 5 adds two items above the list, because they outrank everything in it.**

- **The residual formatter instability is accepted, and a reviewer may
   reasonably think it should not be.** §(b) withdraws the stability claim
   rather than repairing it, on the argument that chasing it is unbounded and
   that §(f)'s measurements make the exposure narrow — markers survive, TOML is
   untouchable, prettier is not installed here. The counter-argument I cannot
   dismiss: a client project *is* the audience, `.doctrine/**` prose *is*
   Markdown, and "we document that you should exclude it" is a weaker guarantee
   than "it round-trips". If the reviewer thinks the derived title must be
   formatter-stable, the honest answer is not another extraction rule — it is a
   Markdown inline parser and a slice to scope it, and this gate should say so
   rather than approximate it. **This is the decision in rev 5 most worth
   attacking**, and it is a judgement about scope, not a fact I can measure.

- **The `.prettierignore` guidance has an unresolved POL-002 tension.** §(f)
   says Doctrine must not write that line into a client repo because it is a
   host-project convention. But it also *recommends* the line, and a
   recommendation the installer cannot act on may simply not reach anyone. I do
   not know whether documentation is sufficient here, and I have not checked
   whether an existing install-time advisory surface could carry it. **IMP-355
   captures the narrower kind-folder-local variant** rather than leaving the
   tension dangling, but filing it is not resolving it: whether a file Doctrine
   writes inside its own tree escapes POL-002 is exactly the question, and this
   gate does not decide it.

The rev 4 list, unchanged in order:

1. **The incumbent table is now executed, and that moves my uncertainty rather
   than removing it.** Nine → twelve → fourteen across four revisions; the last
   two came from running the rows, not reading them. What each row *asserts* is
   now backed by a transcript. What the table does not have is any argument that
   it is **complete** — I enumerated scenarios, and a scenario I did not think
   of is exactly what rows 13 and 14 were. The honest statement is that the
   table's individual claims are now evidence and the table's *closure* is still
   enumeration. A generator over document mutations (see 3) is what would close
   it, and it does not exist.
2. **The framing repair is specified but not yet exercised against the real
   writer.** Rev 4's uniform-affix rule and its prettier-convergence result were
   verified against a model of the *proposed* emit/parse pair, because
   `render_document` does not implement it yet — the executed evidence is about
   the *incumbent*, which is what falsified rev 3. That is the correct division,
   but it means the strongest new claim in this revision — that
   `materialise(adopt(prettier(D)))` = `prettier(D)` — rests on a model of code
   PHASE-06 has not written. EX-9's property test over section counts 1, 2 and 3
   is what converts it, and until then it is the claim in this document most
   likely to be wrong. **Rev 5 update: one of its three legs turned out to be
   false** — prettier is not idempotent on its own output — and rev 4 asserted it
   from a single-pass measurement. The other two legs stand and the substance
   survives, but this item was right that this was the weak claim, and the way it
   broke (a universal asserted from the cases at hand) is the document's
   recurring defect rather than a one-off.
3. **Escaping is lexical, and a materialised document is therefore not
   byte-stable under a user's own editor.** If a user hand-writes a lookalike
   into a body and re-adopts, the next materialise escapes it — so the file
   gains a colon the user did not type. The round trip is correct on the *map*
   and correct on the *stored body*, but the authored bytes acquire an edit the
   user did not make, and their next diff shows it. I believe this is
   unavoidable for any escaping scheme and preferable to the alternatives, but
   it is a real surprise and I have not found a way to warn about it at the
   moment it happens.
4. **The seven refusals now carry a totality proof, and the proof is about the
   decomposition rather than about the list.** § *(e)* argues that the
   head-plus-regions decomposition is defined on every byte string and that each
   part is classified by a decidable test. I believe that argument. What it does
   *not* cover is whether the seven refusals are the right *semantic* carve-up —
   two document states can both be refused, correctly, and still deserve
   different messages, and VA-2's anti-theatre reading is the only thing
   checking that. The generator over document mutations — delete a random line,
   duplicate a random region, corrupt a random marker — asserting *some* refusal
   always fires would independently confirm the totality proof, and it still
   does not exist.
5. **The uniform charset rule may be over-reach.** (a) applies the ASCII
   restriction to all four id kinds when only sections need it, and argues from
   change-row encoding plus the cost of two rules kept in agreement. A reviewer
   may reasonably hold that a marker-driven constraint should not narrow the
   inquiry, checkpoint and attestation namespaces, and that the honest form is a
   `Section`-only rule. I chose uniform because divergence between two id rules
   is the specific failure this project has already paid for once, but the
   argument is about method rather than evidence.
6. **The `sec-11` before `sec-2` fixture in (d) is my own construction.** I
   arranged it so a prefix matcher misassigns, which means it tests the failure I
   thought of. A generated fixture over prefix-related id sets would be stronger,
   and I have specified the fixture rather than the generator.
7. **(f)'s evidence is one formatter in one jail at one version.** Stated in
   § *The limits of this evidence*, and I do not think it is closable here —
   but if the reviewer holds that "tested against at least one real formatter"
   should mean the test lives in the suite rather than in this document, that is
   a fair reading of EN-2(f) and it is not currently PHASE-06's obligation. EX-9
   requires a round-trip property test; it does not require prettier to be in
   the loop, and adding a formatter to the test environment is a cost I have not
   priced.

---

## Revision history

### Rev 5 — RV-323's fifth round: the oracle, and the threat model

F-1 verified. F-3 and F-6 contested, both on one defect seen at two altitudes.

| point | disposition |
|---|---|
| F-3 contested: the specified extractor derives `###` from `## ###`, but the closing sequence makes that heading **empty**, and a format run to `##` then refuses `SectionTitleEmpty` — accepted at declare, unadoptable after formatting | **Conceded, and verified independently before answering.** The rule drops the delimiter whitespace *before* testing for a closing run, so on all-hashes content the test can never fire. Fixed by keeping the leading whitespace until after the test. But building the oracle rather than patching the case found a **second, larger defect the contest did not reach**: the strip ran once where a formatter strips repeatedly, so `## # # #` → `# #` → `#` → nothing, a cascade. Strip now runs to exhaustion. |
| F-6 contested: `VT-6`'s enumerated table omits that case, so the test passes while EX-13(b) is false | **Conceded, and the repair is the instrument, not the row.** Adding the missing row would have been the third patch to a hand-written enumeration — rev 3's table twice, now rev 4's oracle. `VA-7`/`VT-6` become a **generated differential**: `derive(B) == derive(format(B))` over a product corpus, with the known-defective rule retained as a positive control. Proof that this was the right call: the fix that looked obviously right for the first family (refuse bare-`#`-run titles) closed **zero** divergences, and reading would never have shown that. |
| — | **The oracle then falsified the property itself.** Families fall one at a time — closing sequences, cascades, whitespace runs, inline emphasis — and a per-character probe over all printable ASCII plus unicode found **no** unstable character, because instability is a *token-pair* property. No charset can close it. A title derived from source bytes cannot be stable under a tool that rewrites source bytes while preserving meaning. **The stability claim is withdrawn**, as (a)'s maximality claim was under F-4. The closing-sequence rule is kept on **CommonMark's** authority instead; the whitespace collapse is **not adopted**, because its only justification was the withdrawn one. |
| — | **And the threat model was measured for the first time.** Markers survive a realistic formatted document byte-identically; prettier **cannot parse TOML at all**, so the authored entity tier was never exposed; and prettier is **not installed in this repository** — no binary, no config, no ignore file, not a dependency of the one `package.json`. Three grammar rules had been derived from an untested hypothesis about a *client* project. §(f) now scopes the posture: `.doctrine/**` is documented as not formatter-safe Markdown, and Doctrine must not write a `.prettierignore` itself (POL-002). |
| — | **One correction disclosed, not quietly amended.** Rev 4's "prettier is idempotent on its own output" is **false**: 2,560 of 39,019 bodies move on a second pass, convergence takes four. F-1 was *verified* partly on it. The non-idempotence is confined to degenerate hash-only headings so the substance survives, but whether the verification stands is the reviewer's call. |

**Rev 4's falsifier resolved correctly.** It said that a defect found in a
`RUN`-marked claim would mean execution was being performed rather than used,
and that a defect in one of the two claims resting on a model of unwritten code
would be *the gate working as designed*. Round 5 found the second kind. The
prescribed answer was to convert those claims — which is what the differential
does for the title half.

**The falsifier for rev 6.** Rev 5's bet is that the remaining exposure is one
of **scope**, not of rules: that withdrawing the stability property and
documenting the formatter boundary is safer than approximating a Markdown
parser. **If the next round finds a formatter-driven defect that a reviewer
judges must be fixed in the grammar, the bet is lost, and the answer is not a
sixth extraction rule — it is to route the inline-parser question to its own
slice and let this gate close without it.** If instead the next round finds a
defect in the differential's *construction* — a corpus dimension it does not
vary, a control it lacks — that is the instrument being sharpened and the answer
is to extend it in place.

### Rev 4 — RV-323's third round, and a change of method

F-1 and F-3 contested a second time; F-6 raised. Rev 3's falsifier fired: a
third partial function shipped as total. So this revision changes **how claims
are made**, not only what they say.

| point | disposition |
|---|---|
| F-1 contested again: the repair is undefined at EOF, and `render_document`'s `blocks.join("\n")` plus each block's own trailing `"\n"` yields two separator newlines, not one | **Conceded, and running it made the concession sharper than the contest.** `od -c` over a three-section document shows an interior block carrying **two** framing newlines and the last carrying **one**, so rev 3's rule described neither. The repair is to delete the join and make the affix **uniform** — every block, including the last, is marker + `\n` + body + `\n`, concatenated with no separator; parse removes exactly one. Position-independence is what makes the EOF case disappear rather than be special-cased. The two states where a region has no trailing newline are classified by (e)'s decomposition, and the proof that the second is EOF-only is now in the document. |
| F-3 contested again: the zero/multiple-heading table is neither exhaustive nor disjoint — an empty body is caught by two rows, and setext, ATX-inside-a-fence and `#hashtag` are unclassified | **Conceded, and the fix is structural.** A table of cases someone thought of is not a partition. Replaced with a decision procedure over a **named domain** (arbitrary `String`), a **spelled-out** ATX recogniser, and four arms each of which is the negation of its predecessors plus one decidable test — disjoint by construction, total by the unguarded remainder. Setext and `#hashtag` are classified as refused, by decision, with the cost stated. ATX-inside-a-fence **cannot occur**, with a proof: every line before *f* is blank and a blank line opens no fence. Prettier's closing-sequence normalisation added an extraction rule nobody had noticed — without it a format run would silently change a section's title. |
| F-6: `VA-7` proves neither half of EX-13(b) — no `deny_unknown_fields`, and both greps are satisfiable by code that does the wrong thing | **Conceded without qualification, and it is the finding I should have raised against myself.** This document's §(d) already says "detecting one spelling is not proving a property", and `VA-7` did exactly that one section later. Answered with a code obligation rather than a better grep: **EX-14** puts `#[serde(deny_unknown_fields)]` on `Declaration`, and `VA-7` becomes a behavioural negative that submits the old `title` key and asserts refusal, plus a semantic table over the measured heading forms. `VA-5` had the identical defect and is replaced too — answering only the one the reviewer found would be the same error in a smaller place. |
| — | **The method change found two defects review had not.** Row 13: the adoption completeness check compares the caller's declared map against the run's held sections and **never against the document's marker set**, so a legal-but-unheld marker is silently ignored and there is no `UnknownMarker` refusal at head. Row 14: `str::lines()` strips `\r`, so a CRLF-saved document is adopted as LF and the next materialise rewrites the user's line endings. Both are executed, both add a refusal to (e). |
| — | **What did not move.** The delimiters, the id grammar and its 32-byte bound, the escaping scheme, (d)'s collision answer, (g), and every (f) measurement. Rows 1–8 and 10–12 all survived execution as written, except row 8, which execution made *worse* — a newline in an id wedges the run rather than merely losing a marker. |

**The falsifier for rev 5.** Rev 2 set one against claims-from-reading and it
did not fire; rev 3 set one against partial-functions-shipped-as-total and it
did. Rev 4's is against the residue both leave behind: **if the next round finds
a defect in a claim this revision marked RUN, then execution is being performed
rather than used, and the answer is not more scenarios but a generator** — the
document-mutation generator named twice in § *For the reviewer* and still not
built. If instead the next round finds a defect in something marked READ, or in
the two claims this revision admits rest on a model of unwritten code (the
framing repair and its prettier convergence), that is the gate working as
designed and the answer is to convert those two.

### Rev 3 — RV-323's verify round

F-2, F-4 and F-5 verified. **F-1 and F-3 contested, and both contests hold.**
No new findings were raised, so rev 2's falsifier — a sixth finding of the
read-rather-than-ran class — did not fire.

| point | disposition |
|---|---|
| F-1 contested: admission normalisation silently alters declared prose, and neither the exact behaviour nor a test oracle was specified | **Conceded on both counts, and the justification was wrong too.** Rev 2 claimed no parser rule could recover a body's trailing blank line "because the framing and the content are the same bytes". That conflates **trimming** with **framing**: `trim_end` consumes greedily and cannot know how much it ate was separator, but a *fixed-width* separator is removable exactly because the amount is known. So nothing is normalised: emit the body verbatim plus exactly one `\n`, and parse removes exactly one. Remove-exactly-one inverts append-exactly-one, so the composition is the identity on **every** body. The oracle strengthens with it — `parse(materialise(S)) == S` byte for byte, where rev 2 could only ever have asserted it modulo its own normalisation. |
| F-3 contested: heading derivation leaves zero and multiple headings undefined | **Conceded.** "The body's first ATX heading" is a *partial* function and rev 2 shipped it as a total one — the second time in this document a partial function has worn a total one's clothes, after (e)'s refusal set. Now tabulated: first non-blank line is the heading, or the declaration is refused; several headings means the first, with the rest ordinary content. Refusing rather than defaulting to an empty title is what makes it total. The rule is *first non-blank line*, deliberately not *first heading anywhere*, which would let prose precede the title and silently promote a third-paragraph heading. |
| F-3 contested: the repair does not prove the `title` wire field is removed | **Conceded** — EX-13(b) asserted the removal and nothing could fail if it were left parsed-but-ignored, which is exactly the serde failure mode `VA-5` already guards for `adopt_record`. Answered with the same instrument: `VA-7` greps the field out of existence. An assertion nothing can falsify was the finding, and a second assertion would not have fixed it. |
| — | Independently, and stronger than the code reading: the landed test over the re-adoption path, `adopt_authored_crosses_divergence_and_rebaselines_alone` (`tests/e2e_design_state.rs:281`), asserts only the watermark and the fingerprint and **never the stored body**. Both its assertions hold while the prose is silently reverted, so the one test covering this path cannot fail on it. This makes `VA-6` empirical rather than predicted. The verify round confirmed the defect independently, by execution. |
| — | **Nothing else moved.** The marker syntax, delimiters, id grammar and its 32-byte bound, the escaping scheme, the five-refusal partition and every (f) measurement stand untouched for a second round. |

**The falsifier, restated for rev 4.** Rev 2 set it against the
read-rather-than-ran class and it did not fire — the round's two contests were
both *incompleteness in a repair*, not false claims about code. So the standard
tightens rather than relaxes: **a third partial function shipped as total means
the remaining answers must each be checked for totality explicitly before this
gate closes.** Two have now been found ((e)'s refusal set, and the title
derivation), both by review rather than by the author.

### Rev 2 — RV-323's first round

Five findings, all verified against the code before disposition, none
contested. Three falsified claims rev 1 asserted about code it had read rather
than run — the projection sketch's F-1 defect class, arriving in a new document
and in the answers to gate questions rather than in a bound.

| finding | disposition |
|---|---|
| F-1 round trip drops terminal body whitespace (blocker) | **Verified.** `render_document` emits `section.body` raw while `authored_sections` `trim_end`s each region, so the loss is real and unrefused — a **tenth** incumbent defect. Repaired at **admission**, not at parse: the ambiguity is structural, because the block separator and a body's trailing blank line are the same bytes, so no parser rule recovers it. Bodies are normalised at declare time and the round trip is the identity on the stored form. Row 10; EX-13. |
| F-2 row 8 is false for space and `>` (blocker) | **Verified, and the correction sharpens the row.** `authored_sections` extracts the whole inter-delimiter substring and trims it, so `sec-a b` and `sec-a>` *do* round-trip at head. Rev 1 conflated an incumbent **defect** with a forward-looking **contract change**. Row 8 now names only the newline, which genuinely cannot be read back; the space and `>` exclusions are justified in (a) as part of the new grammar and no longer claimed as incumbent breakage. |
| F-3 retitle answer relies on a nonexistent projection (blocker) | **Verified, and its evidence exposed worse.** No heading-to-title path exists: `title` and `body` are independent wire fields, `render_document` never emits `title`, `authored_sections` never reads a heading. Repaired by making the heading *be* body and deriving `title` from it — one source, and a retitle now moves the fingerprint, which under rev 1's account it would not have. Following the finding's citation of `adopt_authored` reached **row 12**: re-adoption updates only the fingerprint and `DerivedInput` carries digests without bodies, so a later `materialise` silently reverts a hand edit and re-baselines the watermark over it. Rows 11 and 12; EX-13. |
| F-4 the maximal-charset claim is not derived (major) | **Verified and withdrawn rather than repaired.** `.`, `:`, `/`, `+`, `=`, `@`, `[`, `]` all satisfy the three stated constraints, so `[A-Za-z0-9_-]` is not maximal. The constraints establish a safety *floor*, not an optimum. The charset is now stated as a **choice** — corpus identifier convention, plus widening-is-cheap — which is what the provenance rule demands of a number or a set that nothing derives. |
| F-5 over-bound ids raise `IdTooLong`, not `MalformedId` (blocker) | **Verified.** `ids.rs:75-80` returns `IdTooLong` from the length guard before the prefix is examined; `MalformedId` is the prefix-or-empty-body arm — which is the arm the new charset rule joins. Both refusals are now named precisely, and EX-10(b)'s enforcement is satisfied by the existing guard rather than by anything new. |

**What did not move.** The marker syntax, the delimiters, the id length bound
(32 = `DESIGN_ID_BYTES`, at equality), the escaping scheme and its invertibility
argument, the five-refusal partition, and every (f) measurement. No finding
touched them, and none was re-derived to look busy.

**The trend worth naming.** Rev 1's three factual errors were all of one kind:
a claim about existing behaviour written from reading rather than from running.
The (f) answer, which *was* measured, produced no findings at all — and the one
measurement rev 1 ran changed the grammar. The falsifier for rev 3 follows from
that asymmetry: **a sixth finding of the read-rather-than-ran class means the
remaining incumbent rows must each be converted into an executed test before
this gate closes, not re-read more carefully.**
