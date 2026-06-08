# SL-023 Design — Ship knowledge tiers (ADR-005)

Canonical design intent for SL-023. Binding upstream: **ADR-005** (accepted) and
its `inquisition.md` resolutions (R-OQ-1…5, R-C1/C3/C5). This design realises the
three tiers; it does not re-decide them.

## D1 — Decisions (locked)

| id | decision | source |
|---|---|---|
| D1.1 | New operator doc named `install/using-doctrine.md` | user, design Q1 |
| D1.2 | Pull-reference pointers live in the **boot digest + targeted skills** | user, design Q2 |
| D1.3 | Glossary ships by **relocation** `doc/glossary.md` → `install/glossary.md` | R-OQ-1 |
| D1.4 | Restate line is canon: MAY name verb / cite rule by name; MUST NOT reproduce flag syntax, option/enum tables, or storage-tier mechanics prose | R-OQ-4 |
| D1.5 | Embed VT asserts via `install::asset_text` + install-plan `Install` step, not a full FS install | user |
| D1.6 | Frozen `doc/glossary.md` path-citations in SL-020 / spec-012 get a **non-destructive breadcrumb** (new path noted under the stale one); history intact | user |

## D2 — Current vs target

**Current.** `doc/glossary.md` is authoritative but unembedded (`#[folder]` roots are
`install/`, `plugins/`, `memory/`). The boot push digest (`install/routing-process.md`)
carries the storage read-rule + use-CLI guardrail (commit 8206b67) but **not**
reference forms. Six skill sites reproduce CLI flag syntax / option tables /
storage-tier mechanics prose. Five shipped templates + `install/governance.md`
cite `doc/glossary.md` — a path **absent from any client install** (latent dead link).

**Target.** Glossary shipped + relocated + pointed-at. One new pointed-at operator
doc. Reference forms resident in the push tier. The six sites reduced to pointers.
Template/governance glossary paths corrected to the client-visible location.

## D3 — Leg 1: ship the glossary (relocation)

- `git mv doc/glossary.md install/glossary.md`. RustEmbed re-embeds (footgun R-A1);
  `install` copies `install/glossary.md` → client `.doctrine/glossary.md`.
- **entity-model link (R-A3).** `glossary.md:4` reads *"see [entity-model](entity-model.md)
  for the architecture."* `entity-model.md` is an unshipped build-spec → **drop the
  link**, reword to a self-contained sentence. No dangling reference in the client copy.
- **Template/governance path fix.** Replace `doc/glossary.md` → `.doctrine/glossary.md`
  (the client-visible install location) in: `install/templates/{spec-product,spec-tech,
  design,plan}.md`, `install/templates/plan.toml`, `install/governance.md:22`. This
  also repairs the pre-existing client-dead path.
- **Frozen-history breadcrumb (D1.6).** In `.doctrine/slice/020/slice-020.md` and
  `.doctrine/spec/product/012/spec-012.md`, add one line under the existing
  `doc/glossary.md` citation noting the new path; leave the original intact. Other
  frozen occurrences (inquisition.md, design.md of SL-020) are name-citations or
  records — untouched.

## D4 — Leg 2: `install/using-doctrine.md` (new, pointed-at)

A client-facing operator's guide. Unique payload (verified absent from `--help`,
glossary, templates):

1. **Task → verb map.** Intent → which verb (names verbs only; defers exact flags to
   `doctrine <kind> --help`). Covers **ad-hoc operations** — *read an entity* →
   `<kind> show`; *record a durable fact* → `memory record`; *check slice status* →
   `slice list`; *hand-edit a lifecycle status* → edit `*.toml`. **Boundary (R1.1):**
   the phase *sequence* (`slice new → design → plan → phases → execute → close`) is
   routing-process.md's sole charge — this map does not restate it.
2. **Read-via-`show` discipline** (full form). Always read an entity through
   `doctrine <kind> show <ID>` — it synthesizes all tiers; a raw `.md` may be empty by
   design. (PUSH carries only the one-line rule; the rationale + worked failure live here.)
