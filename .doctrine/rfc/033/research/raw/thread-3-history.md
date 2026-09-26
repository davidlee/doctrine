## Thread 3 — prior attempts

Grouped by surface. Each entry: **id — what it did — status — what it left open.**

### Governance / durable rules that bound the surface

- **ADR-002** — defined the global-orientation (shipped) memory class: `repo=""`, `anchor_kind=none`, scoped, evergreen. **accepted.** Left open: nothing material except the corpus's own coverage, which SL-069 then chased.
- **ADR-005** — fixed the knowledge-tier doctrine: "skills route, reference docs explain", tiered by access pattern. **accepted.** Its R-C1/C3 evidence-bound scoping left the PULL-tier "which verb for what" doc, the restate-line enforcement (R-OQ-4), the PUSH-tier reference-forms block (R-OQ-5), and the `install/*.md` IA **unbuilt** — routed to SL-144/CHR-023.
- **ADR-019** — pulled apart embedding / publication / projection as independent asset policies, and mandated *minimal projection*: a client gets only files that need a stable path. **accepted.** Deferred the trust-acceptance gate to RFC-021 Stage 5, and did not build any client-refresh path (that gap becomes IDE-030).
- **ADR-024** — the shipped-corpus grounding rule: every claim in a shipped asset (`install/`, `memory/`, `plugins/`) must rest on a client-resolvable address (inline prose, `reference/<name>.md`, shipped memory key, skill name, CLI verb, in-corpus relative path); repo-private ids/paths are never admissible at any tier. **accepted.** Delivered during SL-267; left the drift gate (ISS-309 part 2) deferred.
- **RFC-021** — "Dynamic behaviours and minimal projection": decided projection follows semantic ownership, publication is distinct from projection. **resolved.** Explicitly left *storage, trust, protocol, taxonomy, activator mechanisms* to narrower specs/experiments (SPEC-026, PRD-017).
- **DEC-010** — "Published set = full projection complement" (published = `{embedded filenames} − {base backings}`). **proposed, and out of date with SL-227.** SL-242 objective 2 must settle or supersede it.
- **RFC-011** (read only for the tests RFC-017 leans on) — measured that dispatch token cost is *orientation*, not execution (the "L0" cluster), and that the corpus's own stale signposts are a recurring bleed. **open.** Its orientation findings are what RFC-017's "heavyweight?" FAQ answer and stranger-test depend on.
- **RFC-016** — "Zero-rescue dispatch": proposes that gotcha memories migrate into verb diagnostics, and defines the **memory-blind orchestrator test** — a fresh orchestrator with zero dispatch memories completes a standard run. **open.** RFC-017's *zero-lore stranger test* is explicitly its human twin.

### Shipped memory corpus (the "learning surface" proper)

