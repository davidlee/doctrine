# Inquisition — SL-018 DESIGN (Shipped orientation memory corpus)

> **HERESIS URITOR; DOCTRINA MANET**

Convened 2026-06-05 upon `design.md` and `slice-018.md`. The accused presented
itself well-shriven: eight load-bearing claims about the source were laid on the
rack, and **all eight confessed true under cross-examination**. The masonry is
sound. Yet a clean foundation does not absolve the edifice raised upon it — and
this edifice harbours a contradiction in its own scripture, a pruner that runs
unsupervised every session armed only with a *convention* for a leash, and a
corpus doomed to rot into a "stale" label it cannot escape. We do not burn the
mason. We burn the design's lies about its own legality.

---

## Part I — Verification of the load-bearing CODE claims

Each was opened and the viscera read. The verdict on each:

1. **CONFIRMED.** `collect_memories(items_root)` is the single-root leaf
   (`src/memory.rs:1069`). Production callers: `load_query` →
   `src/retrieve.rs:633` (shared by **both** `run_find:661` and
   `run_retrieve:750` — so switching 633 alone catches find *and* retrieve),
   `run_list` `src/memory.rs:1092`, `list_rows` `src/memory.rs:1109`. Existing
   tests call it directly at `src/memory.rs:2896,2900`. The design's enumeration
   of the three production switch-sites is **complete and precise** — no fourth
   surface is orphaned.

2. **CONFIRMED.** `base_filter` (`src/retrieve.rs:168-182`) admits a `repo=""`
   memory in any partition and drops a non-empty repo ≠ query repo. The global
   hatch is line **174** (design cites 172-174 — immaterial drift).

3. **CONFIRMED.** The write gate (`src/memory.rs:750-758`) bails iff
   `!frame.repo.repo_id.is_empty() && frame.anchor_kind == AnchorKind::None`. It
   keys on **repo non-empty, NOT on scope.paths** — the comment states verbatim
   "Path/glob/command scopes alone do not gate." This confession damns Charge I.

4. **CONFIRMED.** `Memory::try_from` (`src/memory.rs:481-543`) validates
   schema_version, uid, type, status, key, non-empty workspace, tags, and trust
   normalization. It does **NOT** re-enforce scoped⇒anchored. A
   `repo="", anchor_kind=none, scope.paths=[…]` row parses fine.

5. **CONFIRMED.** `boot.rs:127` calls `memory::list_rows`. Routing list_rows
   through `collect_all` would surface shipped memories in the boot snapshot.

6. **CONFIRMED.** `read_body` (`src/memory.rs:1059-1063`, called at
   `src/retrieve.rs:780`) joins `items_root` only and swallows any miss to an
   empty string. A shipped-root fallback is genuinely required.

7. **CONFIRMED — the inverted trap is real.** `.gitignore`: blanket
   `.doctrine/*` (line 12), negation `!.doctrine/memory/` (line 17), narrow
   re-ignores `index|embeddings|state/*` (lines 32-34). `.doctrine/memory/shipped/`
   is therefore **committed-by-default in doctrine's own repo** unless added.

8. **CONFIRMED.** Skills ships masters at repo-root `plugins/` with a *separate*
   `#[derive(RustEmbed)] #[folder="plugins/"]` (`src/skills.rs:24-26`),
   materialized by a dedicated verb with an active ownership trichotomy
   (`classify_link`, `src/skills.rs:287`), NOT folded into install/boot. The only
   embeds in the crate are `install/` and `plugins/` (`grep RustEmbed`) — the
   parallel the design claims exists.

**Judgement on the code claims: the accused did its homework. No charge lies
against its reading of the source.** Would that all heretics were so diligent.

---

## Part II — Charges (design heresy)

### CHARGE I — The False Confession of Legality (D8) · **MAJOR**

**Doctrine violated:** internal consistency; "ask, don't infer"; the design must
not assert a capability the tool denies.

