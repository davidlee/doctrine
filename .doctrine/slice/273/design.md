<!-- doctrine:section sec-1 -->
## 1. Design problem

The published reference library (`install/*.md`, read in a client as
`reference/<name>.md` through `doctrine library show`) is now the declared owner of
doctrine's rules and concepts (ADR-005 as revised by REV-067). The corpus has not
caught up in three ways:

- **Citations are unrecognisable.** Skills, templates and published docs cite
  library docs as bare `<name>.md`, the same shape as a slice's own `design.md`
  or a client's `.doctrine/governance.md`. Neither a reader nor a tool can tell a
  library citation from a filename, and nothing checks that one resolves.
- **Ownership is blurred.** `routing-process.md` rides boot whole and also
  carries copies of rules owned by `glossary.md` and `using-doctrine.md`; the
  three duplicate each other; `boot-footer.md` describes a mechanism that no
  longer exists; `shipped-corpus-authoring.md`, the client copy of ADR-024,
  teaches the old citation form.
- **Skills restate what others own** — flag shapes owned by `--help`, concepts
  owned by a library doc.

This design gives library citations one recognisable, checkable form —
`lib:<address>` — makes each rule have one owning doc, and moves the shipped
non-memory corpus onto both.

**Boundary.** In: the library docs, the boot text built from them, skills,
templates, hymns, the `library show` command, a citation resolution check.
Out: shipped memories (RFC-033 S2), human documentation (S3), SL-242's
projection residue, project override of library rules (QUE-228), and
deterministic inlining of citations (IDE-060).

## 2. Current state

- **Resolution.** `src/publication.rs` owns the publication register: a
  `PublicationManifest` admitted from `publication/manifest.toml`, a validated
  `LogicalAddress` (safe relative path), and `Resolver<A: SourceAdapter>` whose
  `resolve(&LogicalAddress)` returns bytes or `UnknownAddress` /
  `BackingSourceMissing`. `doctrine library show <address>` (`src/commands/library.rs`
  `show_with`) parses the argument as a `LogicalAddress` and emits the bytes; a
  `lib:`-prefixed argument is rejected as unknown.
- **Boot.** `src/boot.rs` embeds `routing-process.md` whole as the "Routing &
  Process" section (`SourceKind::Static`). Its reference-docs paragraph
  (`install/routing-process.md:80-88`) tells agents docs are "cited bare as
  `<name>.md`".
- **Checks.** `doctor`'s `prose_cite` scans `.doctrine/**/*.md` for entity ids
  and deliberately skips inline code spans, where `lib:` citations will live.
  No check knows about library citations. Zero `lib:` occurrences exist in
  `plugins/`, `install/`, `src/` or `memory/`.
- **Corpus.** ~120 bare library citations across 26 skills and 20 install
  assets, plus citations in the `doctrine library show reference/…` form
  (research.md, thread 2). Bare basenames collide: `design.md`, `notes.md`,
  `plan.md`, `memory.md` are template addresses and slice artefacts;
  `governance.md` is a library doc and a client file; `AGENTS.md` is an
  integration asset and a repo file.
<!-- doctrine:section sec-2 -->
## 3. The `lib:` citation form

The foundation every later section depends on.

### 3.1 Grammar

A library citation is the marker `lib:` immediately followed by a publication
logical address:

```text
lib:reference/glossary.md
lib:templates/design.md
```

- The marker is one named constant, `LIB_PREFIX = "lib:"`, in
  `src/publication.rs` (STD-001). Stripping it yields exactly the argument
  `LogicalAddress::parse` accepts (ADR-024 form 2).
- **Recognition** (scanner, section 4): the marker preceded by start-of-text or a
  non-alphanumeric character, followed by a maximal run of `[A-Za-z0-9_./-]`.
  Trailing `.`, `,`, `;`, `:` are trimmed, so a citation ending a sentence
  still parses. A placeholder such as `lib:<address>` has no address
  characters after the marker and is not a citation.
- Citations are written in inline code in Markdown (`` `lib:reference/glossary.md` ``);
  the scanner reads code spans and fenced blocks alike — the marker, not the
  formatting, is what identifies a citation.
- No anchors: a section is named in prose after the citation
  (`lib:reference/glossary.md` § reference forms).

### 3.2 Resolution

`doctrine library show` accepts the prefix verbatim: `show_with` strips
`LIB_PREFIX` when present, then parses and resolves as today. A citation
copied from prose resolves with no editing, and the four existing error
classes are unchanged.

```text
$ doctrine library show lib:reference/glossary.md   # same bytes as
$ doctrine library show reference/glossary.md
```

### 3.3 Where it is taught (the bootstrap)

A library doc cannot be the only teacher of how to read the library, so
teaching is split by access tier (DEC-342):

| tier | surface | carries |
|---|---|---|
| push (every session) | the boot onboarding summary (section 5) | the rule: a `lib:<address>` citation is read with `doctrine library show` — nothing is on disk — and a retrieval a skill or reference doc specifies is mandatory, not optional reading (DEC-345) |
| pull | `lib:reference/using-doctrine.md`, new § publication | the model: published vs projected (ADR-019), what the library holds, same-named client files |
| pull (authors) | `lib:reference/shipped-corpus-authoring.md` | writing citations in shipped text; cites `using-doctrine.md` rather than restating it |

The push rule is two or three lines; everything else is reachable through it.