3. **Storage read/write by tier.** Structured→TOML, prose→MD, never queried/derived in
   prose; which tier to hand-edit (e.g. lifecycle `status` in `*.toml`) vs reach for a verb.
4. **Edit-preserving rules.** Immutable ids (`PHASE-NN`, `EN-/EX-/VT-`); append, never
   renumber.
5. **Pointers out:** glossary (vocabulary/ids), `doctrine --help` (exact shapes).

**Invariant:** reproduces no `--help` flag table. Names verbs, points for shapes.

## D5 — Leg 3: reference-forms PUSH delta

- **Append** a compact block to `install/routing-process.md` (one Static asset; the
  read-rule already rides it — R-OQ-5). Content = **rules only**, no tables/examples:
  - entity ids — prefixed, 3-digit zero-padded; cite durable id not membership label;
  - doc-local enums — bare (`OQ-1`, `D1`, `R1`, `Q1`, `C1`);
  - criteria modes — `VT` (test) / `VA` (agent) / `VH` (human).
  - The full tables + examples stay in `glossary.md` (the PULL tier). PUSH holds the
    irreducible rule statements only — deliberately *not* a second full copy (drift guard).
- **Reference-docs pointer line** in the same digest: "Reference docs (read on demand):
  `glossary.md` — vocabulary/ids; `using-doctrine.md` — verbs & hand-editing." Makes both
  reachable from the resident tier (R-A4).
- **Mechanism.** Pure asset edit; flows through `produce(Static)`→`install::asset_text`.
  **No Rust change** to `boot_sequence`.

## D6 — Leg 4: de-dup the 6 named sites (evidence-bound)

Per-site disposition under D1.4. Pointers target `using-doctrine.md` / `glossary.md` /
`--help`.

| site | remove | replace with |
|---|---|---|
| `record-memory:26-27,36-38,76` | `record`/`find` command templates + `--glob/--command/--tag` option table | keep the scope *concepts* (path/glob/command/tag as a model) in prose; pointer to `using-doctrine.md` + `memory record --help` for flags |
| `retrieve-memory:27` | `retrieve --path-scope <f> --command <t> --tag <t>` template | name the verb + pointer |
| `spec-product:246-249` | command block + `--kind functional\|quality` enums | pointer (already cites `--help` at :242 — keep that line) |
| `spec-tech:13` | `--kind functional\|quality` syntax | name the verb + pointer |
| `execute:27,47` + `phase-plan:43` | `doctrine slice phase <ID> PHASE-NN --status …` (3×) | name the transition ("flip the phase in_progress/completed") + pointer to `using-doctrine.md` |
| `canon:25-27`, `inquisition:39-40` | storage-tier mechanics **prose** | thin to a one-line pointer — the rule is now PUSH-resident + in `using-doctrine.md` |

**Not touched** (already clean pointers, MAY-permitted): `slice:37`, `plan:40`
one-line storage-rule references; `slice:22` `slice new "<title>"` create incantation
(minimal, canonical — borderline but kept; revisit only if review insists).

## D7 — Verification

- **VT-1 (embed/ship).** Unit test: `install::asset_text("glossary.md")` and
  `…("using-doctrine.md")` return non-empty; the install plan lists both as `Install`
  steps. (src/install.rs test surface; no FS round-trip — D1.5.)
- **VT-2 (push presence).** Extend the boot integration test (boot.rs ~:1041) to assert
  the rendered snapshot contains the reference-forms rules + the reference-docs pointer.
- **VA-1 (restate line).** The 6 sites carry no flag syntax / option table / tier-mechanics
  prose; pointers present. (Review check, criterion text = D1.4.)
- **VA-2 (no --help dup + reachability).** No shipped doc reproduces `doctrine --help`;
  every pull-reference is pointed-at by boot or a skill.
