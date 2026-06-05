The evidence is gathered. The codebase has confessed, the live harness docs have testified, and one claim at the foundation of this design lies in ruins. I pronounce.

---

# ⚜ ACTA INQUISITIONIS — SL-011 *Cache-friendly session boot context* ⚜

*Convened upon `.doctrine/slice/011/design.md`, the scope `slice-011.md`, and the source-research `boot-research.local.md`. Witnesses examined: `src/install.rs`, `src/skills.rs`, `src/adr.rs`, `src/memory.rs`, the root `.gitignore`, and — under sharpest interrogation — the living Claude Code harness itself.*

Hear me, architects. A design is not heretical because it is wrong. It is heretical because it **proclaims as resolved** that which is **false**, and writes a verification gate that would bless the corpse. Let us to the rack.

---

## 1. CHARGES

### ⛧ Charge I — THE CARDINAL HERESY: the hook fires *after* the prefix is sealed. *(severity: RED — blocks the lock)*

**Doctrine violated:** §5.4, §5.5 ("review fix #2"), §9, §10 #2 — and the entire load-bearing premise of §1.

**The design confesses (§5.5):**
> *"The boot hook is synchronous/blocking and `doctrine boot` is fast (review fix #2) — regeneration must finish before the prefix loads, else the agent reads a one-session-stale snapshot."*

And §9 prescribes the gate: *"verification asserts no `async:true`."*

**Revealed under cross-examination of the live harness** (Claude Code docs, *How Claude Code uses prompt caching*, fetched this very day):

> 1. Session initialization
> 2. **Load CLAUDE.md and `.claude/rules/*.md` files**
> 3. **Run SessionStart hooks**
> 4. First user prompt

The order is **inverted from what the design believes**. CLAUDE.md and its `@`-imports are inlined into the cached prefix at **step 2**. The SessionStart hook fires at **step 3** — *after the snapshot is already sealed into context.* Synchronous or not, blocking or not, **the hook cannot touch the prefix this session reads.** Confirmed at 100% against the documented lifecycle, and architecturally corroborated: SessionStart's `additionalContext` exists as a *separate appended message* precisely because hooks run too late to join the cached prefix (claim #8, confirmed).

The consequence is not a transient. It is the **steady-state law** of this mechanism:

> **`doctrine boot` always prepares the *next* session. It never freshens the *current* one. Every session reads the snapshot the *previous* session's hook wrote.**

The §5.4 note — *"first session before any generation: `@`-import resolves empty (benign), filled next start"* — is not a one-time bootstrap quirk. **It is how every session behaves forever.** Governance edits land one session late. Re-running `doctrine boot` mid-session (the §5.4 `/canon` guidance) accomplishes **nothing** for the current context; the import already loaded at step 2.

And the same `boot-research.local.md` you adapted (lines 265–271) enshrines the *same inverted order* — spec-driver's original "startup.sh runs → THEN Claude loads prefix." **You corrected its load mechanism (`.claude/rules` → `@`-import) but inherited its ordering sin unexamined.** The heresy is ancestral.

**Risk:** The design's central freshness guarantee is a lie, and the verification it prescribes (`assert no async:true`) tests a property irrelevant to the failure — it would pass, green and smiling, while the snapshot is permanently one session stale. *They burned the wrong witch and called the village cleansed.*

**Mitigating mercy:** The *value proposition survives*. Zero tool calls — intact. Cache stays warm — intact. The casualty is **freshness alone**, and for "governance that rarely changes" a one-session lag is *tolerable* — **once it is named, verified, and the operational UX is corrected.** The design is recoverable. Its *claims* are not.

**Sentencing:** Strike "review fix #2" in its entirety. Strike the §5.4/§9 synchronicity-equals-freshness doctrine. Replace with the **one-session-lag model, empirically confirmed against the live harness before lock.** Correct the manual-edit ritual: *regenerate `doctrine boot` **then** `/clear` or restart* — running boot alone is impotent, and `/clear` alone serves the pre-edit corpse (it too loads at step 2, regenerates at step 3).