- **SL-069** — shipped-memory-corpus-as-onboarding-anchor: authored 13 memories to fill capability gaps, trimmed the boot Memory section, added `boot --check` populatedness. **done.** Left open a self-updating mechanism (OQ-1) and the Review-kind memory (deferred to SL-068); folded candidate-workflow orientation in as CHR-009.
- **SL-143** — the corpus overhaul: audited all 29 shipped memories, restructured them as a curated onboarding path (overview → concepts → workflow → reference), fixed wikilink reachability, net 32 memories. **done.** Left open the self-correction gate (deferred to IMP-163, needs SL-147's domain map), the full ADR-005 IA audit (→ SL-144), and skipped knowledge/backlog-kind signposts.
- **SL-144 / CHR-023** — ADR-005 full compliance: `install/*.md` IA audit, user-hook contracts, restate-line audit, glossary/using-doctrine currency, reference-doc reachability. **`ready`, 0/5 phases.** Left open: everything it proposes, *plus* a staleness problem — its 2026-07-03 scope reconciliation **predates ADR-019**, and it still treats `boot-footer.md` as a live hook while SL-187 retired it. Its own OQ set is now closed, but the slice is unresolved on sequencing (SL-242's OQ-2).
- **SL-178** — close drift-discharge legibility: richer error, `/close` recipe, and promoted a local recipe memory to a **shipped** one. **done.** Left open broader migration of ~46 local operational memories (IMP-216).
- **SL-200** — canonicalised the shipped corpus's body wikilinks to `[[mem.<type>.<domain>.<subject>]]` form so the extractor resolves them. **done.** Explicitly chose **Path 2 — single link store** (body prose wikilinks are the one expression); deferred a resolver "complain" guardrail as a follow-up.
- **SL-202** — made memory body wikilinks first-class catalog edges (web map). **done.** Deferred clickable focus links (→ IMP-264, later done).
- **SL-187** — prompt-cascade delivery: inlined `onboarding`-tagged memory bodies into the cache-stable boot sector and **retired the `boot-footer.md` round-trip**. **done.** Left full boot-subsumption and on-model-change injection open.
- **SL-267** — shipped-corpus conformance: swept 239 repo-private citation sites, corrected stale CLI claims, dispositioned missing orientation, and authored ADR-024 + DEC-311/315/316. **done.** Left a drift gate deferred (ISS-309 part 2).
- **IMP-488** — resolved by promoting `doctrine search` scope into `reference/using-doctrine.md` (one fact, one home) and the boot digest. **resolved/fixed.**
- **DEC-127 / DEC-313** — objective-6 diagrams ship on the *published, generated* surface; hard-case rationale inlines per-knob whys. **accepted.**

### Reading UI (memory browsing / map explorer)

- **SL-072, SL-073** — the Map Server and its interactive browser frontend. **done.**
- **SL-076** — loaded concept maps (`CM`) into the explorer and shipped a web authoring surface. **done.** Left entity-ref node labels and web creation as follow-ups.
- **SL-201** — `--focus` accepts memory refs; added `doctrine onboard` ("Open the map focused on the onboarding memory (human onboarding entry)"). **done.** This is the *reading-UI* half of RFC-017's concern, already delivered before RFC-017 was written.
- **IMP-264** — clickable `[[mem.…]]` wikilinks in the web map markdown pane, resolved server-side via the existing `links.rs` resolver. **resolved/done.**

### Backlog items (the id list, plus search catches)

- **CHR-021** — audit/improve the shipped corpus (currency, completeness, onboarding, cross-linking). **closed/done**, fulfilled by SL-143.
- **CHR-023** — ADR-005 full compliance chore. **open**, fulfilled-by SL-144; carries the same stale scope.
- **IMP-315** — projected reference docs under `.doctrine/` are stale and unrefreshable post-SL-227; surfaces three options (delete/repoint; restore projection; doctor stale-detection). **open.** SL-242 effectively selects options 1+3.
- **IMP-232** — seed schema carries tags so installs are born with onboarding-tagged orientation. **open but likely moot**: the eager orientation-memory seed is retired (`install/manifest.toml:20-24`), so nothing seeds an onboarding-tagged memory any more. [inferred staleness — I did not find a supersede/close reason.]
- **IDE-020** — seed a project-orientation memory at install. **closed/done** — and then *reversed*: the seed was retired in favour of a projected `project-orientation.md` file (SL-227/REV-031). The idea's outcome no longer matches the product.
- **IDE-030** — client-repo doc currency under write-if-absent install: needs a refresh verb and a divergence detector. **open, undesigned.** SL-144 and SL-242 both name it as the carved-out general form.
- **IMP-154** — `search --all`: index non-entity markdown + a path column. **open.** The widening that IMP-488 left to it.
- **IMP-326** — declare `scope.unobservable` on the non-contributing memory scope entries. **open, blocked on SL-232.** Corpus hygiene, not the reading surface.
- **CHR-078** — name the unified `doctrine show` router in the remaining agent-facing guidance (memories + skills + `install/authority-model.md`). **open.**
- **QUE-172** — "Is the shipped memory corpus published-for-copy?" **answered** — DEC-010 records the answer as **no** (not part of leg-2 projection; the eager install seed is gated, not published) — *but* the record's own facet fields (`answer`, `answered_by`, `answered_on`) are `null`, so the answer lives only in DEC-010, which is itself `proposed`.
- **IMP-163** — wire the SL-143 self-correction gate. **open**, still unbuilt.
- **IMP-164** — seed-onboarding template / project orientation were empty stubs. **closed/done** (template rewritten, orientation filled).
- **ISS-313** — the **published** glossary's knowledge status vocabulary is stale (`pending|active|superseded|withdrawn` vs the CLI's `proposed|accepted|rejected|superseded`). **open.**
- **CHR-079** — `install/review-ledger.md` has drifted from the verb surface; add a doc-to-help consistency check. **open.**
- **SL-242** — retire the projected reference-doc model: re-anchor the corpus on the published model, settle DEC-010, untrack the `.doctrine/` residue, add a doctor citation check, close CHR-043. **proposed.**
- **REV-031** — renamed the base orientation surface to `project-orientation.md`. **done.**

**Search catches the id list missed:** SL-267, ADR-024, DEC-127/311/312/313/315/316, QUE-226, ISS-309/313, CHR-079, IMP-488, SL-265, REV-031, IMP-163/164/264, SL-201/202. **Negative finding (with control):** I enumerated the entire knowledge corpus — 10 ASM, 6 CON, 310 DEC, 29 EVD, 1 HYP, 17 QUE — and **no EVD/ASM/HYP record exists about docs, the memory corpus, or publication**; only DECs and QUEs do. The one HYP (HYP-001) is about `HYP` routing, not docs.

---

## Thread 3 — contradictions and live tensions

1. **`install/manifest.toml` projects `project-orientation.md` as a base backing — does that conflict with SL-242 retiring the projected reference-doc model?**

   **Pro-conflict side:** `install/manifest.toml:11` lists `[base] backings = [".gitignore", "doctrine.toml", "project-orientation.md"]`, and `install/manifest.toml:20-24` says the orientation content "now ships as the projected `project-orientation.md` base backing". SL-242's title is "Retire the projected reference-doc model", and ADR-019's own logic is "everything else is framework-owned and exposed intentionally, not installed by default."

   **Contra-conflict side:** SL-242 is scoped to *reference-doc copies under `.doctrine/`*, not to base backings. It says explicitly that changing the projection/publication split is out of scope — "ADR-019 is accurate and is a premise, not a target" (SL-242 A1) — and its residue inventory lists the nine reference-doc copies plus `agents/`, `templates/`, `hymns/`, `workflows/`, `mod.just`, `doctrine.toml.example`, `rules/AGENTS.md`; `project-orientation.md` is absent, and `.doctrine/governance.md` is explicitly *retained and tracked* as user-owned and boot-read. `project-orientation.md` is the exact analogue: user-editable orientation that must occupy a stable path.

   **Judgement:** **no direct conflict.** SL-242 retires the *projected reference-doc copies*, not projected base backings, and deliberately keeps ADR-019 standing. The genuine tension is one level out: SL-242 vs **SL-144**, which audits the same `install/manifest.toml` + reference-doc surface but was scoped on 2026-07-03, before ADR-019 (2026-07-18). That is SL-242's own OQ-2, unanswered. [inferred: that the two slices' `install/manifest.toml` edits will collide is stated by SL-242, not proven by me.]

