# Notes SL-170: Dispatch handover trust-gate

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-03 — F-3 comment-strip walked back (POL-002), Option D

**Decision (`/consult`, authorized):** the S3 keyword match is plain **raw
substring** over `test_file` — NO comment/string-literal stripping. `patterns`
(line-anchored regex) stays as the opt-in language-agnostic shape escalation.

**Why F-3 was wrong:**
1. **POL-002 heresy.** Comment syntax (`//` Rust, `#` Python, …) and string-literal
   syntax are host-LANGUAGE conventions. A gate stripping them load-bears the
   gate's Pass/Fail *correctness* on the host project's language — exactly what
   POL-002 facet-1 forbids ("conventions may inform a default; never carry
   correctness"). doctrine governs arbitrary repos.
2. **Zero-false-fail breach.** Dogfooding `verify-vt 170` against PHASE-02's
   *completed* e2e: keyword `check` lives only as `cmd.arg("check")` (a string
   literal), `fingerprint` only in doc-comments → both stripped → FALSE FAIL on
   legitimate, shipped work. e2e tests reference CLI tokens as string args and
   output assertions as `stdout.contains("…")` — string-stripping is structurally
   incompatible with the dominant e2e keyword case.

**Threat-model consistency:** §5.2 already scopes the gate to worker **OMISSION**,
not adversary. Comment/dead-string *bait* is an adversarial move — out of scope by
the design's own words. Raw substring fully serves omission (omitted work ⇒ keyword
absent entirely ⇒ Fail).

**Accepted weakness (documented):** a keyword present only in a comment now
satisfies. Tolerable — still catches genuine omission; the bait that beats it is
out of the threat model; `patterns` is available for an author who wants a code
shape.

**Surfaces edited:** `src/vtgate.rs` (removed `strip_noncode`), design.md §5.2 +
edges + F-3 finding-log, plan.toml PHASE-03 EX-2 + VT-2 (re-authored, id immutable).
At reconcile/close: F-3's design-record entry already annotated WALKED-BACK in
design.md §"codex review findings".

## PHASE-04 — S6 VT-status summary at conclude / handover

**Shape: thin code, mostly skill wiring.** The gate (`slice verify-vt`) already
existed (PHASE-03). PHASE-04 = (a) wire it into the `/dispatch` conclude cadence,
(b) prove the conclude contract by e2e, (c) close the one VT-1 render-unit gap.

**Two scoping decisions (within design §5.4, not new gaps):**
- **D2 — conclude is the skill cadence, NOT a new CLI verb.** EX-1's "dispatch
  conclude" names the *beat*. Design §5.4 rejected folding into `prepare_review`
  (git-tree blob reads, ADR-001 cohesion) and reserved verify-vt's injected-reader
  seam for the DEFERRED committed-graph hardening. So conclude runs as the
  `/dispatch` prose cadence `slice verify-vt → on green prepare-review → remove
  worktree`. fs reader suffices (orchestrator commits any waiver before the gate;
  coord working tree == committed graph).
- **D1 — one-line S1 status is carried orchestrator prose, NO new renderer.** The
  regression diff runs at the *verify* beat (pre-commit); at conclude (post-commit)
  it is recalled, not re-run — nothing fresh to render. `render_delta` already
  yields a liftable one-line status. A second renderer would be parallel
  implementation. (Door left open in the sheet if prose proves too loose.)

**Surfaces edited:** `src/vtgate.rs` (render unit now covers all four verdicts incl.
Fail — VT-1); `tests/e2e_dispatch_verify_vt.rs` (NEW — conclude-gate contract: VT-2
clean→exit0+block / Fail→non-zero halt; VT-3 committed-coord-tree waiver honoured
non-halting, EX-3 positive half); `plugins/doctrine/skills/dispatch/SKILL.md`
(conclude cadence + S3/S6 prose); `plugins/doctrine/skills/handover/SKILL.md`
(embed VT block + one-line S1).

**Footgun hit (already a memory):** edited the gitignored *projection*
`.doctrine/skills/...` first — authored source is `plugins/doctrine/skills/...`
(`.gitignore:27 .doctrine/*`). See mem.pattern.distribution.skills-source-vs-installed
/ mem.signpost.doctrine.skill-masters. git refuses a path-limited commit *through*
the `.claude/skills` symlink ("beyond a symbolic link") — another tell.

**EX-3 scope line:** the NEGATIVE half of INV-6 (mechanically rejecting a
working-fs-only waiver) stays OUT — deferred prepare_review committed-graph reader.
The injected-reader seam is built for it; the cadence (commit-waiver-before-gate)
carries it until then.

**VH-1 dogfood:** `verify-vt 170` renders clean PHASE-01..04 (PHASE-04 VT-3
UNCHECKABLE — the INV-6 waiver case is behavioural, not greppable; correct).
`check commit` green. Commit 1fef2e2a (feat).