**The heresy.** D8, §5.4, and slice Non-Goals proclaim that a client's local
orientation memory "is **already legal** and needs **no new machinery**" — a
memory described thrice as `repo=<client>, unscoped/tag-scoped, **unanchored**`,
living in committed `items/`, composed automatically by `collect_all`.

**Evidence of guilt.** The write gate (`src/memory.rs:753`, Claim 3, confessed)
**bails** on exactly `repo` non-empty + `anchor_kind == None`. `doctrine memory
record` therefore **cannot create** the artifact D8 describes:
- In a *born* client repo, `record` derives `repo=<client>` AND a real born
  anchor → the memory is **anchored**, not unanchored. (Creatable, but not what
  D8 says.)
- In an *unborn / non-git* context, `record` bails. (Not creatable at all.)
- The "unanchored + non-empty repo" combination is reachable only by
  hand-authoring into `items/` — which violates the store's own invariant
  ("the store is tool-authored, a bad row is a real fault", `src/memory.rs:1067`).

**The design indicts itself.** §Follow-Ups (M3) concedes the gate must be
relaxed: *"possibly relaxing `record`'s gate to permit unscoped+unanchored with
a repo present, for local convention memories."* This is a direct admission that
the gate **blocks** the very artifact D8 calls "already legal." The two
statements cannot both stand.

**Risk.** D8 is the load-bearing answer to the consult's foundational concern
("local changes not overwritten") and the justification for deferring M3. If the
local-content path is half-fiction, the resolution is illusory: an agent told to
"just put it in `items/`" will hit a hard bail.

**Penance.** Strike the word **"unanchored"** from D8 / §5.4 / Non-Goals. A
memory captured in a real born client repo carries a *real* anchor — that is the
honest and supported encoding, and with it the "already legal, no new machinery"
claim becomes TRUE. If genuinely-unanchored local memories are wanted, promote
the M3 gate-relax from "possibly" to a **declared dependency** and cease calling
it "already legal." Either way, reconcile §5.4/D8 with §Follow-Ups so the design
stops bearing false witness against itself.

---

### CHARGE II — A Class Conceived in Sin Against Frozen Scripture · **MAJOR**

**Doctrine violated:** `doc/memory-spec.md:307` — *"an unanchored memory,
permitted only for unscoped memory"* (the frame algorithm declared **frozen**,
§310).

**The heresy.** The shipped corpus is **path-scoped** (`scope.paths`, the
whole retrieval payoff) AND **unanchored** (`anchor_kind=none`). A path-scoped
memory IS "scoped" by the spec's own definition (`:299` — actionable memory
carries paths/globs/commands). Thus every shipped memory stands in direct
violation of §307. The design knows this (§7, OQ-2, R3, §10) and pleads a
*planned* memory-spec amendment + ADR as the reconciliation.

**Risk.** The reconciliation is a **trailing promise**, sequenced vaguely "in a
later phase." Until the ADR is accepted and the spec carved out, the corpus is a
standing contradiction of frozen doctrine, and the `repo=""`/`anchor=none`
admission path is unblessed. A design that builds a spec-violating class first
and amends scripture later inverts the order of authority (`/canon`: the spec is
not subordinate to the slice).

**Penance.** Make the **ADR + spec amendment a hard PHASE GATE that PRECEDES**
corpus authoring and the admission-path golden test — the first phase, not a
trailing doc chore. The amendment must explicitly define the new
*global / unanchored / path-scoped / derived* class AND its staleness
disposition (see Charge IV). No master may be authored before scripture sanctions
the class.

---

### CHARGE III — The Unsupervised Pruner, Leashed Only by Custom · **MAJOR**

**Doctrine violated:** "correctness comes first and last"; R6 (foundational-
process drift); the consult's own concern ("local changes not overwritten").