2. **RFC-017's founding premise — "There is no human-facing on-ramp" — is partly stale.** RFC-017 was created 2026-07-05. Since then: SL-201 added `doctrine onboard` (a one-liner into the onboarding memory graph), SL-202 + IMP-264 made memory body wikilinks clickable edges in the web map, and SL-227/REV-031 projected `project-orientation.md` as the base orientation surface. So the "no path in" claim is weaker now than when written; what remains is exactly RFC-017's three-fold thesis (why/mental-model/first-success), which none of those delivered. [inferred from dates + `doctrine onboard --help`.]

3. **QUE-172 is marked `answered` but its answer facet is empty.** `doctrine knowledge inspect QUE-172` shows `answer: null, answered_by: null, answered_on: null`; the substantive answer is carried only in **DEC-010**, which remains `proposed` and is acknowledged out of date with SL-227. Two records, neither of which is settled at the altitude its status implies.

4. **`boot-footer.md` is dead but undeleted, and two slices disagree about whose job it is.** SL-187 retired the boot-footer round-trip; SL-144 objective 2 says "`boot-footer.md` is retired" and assigns the deletion to itself; SL-242 says "`boot-footer.md`'s retirement — SL-144 objective 2 owns the deletion". Meanwhile `.doctrine/boot-footer.md` and `install/boot-footer.md` are still tracked (verified via `git ls-files`). The asset is described as dead in three places and removed by none.

