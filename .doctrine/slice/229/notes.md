# SL-229 — working notes

## Harvest

fresh-as-of: `PHASE-02 completed` @ `14a9f9f8`

**Produced**
- `design.md` (D1–D6; internal adversarial pass integrated, 4 findings) —
  commits `1f673ca4`, `1399a647`, amendment in `de03c23f`.
- `plan.toml` + `plan.md` (3 phases, VT mandates) — `535ec090`, `de03c23f`.
- design-target selectors (12) recorded; cli.rs removed post-review (RV-297
  declared-but-undelivered class).
- Dogfood research artefact `research/` (research.md, baseline.toml, raw/) —
  gitignored by design; the design's evidence base.
- IDE-044 storage bullet corrected (D1 supersedes shaping) — `1f673ca4`.
- RFC-011 case-notes entries: `[slice/research dogfood; db8e41f5]`,
  `[plan; db8e41f5]`, `[SL-229-PHASE-01-a1f]`.
- **PHASE-01** (`73b6c29d`, path-limited to 6 source files): `src/research.rs`
  engine leaf (`mint`/`check` over `contentset::diff`; `baseline.toml` serde —
  `slice`/`date`/`[hashes]` over the fixed intent-doc set); `SliceCommand::
  Research { id, restamp }` + thin `run_research` (advisory, always exit 0,
  D6); guard `Write("slice research")`; `contentset::is_stale_against` removed
  (EX-2) with the bool relocated to `SetDrift::is_empty` (now `pub(crate)`,
  consumed by `run_research`); `layering.toml` `research = "leaf"`. VT-1/2/3 ✓,
  VA-1 confirmed; full suite green; `check gate` clean.
- **PHASE-02** (`14a9f9f8`, path-limited to 3 files): `plugins/doctrine/
  skills/research/SKILL.md` new master — trigger-form description; five
  contract items (Shape / Citation forms / Verification discipline / Runner
  deferral + raw capture / Prompt duties); inverse pointers single-sourced;
  ADR-005 restate line held (verb named + when, no flag syntax / baseline
  format); D1 storage mandated (direct in-slice, gitignored) — dogfood's
  stale state-dir prose NOT propagated. `install/governance.md` § Research
  agents socket stub; `install/glossary.md` `research/` contents line
  (research.md, raw/, baseline.toml). Authoring-only per plan — embed ritual
  deferred to PHASE-03. VT-1/2/3 ✓ (post `record-delta --commit 14a9f9f8`),
  VA-1 self-review confirmed; `check gate` clean.

