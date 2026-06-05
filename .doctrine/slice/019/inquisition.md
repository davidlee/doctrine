# Inquisition — SL-019 "Backfill Doctrine product-spec corpus"

*Convened upon the design (`design.md`), scope (`slice-019.md`), and metadata
(`slice-019.toml`). The accused is presumed guilty until the code and the
Doctrine acquit it. Verdicts below are marked VERIFIED (the Inquisitor put the
binary, the source, and the tree to the question) or SUSPICION (interrogation
pending).*

---

## Charges

### CHARGE I — The build prerequisite (§7a) names a binary path that is a LIE in this jail · **MATERIAL**

**Doctrine violated:** "use the CLI, don't guess … don't guess … paths"
(CLAUDE.md); the jail contract ("if you need a rw doctrine use the build
target").

**Evidence (VERIFIED):** design.md:153–155 commands the author to:
> 3. **Author with `./target/debug/doctrine`** (carries the new embedded
>    template), not the stale `~/.cargo/bin/doctrine`.

But `cargo metadata` reveals the true target directory is redirected out of the
repo:
```
target_directory: /home/david/.cargo/doctrine-target-jail
```
The freshly-built, new-template binary lands at
`/home/david/.cargo/doctrine-target-jail/debug/doctrine` (strings count of the
new template text "Non-negotiable product/domain stances" = 4 after rebuild).
The repo-relative `./target/debug/doctrine` is a STALE leftover — timestamp
`2026-06-06 01:15:47`, embedding the OLD 5-section template ("The need this
product capability answers" ×2, zero copies of the new). `./target-jail/debug/
doctrine` is likewise stale (0 copies of new text; emits `## Problem`).

**Risk:** An author who obeys §7a literally invokes a stale binary and scaffolds
the OLD `Problem/Value/Principles/Outcomes/Out of scope` template across the
entire corpus — the exact "silent corpus-wide defect" §7a claims to prevent. The
prerequisite that exists to stop the defect *causes* it.

**Sentencing:** Replace the literal path with a derived one: instruct the author
to resolve the binary via `cargo metadata --format-version=1` (the
`target_directory`) or invoke through `cargo run -- spec …`. Add a hard entry
check: scaffold one throwaway product spec and grep `spec show` for `## 1.
Intent` (new) vs `## Problem` (stale) BEFORE authoring any real spec. Burn the
stale `target/` and `target-jail/` copies from the author's PATH expectations.

---

### CHARGE II — §7a's remedy (`commit template + cargo build`) is INSUFFICIENT: rust-embed does not rebuild on a lone asset edit · **MATERIAL**

**Doctrine violated:** correctness first and last (CLAUDE.md); a "blocking"
prerequisite that does not actually unblock.

**Evidence (VERIFIED):** design.md:147–148 correctly cites
`rust_embed` + `#[folder = "install/"]` (src/install.rs:17–19) and the
`debug-embed` feature (Cargo.toml:74: `rust-embed = { version = "8", features =
["debug-embed"] }`). But there is **no `build.rs`** (`ls build.rs` → not found)
and therefore **no `cargo:rerun-if-changed=install/`**. The Inquisitor edited
only `install/templates/spec-product.md`, ran `cargo build` — it reported
`Finished` and the binary STILL embedded the OLD template (`spec show` → `##
Problem`). Only after forcing recompilation of the embedding crate (`touch
src/install.rs`) did the new template embed (strings count → 4, `spec show` → `##
1. Intent`). A plain `cargo build` after a lone template edit is a no-op for the
embedded asset.

**Risk:** §7a step 2 ("`cargo build`") silently fails to re-embed. The author
believes the prerequisite satisfied, scaffolds with a binary still carrying the
old template, and the corpus-wide defect ships anyway. Worse than CHARGE I
because it bites even the author who finds the *correct* binary path.

**Sentencing:** §7a must mandate a rebuild that actually re-embeds: `touch
src/install.rs && cargo build` (force the embedding crate), or `cargo clean -p
doctrine && cargo build`, or `cargo run`. Then VERIFY by the `spec show` grep in
CHARGE I — never trust `Finished` as proof of embed. Consider a follow-up note:
the absence of `rerun-if-changed` for `install/` is a latent footgun for every
template edit, not just this slice (candidate `/record-memory`).

---

### CHARGE III — Scope's §7a premise "the template edit is currently uncommitted" is TRUE, but the design's framing rests on quicksand the slice does not control · **MINOR**

**Doctrine violated:** the storage rule's authored/runtime boundary; ask-don't-infer.

**Evidence (VERIFIED):** `git diff HEAD -- install/templates/spec-product.md`
shows the 8-section rewrite as an **uncommitted working-tree modification** ( ` M`
in `git status`). The committed HEAD (and the design commit `ec7a5ae`) both still
carry the OLD 5-section template (`git show ec7a5ae:install/templates/
spec-product.md` → `## Problem/Value/Principles/Outcomes/Out of scope`). So the
design's repeated assertion that "the product template was restructured into
eight sections" (design.md:6, slice-019.md:5–8) describes an **uncommitted,
unreviewed, un-inquisitioned edit** — the eight sections exist only in the dirty
working tree. The design treats it as settled fact ("the fixed target",
slice-019.md:50) while it is in truth unlanded.

**Risk:** The whole corpus shape is pinned to an artifact that has not passed any
gate. If the 8-section template is itself wrong (see CHARGE V on the §4 line),
the corpus inherits the flaw. SL-019's non-goal "Re-editing the template"
(slice-019.md:49–51) forecloses fixing it mid-slice.

**Sentencing:** Confirm with the User that the 8-section template is the
sanctioned final shape and commit it (with the reworked skill) as the literal
first act of PHASE-02 entry, BEFORE the build step — as §7a step 1 already says,
but elevate it: a dirty uncommitted template is not a "fixed target." If the
template needs any further edit, that is a scope renegotiation, not a silent
working-tree change.

---

### CHARGE IV — The skill's three D-2 conflicts are REAL and correctly diagnosed — acquittal, recorded for the record · **NO CHARGE (ACQUITTED)**

**Evidence (VERIFIED):** design.md:60–64 alleges three conflicts in
`plugins/doctrine/skills/spec-product/SKILL.md`. All three hold:

1. **§4 prescribes prose FR/NFR rows** — SKILL.md:106–126 literally prints
   `### Functional Requirements` / `- FR-001 — The system must …` and
   `- NFR-001 — …` as markdown prose rows. VERIFIED collision with the REQ-entity
   model.
2. **Internal contradiction** — SKILL.md:234 commands "Do not duplicate canonical
   requirement content into narrative prose when it belongs as a requirement
   entity," directly contradicting its own §4 prose example above. VERIFIED.
3. **Label drift `NFR-` vs `NF-`** — SKILL.md:114–115 emit `NFR-001`/`NFR-002`;
   the code's `label_prefix` (src/spec.rs:497–502) returns `"NF"` for
   `ReqKind::Quality`, and a live `spec req add … --kind quality` produced
   `NF-001` (not NFR-). VERIFIED.

The design earns full acquittal on this count. The diagnosis is exact.

---

### CHARGE V — The composition decision D-1 is SOUND and the "double Requirements heading" claim is empirically TRUE — but §4 of the *template itself* still invites the prose-FR heresy · **MATERIAL**

**Doctrine violated:** "no parallel implementation" / no double-storage
(doc/spec-entity-spec.md § Diagnosis "Self-inflicted drift").

**Evidence (VERIFIED):** The Inquisitor built the new-template binary and put it
to the question on a throwaway root:
- `spec show` of an empty-members product spec emits `## 1. Intent` … `## 4.
  Requirements` (line 15) … `## 8. Open Questions` (line 27) AND a synthesized
  `## Requirements` (line 30) — the **double heading** design.md:44–46 / 196–197
  predicts. VERIFIED, and it is coherent (empty synthesized section is a bare
  heading, no crash).
- `spec req add` produced `FR-001 (REQ-001)` and `NF-001 (REQ-002)`; `spec
  validate` → `corpus clean`. VERIFIED that prose §4 + entity requirements
  coexist without a validate failure.

D-1's logic (FR/NF as entities only; §4 prose = constraints/invariants) is
correct AND matches `spec show`'s synthesis order (src/spec.rs:324–423,
verbatim-prose-then-synthesized-Requirements). **However:** the new template's §4
body line reads (install/templates/spec-product.md:12–13):
> ## 4. Requirements
> Functional requirements, non-functional requirements, constraints, and
> invariants.

This template **instructs the author to put functional and non-functional
requirements in the prose §4** — the precise double-storage D-1 exists to kill.
D-1 lives only in design.md and the (to-be-)reworked skill; the **template the
author scaffolds from contradicts D-1 on its face.** An author following the
scaffold, not the skill, reintroduces the heresy.

**Risk:** The corpus double-stores FR/NF (once as prose §4 rows, once as
entities) — exactly `doc/spec-entity-spec.md`'s "Self-inflicted drift" pathology
(lines 55–58). `spec validate` will NOT catch it (it checks FK integrity, not
prose content — VERIFIED: validate passed with both present).

**Sentencing:** Either (a) the slice must reword template §4 to
"Constraints & Invariants" / strip the FR/NF mention — but that collides with the
non-goal "Re-editing the template" (slice-019.md:49–51), forcing a scope
decision via `/consult`; or (b) make D-1 a *blocking authoring rule in the
reworked skill* AND have the exemplar (PHASE-02) demonstrate an FR/NF-free §4, so
every fan-out author copies the right shape. Surface this template-vs-D-1
collision as an explicit open question — it is currently SILENT in §10.

---

### CHARGE VI — The reservation-contention concurrency claim (§6) is VERIFIED accurate · **NO CHARGE (ACQUITTED)**

**Evidence (VERIFIED):** design.md:124–127 claims `spec req add` reserves
`REQ-NNN` via an atomic `mkdir` claim with a bounded retry loop in
`entity.rs allocate_fresh` / `MAX_CLAIM_RETRIES`, lost races recompute-and-retry,
sole failure mode is retry exhaustion. Confirmed:
- src/entity.rs:23 `const MAX_CLAIM_RETRIES: u32 = 128;`
- src/entity.rs:268 `for _ in 0..MAX_CLAIM_RETRIES {` (the loop in `allocate_fresh`)
- src/entity.rs:50–51 the `mkdir` claim: `fn claim(…) { match fs::create_dir(claim) … }`
- src/entity.rs:300 `bail!("Could not reserve an id after {MAX_CLAIM_RETRIES} attempts")`

The claim is accurate. The mitigation (cap fan-out width; ~6 specs) is
proportionate. Acquitted.

---

### CHARGE VII — Storage-rule compliance of §4 taxonomy: ACQUITTED in intent, but "not committed" is UNENFORCED · **MINOR**

**Evidence (VERIFIED):** design.md:81–84 and slice-019.md:84–85 declare the
taxonomy + source-map "disposable runtime context (handover / phase sheet) …
not a persisted authored artifact." This is the correct storage tier — taxonomy
is derived/scaffolding, not authored canon. No queried/derived data is committed
in the design prose itself (the candidate list at design.md:92–95 is design
*reasoning* — "~7, to confirm in PHASE-01" — not a derived index; legitimate).

**Risk (SUSPICION):** Nothing structurally prevents an author from writing the
taxonomy into a committed `doc/*` or slice file. The discipline is asserted, not
gated. Minor — the convention is clear and the author is trusted — but worth a
verification step.

**Sentencing:** At PHASE-05, grep the committed diff for a taxonomy/source-map
file under `doc/` or `slice/019/` and reject it if found. State the taxonomy
lives in the (gitignored) phase sheet / handover only.

---

### CHARGE VIII — Phase ordering is SOUND; reconcile-before-fan-out holds · **NO CHARGE (ACQUITTED)**

**Evidence (VERIFIED by reasoning against the design):** §8 orders
taxonomy(01) → exemplar(02) → reconcile-skill(03) → fan-out(04) → validate(05).
design.md:165–166 correctly insists PHASE-03 (skill rework) "must precede
fan-out so agents read corrected guidance." The dependency is real: fan-out
authors consume both the exemplar (02) and the reconciled skill (03). One latent
ordering note (SUSPICION): §3 calls the skill rework "**exemplar-driven**" and
"Point at the locked exemplar" (design.md:69, 77) — so PHASE-03 *depends on*
PHASE-02's exemplar being locked. That is the stated order (02 before 03), so it
holds; but the phase plan must make 03's entry condition "exemplar locked"
explicit, else 03 could start against an unlocked bar.

**Sentencing:** In `/plan`, set PHASE-03 entry criterion = "PHASE-02 exemplar
accepted/locked." Otherwise no charge.

---

### CHARGE IX — §10's coverage-table open question is well-raised but the resolution is left DANGLING into the exemplar · **MINOR**

**Evidence (VERIFIED):** design.md:190–194 flags that the skill's §7 coverage
table (`| Requirement | … |`, SKILL.md:179–183) referencing mobile `FR-`/`NF-`
labels in committed prose is "fragile under relabel" — a genuine and correct
concern (labels are mobile per doc/spec-entity-spec.md:123–130; durable id is
`REQ-NNN`). The design defers resolution to PHASE-02 ("likely reference
behaviour/durable id, or omit"). This is a real, correctly-identified fragility.

**Risk:** "Resolve in the exemplar" with three candidate answers ("likely … or …
or omit") is not a decision — it is a deferred decision wearing a decision's
clothes. The fan-out (PHASE-04) inherits whatever the exemplar happens to do; if
the exemplar author picks "reference `FR-` labels," every spec bakes the
fragility in.

**Sentencing:** Either decide now (the durable `REQ-NNN` is the only relabel-safe
referent; coverage@v1 is deferred per doc/spec-entity-spec.md:106 so *omitting*
the per-requirement table is defensible) or make it a hard PHASE-02 gate with a
single recorded answer the fan-out must follow. Do not let three options ride
into the exemplar.

---

### CHARGE X — `spec show` identity-line format claim (§2) is ACCURATE · **NO CHARGE (ACQUITTED)**

**Evidence (VERIFIED):** design.md:24–28 sketches the `spec show` head as
`` `PRD-NNN` — <title> `` then flat fields then verbatim prose then synthesized
`## Requirements`. Live output:
```
`PRD-001` — Slices
slices · draft · product

# PRD-001: Slices
…
## Requirements
### FR-001 (REQ-001) — lifecycle ops
```
Matches src/spec.rs:339 (`` `{canonical_ref}` — {title} ``), :340–345 (flat
fields), :372–377 (verbatim body), :380–388 (synthesized headings
`### {label} ({req_ref}) — {title}`). Acquitted.

---

## Questions (interrogatories)

1. **Is the 8-section template the User's sanctioned final shape?** It is
   uncommitted and un-inquisitioned (CHARGE III). The entire corpus pins to it.
2. **What is the canonical author binary in this jail?** §7a's
   `./target/debug/doctrine` is stale; the real build is
   `/home/david/.cargo/doctrine-target-jail/debug/doctrine` (CHARGE I). How will
   the phase plan resolve it portably?
3. **How is the rust-embed re-embed proven?** Given `cargo build` alone does not
   re-embed a lone template edit (CHARGE II), what is the mandated rebuild +
   verification ritual?
4. **Does template §4 stay "Functional/non-functional requirements …"?** That
   line invites the prose-FR double-storage D-1 forbids (CHARGE V). Reword the
   template (violates a non-goal) or override it in skill + exemplar?
5. **Coverage table: `REQ-NNN`, `FR-`/`NF-`, or omit?** Decide now or gate it in
   PHASE-02 (CHARGE IX).
6. **Fan-out execution mechanism** (Workflow vs serial `/execute`) — design.md:189
   defers to `/phase-plan`. Acceptable to defer, but note the Workflow harness's
   own readiness is an external dependency the slice does not control.

---

## Pronounce Judgement

The design is **substantially sound and largely TRUE to the code.** The
Inquisitor put every load-bearing technical claim to the question and the design
emerged with honour on most counts: the `spec show` synthesis order (src/spec.rs:
324–423), the `FR-`/`NF-` labels (src/spec.rs:497–502), the
`--kind functional|quality` surface, the `MAX_CLAIM_RETRIES = 128` /
`allocate_fresh` / `mkdir`-claim concurrency story (src/entity.rs), the
double-`Requirements`-heading, and all three skill D-2 conflicts — **all
VERIFIED accurate.** D-1's composition logic is correct. This is rigorous,
code-grounded design, not hand-waving.

But it is **not yet clean enough to fan out blind.** Two MATERIAL build-mechanism
defects (CHARGES I, II) mean an obedient author may scaffold the entire corpus
with the OLD template and never know — the precise silent corpus-wide defect §7a
was written to prevent. And one MATERIAL composition gap (CHARGE V): the very
template the author scaffolds from instructs them to write FR/NF as prose §4
rows, contradicting D-1, and `spec validate` will not catch it.

**Verdict: proceed to /plan — YES, conditionally.** The architecture is locked
and trustworthy. The conditions are narrow and all live in PHASE-02 entry /
PHASE-03: (a) correct the build prerequisite (real binary path + a rebuild that
re-embeds + a `spec show` grep gate), (b) resolve the template-§4-vs-D-1
collision (reword or skill-override + exemplar demonstration), (c) commit the
template+skill before authoring. These are plan-time fixes, not design teardown.
No BLOCKING heresy. The design is good bones with two cracked load-bearing
prerequisites — mend them in /plan and proceed.

---

## Sentencing (ordered corrective actions)

1. **[MATERIAL · CHARGE I+II] Rewrite §7a's build prerequisite.** Replace
   `./target/debug/doctrine` with a derived path (`cargo metadata` →
   `target_directory`, or `cargo run -- spec …`). Mandate a re-embedding rebuild
   (`touch src/install.rs && cargo build`, or `cargo clean -p doctrine`).
   - *Verify:* scaffold a throwaway product spec; `spec show` MUST contain
     `## 1. Intent` and MUST NOT contain `## Problem`. Make this the literal
     PHASE-02 entry gate.

2. **[MATERIAL · CHARGE V] Resolve template §4 vs D-1.** Decide (via `/consult`
   if it touches the "no template re-edit" non-goal): reword template §4 to
   "Constraints & Invariants," or override in the reworked skill + demonstrate an
   FR/NF-free §4 in the exemplar. Add it as an explicit §10 open question now.
   - *Verify:* the exemplar's `spec-001.md` §4 carries NO `FR-`/`NF-` prose rows;
     all FR/NF are entities (`spec show` shows them only under synthesized
     `## Requirements`).

3. **[MINOR · CHARGE III] Commit the template + reworked skill** as the first act
   of PHASE-02 entry, after User confirmation that 8 sections is final. A dirty
   working tree is not a "fixed target."
   - *Verify:* `git status` clean for `install/templates/spec-product.md` and the
     SKILL.md before any spec is scaffolded.

4. **[MINOR · CHARGE IX] Decide the coverage-table referent now** — `REQ-NNN` or
   omit (coverage@v1 deferred). Record the single answer; do not ride three
   options into the exemplar.
   - *Verify:* the exemplar follows the recorded choice; the reworked skill's §7
     matches it.

5. **[ACQUITTAL · CHARGE IV] Execute the D-2 skill rework** (the three conflicts
   are real): kill the prose-FR/NFR example (SKILL.md:106–126), fix `NFR-`→`NF-`
   (SKILL.md:114–115), remove the internal contradiction vs SKILL.md:234, point
   at the locked exemplar.
   - *Verify:* `grep -n 'NFR-' SKILL.md` returns nothing; no `### Functional
     Requirements` prose-row block remains.

6. **[MINOR · CHARGE VII+VIII] Gate the storage rule and phase entry in /plan.**
   PHASE-03 entry = "exemplar locked." PHASE-05 = grep the committed diff for a
   taxonomy/source-map artifact and reject it.
   - *Verify:* no taxonomy file committed under `doc/` or `slice/019/`; PHASE-03
     does not start before PHASE-02 acceptance.

7. **Harvest a memory** (post-slice): rust-embed `debug-embed` + no
   `rerun-if-changed` for `install/` ⇒ a lone template edit is invisible until the
   embedding crate is forced to recompile. A footgun for every future template
   change.

---

## Disposition (by the slice author, 2026-06-06)

Verdict accepted: proceed to `/plan`, conditions folded into the design.

| Charge | Sev | Disposition |
|---|---|---|
| I — build path is a lie | MATERIAL | **ACCEPT (re-verified).** `cargo metadata target_directory = /home/david/.cargo/doctrine-target-jail`; `./target-jail/` + `~/.cargo/bin` are stale. Snapshot shifts (a concurrent agent is building). Fixed §7a: derive path via `cargo metadata` or `cargo run`; the `spec show` grep gate is the real guard, not any path. |
| II — `cargo build` doesn't re-embed | MATERIAL | **ACCEPT (re-verified):** no `build.rs`, `debug-embed`. §7a now mandates `touch src/install.rs && cargo build` + grep gate. Memory harvest queued (§11). |
| III — uncommitted template | MINOR | **ACCEPT.** §7a step 1 elevated: commit template+skill as first act of PHASE-02 entry. |
| IV — D-2 skill conflicts real | ACQUITTAL | Noted; the rework executes all three fixes (prose-FR kill, `NFR-→NF-`, contradiction). |
| V — template §4 line invites prose-FR | MATERIAL | **ACCEPT + DECIDED (user):** reword the one §4 guidance line (option a) → "Constraints and invariants. (Functional/quality requirements are `REQ` entities — add via `spec req add`.)" Sanctioned one-line edit; lands at PHASE-02 entry. Non-goal updated. |
| VI — concurrency claim | ACQUITTAL | Re-verified: `MAX_CLAIM_RETRIES=128`, `allocate_fresh`, `mkdir` claim. |
| VII — taxonomy not gated | MINOR | **ACCEPT.** PHASE-05 grep gate added (§11). |
| VIII — phase ordering | ACQUITTAL/MINOR | **ACCEPT.** PHASE-03 entry = "exemplar locked" (§11). |
| IX — coverage-table dangling | MINOR | **ACCEPT + DECIDED:** no per-requirement coverage table (coverage@v1 deferred); references use durable `REQ-NNN`, never labels (§10). |
| X — identity-line format | ACQUITTAL | Re-verified accurate. |

Net: 0 design teardown. Two user decisions outstanding before `/plan` locks:
CHARGE V (template §4 reword y/n) and confirmation the 8-section template is final.

> **HERESIS URITOR; DOCTRINA MANET**
