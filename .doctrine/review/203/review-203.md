# Review RV-203 — design of SL-183

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Target.** `SL-183/design.md` — the macOS Seatbelt write-confinement arm, freshly
ratified (D-mac1..4, OQ-mac4 resolved 2026-07-01), arraigned **before lock**. One
facet: `design`. Posture: `inquisitor`. External adversary: codex MCP / GPT-5.5.

**Sanctioned doctrine held over the accused.**
- ADR-001 (layering leaf←engine←command; pure/imperative split — no realpath/exec
  in the pure builders).
- ADR-008 (project-local jail isolation), ADR-006 (worktree posture).
- POL-002 (platform independence / fail-closed: ambiguity ⇒ `deny`, never
  unwrapped pass-through).
- STD-001 (no magic strings — profile tokens, `-D` param names, device-sink set,
  the `xcrun_db` regex must be single-sourced constants).
- Behaviour-preservation gate: SL-182's `resolve_target`/`decide_bash`/`decide_write`
  /`pathcheck`/`opaque_wrap`/`validate_policy` reused UNCHANGED.

**Lines of interrogation.**
1. **The F-A ordering invariant.** SBPL is last-match-wins; the profile now carries
   TWO coarse denies (PTMP, DUTMP) that MUST precede the specific WT/TMP/xcrun_db
   allows. Does §5.1 guarantee the ordering under BOTH denies? Does the `xcrun_db`
   allow (specific) correctly sit AFTER the DUTMP deny (coarse)? Any reordering =
   floor-shadow heresy.
2. **The OQ-mac4 containment hole.** The `xcrun_db` allow re-opens a host-shared,
   GC-uncontrolled path OUTSIDE the floor. Is the regex `DUTMP/xcrun_db` actually
   narrow (anchored?), or does it match siblings? Is the cross-subagent write
   channel honestly disclosed as a floor breach, or dressed as cosmetic?
3. **F-G git-derivation.** The Jailer binds the worktree from `cwd` via git, not a
   path template. Is this stated as a load-bearing INV the Rust MUST honour, or left
   as prose the /plan can lose? What happens if `cwd` is NOT inside a git worktree
   (nested-repo, submodule, detached)? Fail-closed or leak?
4. **D-mac2 seam / behaviour-preservation.** Does slotting into SL-182's
   `select_jailer` as-is actually hold, given SL-182 is UNBUILT? Is the design
   asserting a fork point that does not yet exist as if it does?
5. **STD-001 magic strings.** Are ALL new tokens (DUTMP param, xcrun_db regex,
   getconf DARWIN_USER_TEMP_DIR literal, the network deny line) named as constants,
   or smuggled in as inline literals?
6. **D-mac4 default-open.** Does "default open, deny only on opt-in" contradict
   POL-002's fail-closed ethos for the NETWORK axis? Is open-by-default defensible
   here, or is it a silent egress hole dressed as forward-compat?
7. **confstr vs $TMPDIR.** §5.5 claims xcrun reads DARWIN_USER_TEMP_DIR via
   `confstr(_CS_DARWIN_USER_TEMP_DIR)`, not `$TMPDIR` — hence redirect fails. Is
   this empirically pinned or asserted? If asserted, it is a heresy of unproven fact.
8. **Probe-fact provenance.** Every "PROVEN" claim (M1-sub, updatedInput, M2
   canonicalization) — is it traceable to results.md, or has confidence inflated
   beyond the evidence?

## Synthesis

**Judgement: the design was TAINTED but not damned — nine heresies confessed under
cross-examination, all now burned out; the design stands, penance discharged, fit
to lock.** The external adversary (GPT-5.5 via codex) and the Inquisitor's own eye
convened as two courts and returned a corroborating verdict: **4 blockers, 3 majors,
2 minors**, zero padding, three lines of attack (F-A ordering, OQ-mac4 disclosure
honesty, PROVEN-claim provenance) tried and acquitted clean.

The four mortal heresies were not cosmetic:
- **F-1 (blocker)** — the design contradicted ADR-001 *in its own words*: realpath +
  `getconf` impurity smuggled into the "pure" `sandbox_exec_argv`. Penance: an impure
  `resolve_inputs` shell split from the pure builders.
- **F-2 (blocker)** — confidence inflated past the evidence: the DUTMP/xcrun_db profile
  wore the "proven shape" vestments though no probe ever touched it. Penance:
  provenance honestly relabelled, verification carried into `/plan` (no re-probe —
  the "design-side only" writ was kept).
- **F-3 (blocker)** — the `xcrun_db` re-allow was an unanchored SBPL substring, a
  wider floor-breach than the "narrowest hole" it claimed. Penance: anchored
  filename regex, bounded to one path segment. *(The Inquisitor's own eye drew this
  blood before the external verdict returned — recorded pre-corroboration.)*
- **F-4 (blocker)** — the load-bearing `cwd`→git derivation had NO fail-closed
  contract, a POL-002 breach. Penance: explicit algorithm + six enumerated failure
  branches, every one ⇒ `deny worktree-subagent Bash`.

The three majors closed real gaps (SL-182 seam falsely dressed as landed code; the
network-policy ambiguity path undefined; §9 self-contradicting on pass-2 status).
The two minors tightened honesty (confstr mechanism de-asserted; a named-constant
catalog laid for STD-001).

**Sentencing — corrective sequence, all DISCHARGED at design time:**
1. §5.2 pure/impure split + named-constant catalog (F-1, F-9).
2. §5.1 anchored `xcrun_db` regex + "proven-shape" provenance correction (F-2, F-3).
3. §5.5 new INV(F-B4) fail-closed `cwd`→git derivation, 6 branches (F-4).
4. §7/§6/§2 seam reframed as SL-182 constraint, not landed API (F-5).
5. §7 D-mac4 network-ambiguity fail-closed (F-6); §9/§8 pass-2 status + R-mac1 (F-7).

**Standing risks carried into /plan (consciously accepted, NOT tolerated drift):**
- **The final DUTMP/xcrun_db profile is design-decided but UNPROBED** (F-2/F-3). Its
  exact rule ordering, canary preservation, and xcrun-tool-still-works must be
  confirmed at `/plan`/first-impl. This is a bounded verification obligation, not an
  open design question — the *decision* is locked, the *empirical confirmation* is
  deferred by the standing "no more probing at design" writ.
- **SBPL `regex` match semantics** (anchoring, param-in-regex composition) are
  asserted from first principles, to be pinned by a cheap `/plan` probe.
- **SL-183 impl remains `needs SL-182`-blocked** — the seam it forks is SL-182's
  design, not disk.

No taint tolerated. No blocker downgraded to dodge the gate. **The design is clean
for lock.**

> **HERESIS URITOR; DOCTRINA MANET**
