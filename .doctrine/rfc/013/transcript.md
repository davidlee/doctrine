[claude opus transcript]:

❯ Persisted

  - notes.md (committed aa3487c0) — durable decision log: the full reframing
  (cargo-fmt inversion, doctrine check/SL-163 seam, POL-002 content-split
  table), locked D1, open Q2/Q3, key code anchors (install.rs:895/910/1713/44),
  constraints, relevant memory, split-offs.
  - design.md (committed) — partial: D1 locked above the line; D2/D3
  provisional below, clearly marked PROVISIONAL / pre-lock.
  - design-target selectors — install/hymns/role/worker.md,
  install/hymns/model/deepseek/_default.md, src/install.rs (the certain set;
  stage/* held pending Q2).
  - handover.md (gitignored) — start-here for the resuming agent.
  - Tree clean on edge; handover.md correctly untracked.

  State snapshot

  - SL-191 at design, mid-clarifying-loop.
  - Locked: D1 — model band arm-asymmetric by data (def's declared model),
  never a baked arm→model rule.
  - Open: Q2 (asked, awaiting answer) — model-from-frontmatter +
  likely-drop-stage-band; Q3 — import-gate shape (not yet raised).
  - Commits: 9524c20e scope · aa3487c0 design/notes · acc0af61 ISS-206
  (split-off).

  ---
  Where we are. Worker-contract hardening rides existing seams (cascade hymns +
  install.rs bake + doctrine check cadence). D1 locked: model band
  arm-asymmetric by data (def's declared model), never a baked arm→model rule.
  Two forks remain: Q2 (bake's source of truth + whether a stage band exists)
  and Q3 (import gate shape).

  ---
  Q2 — where the bake learns each def's (model, stage), single source.

  Today the marker {{ prompt resolve --role worker }} resolves role band only —
  model/stage forced None. To land the new model-keyed content in a baked claude
  def, the bake must widen its ContextVector. Two sub-decisions:

  (a) Model source. Recommendation: read the band from the def's frontmatter
  model: — the same field the harness's own model selector reads, so the
  resolved band can never drift from the model the worker actually runs. The
  marker stays a bare sentinel meaning "resolve my full worker context."
  Rejected alt: a parametric marker carrying --model … — two sources of truth,
  drift risk.

  (b) Stage band — exist or not? Recommendation: no stage band. The
  hermetic-fixture and component-anchored-path directives are worker-role
  conduct, not phase-stage conduct — fold them into role/worker, drop --stage
  from the bake, ship no install/hymns/stage/*. Smallest coherent change,
  matches band semantics. Reconsider only if you want per-stage (execute vs
  dispatch) content that genuinely differs — I don't see any.

  Net if you confirm both: widen resolve_worker_role_body to set model from
  frontmatter + bands: Only([Role, Model]); no stage threading.

  Note the slice-191.md scope still lists install/hymns/stage/* as affected
  surface — that predates this recommendation; confirming (b) prunes it.

  ---
  Next after Q2: I raise Q3 (import-gate: reject unformatted/lint-red delta vs
  auto-check+re-import), present D2/D3, finalize design.md, adversarial pass,
  then slice status plan.

  Your call on Q2(a) and Q2(b) — free-text, reframe if the question's wrong.

  [SL-191 design]: resumed clarifying-loop; re-put Q2 (model source + stage
  band)

✻ Cooked for 39s

❯ read the SL-186 design

● Read. SL-186 changes the Q2 calculus — flagging the tension before proceeding.

  Decisive find — SL-186 D7/OQ-3. SL-186 made the bake marker {{ prompt resolve
  --role worker }} deliberately role-band-only. Its stated rationale (D7):

  ▎ "Only the role axis is bound — model/arm/stage are unknown at install and
  ▎ reach the worker via SL-187's spawn envelope (which excludes role, so the
  ▎ two surfaces don't duplicate). Rejected: spawn-time expansion … that is
  ▎ SL-187's spawn-envelope surface, and pulling it here reintroduces the
  ▎ blast-radius overlap the SL-186/SL-187 split exists to avoid."

  So my standing Q2(a) recommendation — widen the bake to carry the model band —
  walks straight back into the seam SL-186 split apart on purpose. That's not
  automatically wrong (SL-191 can supersede a prior decision with cause), but
  it's load-bearing and I shouldn't have glossed it.

  The pivot is SL-191 OQ-3, which I'd parked: does a spawned claude worker
  actually receive SL-187's spawn envelope, or is the baked def its sole
  delivery?

  - If the claude arm does get the SL-187 spawn envelope → model band already
  reaches the worker there; widening the bake duplicates it (the exact overlap
  SL-186 D7 forbids). Then Q2(a) answer = author the model hymn, leave the bake
  role-only, let the envelope carry model.
  - If the baked def is the claude worker's sole static delivery → SL-187's
  envelope doesn't fire for it, and widening the bake is the only way model
  content lands. Then Q2(a) = widen, and it doesn't duplicate anything.

  Q2 can't lock until OQ-3 is resolved. That's an SL-187 fact, not a preference
  — reading it now.

❯ read SL-187's design.md

● Both designs read. Now I can resolve Q2 properly — and the key that unlocks it
  isn't a preference, it's a fact about what SL-187 actually mechanized vs.
  what SL-186 D7 assumed it would.

  What the two designs actually deliver

  SL-186 D7 baked the worker def role-only, explicitly deferring model/arm/stage
  to "SL-187's spawn envelope."

  SL-187, as built, has no worker spawn-envelope that mechanically injects the
  model band. What it mechanized:
  - Session-start injection resolves --role orchestrator — hardcoded, for the
  top-level orchestrator session, not a spawned worker (D5, §5.2, Code Impact).
  - Model band = floor directive + doctrine_onboard supplement — both
  agent-invoked, best-effort, explicitly NOT a ceiling (D4, F14). "No
  correctness invariant rests on the model band."
  - SL-187's Code Impact builds boot.rs orchestrator delivery + onboard + memory
  tag. No --role worker injector anywhere.

  So the "spawn envelope (+model +arm +stage)" that D7 pointed at is a contract
  line in §5.4, not built code. For a claude subagent worker, the baked def is
  its sole guaranteed static delivery (confirms SL-191 OQ-3). SL-187's
  best-effort self-resolve is a supplement, not a mechanism a worker reliably
  hits.

  The reframe that dissolves the duplication worry

  The deepseek delivery patterns exist for deepseek workers. Deepseek runs on
  the subprocess arm (pi/codex spawning it). A claude-arm worker runs claude —
  it is not deepseek and does not want those patterns. So:

  - claude def (no model: pinned) → no model band → bakes negative contract
  (role, model-agnostic) only. Correct: a strong-model worker needs the
  where/what-not contract, not deepseek crutches.
  - universal/subprocess def (model: deepseek/deepseek-v4-pro) → model band =
  deepseek → bakes the deepseek patterns in.

  That is exactly D1 — arm-asymmetric by the def's declared model (data), zero
  arm→model logic. And it's not the D7 duplication, because there's no
  mechanical SL-187 worker-envelope to double against; baking is the reliable
  floor, self-resolve is gravy.

  Q2, resharpened

  - Q2(a) — model from frontmatter model: → confirmed as the right call, and now
  for a stronger reason than "single source": for the claude worker the bake is
  the only reliable delivery, and frontmatter is the only per-def model signal
  available at install time. Widening resolve_worker_role_body to bands:
  Only([Role, Model]) with model read from frontmatter is load-bearing, not
  redundant.
  - Q2(b) — no stage band → unchanged; nothing in either design argues for
  per-stage worker content.

  One thing I want to confirm in code before locking

  Whether the subprocess (pi/codex) worker also picks up a runtime prompt
  resolve (e.g. its before_agent_start firing) — if it does, baking deepseek
  into the pi def could double-deliver on that arm. Cheap to check: the dispatch
  spawn path + pi-spawn-confined.sh + whether the pi def bake targets which
  arms. If it double-delivers, the fix is trivial (bake reliably; runtime
  self-resolve is idempotent hymn content anyway), but I'd rather know than
  assume.

  Want me to (a) run that code probe now and then lock Q2 + raise Q3, or (b)
  lock Q2 on the above and probe during /plan? I lean (a) — it's ~2 greps and
  removes the last unknown from D2.

  [SL-191 design]: read SL-186/187; OQ-3 resolved (baked def = sole
  claude-worker delivery); Q2(a/b) confirmed, one code probe pending

✻ Cogitated for 1m 41s

❯ first: I want to step back from what is (mechanism) to what ought to be
  (category design).

  We can choose ultimately whether to bake something into an agent def, or have
  the agent def call doctrine prompt resolve to fetch it dynamically. That's
  implementation detail.

  the important part is to make the composition of prompts 'as simple as
  possible, but no simpler'.

  workers need to get context appropriate for workers, which might include a
  subset of what `doctrine boot --emit --harness ...` or whatever it is
  currently produces.

  workers which are loose as fuck like deepseek need additional guidance to
  stop them using chainsaws to shake hands. This is not about
  deepseek-the-model, it's about loose-as-fuck-adherence.

  deepseek is also pretty smart. we don't want to tell it things which are
  appropriate to dumb strongly-aligned agents, nor only dumb poor-adherence
  agents.

  this is why the design of the cascade allowed for user-definable, composable
  selectors.

  What we want a deepseek worker to know is stuff for:
  - pretty smart agents
  - with questionable adherence
  - performing their role

  and most of the *interesting* things to probe in terms of what's been
  delivered already by the cascade (as opposed to boring but necessary
  questions like 'is this wired') are in the realm of: when must these compose
  using AND or OR, and does that suggest anything conceptual is missing or
  under-developed in the cascade feature?

● Stepping to category design. Agreed the mechanism (bake vs. runtime fetch) is
  downstream — settle the categories first.

  What actually varies

  Not "deepseek." Three orthogonal axes decide what a worker must be told:

  - capability (smart ↔ dumb) — how much reasoning you can lean on
  - adherence (tight ↔ loose) — how far it colours outside the lines
  - role (worker vs orchestrator) — the job + its subset of boot context

  "deepseek guidance" is really {smart × loose × worker}. Model identity is a
  proxy for a trait-tuple — and a leaky one: it doesn't reuse to the next loose
  model, and it mis-fires the day deepseek tightens or a "smart-tight" model
  ships under the same vendor.

  Map onto the cascade's selector algebra

  The engine today (SL-186 §5.1–5.2):
  - within a selector: every axis constraint must hold → AND.
  - across snippets: every matching snippet concatenates → union (P1/P2).

  That spine is right. A loose-smart worker's context = the union of every
  snippet whose conjunctive selector its trait-set satisfies. AND-within +
  union-across is sufficient for conjunctive traits — provided the traits are
  axes.

  They aren't. Axes today: role · harness · model · arm · stage. There is no
  capability/adherence axis — only model, an identity path (vendor/name, matched
  left-to-right, _default wildcard). So "guidance for loose agents" has nowhere
  to live but model/deepseek/_default.md, which:
  1. doesn't reuse to another loose model (copy → breaks P1's single-home),
  2. fuses identity with trait,
  3. silently mis-targets when the model's real trait ≠ its vendor path.

  That is the conceptual under-development: the model band is identity, not
  classification. P5 said "model is a classification key" but the classification
  is one-dimensional (the path). Real classification is a multi-trait tuple,
  and the cascade can't carry it.

  The AND/OR probe — the sharp part

  Where does OR genuinely show up? "This applies to agents that need guardrails
  = loose OR dumb." That's a disjunction across two different axes.
  Inexpressible as one conjunctive selector. The workaround — author the body
  twice ({adherence=loose} + {capability=dumb}) — breaks P1 (two homes) and
  double-delivers to an agent that is both.

  But the right read isn't "add OR to the selector grammar." Wanting OR is the
  smell that your trait axis is too primitive. The fix is a derived, coarser
  trait — guardrail-level: high — classified from loose/dumb, so selection stays
  purely conjunctive. Keep the engine at AND+union (simple); push the
  disjunction down into classification. That's "as simple as possible, but no
  simpler" applied exactly: don't grow a boolean grammar, grow the right
  vocabulary.

  So the AND/OR analysis lands on: OR is not the missing engine primitive. The
  missing thing is a classification layer — orthogonal trait axes + a
  model→traits declaration that feeds the context vector.

  The registry tension — resolved, not violated

  P4/P6 fenced a models registry — but specifically the weekly-churning model-id
  → spawn-param list. A model→traits map (deepseek = {smart, loose}, claude =
  {smart, tight}) is small and changes on the order of model generations, not
  weekly. It is a different artifact than the one they fenced. The design
  collapsed both under one model/ path and thereby fused the
  churny-identity-we-must-not-registry with the
  slow-classification-we-legitimately-can. Splitting them is the repair, and it
  doesn't reopen R1 — the trait map is the anti-churn stance, not a breach of
  it.

  Where this leaves SL-191

  Right categories for the content we're authoring:
  - negative contract (touch-only-declared-set, hermetic goldens,
  component-anchored paths, only-git-verb-is-commit) → role=worker. Applies to
  every worker, any model. ✓ already the plan.
  - "don't shake hands with a chainsaw" (concrete>abstract, explicit negatives,
  high-density) → an adherence/guardrail trait, not model=deepseek. Reuses
  across loose models; absent for a tight worker.

  But the trait axis doesn't exist in SL-186's engine. So SL-191 hits a real
  fork:

  - (A) Pragmatic: ship the loose-agent guidance keyed
  model/deepseek/_default.md now (works today), and file the trait-axis as a
  cascade RFC. Justified iff deepseek is effectively our only loose dispatch
  target — then trait-vs-identity is a distinction without a difference yet
  (YAGNI).
  - (B) Correct-category: extend the cascade with a capability/adherence (or
  derived guardrail-level) axis + a slow model→traits declaration, then author
  SL-191's content against it. Bigger — touches SL-186's band registry and
  ContextVector — but it's the category design you're reaching for, and it's
  what makes the composition honest.

  My lean: the finding is real and worth capturing regardless (RFC/backlog:
  "trait axes distinct from model identity; conjunctive selection + derived
  traits over an OR-grammar; model→traits is a slow artifact, not the fenced
  registry"). Whether SL-191 builds (B) or defers to (A) turns on one fact I
  can't set for you: how many genuinely loose dispatch targets do we have or
  expect? One → (A) + RFC. Two+ diverging from vendor identity → (B) earns its
  scope now.

  Which way does the loose-target count point — and do you want the trait-axis
  finding filed as an RFC against the cascade either way?

  [SL-191 design]: category probe — model band conflates identity vs trait;
  OR-smell → derived conjunctive traits + model→traits classification (not the
  fenced churny registry); SL-191 fork A(model-keyed now+RFC) vs B(build trait
  axis)

✻ Churned for 2m 18s

❯ so the design conversation, though it seems the design.md document lost
  important nuance, specified model as user-definable, composable, and category
  based.

  it's explicitly designed to allow multiple axes to compose.

  I don't intend to write hymns with a model axis of deepseek/deepseek-v4-pro
  I'll write them with the model axes eg.
    adherence/low
    capability/code/high
    capability/reasoning/medium
    capability/reasoning/high

● You're right — I was reading the delivered engine's narrowed surface
  (design.md §5.2's --model <id> single-path-descent) and treating it as the
  category model. The design intent you describe — model/ as a space of
  composable, user-definable category axes (adherence/low, capability/code/high,
  capability/reasoning/high) that AND-compose — is exactly the honest version,
  and it dissolves my "conceptual gap": the concept isn't missing from the
  design, it's that design.md's compaction flattened it and I inherited the
  flattening.

  This reframes what's worth probing. The interesting question is no longer
  conceptual — it's whether the delivered code can actually compose multiple
  model/-category keys for one worker, because everything about SL-191's
  authoring depends on it:

  - Your snippets key on model/adherence/low, model/capability/code/high, etc.
  - A worker must carry a set of category keys, and matches must do
  membership/prefix over that set, so all applicable snippets union
  (AND-of-traits via composition).
  - The --model surface must therefore be repeatable (or the context otherwise
  carries a multi-key set) — not a single id descended down one path.

  If the delivered hymns.rs only descends a single --model <id> against the
  model/** tree (which is what design.md §5.2's model-key grammar literally
  says), then the category-composition intent is authored-but-not-delivered —
  and SL-191 either extends the engine or can't express adherence/low ∧
  capability/code/high. That's the load-bearing fact. Verifying it directly
  (pasted design lost nuance; I trust the code over both of us here).

  Searched for 1 pattern, read 1 file (ctrl+o to expand)

● Probe done. Verified against the code, not the compacted design:

  Delivered engine is single-key. ContextVector.model: Option<String>
  (hymns.rs:202) — one model key per context. model_matches (hymns.rs:290)
  descends one pattern against one ctx key, left-to-right, _default wildcard.
  Selector.model: Option<String> likewise.

  So the delivered model band is a single hierarchical taxonomy path, not a set
  of composable orthogonal categories. A worker can carry adherence/low xor
  capability/code/high — not both. Your intended authoring (adherence/low ∧
  capability/code/high ∧ capability/reasoning/high all composing) is
  authored-but-undeliverable on the current engine. The design intended
  composition; SL-186's delivery (and design.md's compaction) narrowed it to
  single-key.

  But the gap to close it is small and well-contained — and this is the good
  news the probe was for:

  - Composition across snippets is already a union (P1/P2). The only thing
  blocking orthogonal-category AND is that the context carries a single model
  value.
  - Fix: ContextVector.model → a set of keys; model_matches(pat, ctx_set) =
  "pattern matches any key in the set" (membership). Namespacing does the
  multi-axis for free — adherence/*, capability/code/*, capability/reasoning/*
  are just keys in the set; no new band needed, no touch to INV-1's closed band
  registry.
  - Specificity/precedence are untouched. specificity() reads the selector
  (still one pattern per snippet — hymns.rs:306–314), never the context. Each
  snippet keeps a single model pattern; only the context goes set-valued. So the
  D3 band-primary-axis machinery and all its goldens stand.
  - Change surface: ContextVector.model type, model_matches, the --model CLI
  arity → repeatable, plus new composition goldens. Existing single-model
  goldens pass as singleton sets.

  And it lands your category design cleanly:
  - AND-of-traits = the existing union, once the context holds the trait set. No
  OR-grammar, ever — orthogonal categories + derived coarser traits keep
  selection purely conjunctive (my earlier point survives, now grounded).
  - D1 restated honestly: "arm-asymmetric by data" = the worker def declares its
  trait-key set in frontmatter (adherence/low, capability/code/high,
  capability/reasoning/high for the deepseek-class def; adherence/high,
  capability/*/high for claude). No central model→traits registry — each def
  self-declares. P4/P6 fully intact.

  The one thing this forces — a scoping fork. Set-valued model matching is a
  cascade-engine capability, not worker-contract content. It's a genuine change
  to SL-186's locked engine. So:

  - (i) Fold it into SL-191 (scope expands: "author worker-trait hymns + deliver
  composable model-categories"). Honest but widens the slice past its "contract
  content + bake + check" frame.
  - (ii) Precursor slice/revision against the cascade (SL-191 after: it) —
  engine capability lands first, SL-191 authors against it. Cleaner category
  separation; two slices.
  - (iii) Frame it as a conformance fix to SL-186 (delivered ≠ designed intent)
  — a revision on SL-186, not new scope. Arguably most accurate: the composition
  was designed, the delivery clipped it.

  I lean (iii)/(ii) — the engine extension is a distinct concern from SL-191's
  worker-contract authoring, and treating it as "SL-186 under-delivered its own
  composable-category design" is both true and the tidiest provenance. SL-191
  then becomes purely: author trait-keyed hymns + declare def trait-sets + bake
  + check-cadence, riding a corrected engine.

  Your call on the fork. And — confirm my read of your intent: one model band,
  set-valued context, category-namespaced keys (not a fistful of new first-class
  bands)? That distinction decides whether INV-1's band registry is touched at
  all (I believe it needn't be).

  [SL-191 design]: verified engine model axis is single-key (Option<String>) —
  composable-category intent authored-but-undelivered; minimal fix = set-valued
  context + membership match, specificity untouched; scoping fork (i/ii/iii)
  raised

❯ > No OR-grammar, ever — orthogonal categories + derived coarser traits keep
  selection purely conjunctive (my earlier point survives, now grounded).

  I feel like this kind of commitment deserves at least some thought
  experiments to see if it holds as 'nice simplifying constraint' rather than
  'perennial straightjacket'
● Fair — "ever" was a slogan, not a proof. Let me actually try to break it.

  First, state the algebra honestly

  Conjunctive selectors + union-across-snippets = DNF at snippet granularity.
  Every snippet is one AND-clause; writing N snippets gives you an OR of N
  clauses. So OR isn't inexpressible — any boolean target region is reachable.
  The costs conjunctive-only actually imposes are two, and they're smaller than
  "can't do OR":

  1. a body wanted by multiple disjuncts must be duplicated (breaks P1
  single-home), and
  2. an agent in the intersection of two duplicated disjuncts gets the text
  twice (union has no dedup-by-body).

  For prose-to-an-LLM, double text is wasteful, not incorrect. So there's
  already a graceful pressure-valve: DNF-by-duplication never corrupts, it just
  isn't pretty.

  Thought experiments

  T1 — loose OR dumb → guardrails (the motivating case). Refactors cleanly:
  derived trait guardrail/high, declared on both the loose def and the dumb def.
  Body lives once. Disjunction moved into classification, which we own (def
  frontmatter). ✓ Holds — because the disjunction is over a single axis with a
  mutable, per-entity declarable set.

  T2 — negation over an open axis. "All workers except deepseek get X." No
  negation in a conjunction. Refactor = positive trait on everyone-who-should.
  Cost: a new def that forgets to declare the trait silently misses X.
  Positive-classification makes the default explicit-or-absent — arguably good
  (forces a decision), but it's real friction, and the failure is silent.
  Watch-item, not a wall.

  T3 — OR over a code-enum axis. "harness=claude OR harness=codex (both
  hosted-tool arms) get Y." The escape hatch (derived trait) needs a declarable
  set to hang the trait on — but harness is a bare enum with equality matching
  (hymns.rs: Option<String>, no prefix/glob), owned by code, not a per-entity
  mutable attribute. So there's nowhere to stamp harness-family/hosted. You
  duplicate, or you add a derived axis in code. The escape hatch requires the
  axis to be classifiable; a code-enum isn't. This is where "derived traits
  always save you" fails.

  T4 — OR correlating axes of different owners/lifetimes (sharpest). "Applies to
  {harness=claude} (framework enum, install-time) OR {project-tagged
  experimental} (user overlay, runtime)." No single entity can precompute a
  derived trait spanning an install-time code axis and a runtime user axis — the
  two disjuncts are matched from independent inputs owned by different parties.
  Classification can't reach it. Here you genuinely want selector-OR, or you
  duplicate. Niche, but not fictional.

  So the "derived-trait" escape is clean iff the disjunction is over a single
  axis with a mutable, single-owner, declarable membership set. T3/T4 are
  exactly the cases that violate that.

  The keystone — why keep selectors conjunctive isn't aesthetics

  Here's the grounding that turns this from taste into structure: specificity is
  only well-defined over conjunctions.

  specificity = (primary-axis depth, Σ other-axis depths) (hymns.rs:306). That's
  a clean scalar because a conjunction has a well-defined per-axis depth. Ask
  "what's the specificity of harness=claude OR model=deepseek?" — depth is
  ill-posed; you'd have to invent an arbitrary rule. And specificity is a total
  order that replaces (INV-3), seal (INV-6), and last-word (INV-2) all rest on.
  OR-selectors don't just complicate the parser — they dissolve the total order
  the suppression/precedence algebra needs.

  So disjunction isn't exiled to classification by minimalist fiat. It's exiled
  because classification is the layer where OR doesn't corrupt the ordering: a
  set-valued context matched by membership keeps every selector conjunctive (one
  pattern each → specificity intact — exactly why the set-valued-model change I
  proposed leaves specificity untouched), while the disjunction lives
  harmlessly in the data.

  Revised claim (defensible version)

  Not "no OR-grammar, ever." Instead:

  ▎ Disjunction belongs in the classification layer (set-membership), not the
  ▎ selector grammar — because conjunctive selectors are what make specificity a
  ▎ total order, and disjunction over declarable sets keeps every selector
  ▎ conjunctive.

  This is a generative constraint, not a straightjacket: when a new OR-need
  appears, it tells you what to do — give that axis a declarable, set-valued
  classification (T1, and by extension T3 if we choose to make harness-family a
  declarable set rather than a bare enum). It jams only at T4 (cross-owner,
  cross-lifetime disjunction), and even there the duplication valve holds —
  bounded, non-corrupting for prose.

  Trade to keep grammar-OR out: you accept T4-class duplication and T2-class
  silent-default friction. Trade to let it in: every author reasons in DNF, and
  you must define specificity over boolean formulas (ill-posed) — poisoning
  replaces/seal/last-word for a case (T4) that hasn't appeared yet.

  Verdict: nice simplifying constraint for this domain, with two named
  watch-items (T2 open-axis negation, T3/T4 OR over non-classifiable/cross-owner
  axes). Revisit grammar-OR only if a real T4 case with intolerable duplication
  shows up — and even then, first ask whether the right fix is making the
  offending axis classifiable, not enriching the grammar. Worth recording as a
  cascade design axiom (with T2–T4 as the stress cases) so the "why" doesn't get
  compacted away again like the composability nuance did.

  [SL-191 design]: stress-tested conjunctive-only — holds as generative
  constraint (disjunction→classification), grounded in
  specificity-is-total-order-only-over-conjunctions; frays at open-axis negation
  + cross-owner OR (T2/T4), duplication valve non-corrupting; propose recording
  as cascade axiom

❯ two probes:
  1. is or expressible by a selector for the root of two axes; ie
  model=anthropic matches model:anthropic/opus/5.6,anthropic/haiku/*, ...

  2. what's expressible across bands, e.g: role=worker, model=[ ... ]; - are
  the role & model (always) AND?

● Both are code-semantics questions — let me verify the exact matches body
  rather than reason from memory.

  Read 1 file (ctrl+o to expand)

● Both verified against matches (hymns.rs:319–346) and model_matches (:290).

  Probe 1 — prefix at a hierarchy root = subtree-OR. Yes, but only contiguous.

  model_matches("anthropic", "anthropic/opus/5.6"): pat=[anthropic],
  ctx=[anthropic,opus,5.6]; 1 ≤ 3; zip first seg anthropic==anthropic → true. A
  snippet keyed at a shallow prefix matches every deeper context key under it.
  So model=anthropic = anthropic/opus/5.6 ∨ anthropic/haiku/* ∨ … — a
  disjunction over its whole descendant subtree, for free.

  Bounds:
  - Contiguous subtree only. opus ∨ haiku minus sonnet — impossible; the prefix
  anthropic drags sonnet in.
  - No cross-root. anthropic/opus ∨ openai/gpt — no shared prefix, no single
  pattern.
  - Direction: shallow selector → deep context. (role/worker selector
  model=capability/code reaches capability/code/{high,medium,low} agents.)
  - Aside: anthropic and anthropic/_default both match anthropic/opus/5.6 and
  both have model_depth 1 → same specificity; near-synonyms.

  So hierarchy is a pre-baked OR. Each axis's tree bakes one
  disjunction-decomposition: "any node under P" is a one-selector OR. That's the
  classification-layer disjunction made concrete — as tree shape. It handles
  the T1/T3 ORs iff you shape the tree so the disjuncts you'll want share a
  parent (adherence/needs-help/{low,med} vs adherence/fine/high gets you
  "low∨med−high"). And since a tree gives you one decomposition, two orthogonal
  groupings on the same conceptual axis (vendor-wise ∧ capability-wise) can't
  both be subtrees — which is exactly why your adherence/, capability/code/,
  capability/reasoning/ want to be separate axes. Probe 1 vindicates the
  multi-axis design.

  Probe 2 — within a selector, all axes are AND. Always. No exceptions.

  matches is a chain of if pinned && mismatch → return false, true only if every
  pinned axis matches (:320–345). No OR branch anywhere across axes.

  And crucially: a selector carries all five axes (harness,model,role,arm,stage)
  in one struct, independent of which band the snippet lives in. The band = the
  slot's first path segment (output position); the selector can pin any axis
  via sidecar. So a role/worker.md with sidecar model=adherence/low → slot
  band=role, selector {role=worker ∧ model=adherence/low} → matches loose
  workers only. role=worker ∧ model∈{…} is a single expressible conjunction. The
  band never enters the matching logic — only ordering.

  The only disjunction-like operator in the whole pipeline is the union across
  snippets (resolve: filter → collect → concat, :359–364) — accumulation, not
  selection-OR.

  The unifying picture (the useful takeaway)

  Put both together — one selector expresses:

  ▎ **(subtree-OR on axis₁) ∧ (subtree-OR on axis₂) ∧ … ** = an axis-aligned box
  ▎ in the taxonomy product-space.

  - Within an axis: OR, but only a contiguous subtree (prefix).
  - Across axes: pure AND.
  - Across snippets: union of boxes → any region, tiled.

  So the earlier "DNF at snippet granularity" sharpens to: each selector is one
  axis-aligned box (product of subtree-ORs); the corpus unions boxes. What one
  selector cannot do: non-sibling OR within an axis (split into ≥2 boxes) or any
  cross-axis OR (≥2 boxes). The cost of a region is box count (snippets) +
  shared-body duplication across boxes — and prefix-OR keeps box-count low when
  the taxonomy is shaped to your disjuncts.

  This also nails how the set-valued context fix reads geometrically: the agent
  stops being a point and becomes a set of points (one per declared trait key);
  a box matches if any of the agent's points lands in it. That's precisely what
  lets adherence/low and capability/code/high both fire — the worker occupies
  both regions at once. Fully consistent with everything above, and it's why the
  fix touches only context+match, not specificity (boxes are still
  single-pattern per axis).

  Net for the conjunctive-only question: it's AND-of-contiguous-subtree-ORs per
  selector, union across — a bounded, tileable fragment with a clean geometry
  and a total-order specificity. The straightjacket risk (T2/T4) is unchanged;
  Probe 1 just shows a big chunk of everyday OR is already free if the taxonomy
  is well-shaped, which is a design-time lever you hold.

  [SL-191 design]: probes resolved — prefix = contiguous-subtree-OR (hierarchy
  is pre-baked OR, one decomposition/axis → vindicates multi-axis); cross-axis
  always AND, selector may pin any axis regardless of band; selector =
  axis-aligned box (∧ of subtree-ORs), corpus = union of boxes; set-valued
  context = agent-as-point-set

❯ ok; i feel like what's needed now is a new RFC to capture the design nuance
  and user-facing design considerations which escaped in the slice design.md,
  which ultimately ends up as a spec for the cascade mechanism.

● RFC-013 created + committed (67cfb488, status open). Captures everything the
  design conversation surfaced that SL-186's compaction had lost:

  - The nuance: model band = composable, user-definable trait-classification
  space, not a vendor/name identity path. Model identity is a leaky proxy for a
  trait-tuple.
  - The verified algebra: selector = axis-aligned box (AND of contiguous-subtree
  ORs); corpus = union of boxes; cross-axis always AND; prefix = subtree-OR;
  hierarchy = one decomposition per axis → orthogonal needs = separate axes.
  - The delivered gap + bounded fix: single-key Option<String> → set-valued
  context + membership match; specificity/precedence/band-registry untouched.
  - The axiom: disjunction lives in classification, not selector grammar —
  grounded in specificity is a total order only over conjunctions (OR-selectors
  would dissolve replaces/seal/last-word).
  - Stress tests T1–T4 (where it holds, where it frays, the duplication valve).
  - Classification source: per-def declared trait set, no registry, P4/P6
  intact; adherence assigned not self-ID'd.
  - Authoring/user-facing considerations + the SL-191 sequencing fork +
  graduation path to the cascade tech spec.

  Linked: SL-186/187/191, IMP-155/197, ADR-011/005/014, POL-002.
