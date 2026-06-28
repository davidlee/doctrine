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
