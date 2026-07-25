# Review RV-307 — design of SL-230

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Arraigned:** `.doctrine/slice/230/design.md` (design intent only — not code,
not the plan, which does not yet exist). Posture: `--raiser inquisitor`, external
tribunal (codex/GPT). One prior *internal* adversarial pass already landed
(§ 10, A1–A4); this is the first external pass. Findings A1–A4 are integrated
canon now, not open items — do not re-raise them, but *do* test whether their
integration is sound.

**Pre-reading.** `slice-230.md` (scope), `research/research.md`,
`design.md`. Governance in force: ADR-001 (layering), ADR-013 (governance→work
routes through a REV), POL-002 (no host-layout load-bearing), STD-001 (no magic
strings), SL-008 D6 (`thread_expiry` is reviewed canon), SPEC-007 (memory system).

### Lines of interrogation

1. **The attestation hole the relaxation may open.** § 5.4 makes a corpus-dirty
   tree yield a `Commit`-anchored frame at HEAD. But memory items live *under*
   `.doctrine/memory/items/**` — inside the excluded set. So a memory whose body
   is uncommitted (or whose directory is entirely untracked — T8's `record` →
   `verify` case) can be stamped `verified_sha = HEAD`, where HEAD demonstrably
   does not contain the body being attested. Interrogate: does § 5.4's
   `validate` own-directory check (`rev-list verified_sha..HEAD -- <dir>`) then
   return 0 forever, so the D5 invalidation is defeated *precisely* in the case
   the relaxation creates? Is OQ-3's deferred body-digest load-bearing rather
   than optional?

2. **Coherence of the exclusion mechanism.** § 5.1 places
   `corpus_guard::is_excluded(path, roots)` at leaf tier, and § 5.2 elaborates a
   *pure exclusion predicate*. Yet § 5.2's `capture_with` performs exclusion via
   **git pathspecs** in all three probes. Is `is_excluded` used at all — or has
   the design specified two mechanisms for one job, one of them dead? § 5.1 also
   still names `source_clean` in the tier diagram and § 5.3 still names it in
   prose, though D3 (revised) replaced it with `capture_with`.

3. **Git pathspec semantics, verified not assumed.** Do `git diff HEAD --binary`,
   `git ls-files --others --exclude-standard -z`, and `git diff-index --quiet
   --cached HEAD` behave as claimed when handed an *exclusion-only* pathspec set
   (`:(exclude)X` with no positive pathspec)? Does `--binary` interact? Probe it
   against the real repo; the design's ✓ marks are claims, and claims are
   confessions until tested.

4. **Ordering, atomicity, and the composed changed-flag.** § 5.4 steps 1–5.
   `write_body` runs *before* `apply_edit` validates its fields. Does a rejected
   metadata field leave a written body and an unwritten TOML — i.e. does the
   argument-validation failure path now mutate the tree? Does R1's stated crash
   asymmetry actually hold given step 3 (`clear_verification`) sits between?

5. **Governance completion.** ADR-013 routes the SPEC-007 amendment through a
   `REV-NNN`. § 3 and § 9 both promise it; no REV id is cited anywhere. Is the
   design closeable with an uninstantiated governance dependency? Likewise:
   `spec-007.toml:22` and `spec-007.md:132` — are those the *only* sites that
   assert the clean-tree contract?

6. **Scope boundary and blast radius.** D6 confines the work to items and leaves
   masters hand-edit-only — but § 5.2's exclusion roots contribute
   `MEMORY_MASTERS_DIR` on existence. Do the two halves agree? E2 claims
   repo-empty masters never hit the dirty gate; verify against
   `anchor_kind`/`capture` reality.

7. **Behaviour preservation (R3, I1, I2).** Is I1 truly by construction? Does
   `untracked_fingerprint`'s new `excludes` parameter have exactly one caller as
   claimed? Does I2's no-`write-tree` claim survive the new probe path?

8. **Test adequacy (§ 9).** T7 asserts a HEAD-commit stamp — does that assertion
   encode line-1's hole as *intended* behaviour? Are T1–T16 sufficient for the
   invariants they claim to pin, and is any invariant unpinned?

9. **Unknown unknowns.** Silent error handling, vague criteria, magic strings,
   duplicated concepts, inconsistent terminology, hidden ownership boundaries.
   All works are potential heresies.

### Standing order to the external tribunal

Raise every charge on **this ledger** (`doctrine review raise RV-307 …`), framed
*expected vs observed*, with file:line evidence. Severity `blocker` only for
heresy that must not ship unreconciled. Do not dispose — disposition is the
architect's answer, entered separately.
