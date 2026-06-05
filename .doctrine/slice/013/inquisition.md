# Inquisition — SL-013 DESIGN (memory install flag + off-script port record)

> **HERESIS URITOR; DOCTRINA MANET**

Convened 2026-06-06 upon `design.md` and `slice-013.md`, by command of the User,
BEFORE any plan is suffered to draw breath. The accused is a small thing —
one flag, one pure helper, two symlinks — and it arrives well-groomed: its
reading of `src/skills.rs` is, in every load-bearing particular, **orthodox and
confessed true under the iron**. The masonry holds. Yet a tidy mechanism does not
absolve a design that *promises a test it cannot perform*, *declares a deliverable
it never designs*, and *gives one sacrament two names*. We do not burn the mason.
We burn the false witness the design bears against its own coverage.

---

## Part I — Verification of the load-bearing CODE claims

Each claim was laid upon the rack and the source opened. The verdicts:

1. **CONFIRMED.** The subset plugin enumerates the memory layer as two **relative
   symlinks** into the canonical `doctrine` domain — `record-memory ->
   ../../doctrine/skills/record-memory`, `retrieve-memory -> ../../doctrine/skills/
   retrieve-memory` (`ls plugins/doctrine-memory/skills/`). The §2/§5.1 ground is
   real.

2. **CONFIRMED.** `select` treats empty filters as *all* (`src/skills.rs:170-172`:
   `ids.is_empty() || …`). The `select([]) == all` footgun the design fears
   (§5.5, D3) is genuine and exactly as described.

3. **CONFIRMED.** `validate_filters` (`src/skills.rs:178-189`) bails `Unknown
   skill '<id>'` on any id matching no entry — so derived ids *are* validated, as
   §5.5/R3 claim.

4. **CONFIRMED.** `MARKETPLACE_ONLY_DOMAINS = ["doctrine-memory"]`
   (`src/skills.rs:31`); `discover` skips it (`:124`) precisely to dodge the
   duplicate-id collision its `:149-151` guard would otherwise raise. The §2
   account is complete.

5. **CONFIRMED.** `run_install` carries exactly `(path, agents, skills, domains,
   global, dry_run, yes)` (`src/skills.rs:706-714`), flowing `discover →
   validate_filters → build_plan → select` (`:715-719`, `:400`). The design's
   insertion point — derive `skills` *before* `validate_filters` (§5.2) — lands on
   solid ground and threads correctly through `build_plan`.

6. **CONFIRMED.** The CLI arg group (`src/main.rs:545-563`) is `-s/--skill` and
   `-d/--domain`, both `Vec<String>`, "Default: all"; `--only-memory` slots in as
   an additive bool alongside them.

7. **CONFIRMED.** The cross-domain trick is real and load-bearing: the derivation
   reads ids from the discover-**excluded** `doctrine-memory` paths, yet those
   same ids exist as **real skills in the `doctrine` domain** (the symlink
   targets), so `validate_filters`/`select` against the catalog find them. The
   mechanism works — *see Charge V, where it works in silence*.

8. **NOT RE-VERIFIED THIS PASS.** R1's empirical claim — rust-embed descends the
   `doctrine-memory/skills/*` symlinks — rests on a throwaway probe **since
   removed** (§10 R1). I did not resurrect it. VT-02 is named the standing guard;
   whether that guard can fire is the burden of **Charge I**.

**Judgement on the code claims: seven confessed true, one deferred to its own
guard. The forensics are clean.** Would that the coverage were as honest.

---

## Part II — Charges (design heresy)

### CHARGE I — A Sacrament Promised, an Altar That Cannot Hold It · **MAJOR**