- Gate: `just check` green; `cargo clippy` zero warnings (bins/lib, not `--all-targets`).

## D8 — Risks & assumptions

- **R-A1 (rust-embed footgun).** A doc edit is invisible until a full crate rebuild:
  `cargo clean -p doctrine && cargo build`, then `./target/debug/doctrine boot`. Never
  trust the stale PATH bin for embed-dependent behaviour.
- **R-A2 (over-ship).** `install/` is flat `*.md` + `templates/` + `rules/`; relocating
  one file ships only that file. Confirm no `doc/`-wholesale embed is introduced.
- **R-A3 (dangling link).** Resolved by dropping the entity-model link (D3).
- **R-A4 (reachability).** Asserted by VA-2 + the D5 pointer line.
- **R-A5 (PUSH/glossary drift).** Two homes for reference forms (compact rules in PUSH,
  full tables in glossary). Mitigated by keeping PUSH to rule statements only; the full
  expression lives once, in glossary.

## D9 — Phasing intent (for /plan)

Natural seams, low-coupling order:
1. **Glossary ship** — relocate, drop link, fix template/governance paths, breadcrumbs,
   VT-1. (Self-contained; unblocks the pointer target.)
2. **using-doctrine.md** — author the doc, VT-1 extension. (Depends on nothing.)
3. **PUSH delta** — reference-forms block + pointer line in routing-process.md, VT-2.
   (Points at 1 & 2.)
4. **Skill de-dup** — the 6 sites, VA-1. (Points at 2 & 3; last so targets exist.)

## R1 — Adversarial review (self, integrated)

- **R1.1 — using-doctrine vs routing-process overlap.** Both are operator-facing; a
  task→verb map risks restating routing-process's "Core process" phase sequence
  (`slice new → design → …`). *Resolution:* the D4 map covers **ad-hoc operations**
  (read an entity, record a fact, check status, hand-edit a status field) — NOT the
  phase sequence, which stays routing-process's sole charge. Boundary added to D4.
- **R1.2 — template path-fix is scope the slice didn't name.** Leg 1's ripple touches
  6 files (5 templates + governance) beyond the 4 legs. It is necessary (else the
  relocation ships dead `doc/glossary.md` links). *Resolution:* in-scope under Leg 1;
  `slice-023.md` affected-surface reconciled to list it.
- **R1.3 — PUSH/glossary duplication = parallel implementation?** Two homes for the
  reference-form rules. *Verdict: sanctioned, not heresy* — the ADR's tiering is the
  whole point; PUSH cannot point-at the forms (that is the pull-squared anti-pattern
  for a rule needed before any skill invoke, e.g. writing a commit ref). Mitigated by
  PUSH = rule statements only; full tables/examples live once in glossary (R-A5).
- **R1.4 — record-memory scope concepts: restate-line violation?** Keeping
  `path/glob/command/tag` as a conceptual model in the skill is **naming concepts**
  (allowed), not reproducing an option table. The scope model is memory-domain
  knowledge whose natural home is the memory skill, not `using-doctrine.md`. Kept;
  only the `--flag <syntax>` form is removed.
- **R1.5 — VT-2 weak (asserts embed contains its own content).** True — it is a
  regression guard (deletion of the block from routing-process.md fails the render
  assertion), not a logic test. Accepted: it is the ADR's required VT and the cheapest
  guard against silent loss. A compactness/size budget assertion is *optional* gold-
  plating — deferred unless trivial at plan time.
- **R1.6 — install walk coverage (plan-time check).** Confirm the install file-walk
  copies `install/*.md` root files (not only known subdirs) so `glossary.md` /
  `using-doctrine.md` reach `.doctrine/`. RustEmbed `#[folder="install/"]` + the
  per-file `dest` join (src/install.rs) indicate yes; verify in VT-1, no manifest
  `[dirs]` change expected (flat root files, not a new entity dir).

No finding blocks planning. R1.1 and R1.2 integrated (D4 boundary + slice reconcile).