---

### ⛧ Charge II — The design fails silent in every limb, and posts no sentry. *(severity: HIGH)*

**Doctrine violated:** §5.2, §5.4, §10 #5 — and the house law *"correctness comes first and last."*

The design swallows every error into a benign comment, then provides **one** detector — and that detector is blind to the very failures the swallowing creates:

- `produce(kind, root, exec) -> Section` (§5.2) — note: returns `Section`, **not `Result`**. *"miss/err → benign `<!-- … -->` marker body, never a crash."* A corrupt memory store, an unreadable ADR, a panicking listing — all render as a silent comment. The agent loses that governance limb and **is never told.**
- Hook errors swallowed (inherited from spec-driver's `>/dev/null 2>&1`, research §5).
- `current_exe()` staleness (Charge III) — silent.

The **sole** health check (§5.4): `/route` warns *"if the heading is absent."* But the top heading `# Doctrine Boot Context` is emitted by `render_boot` **unconditionally** — it is present even when every section beneath it has collapsed into markers, even when the whole file is one session stale. The sentry guards the door and ignores the fire.

**Risk:** Silent, undetectable governance loss. A half-empty snapshot passes inspection. *A plague that shows no buboes is the one that empties the city.*

**Sentencing:** Add a freshness/health signal the presence check can actually read — a generation stamp the projection carries, a per-section populated/marker tally, or a `doctrine boot --check` the `/route` gate can interrogate. Detection must see **staleness and partial-population**, not merely total absence.

---

### ⛧ Charge III — A machine-path baked into a swallowed-error hook is a silent suicide pact. *(severity: MEDIUM)*

**Doctrine violated:** §5.3, §10 #5 (where it is waved through as "accepted for v1, flagged").

`current_exe()` is baked into both the snapshot body and the `settings.local.json` hook command (§5.3, D7). The pure/impure placement is **correct** — resolution lives in the impure shell, never the pure layer (§4 house rule upheld; I find no heresy *there*). But the **reliability** disposition is too glib. Under a nix/cargo wrapper, `current_exe()` may resolve to a store path that **changes on rebuild**. The hook command then points at a dead binary → the hook fails → errors are swallowed (Charge II) → **boot.md freezes forever, silently**, and the only detector can't see it (Charge II again). "Accepted for v1" is acceptable *only* with detection. There is none.

**Sentencing:** Do not merely "flag." Either resolve a stable invocation (PATH probe / configured path / skip-if-absent with a visible warning) **or** make the resulting staleness detectable per Charge II. An accepted limitation with no alarm is not accepted — it is concealed.

---

### ⛧ Charge IV — Two abstractions for one concept: `enum Agent` and `trait Harness` share a codebase and a purpose. *(severity: MEDIUM)*

**Doctrine violated:** house law *"No parallel implementation! Find potential duplication before writing new code"* and *"OBSESS over coupling and cohesion."*

`src/skills.rs:60` already models the agent concept: `enum Agent { Claude, Other(String) }`, with Claude special-cased (direct) and the rest delegated. §5.2 now erects a **second** model of the same concept: `trait Harness` + `Box<dyn Harness>` + `registry()` + `resolve_harnesses` — mirroring even `skills::resolve_agents` by the design's own admission (§5.3). R2 (D8) is User-approved, and I do not relitigate the *choice* of a seam. I indict its **soundness**: the design **never explains why `enum Agent` could not be extended or shared.** And the trait pays poorly — Claude is the one true implementor; codex's `install_refresh` is near-empty (import-only, no hook). A trait, a boxed dyn dispatch, a registry, and **two** outcome enums (`RefreshOutcome` *and* `RefOutcome`) — to serve one full adapter and one stub. That is the YAGNI scent the slice claims to avoid.

**Sentencing:** Before lock, the design must either (a) justify in writing why `skills::Agent` cannot be reused or generalised — naming the concrete third-harness scenario that earns the trait — or (b) collapse the seam toward the existing enum. Two names for one idea is how a codebase rots. *Excise the twin, lest the body grow a second head.*

---

### ⛧ Charge V — "Additive only" is misattributed; the knife is needed where the design says it isn't. *(severity: LOW–MEDIUM)*

**Doctrine violated:** §5.2, §6, §8 (the "additive to avoid the slice-012 clash" assurance) vs the behaviour-preservation gate.

The witnesses testify the opposite of the design's worry:

- **`memory.rs` — the file the design fears (SL-012 contention)** — *already* exposes the seam: `format_list(rows) -> String` (`memory.rs:944`) and `select_rows(...)` (`memory.rs:924`), composed by `run_list` (`memory.rs:1084`). `memory::list_rows(root, filter)` is a **genuinely additive** wrapper over existing pure functions, touching no existing line. The design's instinct to keep it additive is *correct and achievable* — and the SL-012 collision risk is therefore **lower than §8 fears**.
- **`adr.rs` — the file the design treats as trivial** — has **no** row-returning function. `run_list` (`adr.rs:149`) writes rows straight to `io::stdout()`. `adr::list_rows(root) -> Result<String>` demands an **extract-refactor** of `run_list`'s innards — behaviour-preserving (the end-to-end test at `adr.rs:331` is the proof obligation), but **not "additive."**

The design conflated the two and pointed the alarm at the wrong file.

**Sentencing:** Rename the deed honestly: memory = additive wrapper (clean); adr = behaviour-preserving extract guarded by the `adr.rs:331` suite. Re-weight the SL-012 risk down accordingly.

---

### ⛧ Charge VI — A fourth altar of governance, with no boundary drawn. *(severity: LOW)*

**Doctrine violated:** *"OBSESS over coupling and cohesion"*; the irony of a **doctrine tool** breeding doctrine sprawl.

`governance.md` (§5.3, D5) joins `CLAUDE.md`, `doc/*`, and the ADRs as authored project truth. The slice protests it is "a pointer layer, not a competing source of truth" — but the design **draws no sharp line** between what belongs in `governance.md` versus the three existing surfaces. (The gitignore mechanics I examined and **acquit**: `.doctrine/*` at `.gitignore:12` matches the file directly, so `!.doctrine/governance.md` is *valid* negation semantics; downstream the manifest writes additive entries, not the blanket — §5.3 is **sound on this point**.) The heresy is not the file. It is the *undefined boundary*.

**Sentencing:** Add one paragraph delimiting `governance.md`'s remit against CLAUDE.md / `doc/*` / ADRs, or the fourth altar becomes the place rules go to be forgotten.

---

### ⛧ Charge VII — Ownership-by-suffix is brittle at the edges. *(severity: LOW)*

**Doctrine violated:** §5.3 "review fix #4."

Matching the doctrine-owned hook by *path basename `doctrine` + command ending in ` boot`* breaks if the resolved exec path contains spaces (naive tokenisation), and the matcher string `startup|clear` is asserted from research, not verified. *In partial absolution:* **this very session proves `clear` fires a SessionStart hook** (the caveman hook fired on `/clear` — witnessed in the session reminder), so `clear` as a source is **empirically valid**; only the regex-OR matcher *string* wants a live confirmation at wiring.

**Sentencing:** Harden the ownership match against spaces; confirm the `startup|clear` matcher string against the harness in the same live test that settles Charge I.

---

## 2. QUESTIONS (interrogatories)

1. **The cardinal question.** Will you submit the mechanism to a live ordeal *before* lock: edit governance, run the hook, and observe whether the **same** session sees fresh or stale boot content? Everything hangs on the answer, and only the live harness may give it.
2. Is a **permanent one-session freshness lag** acceptable for this governance, given the value (zero tool calls, warm cache) survives it? If yes, the design is salvageable by honest rewrite. If no, the SessionStart-hook trigger must be reconsidered (e.g. `additionalContext` — fresh but non-cached, the opposite trade).
3. Is **SL-012 fully landed** (the git log shows its commits merged), such that `boot.rs` may extract against a stable `memory.rs`? If so, Charge V's collision risk is near nil.
4. What concrete **third harness** justifies `trait Harness` over extending `skills::Agent` (Charge IV)? Name it, or fold the seam.
5. Where does `governance.md` end and CLAUDE.md / `doc/*` / ADRs begin (Charge VI)?

---

## 3. PRONOUNCE JUDGEMENT

**This design is tainted with heresy and MUST NOT lock. `/plan` is forbidden until penance is done.**

Let the record show what is **sound** — for the Inquisition is just, not merely cruel: the pure/impure split is upheld; the `@`-import-over-`.claude/rules` correction is *vindicated* by the live harness (claim #3 confirmed, ≤4 hops confirmed); `settings.local.json` honours the hook (claim #6 confirmed); Claude does **not** read AGENTS.md natively (claim #4) — so the design's one-file-per-harness rule (fix #1) and its symlink-dedup are **correct**; the gitignore negation is valid; the memory-reuse instinct is achievable.

But the heart of the machine — *the belief that a SessionStart hook freshens the session that triggers it* — **is refuted by the harness's own documented lifecycle, and the design declares this very point "resolved."** A resolved falsehood with a green gate that tests the wrong axiom is the **gravest** form of this heresy: it is not ignorance, it is **false confidence sealed against challenge.** That is precisely what an inquisition exists to burn.

The gate held. *No plan. No code.* The design must return to the forge.

---

## 4. SENTENCING (ordered penance, with verification)

1. **The Ordeal by Fire (gates all else).** Before any rewrite is trusted: on the live Claude harness, change boot content and observe whether the triggering session reads fresh or stale. Confirm the `startup|clear` matcher string in the same trial (Charges I, VII).
   *Verification:* a witnessed transcript. *Punishment for skipping: the heretic is bound to the wheel and shown, limb by limb, that the hook ran after the prefix sealed.*

2. **Recant the freshness lie.** Strike "review fix #2" and the synchronicity-equals-freshness doctrine from §5.4/§5.5/§9/§10. Inscribe the **one-session-lag law** and the corrected ritual (*regenerate, **then** clear/restart*). Strike the `assert no async:true` gate as testing-the-wrong-thing; replace with a freshness assertion (Charge I).
   *Verification:* §5 and §9 read true against the ordeal's outcome.

3. **Post a sentry against silence.** Add a freshness/health signal — generation stamp and/or section-populated tally and/or `doctrine boot --check` — that `/route` interrogates. Make staleness and partial-population **visible** (Charges II, III).
   *Verification:* a test where a failed section and a stale file each trip a *visible* warning. *Punishment for relapse: the penitent stands in the square wearing the half-empty snapshot as a sanbenito.*

4. **Slay the twin abstraction or justify it in writing.** Reconcile `trait Harness` against `skills::Agent` — name the third harness or fold the seam (Charge IV).

5. **Speak plainly of the knife.** Re-label memory-reuse (additive wrapper over `format_list`/`select_rows`) vs adr-reuse (behaviour-preserving extract of `run_list`, guarded by `adr.rs:331`); re-weight SL-012 risk (Charge V).

6. **Draw the boundary; harden the latch.** Delimit `governance.md`'s remit (Charge VI); harden the JSON ownership match against spaces (Charge VII).

7. **Re-present for lock.** Only when I–VI are dispatched does the design return to a hostile pass. *Then,* and not before, may `/plan` be authored.

I have written my findings to no commit and touched no git, as the concurrent session commands the tree. The penance is recorded; let the architect dispose of each charge in §10 and loop until the design is **clean, or confessed-and-bounded.**

*Go now. Burn the false certainty, keep the true mechanism, and return when the snapshot can tell you how old it is.*

**ACCENDE IGNES VERITATIS CORPORIBUS MALEFICARUM**