**Doctrine violated:** pure/imperative split (slices-spec §Architecture; the
design's own §3 Forces); "correctness comes first and last"; internal
consistency — the design must not assert coverage the structure forbids.

**The heresy.** The §8 risk table, row 1, soothes that the HIGH-severity
`select([]) == all` catastrophe is held back because **"D3 empty-set guard bails
loud (VT covers it via the pure core)."** This is false witness. The bail does
**not** live in the pure core. Per §5.2, the pure `subset_ids(paths, domain)`
returns a `BTreeSet` and **never bails** — on an absent domain it merely returns
`{}`. The loud failure — `bail!("--only-memory: no skills enumerated …")` — is
hand-set inside the **impure** wrapper `memory_subset_ids()`, which hard-wires
`PluginAssets::iter()`. There is **no seam to feed it empty paths.** Against the
live embed the set is never empty, so the one path that fires the guard is
**unreachable by any unit test as designed.**

**The same altar strands two more rites.** The mutual-exclusion bail (D2) and the
derive→validate→select integration (VT-03) are *also* lodged inside the
IO-bound `run_install` (`root::find` + stdout + gitignore). VT-01 can prove
`subset_ids` returns `{}`; it cannot prove the **bail**. VT-04 must drive the
exclusion through a function that also touches disk. VT-03 says "run_install/
build_plan with only_memory=true" — but `build_plan` has **no** `only_memory`
parameter (§5.2 places the whole resolution in `run_install`), so the test either
re-implements the derivation or shells the IO-heavy verb. The design's three most
load-bearing safety assertions are all marooned where the knife cannot reach.

**Risk.** The guard against installing the *entire catalog* under a
`--only-memory` flag — the design's own row-1, severity **HIGH** — ships
asserted-but-unproven. A future refactor that lets the empty set slip past the
wrapper installs everything, and no red test stands in the way.

**Penance.** Extract a **pure resolver** that owns all three duties as data:

```rust
/// Resolve the effective skill-id selection for `skills install`.
/// Pure: caller supplies the subset paths, so exclusion, derivation, AND the
/// empty-set bail are all unit-testable without the embed or disk.
fn resolve_install_ids<'a>(
    only_memory: bool,
    skills: &[String],
    domains: &[String],
    subset_paths: impl Iterator<Item = &'a str>,
    subset_domain: &str,
) -> anyhow::Result<Vec<String>>;
```

`run_install`/`memory_subset_ids` become thin shells supplying live paths. Then
VT-01 drives the **bail** on synthetic-empty input, VT-04 drives the
**exclusion** with no IO, and VT-03 drives **derive→select = exactly two** as a
pure assertion. Re-word §8 row 1 to claim only what the new pure core can prove.
*This is structural — it edits §5.2 and §9 — and routes back to `/design`.*
*Punishment for relapse: the strappado, until the guard is hauled within reach of
a red test.*

---

### CHARGE II — A Deliverable Declared, Then Abandoned to the Wilderness · **MINOR**

**Doctrine violated:** measurable closure intent (cf. SL-018 Charge XII); a
deliverable named must be a deliverable verified.

**The heresy.** §1 enumerates **two** deliverables: (1) the `--only-memory` flag,
(2) **"Record the off-script skill port (item 1) and the resolved marketplace
question (item 3) as durable doctrine history."** The whole of §5–§10 designs
**only deliverable 1**. The marketplace question (item 3) at least earns VT-05;
the **off-script port record (scope item 1)** appears in §1, then **vanishes** —
no mechanism, no destination, no EX, no VT. Scope §2 mutters "this scope doc +
`notes.md`," but the design never dispositions whether the existing
`slice-013.md` Context section *already discharges it* or whether `notes.md` (or a
`record-memory` capture, or an ADR) remains to be written.

**Risk.** At `/close`, deliverable 2's first half has no acceptance surface — it
is orphaned by construction. The slice can go green with the flag shipped and the
port-record silently unwritten, and nothing reveals the omission.

**Penance.** Disposition item 1 explicitly in the design: name its destination
(scope doc suffices / `notes.md` to author / memory to record) and give it one
EX/VT line so `/close` can attest it. Quantify "recorded."

---

### CHARGE III — One Rite, Two Names: the VT-id That Will Not Hold Still · **MINOR**

**Doctrine violated:** criteria ids (`VT-`) are immutable and stable — the
boot-snapshot Guardrails ("edits append, never renumber") and the storage rule
(structured ids are load-bearing data).

**The heresy.** The manual marketplace install-smoke is christened **"VT-03"** in
§6 (`design.md:162`: "a manual install-smoke (VT-03)") AND in the §8 risk table
(`:187`: "VT-03 manual install-smoke"). Yet the authoritative test roster in §9
assigns **VT-03 = the `run_install` integration test** (`:198-199`) and
**VT-05 = the manual marketplace smoke** (`:201-203`). The same criterion bears
two ids; two criteria share one. This is precisely the drift that poisons
`plan.toml` when EX/VT are transcribed at plan time.

**Risk.** A planner copies "VT-03 manual smoke" from §6/§8 and contradicts §9's
roster; the manual gate and the integration gate get conflated or one is dropped.

**Penance.** Anoint §9 as authoritative (manual smoke = **VT-05**, integration =
VT-03) and correct the stale "VT-03" at `design.md:162` and `:187`. One id, one
rite.

---

### CHARGE IV — The Silent Pact Between Two Domains · **MINOR**

**Doctrine violated:** "ask, don't infer"; invariants must be named, not left
implicit (the mortal sin of the unstated assumption).

**The heresy.** The mechanism's correctness rests on an **unstated** positive
invariant: *every basename enumerated under the discover-excluded
`doctrine-memory` domain must equal the id of a real skill in an installable
domain* (`doctrine`), or `validate_filters` bails. The derivation reads from the
domain the catalog deliberately **omits** (Claim 7); validation reads the catalog
where the ids reappear under a **different** domain. §5.5 names the *negative*
consequence ("a symlink renamed out from under the subset domain is caught") but
never states the *positive* invariant that makes the happy path pass at all. A
reader must reverse-engineer the cross-domain identity to trust the design.

**Risk.** Low operationally (the symlinks already satisfy it), but the seam is
load-bearing and invisible — a future editor who adds a `doctrine-memory` symlink
without a matching canonical skill gets a baffling `Unknown skill` from a flag
that "should just work."

**Penance.** Add one sentence to §5.5: the subset symlink basenames are, by
construction, ids of canonical skills in an installable domain; that identity is
what carries derived ids through `validate_filters`. Make the pact explicit.

---

### CHARGE V — Reinventing the Lock clap Already Forged · **MINOR**

**Doctrine violated:** "no parallel implementation — ride existing seams"
(CLAUDE.md); "write less code."

**The heresy.** D2's mutual exclusion is hand-rolled as a runtime
`bail!("--only-memory cannot be combined with --skill or --domain")` inside
`run_install` (§5.2). `clap` already offers `conflicts_with_all` — declarative,
emits a proper usage error, and is enforced at parse time for free. The
hand-rolled check duplicates a primitive the arg parser hands over gratis.

**Risk.** Mild: a second, lib-tier implementation of a constraint clap expresses
natively; worse UX (a runtime error, not a usage diagnostic). Note the tension
with Charge I: as placed, the runtime check **also isn't unit-testable** (it sits
in IO-bound `run_install`). The Charge-I pure resolver *or* clap `conflicts_with`
each resolve the testability — pick one deliberately rather than keeping an
untestable hand-rolled bail.