**The heresy.** D6 drops skills' ownership trichotomy (`classify_link`) on the
ground that "shipped/ is doctrine-only (D8), so there is no foreign-file
question." But shipped/-only is enforced by **nothing but convention** —
§5.5 admits the guarantee rests on "the tier convention + gitignore + docs."
Gitignored is not unwritable. The slice then arms a **prune** ("remove shipped
entries whose master no longer exists") and schedules it to run **every session**
via the M1 SessionStart hook. Convention-only safety + an auto-running file
deleter is precisely the R6 drift the consult raised — answered with a
convention, not a mechanism. Skills, the cited precedent, declined this exact
shortcut and built `classify_link` (`src/skills.rs:287`) to actively distinguish
"ours" from "foreign" before touching anything.

**Risk.** Any file a user or a bug deposits under `shipped/` is silently rm'd by
a hook the user never invoked. The design itself names this: "Violating this … is
the only way sync could prune local work."

**Penance.** Restore a bounded blast radius. Minimal acceptable: **prune only
directories whose `memory.toml` parses AND bears the shipped INV signature**
(`repo=""`, `anchor_kind=none`) — never arbitrary files/dirs. Better: have sync
write a provenance manifest of the uids it materialized and prune **only** uids
it previously wrote (the moral equivalent of skills' "ours"). Do not let an
auto-hook wield `rm` on the strength of a doc paragraph.

---

### CHARGE IV — The Corpus Damned to Rot into a Lie · **MAJOR**

**Doctrine violated:** `doc/memory-spec.md:357-368` (staleness modes); honest
signal to the downstream agent.

**The heresy.** Shipped memories are scoped-but-unattested (`no verified_sha`),
so per the staleness table (`:361`) they resolve to **"days since `reviewed`"**.
OQ-D leans on seeding `reviewed` to the authoring date. The corpus is
**evergreen by construction** (§5.4, R5) — yet days-since-`reviewed` grows
without bound, so the most authoritative orientation knowledge in the system is
progressively branded **stale** to the very agents meant to trust it. The design
waves this off as "weak signal; acceptable."

**The compounding trap.** You cannot fix it at sync time. Bumping `reviewed` on
each `memory sync` would make sync's output differ every session → the diff is no
longer idempotent → M1's "near-zero cost" claim (D9) collapses. So OQ-D
(seed-and-freeze) and D6 (idempotent diff) and D9 (per-session hook) are
mutually constraining: the corpus must rot, because the only cure breaks the
hook's economics.

**Risk.** An agent dropped into a client a year after the binary was built
retrieves correct framework orientation labelled "stale," and down-ranks or
distrusts it. The slice's entire purpose is undermined by its own staleness
metric.

**Penance.** Define a **fourth staleness disposition for the global / unanchored
/ derived class** in the same ADR + spec amendment Charge II demands: this class
is *evergreen / reference-grade*, exempt from the days-since-`reviewed` decay
(render an explicit "reference"/"unanchored" state, never a decaying "stale").
This keeps sync idempotent AND stops the corpus lying about its own freshness.

---

### CHARGE V — Bearing False Witness to Purity (§10) · **MINOR**

**Doctrine violated:** the pure/imperative split (a named project rule, ADR-001
rule 3); naming precision.

**The heresy.** §10 ("Doctrinal alignment") declares "`collect_all`/`plan_corpus`
are **pure** engine-leaf helpers." `plan_corpus` (assets→plan) may be pure, but
`collect_all` calls `collect_memories`, which performs `fs::read_to_string`
(`src/memory.rs:1073`) — it is **impure**, and it lives in `memory.rs`, a
**command-tier** module (ADR-001 table). The actual layering is acceptable
(command tier may do IO), but the design's *self-description* is false.

**Risk.** A future reader trusts §10 and treats `collect_all` as a pure function
(memoizable, testable without disk), then is surprised. Precision in the pure/
impure ledger is doctrine, not decoration.

**Penance.** Re-label: `plan_corpus` pure; `collect_all`/`sync_corpus` impure
shells (or command-tier IO helpers). One sentence.

---

### CHARGE VI — The Seam That Is Not a Seam (M3) · **MINOR**

**Doctrine violated:** "the seam left open, not foreclosed" (R6) must be a real
seam.

**The heresy.** §5.2 dedups `collect_all` by **uid** (`items` wins). §5.5 calls
a uid collision "practically impossible … logged at `find` debug, not an error" —
an *anomaly*. Yet R6/D8 claim "`collect_all` uid-dedup → future `memory_key`
precedence" is the open seam for M3 override. These are incoherent: (a) M3
override is **key**-precedence, but `collect_all` dedups by **uid** — adding key
precedence is a *rewrite of the dedup predicate*, not an "open seam"; (b) the one
path that *would* let uid-dedup serve override (a local `items/` memory
hand-copying a shipped uid so items wins) is simultaneously branded a
practically-impossible anomaly. The design cannot have uid-collision be both a
debug-logged accident and the M3 mechanism.

**Risk.** Overstated continuity. A future M3 author expects a seam and finds a
predicate to redesign; the "not foreclosed" reassurance is softer than claimed.

**Penance.** State plainly: M3 will replace uid-dedup with key-precedence
dedup; today's uid-dedup is a placeholder, not the seam. Decide whether uid
collision is an error or a (future) override channel — not both.

---

### CHARGE VII — Masters Minted by No Hand and No Tool · **MINOR**

**The heresy.** §5.3: master uids are "minted once at authoring (uuid v7)." But
`record` (the only minting path, `src/memory.rs:762`) forces an anchor and writes
to `items/` — it cannot mint a master. No authoring tool is specified; "uuid v7
at authoring" names an algorithm with no owner. Further, `is_uid`
(`src/memory.rs:484`) accepts **any** `mem_<32 hex>` — v7 is never verified, so
the "v7" requirement is decorative.

**Penance.** Specify the master-authoring mechanism in the plan (a
`doctrine memory new-master` verb? a documented `scripts/` one-liner?
`uuidgen`?). Drop the "v7" pretence or enforce it.

---

### CHARGE VIII — `reference`: a Parse-Blocker Dressed as a Nicety · **MINOR**

**The heresy.** OQ-B treats the missing `reference` memory_type as a cosmetic
"map onto signpost (no enum churn)" decision. But `MemoryType::parse`
(`src/memory.rs:68`) **bails** on any unknown type. The OQ-A skeleton lists a
`reference` type ("reference: CLI command map"), and §5.3's example value set
includes `reference`. Any authored master with `memory_type="reference"` makes
`Memory::parse` — hence `collect_all`, the master-lint test, and `memory sync` —
**error hard**. This is a hard blocker on authoring, not an aesthetic choice.

**Penance.** Resolve OQ-B **before** any master is authored. Either map all
references onto `signpost` (and forbid the literal in master-lint) or add the
enum variant. Elevate from "confirm in review" to "blocks PHASE: author corpus."

---

### CHARGE IX — Reassurance by a Baseline That Does Not Exist · **MINOR**

**The heresy.** §5.5 and R2 soothe: globals "pollute focused queries? … Same
behaviour as any global memory today." But `record` always derives a **non-empty
repo** (gate + cwd capture), so **zero `repo=""` memories exist in production
today**. The `base_filter` global hatch (`src/retrieve.rs:174`) is **dormant** —
SL-018 lights it up for the first time. There is no lived baseline to be
"the same as."

**Risk.** False comfort. The repo="" admission path is effectively new and
untested in practice; the design's confidence rests on a phantom precedent.

**Penance.** Reword the reassurance honestly ("this activates a hitherto-dormant
admission path"), and make R3's golden test on the `repo=""`/`anchor=none`
admission a **required** artifact, not optional.

---

### CHARGE X — Will the Broad Memories Even Surface? · **MINOR**

**The heresy.** OQ-A's broadest, most valuable entries (overview, file-map,
skill/route map) are inherently non-path-specific and would lean on **tags**.
But `doc/memory-spec.md:299` says an actionable memory "carries at least one of
`scope.paths`/`scope.globs`/`scope.commands`" — **tags are not listed** — and
`:333` excludes "records without scope" from scope-filtered queries, while the
scope-match table (`:335-340`) *does* score tags (specificity 0). The spec is
itself ambiguous on whether a tag-only memory is "scoped." If it is not, the
flagship orientation memories surface only on a bare `--query` — gutting the
"per-scope retrieval is the whole point" thesis.

**Penance.** Resolve in the Charge-II amendment whether tag-only is retrievable;
ensure each broad skeleton memory carries at least one path/glob/command scope
(even a coarse one, e.g. `.doctrine/`), or accept they are `--query`-only and say
so.

---

### CHARGE XI — The Hook That Errs in Foreign Lands · **MINOR**

**The heresy.** M1 (D9) installs a global SessionStart hook running
`doctrine memory sync` every session. In a non-doctrine repo, `root::find`
errors (cf. `boot.rs:234`) → sync errors. The slice ships **no `--check`
sentry** (Non-Goal) and the design never discusses hook-failure semantics or the
no-root case. It inherits boot's behaviour but does not say so or confirm it
degrades gracefully.

**Penance.** State that `memory sync` (like `boot`) must no-op-and-exit-clean
when no root is found, and confirm SessionStart tolerates it. One invariant +
one test ("sync outside a doctrine repo is a clean no-op").

---

### CHARGE XII — A Corpus That May Pass Empty · **MINOR**

**The heresy.** §9 names machinery tests precisely, but the **content** quality
bar is deferred (OQ-A: "executed in a later phase"). The slice could go green
with the plumbing tested and a trivial or empty corpus — the closure intent "a
doctrine orientation corpus is authored" has no measurable threshold.

**Penance.** Add a concrete acceptance criterion to the plan: every OQ-A
skeleton topic has ≥1 master; master-lint passes on all; the triage table
dispositions all 86. Quantify "authored."

---

## Part III — Questions for the User to Disposition

1. **D8:** Do you want genuinely *unanchored* local orientation memories (⇒ M3
   gate-relax becomes a hard dependency, "already legal" is struck), or is the
   real born anchor acceptable (⇒ drop "unanchored", D8 becomes true as written)?
2. **Charge II/IV:** Will the ADR + spec amendment be the **first** phase
   (gating authoring), and will it define the evergreen-class staleness exemption?
3. **Charge III:** Provenance manifest, INV-signature-gated prune, or accepted
   convention-only risk for the auto-running pruner?
4. **Charge VIII/OQ-B:** `reference` → map onto `signpost`, or add the enum
   variant? (Blocks authoring either way.)
5. **Charge X:** Are tag-only orientation memories retrievable in your intended
   semantics, or must every master carry a path/glob/command scope?
6. **OQ-E:** Separate SessionStart entry vs chaining onto boot's — and either
   way, confirmed graceful in non-doctrine repos (Charge XI)?

---

## Part IV — Pronounce Judgement

**The reading of the source is orthodox — eight claims, eight confessions, no
falsehood.** The accused is, in its forensics, a model penitent.

**But the design tells a lie about its own law (Charge I): it sells a local-
memory path the write gate forbids, and is convicted by its own Follow-Ups.** It
conceives a class in violation of frozen scripture and defers the absolution
(Charge II). It looses an unsupervised pruner restrained by mere custom (Charge
III). And it dooms its evergreen corpus to be branded "stale" with no escape that
does not shatter its own idempotency economics (Charge IV).

These are not fatal to the *architecture* — the three-tier model, the
`collect_all`-over-`collect_memories` gate-preservation, the separate embed, the
derived/gitignored tier are all sound and well-evidenced. They are heresies of
**overclaim, sequencing, and an unguarded blade**. The design is **fit to
proceed once shriven** — it must not be planned until Charges I–IV are
dispositioned and the ADR/spec amendment is made the first gate.

**Verdict: HERESY PRESENT — four MAJOR, eight MINOR. Penance before plan.**

---

## Part V — Sentencing (ordered)

1. **Recant Charge I.** Strike "unanchored" from D8/§5.4/Non-Goals OR declare the
   M3 gate-relax a dependency; reconcile with §Follow-Ups. *Verify:* no statement
   asserts an artifact `record` (`memory.rs:753`) rejects. *Punishment for
   relapse: the strappado, until the contradiction is recanted in full.*
2. **Sequence the scripture (Charge II + IV).** Make the ADR + `memory-spec`
   amendment **PHASE-01**, defining the global/unanchored/path-scoped/derived
   class AND its evergreen-staleness exemption. No master authored prior. *Verify:*
   plan.toml shows the amendment phase gating the authoring phase.
3. **Sheathe the pruner (Charge III).** Constrain prune to INV-signatured shipped
   dirs and/or a provenance manifest of self-written uids. *Verify:* a test —
   sync leaves a foreign file under `shipped/` **untouched**. *Failure to comply:
   breaking on the wheel for the careless deletion of a penitent's labour.*
4. **Resolve the parse-blockers and phantoms** (Charges VIII, IX, X) and the
   minor mislabels (V, VI, VII, XI, XII) in the plan. *Verify:* OQ-B/OQ-D/OQ-E
   closed; master-lint + golden admission test + no-root no-op test enumerated.
5. **Quantify "authored" (Charge XII).** Concrete corpus acceptance in plan.toml
   EX/VT. *Verify:* every OQ-A topic ≥1 master; triage table complete.

Let the auto-da-fé be stayed only so long as the penance proceeds. The masonry is
spared; the design's false confessions are committed to the flame.

> **HERESIS URITOR; DOCTRINA MANET**

---

## Part VI — Disposition (author, 2026-06-06)

All 12 charges **ACCEPTED**. Penances integrated into `design.md` + `slice-018.md`.

| # | sev | disposition | integrated |
|---|---|---|---|
| I | MAJOR | ACCEPT — struck "unanchored" from the "already legal" claim; local additive orientation = `record`-captured with a **real anchor**; genuinely-unanchored = M3 gate-relax dependency. | D8, §5.4, slice Non-Goals |
| II | MAJOR | ACCEPT — ADR + memory-spec amendment is **PHASE-01**, gating all corpus authoring. | D10, §5.4 |
| III | MAJOR | ACCEPT — **bounded prune**: only INV-signatured (`repo=""`/`anchor=none`) orphan masters; foreign/unparseable left untouched + test. | §5.2, §9 |
| IV | MAJOR | ACCEPT — amendment defines a **4th staleness disposition**: evergreen/`reference`, decay-exempt; staleness fn special-cases the class. | §5.4, §9, affected-surface |
| V | MINOR | ACCEPT — relabel `collect_all`/`sync_corpus` **impure** command-tier. | §10 |
| VI | MINOR | ACCEPT — M3 *adds* key-precedence; uid-dedup is v1 behaviour, not the seam. | D8, §5.2 |
| VII | MINOR | ACCEPT — authoring path = `record --global` mode (or scripts/ helper); drop "v7". | §5.3, affected-surface |
| VIII | MINOR | ACCEPT — OQ-B resolved: `reference`→`signpost`; master-lint forbids the literal. | OQ-B, §5.3, §9 |
| IX | MINOR | ACCEPT — reworded "dormant path lit first time"; golden admission test **required**. | §5.5, R2, §9 |
| X | MINOR | ACCEPT — **scope floor**: every master ≥1 path/glob/command (no tag-only); broad memories get coarse scope. | §5.3, §9 |
| XI | MINOR | ACCEPT — `memory sync` no-root **clean no-op** + test. | §5.2, §9 |
| XII | MINOR | ACCEPT — corpus acceptance quantified (all 86 dispositioned, every topic ≥1 master, lint green) → plan EX/VT. | §9 |

**Question answers (Part III):** Q1 → real anchor acceptable; "unanchored" struck;
truly-unanchored deferred to M3. Q2 → yes, ADR/amendment is PHASE-01 + defines the
evergreen staleness exemption. Q3 → INV-signature-gated prune (manifest = paranoid
alt, deferred). Q4 → `reference`→`signpost`. Q5 → every master carries ≥1 path/
glob/command scope (no tag-only reliance). Q6 → lean separate SessionStart entry,
graceful no-op in non-doctrine repos (OQ-E, plan-level).

**Verdict accepted: penance done; design fit to plan. Charges I–IV resolved;
PHASE-01 = ADR + spec amendment.**
