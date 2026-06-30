# Review RV-201 — design of SL-182

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Second adversarial round on `SL-182/design.md`, convened AFTER the RV-200
inquisition reconciled and the design was re-locked. Posture: `--raiser
inquisitor`; aspect: design. The first round changed the design materially;
this tribunal's charge is narrow and surgical — **does the reconcile itself
harbour new heresy, and do the freshly-rewritten sections (§5.1, §5.3, §5.4, §7)
cohere?** RV-200's ten findings are SETTLED and are not re-tried (the inquisitor
who re-prosecutes the acquitted bears false witness).

Doctrine held to: ADR-008 (the claude-arm confinement gap this discharges),
ADR-006 (D-sole-writer), POL-002 (fail-closed on the unsupported), STD-001 (no
magic strings), and the RSK-014 probe-h1 EMPIRICAL ground truth. Every hook
claim tried against the local `docs/claude` cache (authoritative over web).

Lines of interrogation (the four soft targets the author flagged, swept against
the seams, not the prose): (1) F-1 shared-profile model §5.3 — is "single arming,
single intent, no differing-sibling to leak" airtight, or asserted? (2) F-3
capture-before-remove §5.4 — is the WorktreeRemove ordering real, and does the
design commit to a hook that can actually be awaited? (3) F-2 fail-closed exec
§5.4/D-reg — does the chosen plugin materialization seam support injecting an
absolute exec path, or is the remediation resting on a capability that isn't in
code? (4) Internal coherence §5.1 ↔ §5.4 — does the resolve_exec relocation leave
contradictions; do §5.3 ↔ D2 ↔ scope tell ONE keying story? An independent
adversarial pass was run via **codex (GPT-5.5, read-only)**; its charges were
corroborated against the source seams (`src/skills.rs`, `plugins/.../hooks.json`,
`src/worktree/create.rs`, `src/dispatch.rs`) and the `docs/claude` cache before
entry. The seam evidence — not the model's word — is what convicts.

## Synthesis

**Judgement: NEW HERESY FOUND. The reconcile that re-locked SL-182 smuggled in a
false fail-closed guarantee and left three load-bearing claims resting on
unspecified or mislocated machinery.** The confinement core remains righteous and
the RV-200 corrections were the right corrections — but the *graduation of those
corrections into the design prose* outran what the code seams actually provide.
The design does NOT hold at LOCKED. One blocker gates; three majors and a minor
must reconcile in the same sitting.

**The one that gates `/plan` (blocker):**

- **F-1 — the preferred registration ships FAIL-OPEN.** D-reg's parenthetical
  "invoking a resolved absolute doctrine (fail-closed exec ... NOT bare PATH)"
  (design.md:381-389) is *false as-built*. The plugin `hooks.json` is materialized
  by a verbatim RustEmbed byte-copy (`install_hooks_plugin_for_claude`,
  src/skills.rs:1046-1049) of an asset that hardcodes **bare** `doctrine`
  (plugins/doctrine/hooks/hooks.json:7,18); `resolve_exec` is never invoked on
  this path. The fail-closed property the reconcile claimed to win for F-2 holds
  ONLY on the settings.local FALLBACK — the *preferred* plugin path inherits the
  exact RSK-014 fail-open (hooks.md:629-643: only exit-2 blocks; a stale/absent
  `~/.cargo/bin/doctrine` runs the tool unconfined). The remediation also leaves
  its own mechanism unresolved ("absolute path OR exit-2 shim", neither wired,
  shim-authorship unanswered). **This carries a remediation OPTION (template the
  embedded JSON through resolve_exec at materialization vs embed+materialize a
  shim) — a User/`/design` decision, deliberately withheld from `verify`.**

**The corroborating majors (reconcile in the same pass):**