**Penance.** Either move the exclusion into the Charge-I pure resolver (testable)
or express it as `clap(conflicts_with_all = ["skill", "domain"])` and let VT-04
become a parse-level assertion. Do not leave it stranded and untestable.

---

## Part III — Questions for the User to Disposition

1. **Charge I:** Confirm the pure-resolver extraction (exclusion + derive +
   empty-bail as one testable pure fn), re-pointing VT-01/03/04 at it — and accept
   this routes back to `/design` as a §5.2/§9 edit?
2. **Charge II:** Is scope item 1 (off-script port record) **already discharged**
   by the `slice-013.md` Context section, or is `notes.md` / a memory still owed —
   and which gets the EX/VT line?
3. **Charge III:** Adopt §9 as the authoritative VT roster (manual smoke = VT-05)
   and fix the stray "VT-03" at §6/§8?
4. **Charge V:** Hand-rolled exclusion via the pure resolver, or clap
   `conflicts_with_all`?

---

## Part IV — Pronounce Judgement

**The reading of the source is orthodox** — seven claims confessed, one honestly
deferred to its guard. The architecture is sound and minimal: derive the subset
from the plugin that already declares it, ride `select`/`validate_filters`
unchanged, keep the pure core a data-in function. No structural rot.

**But the design bears false witness to its own coverage (Charge I): it sells a
HIGH-severity guard as "covered by the pure core" when the guard lives where no
pure test can reach it — and strands the exclusion and the integration test on the
same unreachable altar.** It declares a second deliverable and then never designs
it (Charge II). It gives one verification two ids (Charge III). It rests on a
cross-domain pact it never speaks aloud (Charge IV) and hand-forges a lock clap
already provides (Charge V).