- **PHASE-03** (`64fcc7ad`, path-limited to the 4 skill masters): advisory
  research hooks in `/slice` (`## Next`), `/design` (`### Explore context`, new
  step 2 *before* the `/canon` step — D-b, following items renumbered), `/plan`
  (appended to step 2's re-grep bullet list), `/phase-plan` (new step 4). Each
  1–3 lines, pointer + consumption note, D6 advisory, project-neutral
  (POL-002); inverse pointers left single-sourced in `/research`. `/phase-plan`
  says **"re-stamp the baseline" in prose, not `--restamp`** (D-c: ADR-005
  restate rule, and `phase-plan` is inside `dedup_skills_route_not_restate`'s
  named set). Re-embed via `touch src/install.rs && cargo build`, all four hook
  strings verified in the fresh binary. `check gate` exit 0; VT-1..4 ✓ after
  `record-delta --commit 64fcc7ad`; 10/10 slice VTs green.

**Decisions carried out of PHASE-03** (durable — the phase sheet is disposable)

- **D-a — EX-2/VA-1 read as a contract, not a path.** SL-227 (minimal
  projection, ADR-019) landed *between* this slice's plan authoring and
  PHASE-03 and removed local skills projection entirely: `install --dry-run`
  emits no skill-file rows; claude installs via the plugin marketplace, other
  harnesses delegate to `npx`; `.agents/` survives only as an auto-detection
  probe (`src/install.rs:1570`); `.doctrine/skills/` never existed in this repo.
  The criteria's named paths are dead. Satisfied as the **contract** — the
  harness-visible copy matches the `plugins/` master — with the marketplace
  cache as the claude-side mirror. The embed ritual stays *necessary*
  (`plugins/` is still a RustEmbed root, `src/install.rs:21`) but is no longer
  *sufficient*. Criteria ids are immutable; this is interpretation, not an edit.
  **Selector consequence:** `.agents/skills/**` was `unmatched` under `selector
  doctor` and was removed (`9aedc079`).
- **VA-1 residual → `/audit`.** Cache comparison: `research/SKILL.md` **matches**
  its master (so the tag + `claude plugin update` path is proven end-to-end),
  and the four hooked skills differ by *exactly* PHASE-03's edits and nothing
  else. The only gap is that `64fcc7ad` postdates the `v0.31.0` tag — release
  work, deliberately not done here (user ruling: leave it to audit absent a
  compelling reason). Note `v0.31.0` is contained in `origin/edge` only, while
  `.claude-plugin/marketplace.json` sources `ref: main`.

**Learned** (routed to sinks)
- Pre-existing research-scratch seams (SL-055 gitignore, SL-116
  `Tier::Research`, doctor skip) → design D1 rationale + research.md
  § Cross-thread.
- VT keyword-mandate selection pitfalls; unmarked-row falsification at plan
  re-grep (scout's main.rs parse-test claim) → case-notes `[plan; db8e41f5]`.
- New `mod` requires an ADR-001 `layering.toml [tiers]` entry or the
  `architecture_layering` gate fails `Unclassified` — not flagged in the
  handover terrain → case-notes `[SL-229-PHASE-01-a1f]`.
- Dogfood `research.md` still carries the *pre-design* storage wording
  (`state/research/` + symlink) that D1 **superseded** (direct
  `.doctrine/slice/NNN/research/`, gitignored in place) — a PHASE-02 authoring
  hazard: mandate the D1 shape, not the dogfood's stale storage claim.
  (PHASE-02: hazard avoided; skill mandates D1.)
- verify-vt attribution reads the source-delta registry — a docs-only phase
  still needs `slice record-delta <id> <PHASE> --commit <S>` or its VTs stay
  `Unattributable`. And that verdict's reason string says "keyword present"
  without evaluating keywords (`vtgate.rs:124`) — misleading at red-phase →
  case-notes `[execute; SL-229-PHASE-02-b7c]`.
- **Two shipped memories are stale post-SL-227 and need correction** (found at
  PHASE-03 phase-plan; not yet fixed):
  `mem.pattern.build.jail-binary-for-skill-install` (**high** trust) tables
  `./target/debug/doctrine` as "stale — CARGO_TARGET_DIR redirects elsewhere",
  but SL-156 / ADR-008 D-B1 removed that redirect; in-tree `target/` is the live
  binary (AGENTS.md, and `--version` → 0.31.0 carrying `slice research`).
  A high-trust memory misdirecting binary choice is the more dangerous of the
  two. `mem.pattern.distribution.skills-source-vs-installed` describes
  `.doctrine/skills/` as the gitignored installed copy — mechanism now dead,
  headline claim (source of truth is `plugins/`) still holds.
- **`dedup_skills_route_not_restate`** (`src/install.rs:~2694`) is a live gate on
  shipped skill wording — no flag-syntax fragments, must retain a tier-1/2
  pointer. Named set covers `phase-plan`, `execute`, `canon`, `spec-*`,
  `record-memory`, `retrieve-memory`, `inquisition` — not `slice`/`design`/`plan`.
  It was absent from the PHASE-02 handover's terrain.
- **Boot-snapshot regression observed and repaired** at PHASE-03 start: the
  spine had lost `research`, `graph`, *and* SL-227's `library` — the
  `mem.fact.doctrine.boot-regen-binary-embed-divergence` footgun (a stale-embed
  binary had run `boot`). Regen from `./target/debug/doctrine` restored all
  three. Gitignored runtime state, so cheap — but it silently degrades every
  session's governance context until someone notices.
- `mem.pattern.skills.yaml-frontmatter-colons` sweep recipe false-positived
  (its own `FILENAME": "` separator matched the `: ` grep); memory body
  corrected in place → case-notes ditto.

- **AUDIT** (`RV-306`, reconciliation facet, parent tree `edge`): 7 findings,
  all terminal, **0 blockers**. `check gate` exit 0; 10/10 VTs pass; conformance
  12/12 design-targets delivered, 0 undelivered, 1 genuine undeclared row.
  Two majors, both distribution rather than defect: **F-1** — PHASE-03
  (`64fcc7ad`) postdates `v0.31.0` and is not on `origin/main`, so the four
  consumption hooks are absent from every harness-visible copy (verified by
  grepping the live plugin cache: `research/SKILL.md` present, hook text zero
  matches in all four) → **CHR-048**; **F-2** — the design VH ("one further real
  slice driven through the round") is consequently unevidenced → CHR-048 step 4.
  Operator elected to close with the release tracked rather than hold. Minors:
  **F-3** `.doctrine/adr/001/layering.toml` undeclared → reconciliation brief
  (selector registry first, design § Code impact mirror second); **F-4**
  conformance noise → existing IMP-175 / IMP-292 (SL-229 is the first *solo*
  datapoint); **F-5** no harvest pointer for `research/` at close → **IMP-314**.
  Nits: **F-6** this repo's `.doctrine/governance.md` had no § Research agents
  section for `/research` to point at → **fixed in audit** (pi-scout /
  pi-research, boot regenerated); **F-7** five of nine projected reference docs
  under `.doctrine/` are stale and unrefreshable post-SL-227 → **IMP-315**.
  Synthesis + reconciliation brief in `.doctrine/review/306/review-306.md`.

**Learned at audit**
- **`mem.pattern.distribution.skills-source-vs-installed` corrected** (was flagged
  stale at PHASE-03, now done): the `.doctrine/skills/` installed-copy mechanism
  is dead post-SL-227; the real route is `plugins/` master → RustEmbed → release
  tag → `origin/main` → `claude plugin update` → the harness cache. Re-attested.
  Its sibling `mem.pattern.build.jail-binary-for-skill-install` was already
  corrected earlier the same day.
- **Authored-and-committed no longer implies reachable.** Post-SL-227 there is no
  verb answering "is this authored asset live in a harness?"; establishing it
  during the audit took five probes across four surfaces (`merge-base
  --is-ancestor` vs the tag, `branch --contains` vs `origin/main`,
  `marketplace.json`, the cache dir, `library show`). For any slice whose product
  *is* shipped prose, that question is the audit. Recorded in CHR-048 § Wider
  point and in the corrected memory.
- **A new `src/` module always implies an ADR-001 `layering.toml` entry** — the
  `architecture_layering` gate fails `Unclassified` without it. Predictable
  enough that a design declaring a new module should declare that path as a
  design-target up front (F-3; already in case-notes from PHASE-01).
- RFC-011 case-notes entry `[audit; SL-229-RV306-a4d]`: `slice show` has no
  plan/phases face (forcing a raw-file read against the boot guardrail), `slice
  phase` has no read-only query form, `backlog new` takes a positional title,
  conformance needed hand-adjudication, and distribution is invisible from the
  authored tree.

**Open**
- A1 (prose runner deferral suffices for both arms), A2 (fixed baseline path
  set incl. plan.toml is enough for v1), R1 (advisory hooks may
  under-deliver; escalation = ADR, not skill tweak) — all in design.md § Open
  questions / risks. No DEC/QUE/ASM entities minted; design.md is the record.
  **Post-audit:** A1 is marginally better supported (F-6 filled this repo's own
  runner socket). **R1 is not merely open but untestable until CHR-048 lands** —
  no agent sees a hook, so the RFC-011 eval that was to judge whether harder
  gating needs an ADR cannot run.