- **F-2 — capture-before-remove leads with the wrong hook.** §5.4 leads with
  `WorktreeRemove (and/or SubagentStop)` and gates OQ-2 on WorktreeRemove
  observing the tree intact — but WorktreeRemove has NO decision control, is
  side-effect-only, failures debug-log-only (hooks.md:680/814/2442); nothing
  documents that Claude awaits it before `git worktree remove`. **SubagentStop**
  *can* prevent the subagent stopping (hooks.md:658) — blocking-capable, awaited,
  carries `agent_id`+`agent_transcript_path` (hooks.md:1930-1957). The funnel
  must COMMIT to SubagentStop and demote WorktreeRemove to cleanup. (Not a blocker:
  the defined abort to Path C / IDE-024 holds, and the gate is pre-execute testable
  — but the design currently bets on the non-controlling hook.)

- **F-3 — the scope doc is split-brained, and "scope doc corrected" is a false
  attestation.** RV-200 F-4 and design §10 both attest the scope corrected, yet
  slice-182.md objective 3 (47-50) still preaches `agent_id` keying, "per-worker"/
  "specific worker" tuning, `extra_ro`, and `strict/loose` — every clause
  repudiated by locked D2/D6/F-1. A current decisions block sits atop a stale
  objectives block in the same file.

- **F-4 — the shared-profile safety proof rests on unspecified machinery.** The
  "no differing-sibling to leak" / "must not interleave a second arming" claims
  (§5.3:199-232) are asserted, not grounded: the profile-declaration file is never
  named, never given an atomicity contract with `base` (which the arming seam
  overwrites idempotently, src/dispatch.rs:200-206), and the create-fork provision
  step is NET-NEW — `classify_create` writes nothing under `jail/` today
  (src/worktree/create.rs:166-187). The per-arming MODEL is sound (acquitted); the
  "must not interleave" property is in fact structurally enforced by the blocking
  Agent call (one-turn batch blocks until all N complete, so the single-threaded
  orchestrator has no turn to re-arm) — but the design grounds it in *discipline*
  rather than that structure.

**The minor (coherence cleanup):**

- **F-5 — vestigial `resolve_exec` in the runtime layer.** §5.1:85 and D1:358
  still list `resolve_exec` as a `pretooluse.rs` (runtime) responsibility; the
  fail-closed-exec fix is install-time (the binary is already running at hook-exec,
  so `current_exe()` there is useless). Scar tissue from the pre-F-2 draft; the
  twin of F-1.

**Acquittals (a clean verdict where the prose held):**

- **V-plugin deferral is DEFENSIBLE.** The design does not claim the probe proved
  the plugin path; it documents settings.local as proven, makes V-plugin the first
  execute gate, and scopes a same-phase settings.local fallback (design.md:257-268,
  446-451). Explicit, ordered before reliance, bounded by a named contingency —
  acceptable at lock.
- **OQ-2 defined-abort still holds** — it is a lock-time risk with a named
  escalation (Path C / IDE-024), not a bare "verify later", and is pre-execute
  testable once F-2's hook commitment is made.
- **F-1 keying MODEL (per-arming, shared parallel profile)** is the right call and
  immune to the differing-siblings leak under one slot — the heresy is in the
  *machinery's specification*, not the model.

**Standing risks after reconcile:** the slice's correctness still hinges on two
unverified harness behaviours (plugin-PreToolUse firing; capture-hook-before-remove
timing) — both already first-execute gates with named fallbacks. This round adds a
third, sharper exposure: the *install seam itself* must be changed to make the
preferred path fail-closed (F-1), or the design must demote the plugin path to the
proven-fail-closed settings.local path. That is not a `/plan` detail — it decides
whether the headline registration is safe at all.

**Sentence:** the design returns to its author for a second reconciliation. **F-1
holds the close-gate** (option-bearing: a User decision between materialization-time
templating and an embedded shim). F-2, F-3, F-4, F-5 carry clear corrective
direction and may reconcile directly. The five charges are withheld from `verify`
until the penance is done and canon tells the truth — `/plan` is NOT yet clear.

> Let the false guarantee be put to the fire; the floor that holds shall remain.

> **HERESIS URITOR; DOCTRINA MANET**