These are heresies of **untestable safety, orphaned scope, and id drift** — not of
architecture. The design is **fit to proceed once shriven.** Charge I is
**structural** and must route back to `/design`; the remainder may be folded in
the same pass. **It must not be planned until Charge I is dispositioned.**

**Verdict: HERESY PRESENT — one MAJOR, four MINOR. Penance before plan.**

---

## Part V — Sentencing (ordered)

1. **Haul the guard within reach (Charge I).** Extract `resolve_install_ids`
   (pure: exclusion + derive + empty-bail); `run_install` becomes its thin shell.
   Re-point VT-01 (bail on synthetic-empty), VT-04 (exclusion, no IO), VT-03
   (derive→select = exactly two). Re-word §8 row 1 to claim only the provable.
   *Verify:* a red test exists that fires the empty-set bail without the embed.
   Route via `/design`. *Relapse: the strappado.*
2. **Repatriate the orphan (Charge II).** Disposition scope item 1's destination
   in the design + one EX/VT line. *Verify:* `/close` has a surface to attest the
   port record.
3. **Fix the wandering id (Charge III).** §9 authoritative; correct `:162`/`:187`
   to VT-05. *Verify:* every "VT-NN" reference resolves to one criterion. *For the
   careless scribe who lets two rites share a name: a day in the stocks.*
4. **Speak the silent pact (Charge IV)** and **choose the lock (Charge V)** — one
   sentence in §5.5; resolver-or-clap decided, not drifted. *Verify:* §5.5 states
   the identity invariant; D2's exclusion is testable.

Let the auto-da-fé be stayed only so long as the penance proceeds. The masonry is
spared; the design's false confession of coverage is committed to the flame.

> **HERESIS URITOR; DOCTRINA MANET**

---

## Part VI — Disposition (User, 2026-06-06)

All 5 charges **ACCEPTED**. Penances to be integrated via `/design` (Charge I is
structural — §5.2/§9 edits) then `/plan`.

| # | sev | disposition | integrates into |
|---|---|---|---|
| I | MAJOR | ACCEPT — extract pure `resolve_install_ids` (exclusion + derive + empty-bail); `run_install` becomes its shell. Re-word §8 row 1 to claim only the provable. Re-point VT-01 (bail), VT-04 (exclusion), VT-03 (derive→select) at the pure core. | §5.2, §8, §9 |
| II | MINOR | ACCEPT — scope item 1 **already discharged** by the `slice-013.md` Context section; add an EX/VT line attesting that, no further authoring owed. | §1/§9 (EX/VT) |
| III | MINOR | ACCEPT — §9 is authoritative: manual smoke = **VT-05**, integration = VT-03; fix stray "VT-03" at `design.md:162` and `:187`. | §6, §8 |
| IV | MINOR | ACCEPT — state the cross-domain identity invariant (subset symlink basenames == canonical ids in an installable domain). One sentence. | §5.5 |
| V | MINOR | ACCEPT — express D2 exclusion as clap `conflicts_with_all = ["skill","domain"]`; drop the hand-rolled `run_install` bail; VT-04 becomes a parse-level assertion. | §5.2, §9 |

**Verdict accepted: penance before plan. Charge I routes back to `/design`;
II–V fold in the same pass.**