5. **IDE-020 (`done`) contradicts the shipped product.** IDE-020's whole outcome is a seeded orientation *memory*; SL-227 replaced that with a projected *file* and `seed_items = []`. A closed idea now describes an architecture the product no longer has.

6. **RFC-017 proposes an in-repo `docs/` home, but the repo's `docs/` is occupied by vendor harness docs.** `docs/` currently contains `docs/claude/**` (Claude Code reference material), not doctrine narrative. README.md opens with "Doctrine is an opinionated but hackable set of tools…" and design goals — not the positioning paragraph RFC-017 wants (OQ-7). Placement is unresolved and the namespace already has a different meaning.

7. **Two overlapping "no repo-private citation" rules.** DEC-127 generalised the rule for `install/` during SL-244; ADR-024/DEC-311 restated it corpus-wide during SL-267. ADR-024 says DEC-127 is "related … not superseded", so the same invariant now has two governance homes with different scopes.

---

## Thread 3 — already-rejected options

An RFC on the learning surface must not silently re-propose these:

- **Web UI / memory corpus as the primary docs.** RFC-017 rejected "expose the memory corpus harder (web UI as the docs)": agent-voiced reference "presumes the mental model it should be teaching". The web UI is a complement, not an on-ramp. (RFC-017, *Alternatives considered*.)
- **A comprehensive manual.** RFC-017 rejected it — the agent-mediated reframe makes most of a manual redundant, and it competes with the CLI-as-source-of-truth discipline.
- **Screencast / video tour.** RFC-017 **deferred** it — "rots faster than an annotated artifact chain".
- **`project-onboarding.md` as the orientation-surface name.** REV-031 considered and rejected it: "onboarding" narrows the standing-orientation role to first-run. (REV-031 rationale.)
- **A second TOML link store for memories (duplicate `[[relation]]` rows).** SL-200 Path 1, rejected; chose Path 2 — body wikilinks as the single link store.
- **A bounded published set (templates + 4 reference docs).** DEC-010 records this was the original decision and was **reversed** after RV-299 X-F1 refuted its completeness; the full projection complement was chosen instead.
- **Restoring projection for the reference tier (IMP-315 option 2).** Rejected — it "re-opens the projection surface ADR-019 deliberately narrowed". SL-242 selects untrack + doctor check.
- **Rewriting every bare `<name>.md` citation to carry `library show`.** SL-242 non-goal — "duplicating it per citation is the parallel-implementation failure"; the boot sector states the rule once.
- **A migration/removal primitive for already-installed clients.** SL-242 explicitly declines to build it; filed as IMP-378 (once-per-repo, version-keyed, isolated from doctrine's own deps).
- **A signpost per verb.** DEC-315 refused it: "the CLI spine is a product surface, not a table of contents, and reproducing `--help` in prose rots."
- **Spec-sibling / duplicated / hand-authored diagrams (objective 6).** DEC-127 refused all three; chose published + generated.
- **"Status quo + better memories", one mega-verb, deleting ref classes.** RFC-016 rejected all three, with reasons recorded.
- **Warm reuse (`SendMessage`) as the default token lever.** RFC-011 demoted it to an experiment.
- **A resolver "complain" guardrail against prefix-less wikilinks.** SL-200 deferred it to a follow-up (ISS-214 extractor).

---

## Thread 3 — what remains genuinely undecided

No artefact answers these:

1. **RFC-017's own OQ-1…OQ-7**, all open:
   - **OQ-1** — which closed slice is the Tour subject (SL-199 candidate; must be closed and representative, not just dramatic).
   - **OQ-2** — Tour format: single long annotated page vs stage-per-page; how much artifact text is inlined vs linked.
   - **OQ-3** — Quickstart's honest floor: minimum viable harness × platform × no-jail environment, and what install must do better to keep the doc short.
   - **OQ-4** — Stranger-test protocol: scripted scenario, pass/fail, cadence, where results are recorded.
   - **OQ-5** — Mental-model page ownership: standalone doc vs promotion into `using-doctrine.md` as a human-voiced preface.
   - **OQ-6** — What a curated "start here" trail needs from the corpus (a tag? an ordered signpost chain?) without forking the memory model.
   - **OQ-7** — README restructure: what moves to `docs/` so the README leads with positioning + Tour link.
2. **Whether the shipped memory corpus is published-for-copy.** DEC-010 says no *for now*, but it is `proposed`/out of date and QUE-172's facet is null; the collection-set decision for the published library is still open.
3. **SL-242's OQ-1/2/3** — which residue paths are derived vs user-overlay; sequencing against SL-144 (which needs a staleness pass first?); whether a shipped `[gitignore]` change can reach already-installed clients.
4. **How the corpus stays current.** IMP-163 (self-correction gate) unbuilt; ISS-313 (stale published glossary) open; CHR-079 (doc-to-help check) open; IMP-309 (doctor: published entries resolve) open. No working currency check exists.
5. **The client-repo currency gap (IDE-030 / IMP-378).** No refresh verb, no divergence detector, no migration primitive; deliberately deferred by SL-242.
6. **RFC-017's relationship to `project-orientation.md`.** Whether the projected orientation surface is part of the human doc set, feeds it, or is out of scope — untouched by any artefact. [inferred: RFC-017 predates the file's naming and projection.]
7. **The status of SL-144 itself.** `ready`/0/5 with a scope reconciliation that predates the flip it audits; whether it gets re-scoped, re-sequenced behind SL-242, or abandoned is unstated.
8. **IMP-232's disposition.** Open, but the seed it targets is retired; no artefact closes or supersedes it.

---

## Limits

- I read via `doctrine show`/`inspect`/`list` and `doctrine search`/`memory search`; I did not open entity TOML/MD directly, and I did not read slice `design.md`/`research.md` bodies (e.g. SL-242's full design, SL-143's audit ledger) — those were out of reach for the `show` surface and may carry OQ resolutions not reflected in the summaries.
- The `project-orientation.md`-vs-SL-242 question is only as sharp as SL-242's text; SL-242 does **not** mention `project-orientation.md`, so my "no direct conflict" is a scope reading, not an explicit statement by the slice.
- IMP-232's staleness is **inferred** from the manifest comment; I found no explicit supersede/close.
- RFC-017 premise-staleness is **inferred** from dates and from `doctrine onboard --help` existing; I did not diff the pre/post-state of the README or the web map.
- I did not verify whether `.doctrine/routing-process.md` is tracked (my `git ls-files` grep surfaced the other copies but not it), so the "nine tracked copies" count is SL-242's claim, not independently confirmed.
- Counts for knowledge records are from `doctrine knowledge list`; the "no EVD/ASM/HYP about docs" finding is a positive-control-checked negative over titles, not over bodies.
