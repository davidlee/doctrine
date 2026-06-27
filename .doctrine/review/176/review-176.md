# Review RV-176 — design of SL-166

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

External adversarial pass on `SL-166/design.md` (three dispatch corpus-loss
guards g1/g2/g3), conducted before design lock. Internal pass already resolved
its findings (design §10); this Inquisition re-tries the design intent with a
hostile external reviewer (codex / GPT-5.5). The accused has confessed its own
weak points in §10 — the Inquisition presses on them and hunts the unconfessed.

**Doctrine the accused is held to.** ADR-012 (dispatch integration topology, esp.
D4 FF-only/CAS contract), ADR-001 (module layering: leaf ← engine ← command, no
cycles), ADR-006 (worktree posture / fork-base ladder), POL-002 (platform
independence from host-project convention), STD-001 (no magic strings), the
pure/imperative split, the behaviour-preservation gate, and the scope locked in
RFC-005 §H6 OQ-7/8/9 + slice-166 Non-Goals.

**Lines of interrogation.**

1. **D6 (apex charge).** Does g3 — an *added refusal* to integrate when a leg
   would clobber `.doctrine/**` — alter ADR-012 D4's FF-only / 3-arg-CAS
   contract, demanding a mechanism-only ADR-012 Revision? Or is it purely
   additive (refuse-more, relax-nothing)? The accused leans "additive, no
   Revision" (§7 D6, §10). Press hardest: would a hostile reader see g3 as
   *redefining* what a legal advance is — and thus the integration contract?

2. **g3 `base = merge-base(new, cur)` correctness** across both legs — the
   pure-ref (not-checked-out integration buffer, CAS-and-done) and the
   checked-out (authoring ref, `merge --ff-only`) advance. Is the claim "a true
   FF advance can never clobber" (design §5.4, §10-A) sound for *both* legs? Does
   `merge-base(new,cur)` equal the leg's actual CAS expected-old (§5.5 assumption)?

3. **g2-strict false-positive surface** beyond "promotion ritual not followed"
   (§10-B accepts the strictness). Any *legitimate* topology — first dispatch
   before any corpus, divergent corpus history, shallow/grafted clone, a corpus
   commit on the buffer but not the authoring ref — where `is-ancestor(corpus_tip,
   base)` refuses a setup that should proceed?

4. **ADR-001 layering** — g2 makes `worktree::coordinate` reach `DispatchConfig`
   / `dtoml`. Does that close an engine→config (or leaf→command) cycle? The
   accused offers a values-not-loader escape (§10) — is it sufficient, or is the
   edge direction itself heresy?

5. **g1 HEAD resolution** — `current_branch(root)` must read the *invoking
   worktree's* HEAD, not the common dir (§10). Verify against the existing
   raw-evidence-ref guard (`dispatch.rs:1067`).

6. **Scope discipline.** The raw-git boundary (manual `git merge` / `fetch
   edge:main`) is a deliberate Non-Goal (slice §Non-Goals, design §8 R1). The
   Inquisitor must NOT expand scope into it — but must confirm the design does
   not *pretend* to close it (honest naming vs false closure).

7. **Unconfessed heresy.** Magic strings (STD-001), silent error handling (the
   `degrade-to-no-op` valves in g2 — do they swallow real failures?), the global
   `--allow-corpus-clobber` across both legs (§10 accepts), OQ-3 (`admit` in the
   g1 verb set), and any invariant left unstated or untested (§9).

## Synthesis — verdict

**Five heresies confessed and burned; the apex charge acquitted.** The accused's
*mechanisms* (g1 refuse-on-buffer, g2 base-corpus freshness, g3 3-way clobber gate)
are sound and survive scrutiny. The taint lay in the **narrative around them** — a
design that misstated the current `integrate` contract and then built its central
rationale on that false model. The corpus-loss guards themselves stand.

**Closure story.** Two blockers, two majors, one minor — all terminal (verified):

- **F-2 (blocker, design-wrong).** §10-A's "both advance legs FF-only today;
  `advance_pure_ref` CAS-checks `is_ancestor`" was false witness, refuted in the
  source: `advance_pure_ref` (`dispatch.rs:1821`) is plain `update_ref_cas`,
  `plan_edge_row` (`:1990`) is explicitly *not* ff-gated. g3 is therefore
  **load-bearing on the `--edge` leg today**, not mere RFC-006 insurance. The whole
  "g3 catches nothing today" defence collapsed; §2/§5.4/§5.5/§9/§10-A corrected and
  the ship decision *strengthened*, not weakened.
- **F-1 (blocker, design-wrong).** The PRIMARY guard silently disabled itself on a
  typo'd `authoring-branch` — the precise silent-catastrophe shape (ISS-056) the
  slice exists to kill. Now fail-closed: a set-but-unresolvable ref refuses setup;
  `last_corpus_commit` made tri-state (§5.2, R4, §9).
- **F-3 (major, design-wrong).** R3 overstated the `Ok(None)` valve; the posture
  contract is now an explicit single linear append-mostly authoring ref, with
  divergent/shallow/multi-branch named unsupported and buffer-only corpus named a
  false negative.
- **F-4 (major, design-wrong).** g1's guarded set was internally inconsistent
  (listed a non-buffer-mutating verb). Principle stated (guard only verbs that
  advance `deliver_to`/`edge`); OQ-3 reduced to a /plan enumeration detail.
- **F-5 (minor, fix-now).** `slice-166.md` reconciled from the abandoned Model A
  (absolute corpus tip) to the locked Model B (relative `merge-base(new,cur)`).

**D6 — the apex charge acquitted.** The Inquisition pressed hardest on whether g3
alters ADR-012 D4's FF-only/CAS integration contract (→ a mechanism-only Revision).
It does **not**: g3 *narrows acceptance* (refuses some CAS-eligible advances) but
relaxes and redefines nothing — no force, no auto-resolve, still 3-arg CAS,
report-never-resolve intact. ADR-012 specifies advancement *shapes*, not a guarantee
that every mechanically-legal CAS move be admitted. **No ADR-012 Revision required.**
The design's D6 lean is vindicated by the external pass.

**Cleared without taint.** ADR-001 layering (`worktree::coordinate` does not import
`dispatch_config` today; config is a leaf, no cycle; values-not-loader guidance
retained). g1 HEAD locality (`current_branch` → `symbolic-ref --short HEAD`,
worktree-local, correct). Raw-git boundary honestly named as open (§8 R1), no false
closure — scope discipline held; the Inquisitor did not expand into the deliberate
Non-Goal.

**Standing risks carried forward (not heresy, watch in /plan).** (1) g3's
catastrophe-path cost (4816-blob clobber set) — R2's batched `diff --name-only` +
capped render must be honoured in implementation. (2) The global
`--allow-corpus-clobber` across both integrate legs — accepted imprecision (§10),
note in verb help. (3) The g1 exact-verb enumeration and the g2 `coordinate()`
config-threading edge direction both confirm mechanically in /plan.

**Sentence.** The design, corrected, is **lock-ready**. `doctrine slice status 166
plan` → `/plan`. The corpus shall not be silently consumed again.

> **HERESIS URITOR; DOCTRINA MANET**
